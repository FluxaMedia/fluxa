use super::*;
use serde_json::{Value, json};

fn member(id: &str) -> WatchTogetherMember {
    WatchTogetherMember {
        id: id.to_string(),
        name: id.to_string(),
        is_host: id == "host",
        buffering: false,
        ready: false,
        avatar: None,
        color: None,
        status: WatchTogetherMemberStatus::Waiting,
    }
}

fn members(ids: &[&str]) -> Vec<WatchTogetherMember> {
    ids.iter().map(|id| member(id)).collect()
}

fn session(id: &str, ids: &[&str]) -> WatchTogetherSession {
    WatchTogetherSession::new(
        "ABC234".to_string(),
        id.to_string(),
        "host".to_string(),
        members(ids),
    )
}

fn episode() -> WatchTogetherContent {
    WatchTogetherContent {
        id: "tt123".to_string(),
        content_type: "series".to_string(),
        video_id: Some("tt123:1:2".to_string()),
        title: "Episode 2".to_string(),
        artwork: None,
    }
}

fn stream() -> SharedStream {
    SharedStream {
        url: "https://cdn.example/video.mkv".to_string(),
        title: "1080p".to_string(),
        headers: BTreeMap::new(),
    }
}

fn broadcasts(effects: &[SessionEffect]) -> Vec<Value> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            SessionEffect::Broadcast(value) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

fn deliver(
    target: &mut WatchTogetherSession,
    effects: &[SessionEffect],
    now: i64,
) -> Vec<SessionEffect> {
    broadcasts(effects)
        .iter()
        .flat_map(|message| target.handle(message, now))
        .collect()
}

#[test]
fn guests_without_a_shared_stream_are_sent_to_the_stream_picker() {
    let mut host = session("host", &["host", "ana"]);
    let mut ana = session("ana", &["host", "ana"]);
    let sent = host.set_content(episode(), None, 0);
    assert_eq!(
        deliver(&mut ana, &sent, 0),
        vec![SessionEffect::PickStream(episode())]
    );
    assert_eq!(ana.phase(), SessionPhase::Selecting);
}

#[test]
fn shared_stream_is_played_without_a_picker() {
    let mut host = session("host", &["host", "ana"]);
    let mut ana = session("ana", &["host", "ana"]);
    let sent = host.set_content(episode(), Some(stream()), 0);
    assert_eq!(
        deliver(&mut ana, &sent, 0),
        vec![SessionEffect::PlaySharedStream {
            content: episode(),
            stream: stream()
        }]
    );
}

#[test]
fn room_starts_only_after_everyone_is_ready() {
    let ids = ["host", "ana", "bo"];
    let mut host = session("host", &ids);
    let mut ana = session("ana", &ids);
    let mut bo = session("bo", &ids);
    let sent = host.set_content(episode(), None, 90_000);
    deliver(&mut ana, &sent, 0);
    deliver(&mut bo, &sent, 0);

    let ana_ready = ana.local_ready(true, None, 10);
    assert!(deliver(&mut host, &ana_ready, 10).is_empty());
    let bo_ready = bo.local_ready(true, None, 20);
    assert!(deliver(&mut host, &bo_ready, 20).is_empty());
    assert_eq!(host.phase(), SessionPhase::Selecting);

    let host_ready = host.local_ready(true, None, 30);
    assert!(host_ready.contains(&SessionEffect::Start {
        position_ms: 90_000
    }));
    assert_eq!(host.phase(), SessionPhase::Playing);
    assert_eq!(
        deliver(&mut ana, &host_ready, 30),
        vec![SessionEffect::Start {
            position_ms: 90_000
        }]
    );
    assert_eq!(
        deliver(&mut bo, &host_ready, 30),
        vec![SessionEffect::Start {
            position_ms: 90_000
        }]
    );
}

#[test]
fn guest_leaving_while_selecting_lets_the_rest_start() {
    let ids = ["host", "ana", "bo"];
    let mut host = session("host", &ids);
    let mut ana = session("ana", &ids);
    let sent = host.set_content(episode(), None, 0);
    deliver(&mut ana, &sent, 0);
    host.local_ready(true, None, 0);
    let ready = ana.local_ready(true, None, 0);
    deliver(&mut host, &ready, 0);
    assert_eq!(host.phase(), SessionPhase::Selecting);

    let effects = host.members_changed("host".to_string(), members(&["host", "ana"]), 5);
    assert!(effects.contains(&SessionEffect::Start { position_ms: 0 }));
}

