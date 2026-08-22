use super::super::*;
use serde_json::{Value, json};

#[test]
fn library_commands_are_storage_effects_owned_by_core() {
    let handle = create_headless_engine(r#"{"profile":{"activeProfileId":"p1"}}"#);
    let requested: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"toggleWatchlistRequested","profile":{"id":"p2"},"item":{"id":"tt1","name":"Movie","type":"movie"}}"#,
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(requested["effects"][0]["type"], "writeLibraryCommand");
    assert_eq!(requested["effects"][0]["payload"]["profileId"], "p2");
    assert_eq!(
        requested["effects"][0]["payload"]["command"]["type"],
        "toggleWatchlist"
    );
    assert_eq!(
        requested["effects"][0]["payload"]["command"]["item"]["id"],
        "tt1"
    );

    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": requested["effects"][0]["id"].as_str().unwrap(),
                "status": "ok",
                "value": { "watchlist": [{ "id": "tt1" }] }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert!(completed["state"]["library"]["lastWriteError"].is_null());
    assert_eq!(
        completed["state"]["library"]["lastWrite"]["watchlist"][0]["id"],
        "tt1"
    );
    assert!(destroy_headless_engine(handle));
}

#[test]
fn playback_progress_write_is_clamped_and_delegated_to_storage_adapter() {
    let handle = create_headless_engine(r#"{"profile":{"activeProfileId":"p1"}}"#);
    let requested: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"savePlaybackProgressRequested","meta":{"id":"tt1","name":"Movie","type":"movie"},"timeOffset":-10,"duration":7200,"lastVideoId":"tt1","lastStreamIndex":2,"lastEpisodeName":null,"lastStreamUrl":"http://a","lastStreamTitle":"A","lastAudioLanguage":"en","lastSubtitleLanguage":"tr"}"#,
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(requested["effects"][0]["type"], "writePlaybackProgress");
    assert_eq!(requested["effects"][0]["payload"]["profileId"], "p1");
    assert_eq!(
        requested["effects"][0]["payload"]["progress"]["timeOffset"],
        0
    );
    assert_eq!(
        requested["effects"][0]["payload"]["progress"]["lastSubtitleLanguage"],
        "tr"
    );

    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": requested["effects"][0]["id"].as_str().unwrap(),
                "status": "ok",
                "value": {}
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert!(completed["state"]["library"]["pendingPlaybackProgress"].is_null());
    assert_eq!(
        completed["state"]["library"]["savedPlaybackProgress"]["meta"]["id"],
        "tt1"
    );
    assert!(destroy_headless_engine(handle));
}

#[test]
fn clearing_playback_progress_drops_the_item_from_home_continue_watching() {
    let handle = create_headless_engine(r#"{"profile":{"activeProfileId":"p1"}}"#);
    headless_engine_dispatch_json(
        handle,
        r#"{"type":"homeLoadRequested","profile":{"id":"p1"}}"#,
    )
    .unwrap();
    let home_loaded: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": "fx-1",
                "status": "ok",
                "value": { "continueWatching": [{ "id": "tt1" }, { "id": "tt2" }] }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        home_loaded["state"]["home"]["continueWatching"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let requested: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"clearPlaybackProgressRequested","meta":{"id":"tt1"}}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let effect_id = requested["effects"][0]["id"].as_str().unwrap();

    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": { "droppedId": "tt1" }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    let continue_watching = completed["state"]["home"]["continueWatching"]
        .as_array()
        .unwrap();
    assert_eq!(continue_watching.len(), 1);
    assert_eq!(continue_watching[0]["id"], "tt2");
    assert!(destroy_headless_engine(handle));
}

#[test]
fn completing_an_effect_does_not_redeliver_already_delivered_siblings() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"detailLoadRequested","contentType":"movie","id":"tt1","language":"en"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    // dispatch_load creates and delivers both effects directly in one response.
    assert_eq!(requested["effects"][0]["type"], "fetchMetaDetail");
    assert_eq!(requested["effects"][1]["type"], "readPlaybackProgress");

    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": requested["effects"][0]["id"].as_str().unwrap(),
                "status": "ok",
                "value": { "meta": { "id": "tt1", "name": "Movie" } }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    // readPlaybackProgress was already handed to the platform alongside fetchMetaDetail.
    // Completing fetchMetaDetail must not hand it out again as if it were fresh work —
    // the platform is presumably still executing it.
    assert!(completed["effects"].as_array().unwrap().is_empty());
    assert!(destroy_headless_engine(handle));
}

#[test]
fn expire_stale_pending_effects_drops_old_but_not_recent_effects() {
    let mut engine = HeadlessEngine::default();
    let action: AppAction = serde_json::from_str(
        r#"{"type":"detailLoadRequested","contentType":"movie","id":"tt1","language":"en"}"#,
    )
    .unwrap();
    let effects = engine.dispatch(action);
    let visible = engine.resolve_visible_effects(effects);
    assert_eq!(visible.len(), 2);

    // Still well within the window — nothing genuinely in flight should be dropped.
    engine.expire_stale_pending_effects(Instant::now());
    assert_eq!(engine.pending_effects.len(), 2);

    // Past the expiry window — abandoned effects (platform never called
    // complete_effect) get swept from all three bookkeeping collections.
    let far_future = Instant::now() + Duration::from_secs(301);
    engine.expire_stale_pending_effects(far_future);
    assert!(engine.pending_effects.is_empty());
    assert!(engine.delivered_effect_ids.is_empty());
    assert!(engine.effect_created_at.is_empty());
}
