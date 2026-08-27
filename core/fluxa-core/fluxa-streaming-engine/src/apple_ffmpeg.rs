use std::ffi::c_char;
use std::ffi::CString;
use std::os::raw::c_void;
use tokio::sync::mpsc;

type WriteCallback = unsafe extern "C" fn(*mut c_void, *const u8, i32) -> i32;

pub(crate) enum RemuxMessage {
    Chunk(Vec<u8>),
    Finished(i32),
}

unsafe extern "C" {
    fn fluxa_ffmpeg_remux_url(
        url: *const c_char,
        headers: *const c_char,
        start_microseconds: i64,
        callback: WriteCallback,
        opaque: *mut c_void,
    ) -> i32;
}

// Keep the FFmpeg shim symbol reachable from the final Apple static library.
// The playback path will call this through the streaming callback once the
// per-target archive is linked by build-rust-core-xcframework.sh.
#[used]
static FLUXA_FFMPEG_REMUX_SYMBOL: unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    i64,
    WriteCallback,
    *mut c_void,
) -> i32 = fluxa_ffmpeg_remux_url;

unsafe extern "C" fn write_chunk(opaque: *mut c_void, data: *const u8, size: i32) -> i32 {
    if opaque.is_null() || data.is_null() || size <= 0 {
        return -1;
    }
    let sender = unsafe { &*(opaque as *const mpsc::Sender<RemuxMessage>) };
    let bytes = unsafe { std::slice::from_raw_parts(data, size as usize) }.to_vec();
    sender
        .blocking_send(RemuxMessage::Chunk(bytes))
        .map(|_| size)
        .unwrap_or(-1)
}

pub(crate) fn start_remux(
    url: String,
    headers: String,
    start_microseconds: i64,
) -> mpsc::Receiver<RemuxMessage> {
    let (sender, receiver) = mpsc::channel(4);
    tokio::task::spawn_blocking(move || {
        let Ok(url) = CString::new(url) else {
            let _ = sender.blocking_send(RemuxMessage::Finished(-22));
            return;
        };
        let Ok(headers) = CString::new(headers) else {
            let _ = sender.blocking_send(RemuxMessage::Finished(-22));
            return;
        };
        let result = unsafe {
            fluxa_ffmpeg_remux_url(
                url.as_ptr(),
                headers.as_ptr(),
                start_microseconds,
                write_chunk,
                &sender as *const _ as *mut c_void,
            )
        };
        let _ = sender.blocking_send(RemuxMessage::Finished(result));
    });
    receiver
}