#[test]
fn only_the_host_can_change_content_or_playback() {
    let mut ana = session("ana", &["host", "ana", "bo"]);
    let forged =
        json!({"type": "content", "from": "bo", "contentId": "other", "contentType": "movie"});
    assert!(ana.handle(&forged, 0).is_empty());
    assert!(ana.content().is_none());
    let state =
        json!({"type": "state", "from": "bo", "sequence": 1, "positionMs": 5, "playing": true});
    assert!(ana.handle(&state, 0).is_empty());
}

#[test]
fn guest_buffering_pauses_everyone_until_it_clears() {
    let ids = ["host", "ana"];
    let mut host = session("host", &ids);
    let mut ana = session("ana", &ids);
    let sent = host.set_content(episode(), None, 0);
    deliver(&mut ana, &sent, 0);
    let ready = ana.local_ready(true, None, 0);
    deliver(&mut host, &ready, 0);
    let started = host.local_ready(true, None, 0);
    deliver(&mut ana, &started, 0);

    let stall = ana.local_buffering(true, 4_000);
    let paused = deliver(&mut host, &stall, 4_000);
    assert!(paused.contains(&SessionEffect::Remote {
        playing: false,
        position_ms: 4_000,
        rate: 1.0
    }));
    assert_eq!(
        deliver(&mut ana, &paused, 4_000),
        vec![SessionEffect::Remote {
            playing: false,
            position_ms: 4_000,
            rate: 1.0
        }]
    );

    let recover = ana.local_buffering(false, 6_000);
    let resumed = deliver(&mut host, &recover, 6_000);
    assert!(resumed.contains(&SessionEffect::Remote {
        playing: true,
        position_ms: 4_000,
        rate: 1.0
    }));
}

#[test]
fn late_joiner_catches_up_to_the_hosts_position() {
    let mut host = session("host", &["host"]);
    host.set_content(episode(), None, 0);
    host.local_ready(true, None, 0);
    host.host_playback(
        &WatchTogetherPlaybackSnapshot {
            position_ms: 60_000,
            duration_ms: 3_000_000,
            is_playing: true,
            is_buffering: false,
            rate: 1.0,
        },
        1_000,
    );

    let mut cy = session("cy", &["host", "cy"]);
    let announce = host.members_changed("host".to_string(), members(&["host", "cy"]), 2_000);
    assert_eq!(
        deliver(&mut cy, &announce, 2_000),
        vec![SessionEffect::PickStream(episode())]
    );
    let ready = cy.local_ready(true, None, 3_000);
    let reply = deliver(&mut host, &ready, 3_000);
    assert_eq!(
        deliver(&mut cy, &reply, 3_000),
        vec![SessionEffect::Start {
            position_ms: 62_000
        }]
    );
    assert_eq!(cy.phase(), SessionPhase::Playing);
}

#[test]
fn guest_correction_follows_the_hosts_clock() {
    let ids = ["host", "ana"];
    let mut ana = session("ana", &ids);
    ana.handle(
        &json!({"type": "content", "from": "host", "contentId": "tt123"}),
        0,
    );
    ana.local_ready(true, None, 0);
    ana.handle(
        &json!({"type": "start", "from": "host", "positionMs": 10_000}),
        1_000,
    );
    assert_eq!(
        ana.correction(10_000, 3_000, false),
        WatchTogetherCorrection::Seek(12_000)
    );
    assert_eq!(
        ana.correction(12_000, 3_000, false),
        WatchTogetherCorrection::None
    );
}

#[test]
fn roster_exposes_ready_and_buffering_per_member() {
    let ids = ["host", "ana"];
    let mut host = session("host", &ids);
    let mut ana = session("ana", &ids);
    let sent = host.set_content(episode(), None, 0);
    deliver(&mut ana, &sent, 0);
    let ready = ana.local_ready(true, None, 0);
    deliver(&mut host, &ready, 0);
    let roster = host.roster();
    assert!(roster.iter().find(|m| m.id == "ana").unwrap().ready);
    assert!(!roster.iter().find(|m| m.id == "host").unwrap().ready);

    let started = host.local_ready(true, None, 0);
    deliver(&mut ana, &started, 0);
    let stall = ana.local_buffering(true, 100);
    deliver(&mut host, &stall, 100);
    let waiting: Vec<_> = host
        .roster()
        .into_iter()
        .filter(|m| m.buffering)
        .map(|m| m.id)
        .collect();
    assert_eq!(waiting, vec!["ana".to_string()]);
}

