use fluxa_effects::{SessionHandle, Storage};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn settle(session: &SessionHandle) {
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut quiet = 0;
    while quiet < 3 {
        assert!(Instant::now() < deadline, "session did not settle");
        std::thread::sleep(Duration::from_millis(50));
        if session.has_queued_dispatches() || session.has_outstanding_effects() {
            quiet = 0;
        } else {
            quiet += 1;
        }
    }
}

fn resolve(video_id: &str, max_height: u32) -> Value {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("live-trailer-{video_id}-{max_height}"));
    let _ = std::fs::remove_dir_all(&dir);
    let session = SessionHandle::open(Storage::open(dir).expect("storage")).expect("session");
    session
        .dispatch(json!({
            "type": "trailerResolveRequested",
            "requestId": "live",
            "videoId": video_id,
            "maxHeight": max_height
        }))
        .expect("dispatch");
    settle(&session);
    session
        .snapshot()
        .pointer("/trailer/resolutions/live")
        .cloned()
        .unwrap_or(Value::Null)
}

#[test]
#[ignore = "hits live YouTube"]
fn live_trailers_resolve_to_1080p_with_separate_audio() {
    for id in ["dQw4w9WgXcQ"] {
        let resolution = resolve(id, 1080);
        println!("RES {resolution}");
        assert!(resolution["streamUrl"].as_str().is_some_and(|url| url.starts_with("https://")), "{id}: {resolution}");
        assert!(resolution["audioUrl"].as_str().is_some(), "{id}: {resolution}");
    }
}
