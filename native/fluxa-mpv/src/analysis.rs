use super::*;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

const FRAME_SAMPLES: u32 = 800;
const SAMPLE_RATE: u32 = 16_000;
const DEADLINE: Duration = Duration::from_secs(90);

#[derive(Clone, Debug, PartialEq)]
pub struct SpeechEnergy {
    pub start: f64,
    pub frame_seconds: f64,
    pub energies: Vec<f64>,
}

pub fn analyze_speech(url: &str, start: f64, length: f64) -> Receiver<Option<SpeechEnergy>> {
    let (sender, receiver) = channel();
    let url = url.to_owned();
    std::thread::spawn(move || {
        let _ = sender.send(run(&url, start, length));
    });
    receiver
}

fn run(url: &str, start: f64, length: f64) -> Option<SpeechEnergy> {
    let output = std::env::temp_dir().join(format!("fluxa-speech-{}.txt", std::process::id()));
    let escaped = output
        .to_string_lossy()
        .replace('\\', "/")
        .replace(':', "\\:");
    let graph = format!(
        "lavfi=[aformat=sample_rates={SAMPLE_RATE}:channel_layouts=mono,asetnsamples=n={FRAME_SAMPLES}:p=0,astats=metadata=1:reset=1,ametadata=mode=print:key=lavfi.astats.Overall.RMS_level:file={escaped}]"
    );
    let _ = std::fs::remove_file(&output);
    let api = MpvApi::load().ok()?;
    let handle = unsafe { (api.mpv_create)() };
    if handle.is_null() {
        return None;
    }
    let set = |name: &str, value: &str| {
        let (name, value) = (CString::new(name).ok()?, CString::new(value).ok()?);
        let result = unsafe { (api.mpv_set_option_string)(handle, name.as_ptr(), value.as_ptr()) };
        (result >= 0).then_some(())
    };
    let configured = (|| {
        set("terminal", "no")?;
        set("config", "no")?;
        set("vid", "no")?;
        set("sid", "no")?;
        set("ao", "null")?;
        set("ao-null-untimed", "yes")?;
        set("audio-display", "no")?;
        set("start", &format!("{start:.1}"))?;
        set("length", &format!("{length:.1}"))?;
        set("af", &graph)?;
        (unsafe { (api.mpv_initialize)(handle) } >= 0).then_some(())
    })();
    let url_c = CString::new(url).ok()?;
    let command = [c"loadfile".as_ptr(), url_c.as_ptr(), ptr::null()];
    let finished = configured.is_some()
        && unsafe { (api.mpv_command)(handle, command.as_ptr()) } >= 0
        && wait_for_end(&api, handle);
    unsafe { (api.mpv_terminate_destroy)(handle) };
    let text = std::fs::read_to_string(&output).ok();
    let _ = std::fs::remove_file(&output);
    if !finished {
        return None;
    }
    let energies = text?
        .lines()
        .filter_map(|line| line.split_once("RMS_level="))
        .map(|(_, value)| match value.trim().parse::<f64>() {
            Ok(db) if db.is_finite() => 10f64.powf(db / 20.0),
            _ => 0.0,
        })
        .collect::<Vec<_>>();
    Some(SpeechEnergy {
        start,
        frame_seconds: f64::from(FRAME_SAMPLES) / f64::from(SAMPLE_RATE),
        energies,
    })
}

fn wait_for_end(api: &MpvApi, handle: *mut MpvHandle) -> bool {
    let started = Instant::now();
    while started.elapsed() < DEADLINE {
        let event = unsafe { &*(api.mpv_wait_event)(handle, 0.25) };
        if event.event_id == MPV_EVENT_END_FILE {
            let data = event.data as *const MpvEventEndFile;
            return !data.is_null() && unsafe { (*data).reason } == MPV_END_FILE_REASON_EOF;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "needs libmpv and FLUXA_TEST_AUDIO"]
    fn measures_tone_bursts() {
        let path = std::env::var("FLUXA_TEST_AUDIO").expect("FLUXA_TEST_AUDIO");
        let result = analyze_speech(&path, 0.0, 60.0)
            .recv()
            .expect("channel")
            .expect("analysis");
        assert!(result.energies.len() > 1000);
        let loud = result.energies.iter().filter(|e| **e > 0.1).count();
        let quiet = result.energies.iter().filter(|e| **e < 0.01).count();
        assert!(loud > 100 && quiet > 300, "loud {loud} quiet {quiet}");
    }
}