#[test]
fn shared_stream_with_a_non_web_url_is_ignored() {
    let mut guest = session("guest", &["host", "guest"]);
    let mut message = json!({
        "type": "content",
        "from": "host",
        "contentId": "tt1",
        "stream": {"url": "file:///etc/passwd"},
    });
    let effects = guest.handle(&message, 0);
    assert!(matches!(effects.as_slice(), [SessionEffect::PickStream(_)]));
    message["stream"]["url"] = json!("https://cdn.example/video.mkv");
    let effects = guest.handle(&message, 0);
    assert!(matches!(
        effects.as_slice(),
        [SessionEffect::PlaySharedStream { .. }]
    ));
}

#[test]
fn oversized_content_is_dropped() {
    let mut guest = session("guest", &["host", "guest"]);
    let message = json!({"type": "content", "from": "host", "contentId": "x".repeat(500)});
    assert!(guest.handle(&message, 0).is_empty());
}

#[test]
fn guest_on_another_episode_is_not_marked_ready() {
    let ids = ["host", "ana"];
    let mut host = session("host", &ids);
    let mut ana = session("ana", &ids);
    let sent = host.set_content(episode(), None, 0);
    deliver(&mut ana, &sent, 0);

    let other = WatchTogetherContent {
        video_id: Some("tt123:1:3".to_string()),
        ..episode()
    };
    assert!(ana.local_ready(true, Some(&other), 0).is_empty());
    assert!(ana.content_mismatch());
    assert!(!ana.is_ready("ana"));

    let right = ana.local_ready(true, Some(&episode()), 1);
    assert!(!right.is_empty());
    assert!(!ana.content_mismatch());
    assert!(ana.is_ready("ana"));
}

#[test]
fn another_show_or_season_is_a_mismatch() {
    let base = episode();
    let other_show = WatchTogetherContent {
        id: "tt999".to_string(),
        video_id: Some("tt999:1:2".to_string()),
        ..episode()
    };
    let other_season = WatchTogetherContent {
        video_id: Some("tt123:2:2".to_string()),
        ..episode()
    };
    let no_episode = WatchTogetherContent {
        video_id: None,
        ..episode()
    };
    assert!(base.is_same_episode(&episode()));
    assert!(!base.is_same_episode(&other_show));
    assert!(!base.is_same_episode(&other_season));
    assert!(!base.is_same_episode(&no_episode));
}

fn snapshot(position_ms: i64, is_playing: bool, rate: f64) -> WatchTogetherPlaybackSnapshot {
    WatchTogetherPlaybackSnapshot {
        position_ms,
        duration_ms: 3_000_000,
        is_playing,
        is_buffering: false,
        rate,
    }
}

fn started(ids: &[&str]) -> (WatchTogetherSession, WatchTogetherSession) {
    let mut host = session("host", ids);
    let mut ana = session("ana", ids);
    let sent = host.set_content(episode(), None, 0);
    deliver(&mut ana, &sent, 0);
    let ready = ana.local_ready(true, None, 0);
    deliver(&mut host, &ready, 0);
    let start = host.local_ready(true, None, 0);
    deliver(&mut ana, &start, 0);
    host.host_playback(&snapshot(0, true, 1.0), 0);
    (host, ana)
}

#[test]
fn someone_joining_does_not_restart_guests_already_watching() {
    let (mut host, mut ana) = started(&["host", "ana"]);
    let announce = host.members_changed("host".to_string(), members(&["host", "ana", "cy"]), 5_000);
    assert!(deliver(&mut ana, &announce, 5_000).is_empty());
    assert_eq!(ana.phase(), SessionPhase::Playing);

    let mut cy = session("cy", &["host", "ana", "cy"]);
    assert_eq!(
        deliver(&mut cy, &announce, 5_000),
        vec![SessionEffect::PickStream(episode())]
    );
}

