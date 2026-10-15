use std::ffi::{CStr, CString, c_char, c_int, c_void};

use libloading::Library;

use crate::remote::{RawKey, decode};

const LIBRARIES: [&str; 2] = ["libSDL2-2.0.so.0", "libSDL2.so"];
const INIT_VIDEO: u32 = 0x20;
const WINDOW_SHOWN: u32 = 0x4;
const WINDOW_FULLSCREEN_DESKTOP: u32 = 0x1001;
const WINDOW_RESIZABLE: u32 = 0x20;
const SYSWM_WAYLAND: u32 = 6;
const EVENT_SIZE: usize = 128;

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Quit,
    Resized(u32, u32),
    Key(RawKey),
    Text(String),
    MouseMoved(f32, f32),
    MouseButton { button: u8, pressed: bool, x: f32, y: f32 },
    Wheel(f32),
}

#[repr(C, align(8))]
struct Raw([u8; EVENT_SIZE]);

pub struct Sdl {
    library: Library,
    window: *mut c_void,
}

pub struct Handles {
    pub display: *mut c_void,
    pub surface: *mut c_void,
}

fn word(raw: &[u8], at: usize) -> i32 {
    raw.get(at..at + 4).map_or(0, |bytes| i32::from_ne_bytes(bytes.try_into().unwrap()))
}

pub fn parse(raw: &[u8]) -> Option<Event> {
    let kind = word(raw, 0) as u32;
    Some(match kind {
        0x100 => Event::Quit,
        0x200 => match raw.get(12).copied()? {
            6 | 5 => Event::Resized(word(raw, 16).max(1) as u32, word(raw, 20).max(1) as u32),
            14 => Event::Quit,
            _ => return None,
        },
        0x300 | 0x301 => Event::Key(decode(raw)?),
        0x303 => {
            let text = raw.get(12..12 + 32)?;
            let end = text.iter().position(|byte| *byte == 0).unwrap_or(text.len());
            Event::Text(String::from_utf8_lossy(&text[..end]).into_owned())
        }
        0x400 => Event::MouseMoved(word(raw, 20) as f32, word(raw, 24) as f32),
        0x401 | 0x402 => Event::MouseButton {
            button: *raw.get(16)?,
            pressed: kind == 0x401,
            x: word(raw, 20) as f32,
            y: word(raw, 24) as f32,
        },
        0x403 => Event::Wheel(word(raw, 20) as f32),
        _ => return None,
    })
}

impl Sdl {
    pub fn open(title: &str, width: u32, height: u32, fullscreen: bool) -> Result<Sdl, String> {
        let library = LIBRARIES
            .iter()
            .find_map(|name| unsafe { Library::new(name) }.ok())
            .ok_or("libSDL2 not found")?;
        unsafe {
            let init: unsafe extern "C" fn(u32) -> c_int = *library.get(b"SDL_Init\0").map_err(|e| e.to_string())?;
            if init(INIT_VIDEO) != 0 {
                return Err(error(&library));
            }
            let create: unsafe extern "C" fn(*const c_char, c_int, c_int, c_int, c_int, u32) -> *mut c_void =
                *library.get(b"SDL_CreateWindow\0").map_err(|e| e.to_string())?;
            let title = CString::new(title).unwrap_or_default();
            let flags = WINDOW_SHOWN
                | WINDOW_RESIZABLE
                | if fullscreen { WINDOW_FULLSCREEN_DESKTOP } else { 0 };
            let window = create(title.as_ptr(), 0x1FFF0000, 0x1FFF0000, width as c_int, height as c_int, flags);
            if window.is_null() {
                return Err(error(&library));
            }
            Ok(Sdl { library, window })
        }
    }

    pub fn size(&self) -> (u32, u32) {
        let (mut width, mut height) = (0, 0);
        unsafe {
            if let Ok(get) = self.library.get::<unsafe extern "C" fn(*mut c_void, *mut c_int, *mut c_int)>(b"SDL_GetWindowSize\0") {
                get(self.window, &mut width, &mut height);
            }
        }
        (width.max(1) as u32, height.max(1) as u32)
    }

