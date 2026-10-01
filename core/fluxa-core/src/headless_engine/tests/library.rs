use super::super::*;







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
