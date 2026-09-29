use super::*;

fn content() -> WatchTogetherContent {
    WatchTogetherContent {
        id: "tt123".to_string(),
        content_type: "series".to_string(),
        video_id: Some("tt123:1:2".to_string()),
        title: "Episode 2".to_string(),
    }
}

#[test]
fn protocol_matches_existing_server_wire_shape() {
    assert_eq!(
        WatchTogetherProtocol::create("Alice"),
        json!({"type": "create", "name": "Alice"})
    );
    assert_eq!(
        WatchTogetherProtocol::join("ABC234", "Alice"),
        json!({"type": "join", "room": "ABC234", "name": "Alice"})
    );
    assert_eq!(
        WatchTogetherProtocol::content(&content())["contentId"],
        "tt123"
    );
}

#[test]
fn playback_state_round_trips_content_and_snapshot() {
    let snapshot = WatchTogetherPlaybackSnapshot {
        position_ms: 12_000,
        duration_ms: 30_000,
        is_playing: true,
        is_buffering: false,
    };
    let value = WatchTogetherProtocol::playback_state(&snapshot, Some(&content()));
    assert_eq!(WatchTogetherProtocol::message_type(&value), Some("state"));
    assert_eq!(WatchTogetherProtocol::position_ms(&value), Some(12_000));
    assert_eq!(WatchTogetherProtocol::playing(&value), Some(true));
    assert_eq!(WatchTogetherProtocol::content_from(&value), Some(content()));
}

#[test]
fn drift_policy_is_shared_across_platforms() {
    assert_eq!(
        WatchTogetherDriftPolicy::correction(1_000, 1_100, true, false),
        WatchTogetherCorrection::None
    );
    assert_eq!(
        WatchTogetherDriftPolicy::correction(1_000, 1_600, true, false),
        WatchTogetherCorrection::Speed(1.03)
    );
    assert_eq!(
        WatchTogetherDriftPolicy::correction(1_000, 2_500, true, true),
        WatchTogetherCorrection::Seek(2_500)
    );
    assert_eq!(
        WatchTogetherDriftPolicy::correction(1_000, 1_100, true, true),
        WatchTogetherCorrection::ResetSpeed
    );
}

#[test]
fn relayed_sync_and_peer_state_decode_the_same() {
    let snapshot = WatchTogetherPlaybackSnapshot {
        position_ms: 12_000,
        duration_ms: 30_000,
        is_playing: true,
        is_buffering: false,
    };
    let peer = WatchTogetherProtocol::playback_state(&snapshot, Some(&content()));
    let relayed = json!({
        "type": "sync", "sequence": 4, "senderId": "abc", "serverTimeMs": 1_700_000_000_000i64,
        "positionMs": 12_000, "durationMs": 30_000, "playing": true, "buffering": false,
        "contentId": "tt123", "contentType": "series", "videoId": "tt123:1:2", "title": "Episode 2",
    });

    let from_peer = WatchTogetherProtocol::decode(&peer, Some(&content()));
    let from_relay = WatchTogetherProtocol::decode(&relayed, Some(&content()));

    assert_eq!(from_peer.kind, WatchTogetherIncomingKind::Sync);
    assert_eq!(from_relay.kind, WatchTogetherIncomingKind::Sync);
    assert_eq!(from_peer.position_ms, from_relay.position_ms);
    assert_eq!(from_peer.playing, from_relay.playing);
    assert_eq!(from_peer.content, from_relay.content);
    assert!(from_peer.content_matches_local);
    assert_eq!(from_peer.sequence, None);
    assert_eq!(from_relay.sequence, Some(4));
}

#[test]
fn room_message_flags_only_the_host_member() {
    let decoded = WatchTogetherProtocol::decode(
        &json!({
            "type": "room", "room": "ABC234", "clientId": "b2", "hostId": "a1",
            "members": [
                {"id": "a1", "name": "Alice", "buffering": false},
                {"id": "b2", "name": "Bob", "buffering": true},
                {"name": "ghost"},
            ],
        }),
        None,
    );

    assert_eq!(decoded.kind, WatchTogetherIncomingKind::Room);
    assert_eq!(decoded.room_code.as_deref(), Some("ABC234"));
    assert_eq!(decoded.members.len(), 2);
    assert!(decoded.members.iter().filter(|m| m.is_host).count() == 1);
    assert!(decoded.members.iter().any(|m| m.id == "a1" && m.is_host));
    assert!(decoded.members.iter().any(|m| m.id == "b2" && m.buffering));
}

#[test]
fn sync_for_a_different_episode_is_flagged_as_mismatched() {
    let decoded = WatchTogetherProtocol::decode(
        &json!({"type": "sync", "contentId": "tt123", "videoId": "tt123:1:9"}),
        Some(&content()),
    );
    assert!(!decoded.content_matches_local);
}

#[test]
fn content_matching_preserves_episode_identity() {
    let same = content();
    assert!(same.matches(&content()));
    assert!(!same.matches(&WatchTogetherContent {
        video_id: Some("tt123:1:3".to_string()),
        ..content()
    }));
}