#[test]
fn party_resumes_after_buffering_even_though_the_host_was_paused_by_it() {
    let (mut host, mut ana) = started(&["host", "ana"]);
    let stall = ana.local_buffering(true, 4_000);
    deliver(&mut host, &stall, 4_000);
    assert!(host.host_playback(&snapshot(4_000, false, 1.0), 4_100).is_empty());

    let recover = ana.local_buffering(false, 6_000);
    let resumed = deliver(&mut host, &recover, 6_000);
    assert!(resumed.contains(&SessionEffect::Remote {
        playing: true,
        position_ms: 4_000,
        rate: 1.0
    }));
}

#[test]
fn guest_pause_reaches_the_host_and_everyone_sees_who_did_it() {
    let ids = ["host", "ana", "bo"];
    let mut host = session("host", &ids);
    let mut ana = session("ana", &ids);
    let mut bo = session("bo", &ids);
    let sent = host.set_content(episode(), None, 0);
    for guest in [&mut ana, &mut bo] {
        deliver(guest, &sent, 0);
        let ready = guest.local_ready(true, None, 0);
        deliver(&mut host, &ready, 0);
    }
    let start = host.local_ready(true, None, 0);
    deliver(&mut ana, &start, 0);
    deliver(&mut bo, &start, 0);
    host.host_playback(&snapshot(0, true, 1.0), 0);

    let pause = ana.local_control(WatchTogetherEventKind::Paused, 3_000);
    let reply = deliver(&mut host, &pause, 3_000);
    assert!(reply.contains(&SessionEffect::Remote {
        playing: false,
        position_ms: 3_000,
        rate: 1.0
    }));
    assert_eq!(host.last_event().unwrap().name, "ana");

    assert_eq!(
        deliver(&mut bo, &reply, 3_000),
        vec![SessionEffect::Remote {
            playing: false,
            position_ms: 3_000,
            rate: 1.0
        }]
    );
    let seen = bo.last_event().unwrap();
    assert_eq!(seen.kind, WatchTogetherEventKind::Paused);
    assert_eq!(seen.name, "ana");
    assert!(deliver(&mut ana, &reply, 3_000).len() == 1);
    assert!(ana.last_event().is_none());
}

#[test]
fn locked_room_ignores_guest_controls() {
    let (mut host, mut ana) = started(&["host", "ana"]);
    host.set_locked(true);
    let seek = ana.local_control(WatchTogetherEventKind::Seeked { position_ms: 60_000 }, 1_000);
    assert!(deliver(&mut host, &seek, 1_000).is_empty());
    ana.set_locked(true);
    assert!(ana.local_control(WatchTogetherEventKind::Paused, 1_000).is_empty());
}

#[test]
fn host_speed_change_is_shared_and_drift_uses_it() {
    let (mut host, mut ana) = started(&["host", "ana"]);
    let sent = host.host_playback(&snapshot(1_000, true, 1.5), 1_000);
    assert_eq!(
        deliver(&mut ana, &sent, 1_000),
        vec![SessionEffect::Remote {
            playing: true,
            position_ms: 1_000,
            rate: 1.5
        }]
    );
    assert_eq!(
        ana.last_event().unwrap().kind,
        WatchTogetherEventKind::Speed { rate: 1.5 }
    );
    assert_eq!(
        ana.correction(4_000, 3_000, false),
        WatchTogetherCorrection::None
    );
}

#[test]
fn roster_reports_what_each_member_is_doing() {
    use WatchTogetherMemberStatus as Status;
    let ids = ["host", "ana"];
    let mut host = session("host", &ids);
    let status = |session: &WatchTogetherSession, id: &str| {
        session.roster().into_iter().find(|m| m.id == id).unwrap().status
    };
    assert_eq!(status(&host, "ana"), Status::Waiting);
    host.set_content(episode(), None, 0);
    assert_eq!(status(&host, "ana"), Status::Picking);
    let (mut host, mut ana) = started(&ids);
    assert_eq!(status(&host, "ana"), Status::Watching);
    host.host_playback(&snapshot(500, false, 1.0), 500);
    assert_eq!(status(&host, "ana"), Status::Paused);
    let stall = ana.local_buffering(true, 600);
    deliver(&mut host, &stall, 600);
    assert_eq!(status(&host, "ana"), Status::Buffering);
}
