use fluxa_effects::{SessionHandle, Storage};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn open(name: &str) -> SessionHandle {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("live-watch-{name}"));
    SessionHandle::open(Storage::open(dir).expect("storage")).expect("session")
}

fn wait(session: &SessionHandle, what: &str, check: impl Fn(&Value) -> bool) -> Value {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let state = session
            .snapshot()
            .get("watchTogether")
            .cloned()
            .unwrap_or(Value::Null);
        if check(&state) {
            return state;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {what}: {state}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
#[ignore = "talks to the live Supabase project"]
fn host_and_guest_start_together() {
    let host = open("host");
    let guest = open("guest");

    host.dispatch(json!({"type": "watchTogetherCreateRequested", "displayName": "Host"}))
        .unwrap();
    let state = wait(&host, "host room", |s| s["roomCode"].is_string());
    let code = state["roomCode"].as_str().unwrap().to_owned();

    guest
        .dispatch(
            json!({"type": "watchTogetherJoinRequested", "code": code, "displayName": "Guest"}),
        )
        .unwrap();
    wait(&guest, "guest room", |s| s["roomCode"].is_string());
    wait(&host, "host sees guest", |s| {
        s["members"].as_array().is_some_and(|m| m.len() == 2)
    });

    host.dispatch(json!({
        "type": "watchTogetherContentSelected",
        "content": {"id": "tt0111161", "contentType": "movie", "title": "Test"},
        "startPositionMs": 0,
    }))
    .unwrap();
    wait(&guest, "guest content", |s| {
        s["content"]["id"] == "tt0111161"
    });

    host.dispatch(json!({"type": "watchTogetherReadyChanged", "ready": true, "nowMs": 1}))
        .unwrap();
    guest
        .dispatch(json!({"type": "watchTogetherReadyChanged", "ready": true, "nowMs": 1}))
        .unwrap();
    wait(&host, "playing", |s| s["phase"] == "playing");
    wait(&guest, "playing", |s| s["phase"] == "playing");

    host.dispatch(json!({"type": "watchTogetherLeaveRequested"}))
        .unwrap();
    guest
        .dispatch(json!({"type": "watchTogetherLeaveRequested"}))
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));
}

#[test]
#[ignore = "talks to the live Supabase project"]
fn host_can_lock_and_remove() {
    let host = open("admin-host");
    let guest = open("admin-guest");
    let late = open("admin-late");

    host.dispatch(json!({"type": "watchTogetherCreateRequested", "displayName": "Host"}))
        .unwrap();
    let state = wait(&host, "host room", |s| s["roomCode"].is_string());
    let code = state["roomCode"].as_str().unwrap().to_owned();

    guest
        .dispatch(
            json!({"type": "watchTogetherJoinRequested", "code": code, "displayName": "Guest"}),
        )
        .unwrap();
    let state = wait(&guest, "guest room", |s| s["roomCode"].is_string());
    let guest_id = state["clientId"].as_str().unwrap().to_owned();
    wait(&host, "host sees guest", |s| {
        s["members"].as_array().is_some_and(|m| m.len() == 2)
    });

    host.dispatch(json!({"type": "watchTogetherLockChanged", "locked": true}))
        .unwrap();
    std::thread::sleep(Duration::from_secs(1));
    late.dispatch(
        json!({"type": "watchTogetherJoinRequested", "code": code, "displayName": "Late"}),
    )
    .unwrap();
    wait(&late, "late join refused", |s| {
        s["connectionState"] == "ERROR"
    });

    host.dispatch(json!({"type": "watchTogetherKickRequested", "memberId": guest_id}))
        .unwrap();
    wait(&host, "guest removed", |s| {
        s["members"].as_array().is_some_and(|m| m.len() == 1)
    });
    wait(&guest, "guest notices", |s| s["roomCode"].is_null());

    host.dispatch(json!({"type": "watchTogetherLockChanged", "locked": false}))
        .unwrap();
    std::thread::sleep(Duration::from_secs(1));
    guest
        .dispatch(
            json!({"type": "watchTogetherJoinRequested", "code": code, "displayName": "Guest"}),
        )
        .unwrap();
    wait(&guest, "banned guest refused", |s| {
        s["connectionState"] == "ERROR"
    });

    host.dispatch(json!({"type": "watchTogetherLeaveRequested"}))
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));
}
