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
fn primitive_player_updates_mark_only_the_player_domain_dirty() {
    let handle = create_headless_engine("{}");
    assert!(headless_engine_set_player_buffering(handle, false));
    assert!(headless_engine_set_player_stream_index(handle, 3));
    assert!(headless_engine_set_player_position(handle, 12_345));
    let response = headless_engine_snapshot_json(handle).unwrap();
    assert!(response.contains(r#"isBuffering":false"#));
    assert!(response.contains(r#"currentStreamIndex":3"#));
    assert!(response.contains(r#"lastPositionMs":12345"#));
    assert!(destroy_headless_engine(handle));
}