    pub fn wayland_handles(&self) -> Result<Handles, String> {
        let mut info = [0u8; 256];
        info[0] = 2;
        info[1] = 0;
        info[2] = 32;
        unsafe {
            let get: unsafe extern "C" fn(*mut c_void, *mut u8) -> c_int =
                *self.library.get(b"SDL_GetWindowWMInfo\0").map_err(|e| e.to_string())?;
            if get(self.window, info.as_mut_ptr()) == 0 {
                return Err(error(&self.library));
            }
        }
        let subsystem = u32::from_ne_bytes(info[4..8].try_into().unwrap());
        if subsystem != SYSWM_WAYLAND {
            return Err(format!("window system {subsystem} is not wayland"));
        }
        let pointer = std::mem::size_of::<usize>();
        let read = |at: usize| usize::from_ne_bytes(info[at..at + pointer].try_into().unwrap()) as *mut c_void;
        Ok(Handles { display: read(8), surface: read(8 + pointer) })
    }

    pub fn poll(&self, timeout_ms: i32) -> Option<Event> {
        let mut raw = Raw([0; EVENT_SIZE]);
        loop {
            let got = unsafe {
                let wait: unsafe extern "C" fn(*mut u8, c_int) -> c_int = *self.library.get(b"SDL_WaitEventTimeout\0").ok()?;
                wait(raw.0.as_mut_ptr(), timeout_ms)
            };
            if got == 0 {
                return None;
            }
            if let Some(event) = parse(&raw.0) {
                return Some(event);
            }
            raw.0 = [0; EVENT_SIZE];
        }
    }

    pub fn exported_window(&self) -> Option<String> {
        unsafe {
            let get: unsafe extern "C" fn(*mut c_void) -> *const c_char =
                *self.library.get(b"SDL_webOSGetExportedWindow\0").ok()?;
            let name = get(self.window);
            (!name.is_null()).then(|| CStr::from_ptr(name).to_string_lossy().into_owned())
        }
    }
}

fn error(library: &Library) -> String {
    unsafe {
        let get: Result<libloading::Symbol<unsafe extern "C" fn() -> *const c_char>, _> = library.get(b"SDL_GetError\0");
        get.map(|get| CStr::from_ptr(get()).to_string_lossy().into_owned()).unwrap_or_default()
    }
}

impl Drop for Sdl {
    fn drop(&mut self) {
        unsafe {
            if let Ok(destroy) = self.library.get::<unsafe extern "C" fn(*mut c_void)>(b"SDL_DestroyWindow\0") {
                destroy(self.window);
            }
            if let Ok(quit) = self.library.get::<unsafe extern "C" fn()>(b"SDL_Quit\0") {
                quit();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(kind: u32, fill: impl Fn(&mut [u8])) -> Vec<u8> {
        let mut raw = vec![0u8; 56];
        raw[0..4].copy_from_slice(&kind.to_ne_bytes());
        fill(&mut raw);
        raw
    }

    #[test]
    fn window_size_change_reports_new_dimensions() {
        let event = raw(0x200, |raw| {
            raw[12] = 6;
            raw[16..20].copy_from_slice(&1280i32.to_ne_bytes());
            raw[20..24].copy_from_slice(&720i32.to_ne_bytes());
        });
        assert_eq!(parse(&event), Some(Event::Resized(1280, 720)));
    }

    #[test]
    fn mouse_button_up_is_not_pressed() {
        let event = raw(0x402, |raw| {
            raw[16] = 1;
            raw[20..24].copy_from_slice(&10i32.to_ne_bytes());
            raw[24..28].copy_from_slice(&20i32.to_ne_bytes());
        });
        assert_eq!(parse(&event), Some(Event::MouseButton { button: 1, pressed: false, x: 10.0, y: 20.0 }));
    }

    #[test]
    fn text_input_stops_at_the_nul() {
        let event = raw(0x303, |raw| raw[12..14].copy_from_slice(b"ab"));
        assert_eq!(parse(&event), Some(Event::Text("ab".into())));
    }

    #[test]
    fn unknown_events_are_skipped() {
        assert_eq!(parse(&raw(0x7fff, |_| {})), None);
    }
}
