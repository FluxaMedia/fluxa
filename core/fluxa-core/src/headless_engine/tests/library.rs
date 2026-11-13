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

mod addon_events {
    use super::super::super::*;
    use crate::runtime::EffectKind;
    use serde_json::{Value, json};

    fn engine_with_tracker(consent: bool) -> HeadlessEngine {
        let mut engine = HeadlessEngine::new(Default::default());
        let mut tracker = json!({
            "transportUrl": "https://tracker.example.com/manifest.json",
            "manifest": {
                "id": "example.tracker",
                "resources": ["library", "player"],
                "types": ["movie", "series"],
                "idPrefixes": ["tt"]
            },
            "trackingConsent": consent
        });
        tracker["enabled"] = json!(true);
        engine.state.addons.installed = json!([tracker]);
        engine
    }

    fn act(engine: &mut HeadlessEngine, action: Value) -> Vec<crate::runtime::EffectEnvelope> {
        engine
            .apply(serde_json::from_value(action).unwrap())
            .effects
    }

    #[test]
    fn season_is_split_into_chunks_of_one_hundred() {
        let mut engine = engine_with_tracker(true);
        let video_ids: Vec<String> = (1..=150).map(|n| format!("tt1:1:{n}")).collect();
        let effects = act(
            &mut engine,
            json!({
                "type": "markWatchedRequested", "seriesId": "tt1",
                "videoIds": video_ids, "watched": true,
                "meta": {"id": "tt1", "type": "series"},
            }),
        );
        let emitted: Vec<_> = effects
            .iter()
            .filter(|effect| effect.kind == EffectKind::EmitAddonEvent)
            .collect();
        assert_eq!(emitted.len(), 1);
        let requests = emitted[0].payload["requests"].as_array().unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0]["extra"]["action"], "watched");
        let first = requests[0]["extra"]["videoId"].as_str().unwrap();
        assert_eq!(first.split(',').count(), 100);
    }

    #[test]
    fn movie_is_marked_without_video_ids() {
        let mut engine = engine_with_tracker(true);
        let effects = act(
            &mut engine,
            json!({
                "type": "markWatchedRequested", "seriesId": "tt1",
                "videoIds": ["tt1"], "watched": false,
                "meta": {"id": "tt1", "type": "movie"},
            }),
        );
        let emitted = effects
            .iter()
            .find(|effect| effect.kind == EffectKind::EmitAddonEvent)
            .unwrap();
        let extra = &emitted.payload["requests"][0]["extra"];
        assert_eq!(extra["action"], "unwatched");
        assert!(extra.get("videoId").is_none());
    }

    #[test]
    fn nothing_is_sent_without_consent() {
        let mut engine = engine_with_tracker(false);
        let effects = act(
            &mut engine,
            json!({
                "type": "markWatchedRequested", "seriesId": "tt1",
                "videoIds": ["tt1"], "watched": true,
                "meta": {"id": "tt1", "type": "movie"},
            }),
        );
        assert!(
            effects
                .iter()
                .all(|effect| effect.kind != EffectKind::EmitAddonEvent)
        );
    }

    #[test]
    fn player_events_carry_milliseconds_and_skip_mid_play_stop() {
        let mut engine = engine_with_tracker(true);
        let scrobble = |action: &str, closing: bool| {
            json!({
                "type": "scrobbleRequested", "token": "", "metaType": "movie",
                "itemId": "tt1", "progress": 10.0, "actionName": action,
                "positionMs": 6000, "durationMs": 60000, "closing": closing,
            })
        };
        let emitted = |effects: &[crate::runtime::EffectEnvelope]| {
            effects
                .iter()
                .filter(|effect| effect.kind == EffectKind::EmitAddonEvent)
                .count()
        };
        let start = act(&mut engine, scrobble("start", false));
        assert_eq!(emitted(&start), 1);
        let request = &start
            .iter()
            .find(|effect| effect.kind == EffectKind::EmitAddonEvent)
            .unwrap()
            .payload["requests"][0];
        assert_eq!(request["extra"]["currentTime"], "6000");
        assert_eq!(request["extra"]["duration"], "60000");
        assert_eq!(emitted(&act(&mut engine, scrobble("stop", false))), 0);
        assert_eq!(emitted(&act(&mut engine, scrobble("stop", true))), 1);
    }
}
