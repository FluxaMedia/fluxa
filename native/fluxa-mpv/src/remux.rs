use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RemuxProgress {
    pub seconds: f64,
    pub duration: Option<f64>,
    pub bytes: u64,
}

pub fn remux(
    url: &str,
    headers: &[(String, String)],
    output: &std::path::Path,
    cancel: &AtomicBool,
    mut progress: impl FnMut(RemuxProgress),
) -> Result<(), String> {
    let api = MpvApi::load()?;
    let handle = unsafe { (api.mpv_create)() };
    if handle.is_null() {
        return Err("mpv_create failed".to_owned());
    }
    let result = run(&api, handle, url, headers, output, cancel, &mut progress);
    unsafe { (api.mpv_terminate_destroy)(handle) };
    result
}

fn run(
    api: &MpvApi,
    handle: *mut MpvHandle,
    url: &str,
    headers: &[(String, String)],
    output: &std::path::Path,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(RemuxProgress),
) -> Result<(), String> {
    let set = |name: &str, value: &str| {
        let (name_c, value_c) = (
            CString::new(name).map_err(|error| error.to_string())?,
            CString::new(value).map_err(|error| error.to_string())?,
        );
        let code = unsafe { (api.mpv_set_option_string)(handle, name_c.as_ptr(), value_c.as_ptr()) };
        if code < 0 {
            return Err(format!("{name}: {}", api.error_string(code)));
        }
        Ok(())
    };
    set("terminal", "no")?;
    set("config", "no")?;
    set("load-scripts", "no")?;
    set("ytdl", "no")?;
    set("sid", "no")?;
    set("vo", "null")?;
    set("ao", "null")?;
    set("ao-null-untimed", "yes")?;
    set("untimed", "yes")?;
    set("stream-record", &output.to_string_lossy())?;
    if !headers.is_empty() {
        let fields = headers
            .iter()
            .map(|(name, value)| format!("{name}: {value}"))
            .collect::<Vec<_>>()
            .join(",");
        set("http-header-fields", &fields)?;
    }
    let code = unsafe { (api.mpv_initialize)(handle) };
    if code < 0 {
        return Err(api.error_string(code));
    }
    let url_c = CString::new(url).map_err(|error| error.to_string())?;
    let command = [c"loadfile".as_ptr(), url_c.as_ptr(), ptr::null()];
    let code = unsafe { (api.mpv_command)(handle, command.as_ptr()) };
    if code < 0 {
        return Err(api.error_string(code));
    }
    let property = |name: &str| -> Option<f64> {
        let name = CString::new(name).ok()?;
        let mut value = 0.0f64;
        let code = unsafe {
            (api.mpv_get_property)(handle, name.as_ptr(), 5, (&mut value as *mut f64).cast())
        };
        (code >= 0).then_some(value)
    };
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".to_owned());
        }
        let event = unsafe { &*(api.mpv_wait_event)(handle, 0.5) };
        if event.event_id == MPV_EVENT_END_FILE {
            let data = event.data as *const MpvEventEndFile;
            if data.is_null() {
                return Err("remux_failed".to_owned());
            }
            let (reason, error) = unsafe { ((*data).reason, (*data).error) };
            return if reason == MPV_END_FILE_REASON_EOF {
                Ok(())
            } else {
                Err(api.error_string(error))
            };
        }
        progress(RemuxProgress {
            seconds: property("time-pos")
                .unwrap_or(0.0)
                .max(property("demuxer-cache-time").unwrap_or(0.0)),
            duration: property("duration").filter(|duration| *duration > 0.0),
            bytes: std::fs::metadata(output).map(|meta| meta.len()).unwrap_or(0),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "needs libmpv and FLUXA_TEST_HLS"]
    fn remuxes_a_playlist_with_the_bundled_libs() {
        let url = std::env::var("FLUXA_TEST_HLS").expect("FLUXA_TEST_HLS");
        let output = std::env::temp_dir().join("fluxa-remux-test.mkv");
        let cancel = AtomicBool::new(false);
        remux(&url, &[], &output, &cancel, |_| {}).expect("remux");
        assert!(std::fs::metadata(&output).unwrap().len() > 0);
        let _ = std::fs::remove_file(output);
    }
}
