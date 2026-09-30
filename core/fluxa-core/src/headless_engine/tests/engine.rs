use super::super::*;

#[test]
fn engines_lock_survives_a_panic_while_held_by_another_thread() {
    // Poison the lock the same way a caught panic in a request would: a
    // thread panics while still holding the guard.
    let poisoner = std::thread::spawn(|| {
        let _guard = engines().lock().unwrap();
        panic!("simulated panic while holding the engines lock");
    });
    assert!(poisoner.join().is_err());

    // A naive `.lock().ok()` would now return None forever; lock_engines
    // must recover the guard so the store keeps working.
    let handle = create_headless_engine("{}");
    assert!(handle > 0);
    assert!(headless_engine_snapshot_json(handle).is_some());
    assert!(destroy_headless_engine(handle));
}

#[test]
fn poisoned_engine_handle_is_rejected_without_affecting_other_handles() {
    let poisoned_handle = create_headless_engine("{}");
    let healthy_handle = create_headless_engine("{}");
    let poisoned_engine = lock_engines().get(&poisoned_handle).unwrap().clone();
    let poisoner = std::thread::spawn(move || {
        let _guard = poisoned_engine.lock().unwrap();
        panic!("simulated engine update panic");
    });
    assert!(poisoner.join().is_err());

    assert!(headless_engine_snapshot_json(poisoned_handle).is_none());
    assert!(headless_engine_snapshot_json(healthy_handle).is_some());
    assert!(destroy_headless_engine(poisoned_handle));
    assert!(destroy_headless_engine(healthy_handle));
}

#[test]
fn player_reset_and_telemetry_use_the_headless_core_state() {
    let handle = create_headless_engine("{}");
    headless_engine_dispatch_json(
        handle,
        r#"{"type":"playerResetForEpisode","videoId":"tt123:1:2"}"#,
    )
    .unwrap();
    let reset =
        serde_json::from_str::<serde_json::Value>(&headless_engine_snapshot_json(handle).unwrap())
            .unwrap();
    assert_eq!(reset["player"]["currentVideoId"], "tt123:1:2");
    assert_eq!(reset["player"]["currentStreamIndex"], 0);

    headless_engine_dispatch_json(
        handle,
        r#"{"type":"playerTelemetryUpdated","positionMs":12345,"streamIndex":2,"buffering":false,"playbackEnded":true,"started":true,"rendered":true}"#,
    )
    .unwrap();
    let telemetry =
        serde_json::from_str::<serde_json::Value>(&headless_engine_snapshot_json(handle).unwrap())
            .unwrap();
    assert_eq!(telemetry["player"]["lastPositionMs"], 12345);
    assert_eq!(telemetry["player"]["currentStreamIndex"], 2);
    assert_eq!(telemetry["player"]["isBuffering"], false);
    assert_eq!(telemetry["player"]["playbackEnded"], true);
    assert_eq!(telemetry["player"]["hasStartedPlaying"], true);
    assert_eq!(telemetry["player"]["isVideoRendered"], true);
    assert!(destroy_headless_engine(handle));
}
