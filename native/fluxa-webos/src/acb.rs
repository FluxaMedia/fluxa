use std::ffi::{c_char, c_int, c_long};

use libloading::Library;

use crate::diagnostics::trace;
use crate::ffi::{Error, c_string, symbol};

const LIBRARY: &str = "libAcbAPI.so.1";
const PLAYER_TYPE_MSE: c_int = 10;
const SINK_TYPE_MAIN: c_int = 0;
const APPSTATE_FOREGROUND: c_int = 1;
const PLAYSTATE_UNLOADED: c_int = 0;
const PLAYSTATE_LOADED: c_int = 1;
const PLAYSTATE_PLAYING: c_int = 2;
const PLAYSTATE_PAUSED: c_int = 3;

type Callback = extern "C" fn(c_long, c_long, c_long, c_long, c_long, *const c_char);
type Create = unsafe extern "C" fn() -> c_long;
type Initialize = unsafe extern "C" fn(c_long, c_int, *const c_char, Callback) -> bool;
type SetText = unsafe extern "C" fn(c_long, *const c_char) -> bool;
type SetSink = unsafe extern "C" fn(c_long, c_int) -> bool;
type SetData = unsafe extern "C" fn(c_long, *const c_char, *mut c_long) -> c_int;
type SetState = unsafe extern "C" fn(c_long, c_int, c_int, *mut c_long) -> c_int;
type SetWindow =
    unsafe extern "C" fn(c_long, c_long, c_long, c_long, c_long, bool, *mut c_long) -> c_int;
type SetCustomWindow = unsafe extern "C" fn(
    c_long,
    c_long,
    c_long,
    c_long,
    c_long,
    c_long,
    c_long,
    c_long,
    c_long,
    bool,
    *mut c_long,
) -> c_int;
type Finalize = unsafe extern "C" fn(c_long) -> c_int;
type Destroy = unsafe extern "C" fn(c_long);

extern "C" fn on_event(
    id: c_long,
    task: c_long,
    kind: c_long,
    app: c_long,
    play: c_long,
    _reply: *const c_char,
) {
    trace(format!(
        "acb {id} task={task} event={kind} app={app} play={play}"
    ));
}

pub struct Acb {
    id: c_long,
    set_media_id: SetText,
    set_sink: SetSink,
    set_video: SetData,
    set_audio: Option<SetData>,
    set_state: SetState,
    set_window: SetWindow,
    set_custom_window: Option<SetCustomWindow>,
    finalize: Finalize,
    destroy: Destroy,
    _library: Library,
}

impl Acb {
    pub fn open(app_id: &str) -> Result<Acb, Error> {
        let library =
            unsafe { Library::new(LIBRARY) }.map_err(|error| Error::Library(error.to_string()))?;
        let (create, initialize): (Create, Initialize) = unsafe {
            (
                symbol(&library, "AcbAPI_create\0")?,
                symbol(&library, "AcbAPI_initialize\0")?,
            )
        };
        let id = unsafe { create() };
        if id == 0 {
            return Err(Error::Rejected("AcbAPI_create"));
        }
        let app = c_string(app_id);
        if !unsafe { initialize(id, PLAYER_TYPE_MSE, app.as_ptr(), on_event) } {
            return Err(Error::Rejected("AcbAPI_initialize"));
        }
        unsafe {
            Ok(Acb {
                id,
                set_media_id: symbol(&library, "AcbAPI_setMediaId\0")?,
                set_sink: symbol(&library, "AcbAPI_setSinkType\0")?,
                set_video: symbol(&library, "AcbAPI_setMediaVideoData\0")?,
                set_audio: symbol(&library, "AcbAPI_setMediaAudioData\0").ok(),
                set_state: symbol(&library, "AcbAPI_setState\0")?,
                set_window: symbol(&library, "AcbAPI_setDisplayWindow\0")?,
                set_custom_window: symbol(&library, "AcbAPI_setCustomDisplayWindow\0").ok(),
                finalize: symbol(&library, "AcbAPI_finalize\0")?,
                destroy: symbol(&library, "AcbAPI_destroy\0")?,
                _library: library,
            })
        }
    }

    fn state(&self, play: c_int) {
        let mut task = 0;
        let result = unsafe { (self.set_state)(self.id, APPSTATE_FOREGROUND, play, &mut task) };
        if result == 0 {
            trace(format!("acb setState {play} rejected"));
        }
    }

    pub fn bind(&self, media_id: &str) {
        let media = c_string(media_id);
        unsafe {
            (self.set_sink)(self.id, SINK_TYPE_MAIN);
            (self.set_media_id)(self.id, media.as_ptr());
        }
        self.state(PLAYSTATE_LOADED);
    }

    pub fn video_data(&self, payload: &str) {
        let payload = c_string(payload);
        let mut task = 0;
        if unsafe { (self.set_video)(self.id, payload.as_ptr(), &mut task) } == 0 {
            trace("acb setMediaVideoData rejected");
        }
    }

    pub fn audio_data(&self, payload: &str) {
        let Some(set) = self.set_audio else {
            return;
        };
        let payload = c_string(payload);
        let mut task = 0;
        if unsafe { set(self.id, payload.as_ptr(), &mut task) } == 0 {
            trace("acb setMediaAudioData rejected");
        }
    }

    pub fn playing(&self, width: c_long, height: c_long) {
        let mut task = 0;
        unsafe { (self.set_window)(self.id, 0, 0, width, height, true, &mut task) };
        self.state(PLAYSTATE_PLAYING);
    }

    pub fn place(&self, frame: (i32, i32), dst: [i32; 4]) {
        let Some(set) = self.set_custom_window else {
            return;
        };
        let mut task = 0;
        let [x, y, w, h] = dst.map(c_long::from);
        let (fw, fh) = (c_long::from(frame.0), c_long::from(frame.1));
        if unsafe { set(self.id, 0, 0, fw, fh, x, y, w, h, false, &mut task) } == 0 {
            trace("acb setCustomDisplayWindow rejected");
        }
    }

    pub fn start(&self) {
        self.state(PLAYSTATE_PLAYING);
    }

    pub fn paused(&self) {
        self.state(PLAYSTATE_PAUSED);
    }

    pub fn unloaded(&self) {
        self.state(PLAYSTATE_UNLOADED);
    }
}

impl Drop for Acb {
    fn drop(&mut self) {
        unsafe {
            (self.finalize)(self.id);
            (self.destroy)(self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_library_is_an_error_not_a_panic() {
        assert!(matches!(Acb::open("app"), Err(Error::Library(_))));
    }
}
