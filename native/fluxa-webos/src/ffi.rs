use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::sync::mpsc::Sender;

use libloading::Library;

use crate::diagnostics::trace;
use crate::events::Event;
use crate::feed::FeedReply;

const LIBRARY: &str = "libplayerAPIs.so.1";
const OBJECT_SIZE: usize = 8192;
const STRING_SIZE: usize = 64;

type Callback = extern "C" fn(c_int, i64, *const c_char, *mut c_void);
type Ctor = unsafe extern "C" fn(*mut c_void, *const c_char);
type Dtor = unsafe extern "C" fn(*mut c_void);
type Load = unsafe extern "C" fn(*mut c_void, *const c_char, Callback, *mut c_void) -> bool;
type Feed = unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_char);
type Verb = unsafe extern "C" fn(*mut c_void) -> bool;
type Text = unsafe extern "C" fn(*mut c_void, *const c_char) -> bool;
type MediaId = unsafe extern "C" fn(*mut c_void) -> *const c_char;
type ChangeCodec = unsafe extern "C" fn(*mut c_void, *const CxxString, *const CxxString) -> bool;

const PLAY_RATE_SYMBOLS: [&str; 1] = ["_ZN17StarfishMediaAPIs11SetPlayRateEPKc\0"];
const CHANGE_CODEC_SYMBOLS: [&str; 1] =
    ["_ZN17StarfishMediaAPIs16changeAudioCodecERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES8_\0"];

#[repr(C)]
struct CxxString {
    data: *const u8,
    len: usize,
    union: [u8; 16],
}

struct BorrowedString {
    raw: CxxString,
    _bytes: Vec<u8>,
}

fn borrowed_string(text: &str) -> BorrowedString {
    let mut bytes = text.as_bytes().to_vec();
    bytes.push(0);
    let mut union = [0u8; 16];
    let capacity = text.len().to_ne_bytes();
    union[..capacity.len()].copy_from_slice(&capacity);
    BorrowedString { raw: CxxString { data: bytes.as_ptr(), len: text.len(), union }, _bytes: bytes }
}

