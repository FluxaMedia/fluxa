use crate::local_stream::{start_local_stream_server, stop_local_stream_server};
use crate::torrent_engine::{start_torrent_server, stop_torrent_server};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

fn read_string(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(value) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

fn write_string(value: Option<String>) -> *mut c_char {
    value
        .and_then(|value| CString::new(value).ok())
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn fluxa_streaming_start_local_stream_server(
    target_url: *const c_char,
    headers_json: *const c_char,
    preferred_port: i32,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let target_url = read_string(target_url)?;
        let headers_json = read_string(headers_json)?;
        start_local_stream_server(&target_url, &headers_json, preferred_port)
    })
    .ok()
    .flatten()
    .map(Some)
    .map(write_string)
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn fluxa_streaming_stop_local_stream_server(server_id: *const c_char) -> bool {
    std::panic::catch_unwind(|| {
        read_string(server_id)
            .map(|server_id| stop_local_stream_server(&server_id))
            .unwrap_or(false)
    })
    .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn fluxa_streaming_start_torrent_server(
    cache_dir: *const c_char,
    preferred_port: i32,
    access_token: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let cache_dir = read_string(cache_dir)?;
        let access_token = read_string(access_token).unwrap_or_default();
        start_torrent_server(&cache_dir, preferred_port, &access_token)
    })
    .ok()
    .flatten()
    .map(Some)
    .map(write_string)
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn fluxa_streaming_stop_torrent_server() -> bool {
    std::panic::catch_unwind(|| stop_torrent_server(None)).unwrap_or(false)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_streaming_string_free(value: *mut c_char) {
    if !value.is_null() {
        let _ = unsafe { CString::from_raw(value) };
    }
}

#[cfg(test)]
mod tests {
    use super::{
        fluxa_streaming_start_local_stream_server, fluxa_streaming_stop_local_stream_server,
        fluxa_streaming_string_free,
    };
    use std::ffi::{CStr, CString};

    #[test]
    #[cfg_attr(
        target_os = "linux",
        ignore = "Linux sandbox may deny loopback listener creation"
    )]
    fn local_proxy_lifecycle_is_available_through_the_apple_bridge() {
        let target_url = CString::new("https://example.invalid/video.mp4").unwrap();
        let headers = CString::new("{}").unwrap();
        let result =
            fluxa_streaming_start_local_stream_server(target_url.as_ptr(), headers.as_ptr(), 0);
        assert!(!result.is_null());
        let response = unsafe { CStr::from_ptr(result) }
            .to_str()
            .unwrap()
            .to_string();
        unsafe { fluxa_streaming_string_free(result) };
        let id = serde_json::from_str::<serde_json::Value>(&response)
            .unwrap()
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap()
            .to_string();
        let id = CString::new(id).unwrap();
        assert!(fluxa_streaming_stop_local_stream_server(id.as_ptr()));
    }
}
