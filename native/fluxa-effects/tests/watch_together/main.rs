mod fake;

use fluxa_effects::{SessionHandle, Storage};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static SERVER: OnceLock<()> = OnceLock::new();
static APPS: AtomicUsize = AtomicUsize::new(0);

fn server() {
    SERVER.get_or_init(|| {
        let url = option_env!("FLUXA_SYNC_URL")
            .expect("run with FLUXA_SYNC_URL=http://127.0.0.1:<port>/functions/v1/fluxa-sync (npm run check:watch-party)");
        let port = url
            .rsplit_once(':')
            .and_then(|(_, rest)| rest.split('/').next())
            .and_then(|port| port.parse().ok())
            .expect("FLUXA_SYNC_URL must be a local http url with a port");
        unsafe {
            std::env::set_var("NO_PROXY", "127.0.0.1,localhost");
            std::env::set_var("no_proxy", "127.0.0.1,localhost");
        }
        fake::serve(port);
    });
}

struct Peer {
    session: SessionHandle,
}

fn peer() -> Peer {
    server();
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "watch-{}-{}",
        std::process::id(),
        APPS.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    Peer {
        session: SessionHandle::open(Storage::open(dir).expect("storage")).expect("session"),
    }
}

impl Peer {
    fn send(&self, action: Value) {
        self.session.dispatch(action).expect("dispatch");
    }

    fn watch(&self) -> Value {
        self.session
            .snapshot()
            .get("watchTogether")
            .cloned()
            .unwrap_or(Value::Null)
    }

    fn wait(&self, what: &str, check: impl Fn(&Value) -> bool) -> Value {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let state = self.watch();
            if check(&state) {
                return state;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for {what}: {state}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn command(&self, what: &str, name: &str) -> Value {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(command) = self
                .session
                .take_watch_commands()
                .into_iter()
                .find(|command| command["command"] == name)
            {
                return command;
            }
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

fn members(count: usize) -> impl Fn(&Value) -> bool {
    move |state| state["members"].as_array().is_some_and(|m| m.len() == count)
}

fn room_with_guest() -> (Peer, Peer, String) {
    let host = peer();
    let guest = peer();
    host.send(json!({"type": "watchTogetherCreateRequested", "displayName": "Host"}));
    let code = host
        .wait("host room", |s| s["roomCode"].is_string())["roomCode"]
        .as_str()
        .unwrap()
        .to_owned();
    guest.send(json!({"type": "watchTogetherJoinRequested", "code": code, "displayName": "Guest"}));
    guest.wait("guest room", |s| s["roomCode"].is_string());
    host.wait("host sees guest", members(2));
    guest.wait("guest sees host", members(2));
    (host, guest, code)
}

fn start_movie(host: &Peer, guest: &Peer) {
    host.send(json!({
        "type": "watchTogetherContentSelected",
        "content": {"id": "tt0111161", "contentType": "movie", "title": "Film"},
        "startPositionMs": 0,
    }));
    guest.wait("guest content", |s| s["content"]["id"] == "tt0111161");
    guest.command("stream picker", "pickStream");
    host.send(json!({"type": "watchTogetherReadyChanged", "ready": true, "nowMs": 1}));
    guest.send(json!({"type": "watchTogetherReadyChanged", "ready": true, "nowMs": 1}));
    host.wait("host playing", |s| s["phase"] == "playing");
    guest.wait("guest playing", |s| s["phase"] == "playing");
}

#[test]
fn guest_joins_and_both_start_together() {
    let (host, guest, code) = room_with_guest();
    assert_eq!(guest.watch()["roomCode"], code.as_str());
    assert_eq!(guest.watch()["role"], "GUEST");
    assert_eq!(host.watch()["role"], "HOST");
    start_movie(&host, &guest);
    guest.command("start", "start");
}

#[test]
fn guest_follows_the_hosts_position() {
    let (host, guest, _) = room_with_guest();
    start_movie(&host, &guest);
    host.send(json!({
        "type": "watchTogetherPlaybackReported",
        "positionMs": 600_000, "durationMs": 7_000_000,
        "playing": true, "buffering": false, "nowMs": 5_000,
    }));
    let deadline = Instant::now() + Duration::from_secs(15);
    let seek = loop {
        guest.send(json!({
            "type": "watchTogetherPositionSampled",
            "positionMs": 1_000, "nowMs": 5_100, "speedCorrectionActive": false,
        }));
        std::thread::sleep(Duration::from_millis(100));
        if let Some(seek) = guest
            .session
            .take_watch_commands()
            .into_iter()
            .find(|command| command["command"] == "seek")
        {
            break seek;
        }
        assert!(Instant::now() < deadline, "guest never asked to seek");
    };
    assert!(seek["positionMs"].as_i64().unwrap() >= 590_000);
}

#[test]
fn guest_pause_is_seen_by_the_host() {
    let (host, guest, _) = room_with_guest();
    start_movie(&host, &guest);
    guest.send(json!({"type": "watchTogetherControlRequested", "kind": "paused", "nowMs": 10}));
    let state = host.wait("host sees the pause", |s| s["lastEvent"]["kind"] == "paused");
    assert_eq!(state["lastEvent"]["name"], "Guest");
}

#[test]
fn chat_reaches_the_other_side() {
    let (host, guest, _) = room_with_guest();
    host.send(json!({"type": "watchTogetherChatSent", "text": "ready?", "nowMs": 1}));
    let state = guest.wait("chat", |s| s["chat"].as_array().is_some_and(|c| !c.is_empty()));
    assert_eq!(state["chat"][0]["text"], "ready?");
    assert_eq!(state["chat"][0]["name"], "Host");
}

#[test]
fn locked_room_refuses_joins_and_kicked_guest_stays_out() {
    let (host, guest, code) = room_with_guest();
    let guest_id = guest.watch()["clientId"].as_str().unwrap().to_owned();

    host.send(json!({"type": "watchTogetherLockChanged", "locked": true}));
    let late = peer();
    std::thread::sleep(Duration::from_millis(300));
    late.send(json!({"type": "watchTogetherJoinRequested", "code": code, "displayName": "Late"}));
    late.wait("late join refused", |s| s["connectionState"] == "ERROR");

    host.send(json!({"type": "watchTogetherKickRequested", "memberId": guest_id}));
    host.wait("guest removed", members(1));
    guest.wait("guest notices", |s| s["roomCode"].is_null());

    host.send(json!({"type": "watchTogetherLockChanged", "locked": false}));
    std::thread::sleep(Duration::from_millis(300));
    guest.send(json!({"type": "watchTogetherJoinRequested", "code": code, "displayName": "Guest"}));
    guest.wait("banned guest refused", |s| s["connectionState"] == "ERROR");
}

#[test]
fn ending_the_room_sends_guests_home() {
    let (host, guest, _) = room_with_guest();
    host.send(json!({"type": "watchTogetherEndRequested"}));
    guest.wait("guest leaves", |s| s["roomCode"].is_null());
    assert!(host.watch()["roomCode"].is_null());
}