fn optional<T: Copy>(library: &Library, names: &[&'static str]) -> Option<T> {
    names.iter().find_map(|name| unsafe { symbol::<T>(library, name) }.ok())
}

#[repr(C, align(16))]
struct Aligned<const N: usize>([u8; N]);

struct Sink {
    events: Sender<Event>,
    webos_major: u32,
}

#[derive(Debug)]
pub enum Error {
    Library(String),
    Symbol(&'static str),
    Rejected(&'static str),
}

pub struct Starfish {
    _library: Library,
    object: Box<Aligned<OBJECT_SIZE>>,
    sink: Box<Sink>,
    ctor: Ctor,
    dtor: Dtor,
    load: Load,
    feed: Feed,
    play: Verb,
    pause: Verb,
    unload: Verb,
    notify_foreground: Verb,
    seek: Text,
    set_hdr_info: Text,
    media_id: MediaId,
    flush: Verb,
    push_eos: Verb,
    set_time_to_decode: Text,
    set_play_rate: Option<Text>,
    change_audio_codec: Option<ChangeCodec>,
}

extern "C" fn trampoline(kind: c_int, number: i64, text: *const c_char, data: *mut c_void) {
    let sink = unsafe { &*(data as *const Sink) };
    let text = (!text.is_null()).then(|| unsafe { CStr::from_ptr(text) }.to_string_lossy());
    trace(format!("event {kind} {number} {}", text.as_deref().unwrap_or("")));
    let _ = sink.events.send(Event::from_callback(kind, number, text.as_deref(), sink.webos_major));
}

pub(crate) unsafe fn symbol<T: Copy>(library: &Library, name: &'static str) -> Result<T, Error> {
    unsafe { library.get::<T>(name.as_bytes()) }.map(|symbol| *symbol).map_err(|_| Error::Symbol(name))
}

pub(crate) fn c_string(text: &str) -> CString {
    CString::new(text.replace('\0', "")).unwrap_or_default()
}

impl Starfish {
    pub fn open(app_id: &str, webos_major: u32, events: Sender<Event>) -> Result<Starfish, Error> {
        let library = unsafe { Library::new(LIBRARY) }.map_err(|err| Error::Library(err.to_string()))?;
        let mut starfish = unsafe {
            Starfish {
                ctor: symbol(&library, "_ZN17StarfishMediaAPIsC1EPKc\0")?,
                dtor: symbol(&library, "_ZN17StarfishMediaAPIsD1Ev\0")?,
                load: symbol(&library, "_ZN17StarfishMediaAPIs4LoadEPKcPFvixS1_PvES2_\0")?,
                feed: symbol(&library, "_ZN17StarfishMediaAPIs4FeedB5cxx11EPKc\0")?,
                play: symbol(&library, "_ZN17StarfishMediaAPIs4PlayEv\0")?,
                pause: symbol(&library, "_ZN17StarfishMediaAPIs5PauseEv\0")?,
                unload: symbol(&library, "_ZN17StarfishMediaAPIs6UnloadEv\0")?,
                notify_foreground: symbol(&library, "_ZN17StarfishMediaAPIs16notifyForegroundEv\0")?,
                seek: symbol(&library, "_ZN17StarfishMediaAPIs4SeekEPKc\0")?,
                set_hdr_info: symbol(&library, "_ZN17StarfishMediaAPIs10setHdrInfoEPKc\0")?,
                media_id: symbol(&library, "_ZN17StarfishMediaAPIs10getMediaIDEv\0")?,
                flush: symbol(&library, "_ZN17StarfishMediaAPIs5flushEv\0")?,
                push_eos: symbol(&library, "_ZN17StarfishMediaAPIs7pushEOSEv\0")?,
                set_time_to_decode: symbol(&library, "_ZN17StarfishMediaAPIs13setTimeToDecodeEPKc\0")?,
                set_play_rate: optional(&library, &PLAY_RATE_SYMBOLS),
                change_audio_codec: optional(&library, &CHANGE_CODEC_SYMBOLS),
                _library: library,
                object: Box::new(Aligned([0; OBJECT_SIZE])),
                sink: Box::new(Sink { events, webos_major }),
            }
        };
        let uid = c_string(app_id);
        unsafe { (starfish.ctor)(starfish.object.0.as_mut_ptr().cast(), uid.as_ptr()) };
        Ok(starfish)
    }

    fn this(&self) -> *mut c_void {
        self.object.0.as_ptr() as *mut c_void
    }

    pub fn notify_foreground(&self) -> bool {
        unsafe { (self.notify_foreground)(self.this()) }
    }

    pub fn load(&self, payload: &str) -> Result<(), Error> {
        trace(format!("load {payload}"));
        let payload = c_string(payload);
        let data = &*self.sink as *const Sink as *mut c_void;
        unsafe { (self.load)(self.this(), payload.as_ptr(), trampoline, data) }
            .then_some(())
            .ok_or(Error::Rejected("Load"))
    }

    pub fn feed(&self, payload: &str) -> FeedReply {
        let payload = c_string(payload);
        let mut reply = Aligned::<STRING_SIZE>([0; STRING_SIZE]);
        unsafe {
            (self.feed)(reply.0.as_mut_ptr().cast(), self.this(), payload.as_ptr());
            let data = *(reply.0.as_ptr() as *const *const c_char);
            if data.is_null() {
                return FeedReply::Failed;
            }
            FeedReply::parse(&CStr::from_ptr(data).to_string_lossy())
        }
    }

    pub fn play(&self) -> bool {
        unsafe { (self.play)(self.this()) }
    }

    pub fn pause(&self) -> bool {
        unsafe { (self.pause)(self.this()) }
    }

    pub fn unload(&self) -> bool {
        unsafe { (self.unload)(self.this()) }
    }

    pub fn seek_ms(&self, millis: u64) -> bool {
        let millis = c_string(&millis.to_string());
        unsafe { (self.seek)(self.this(), millis.as_ptr()) }
    }

    pub fn set_hdr_info(&self, payload: &str) -> bool {
        trace(format!("hdr {payload}"));
        let payload = c_string(payload);
        unsafe { (self.set_hdr_info)(self.this(), payload.as_ptr()) }
    }

    pub fn flush(&self) -> bool {
        unsafe { (self.flush)(self.this()) }
    }

    pub fn push_eos(&self) -> bool {
        unsafe { (self.push_eos)(self.this()) }
    }

    pub fn set_time_to_decode(&self, payload: &str) -> bool {
        let payload = c_string(payload);
        unsafe { (self.set_time_to_decode)(self.this(), payload.as_ptr()) }
    }

    pub fn set_play_rate(&self, payload: &str) -> Option<bool> {
        let set = self.set_play_rate?;
        let payload = c_string(payload);
        Some(unsafe { set(self.this(), payload.as_ptr()) })
    }

    pub fn change_audio_codec(&self, codec: &str, options: &str) -> Option<bool> {
        let change = self.change_audio_codec?;
        let (codec, options) = (borrowed_string(codec), borrowed_string(options));
        Some(unsafe { change(self.this(), &codec.raw, &options.raw) })
    }

    pub fn media_id(&self) -> Option<String> {
        let id = unsafe { (self.media_id)(self.this()) };
        (!id.is_null()).then(|| unsafe { CStr::from_ptr(id) }.to_string_lossy().into_owned())
    }
}

impl Drop for Starfish {
    fn drop(&mut self) {
        unsafe { (self.dtor)(self.this()) };
    }
}

const SYMBOLS: [&str; 15] = [
    "_ZN17StarfishMediaAPIsC1EPKc",
    "_ZN17StarfishMediaAPIsD1Ev",
    "_ZN17StarfishMediaAPIs4LoadEPKcPFvixS1_PvES2_",
    "_ZN17StarfishMediaAPIs4LoadEPKcPFvixS1_E",
    "_ZN17StarfishMediaAPIs4FeedB5cxx11EPKc",
    "_ZN17StarfishMediaAPIs4FeedEPKc",
    "_ZN17StarfishMediaAPIs4PlayEv",
    "_ZN17StarfishMediaAPIs5PauseEv",
    "_ZN17StarfishMediaAPIs10setHdrInfoEPKc",
    "_ZN17StarfishMediaAPIs10getMediaIDEv",
    "_ZN17StarfishMediaAPIs5flushEv",
    "_ZN17StarfishMediaAPIs7pushEOSEv",
    "_ZN17StarfishMediaAPIs13setTimeToDecodeEPKc",
    "_ZN17StarfishMediaAPIs11SetPlayRateEPKc",
    "_ZN17StarfishMediaAPIs16changeAudioCodecERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES8_",
];

pub fn probe_symbols() -> Vec<(String, bool)> {
    let Ok(library) = (unsafe { Library::new(LIBRARY) }) else {
        return vec![(LIBRARY.to_owned(), false)];
    };
    let mut found = vec![(LIBRARY.to_owned(), true)];
    for name in SYMBOLS {
        let present = unsafe { library.get::<*const c_void>(format!("{name}\0").as_bytes()) }.is_ok();
        found.push((name.to_owned(), present));
    }
    found
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use super::*;

    #[test]
    fn missing_library_is_an_error_not_a_panic() {
        let (tx, _rx) = channel();
        assert!(matches!(Starfish::open("app", 5, tx), Err(Error::Library(_))));
    }

    #[test]
    fn borrowed_string_is_nul_terminated_and_records_capacity() {
        let text = borrowed_string("abc");
        assert_eq!(text.raw.len, 3);
        assert_eq!(unsafe { *text.raw.data.add(3) }, 0);
        assert_eq!(&text.raw.union[..std::mem::size_of::<usize>()], 3usize.to_ne_bytes());
    }

    #[test]
    fn interior_nul_is_dropped() {
        assert_eq!(c_string("a\0b").to_bytes(), b"ab");
    }
}
