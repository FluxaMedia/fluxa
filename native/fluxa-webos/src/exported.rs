use std::ffi::{CString, c_char, c_int};

use libloading::Library;

use crate::diagnostics::trace;

const LIBRARIES: [&str; 2] = ["libSDL2-2.0.so.0", "libSDL2.so"];

#[repr(C)]
struct Rect {
    x: c_int,
    y: c_int,
    w: c_int,
    h: c_int,
}

type SetExportedWindow = unsafe extern "C" fn(*const c_char, *mut Rect, *mut Rect) -> c_int;

pub struct ExportedWindow {
    id: CString,
    set: SetExportedWindow,
    _library: Library,
}

impl ExportedWindow {
    pub fn open(id: &str) -> Option<ExportedWindow> {
        let id = CString::new(id).ok()?;
        let library = LIBRARIES.iter().find_map(|name| unsafe { Library::new(*name) }.ok())?;
        let set = *unsafe { library.get::<SetExportedWindow>(b"SDL_webOSSetExportedWindow\0") }.ok()?;
        Some(ExportedWindow { id, set, _library: library })
    }

    pub fn place(&self, frame: (i32, i32), dst: [i32; 4]) {
        let mut src = Rect { x: 0, y: 0, w: frame.0, h: frame.1 };
        let mut dst = Rect { x: dst[0], y: dst[1], w: dst[2], h: dst[3] };
        if unsafe { (self.set)(self.id.as_ptr(), &mut src, &mut dst) } == 0 {
            trace("SDL_webOSSetExportedWindow rejected");
        }
    }
}
