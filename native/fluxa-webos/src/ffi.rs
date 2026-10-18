use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::sync::OnceLock;
use std::sync::mpsc::Sender;

use libloading::Library;

use crate::diagnostics::trace;
use crate::events::Event;
use crate::feed::FeedReply;

const LIBRARY: &str = "libplayerAPIs.so.1";
const OBJECT_SIZE: usize = 65536;
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

const CTOR: &str = "_ZN17StarfishMediaAPIsC1EPKc";
const DTOR: &str = "_ZN17StarfishMediaAPIsD1Ev";
const LOAD: &str = "_ZN17StarfishMediaAPIs4LoadEPKcPFvixS1_PvES2_";
const FEED: &str = "_ZN17StarfishMediaAPIs4FeedB5cxx11EPKc";
const FEED_COW: &str = "_ZN17StarfishMediaAPIs4FeedEPKc";
const COW_STRING_DTOR: &str = "_ZNSsD1Ev";
const PLAY: &str = "_ZN17StarfishMediaAPIs4PlayEv";
const PAUSE: &str = "_ZN17StarfishMediaAPIs5PauseEv";
const UNLOAD: &str = "_ZN17StarfishMediaAPIs6UnloadEv";
const NOTIFY_FOREGROUND: &str = "_ZN17StarfishMediaAPIs16notifyForegroundEv";
const SEEK: &str = "_ZN17StarfishMediaAPIs4SeekEPKc";
const HDR_INFO: &str = "_ZN17StarfishMediaAPIs10setHdrInfoEPKc";
const MEDIA_ID: &str = "_ZN17StarfishMediaAPIs10getMediaIDEv";
const FLUSH: &str = "_ZN17StarfishMediaAPIs5flushEv";
const PUSH_EOS: &str = "_ZN17StarfishMediaAPIs7pushEOSEv";
const TIME_TO_DECODE: &str = "_ZN17StarfishMediaAPIs15setTimeToDecodeEPKc";
const PLAY_RATE: &str = "_ZN17StarfishMediaAPIs11SetPlayRateEPKc";
const CHANGE_CODEC: &str =
    "_ZN17StarfishMediaAPIs16changeAudioCodecERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES7_";

static LIBRARY_HANDLE: OnceLock<Result<Library, String>> = OnceLock::new();

fn library() -> Result<&'static Library, Error> {
    LIBRARY_HANDLE
        .get_or_init(|| unsafe { Library::new(LIBRARY) }.map_err(|err| err.to_string()))
        .as_ref()
        .map_err(|err| Error::Library(err.clone()))
}

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

fn optional<T: Copy>(library: &Library, name: &'static str) -> Option<T> {
    unsafe { symbol::<T>(library, name) }.ok()
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
    object: Box<Aligned<OBJECT_SIZE>>,
    sink: Box<Sink>,
    ctor: Ctor,
    dtor: Dtor,
    load: Load,
    feed: Feed,
    feed_cow: bool,
    feed_reply_dtor: Option<Dtor>,
    play: Verb,
    pause: Verb,
    unload: Verb,
    notify_foreground: Verb,
    seek: Text,
    set_hdr_info: Option<Text>,
    media_id: MediaId,
    flush: Verb,
    push_eos: Verb,
    set_time_to_decode: Option<Text>,
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
        let library = library()?;
        let mut starfish = unsafe {
            Starfish {
                ctor: symbol(library, CTOR)?,
                dtor: symbol(library, DTOR)?,
                load: symbol(library, LOAD)?,
                feed: symbol(library, FEED).or_else(|_| symbol(library, FEED_COW))?,
                feed_cow: optional::<*const c_void>(library, FEED).is_none(),
                feed_reply_dtor: unsafe { cow_string_dtor() },
                play: symbol(library, PLAY)?,
                pause: symbol(library, PAUSE)?,
                unload: symbol(library, UNLOAD)?,
                notify_foreground: symbol(library, NOTIFY_FOREGROUND)?,
                seek: symbol(library, SEEK)?,
                set_hdr_info: optional(library, HDR_INFO),
                media_id: symbol(library, MEDIA_ID)?,
                flush: symbol(library, FLUSH)?,
                push_eos: symbol(library, PUSH_EOS)?,
                set_time_to_decode: optional(library, TIME_TO_DECODE),
                set_play_rate: optional(library, PLAY_RATE),
                change_audio_codec: optional(library, CHANGE_CODEC),
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
            let reply_slot: *mut c_void = reply.0.as_mut_ptr().cast();
            (self.feed)(reply_slot, self.this(), payload.as_ptr());
            let data = *(reply.0.as_ptr() as *const *const c_char);
            if data.is_null() {
                return FeedReply::Failed;
            }
            let reply = FeedReply::parse(&CStr::from_ptr(data).to_string_lossy());
            if self.feed_cow {
                if let Some(release) = self.feed_reply_dtor {
                    release(reply_slot);
                }
            }
            reply
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
        let Some(set) = self.set_hdr_info else {
            return false;
        };
        let payload = c_string(payload);
        unsafe { set(self.this(), payload.as_ptr()) }
    }

    pub fn flush(&self) -> bool {
        unsafe { (self.flush)(self.this()) }
    }

    pub fn push_eos(&self) -> bool {
        unsafe { (self.push_eos)(self.this()) }
    }

    pub fn set_time_to_decode(&self, payload: &str) -> bool {
        let Some(set) = self.set_time_to_decode else {
            return false;
        };
        let payload = c_string(payload);
        unsafe { set(self.this(), payload.as_ptr()) }
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

unsafe fn cow_string_dtor() -> Option<Dtor> {
    let global = libloading::os::unix::Library::this();
    unsafe { global.get::<Dtor>(COW_STRING_DTOR.as_bytes()) }.ok().map(|symbol| *symbol)
}

const SYMBOLS: [&str; 16] = [
    CTOR,
    DTOR,
    LOAD,
    "_ZN17StarfishMediaAPIs4LoadEPKcPFvixS1_E",
    FEED,
    "_ZN17StarfishMediaAPIs4FeedEPKc",
    PLAY,
    PAUSE,
    UNLOAD,
    NOTIFY_FOREGROUND,
    SEEK,
    HDR_INFO,
    MEDIA_ID,
    FLUSH,
    PUSH_EOS,
    TIME_TO_DECODE,
];

const EXTRA_SYMBOLS: [&str; 2] = [PLAY_RATE, CHANGE_CODEC];

pub fn probe_symbols() -> Vec<(String, bool)> {
    let library = match library() {
        Ok(library) => library,
        Err(_) => return vec![(LIBRARY.to_owned(), false)],
    };
    let mut found = vec![(LIBRARY.to_owned(), true)];
    for name in SYMBOLS.iter().chain(EXTRA_SYMBOLS.iter()) {
        let present = unsafe { library.get::<*const c_void>(name.as_bytes()) }.is_ok();
        found.push(((*name).to_owned(), present));
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
    fn bound_names_exist_in_firmware_dumps() {
        let known = include_str!("../tests/fixtures/starfish-symbols.txt");
        for name in SYMBOLS.iter().chain(EXTRA_SYMBOLS.iter()) {
            assert!(known.lines().any(|line| line == *name), "{name} is in no firmware dump");
        }
    }

    #[test]
    fn interior_nul_is_dropped() {
        assert_eq!(c_string("a\0b").to_bytes(), b"ab");
    }
}
