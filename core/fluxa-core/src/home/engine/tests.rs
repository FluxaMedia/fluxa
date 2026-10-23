use super::{add_continue_watching_episode_labels, dispatch_load};
use crate::headless_engine::HeadlessEngine;
use crate::headless_engine::contracts::{EffectResultInput, EffectStatus};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde_json::json;

#[test]
fn continue_watching_episode_label_includes_locator_and_episode_name() {
    let items = add_continue_watching_episode_labels(json!([{
        "lastEpisodeName": "The Day I Become a Shinigami",
        "lastVideoId": "show-id:1:1",
    }]));

    assert_eq!(
        items[0]["episodeLabel"],
        "S1:E1 The Day I Become a Shinigami"
    );
}

fn answer(
    engine: &mut HeadlessEngine,
    effect: &EffectEnvelope,
    value: serde_json::Value,
) -> Vec<EffectEnvelope> {
    engine.complete_effect(EffectResultInput {
        effect_id: effect.id.clone(),
        status: EffectStatus::Ok,
        value,
        error: serde_json::Value::Null,
    })
}

#[test]
fn rows_land_in_plan_order_whatever_order_feeds_answer() {
    let mut engine = HeadlessEngine::default();
    let load = dispatch_load(
        &mut engine,
        Some(json!({"id": "p1"})),
        Some("en".to_owned()),
        Some(false),
    );
    let started = answer(
        &mut engine,
        &load[0],
        json!({"plan": {"feeds": [{"key": "a"}, {"key": "b"}]}}),
    );
    let feeds: Vec<_> = started
        .iter()
        .filter(|effect| effect.kind == EffectKind::FetchHomeFeed)
        .collect();
    assert_eq!(feeds.len(), 2);
    assert!(engine.state.home.is_loading);

    let row = |id: &str| json!({"row": {"id": id, "type": "movie", "items": [{"id": format!("{id}1"), "type": "movie"}]}});
    answer(&mut engine, feeds[1], row("b"));
    assert_eq!(engine.state.home.categories.as_array().unwrap().len(), 1);
    assert_eq!(engine.state.home.pending_feeds, 1);

    answer(&mut engine, feeds[0], row("a"));
    let ids: Vec<_> = engine
        .state
        .home
        .categories
        .as_array()
        .unwrap()
        .iter()
        .map(|category| category["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(ids, ["a", "b"]);
    assert_eq!(engine.state.home.billboard["id"], "a1");
    assert!(engine.state.home.is_loading);
}

#[test]
fn cache_is_written_once_every_feed_and_continue_watching_settled() {
    let mut engine = HeadlessEngine::default();
    let load = dispatch_load(&mut engine, Some(json!({"id": "p1"})), None, Some(false));
    let started = answer(
        &mut engine,
        &load[0],
        json!({"plan": {"feeds": [{"key": "a"}]}}),
    );
    let feed = started
        .iter()
        .find(|effect| effect.kind == EffectKind::FetchHomeFeed)
        .unwrap();
    let continue_watching = started
        .iter()
        .find(|effect| effect.kind == EffectKind::RefreshContinueWatching)
        .unwrap();

    let after_feed = answer(
        &mut engine,
        feed,
        json!({"row": {"id": "a", "type": "movie", "items": [{"id": "a1", "type": "movie"}]}}),
    );
    assert!(
        after_feed
            .iter()
            .all(|effect| effect.kind != EffectKind::WriteHomeCache)
    );
    assert!(engine.state.home.is_loading);

    let after_continue = answer(
        &mut engine,
        continue_watching,
        json!({"continueWatching": []}),
    );
    assert!(
        after_continue
            .iter()
            .any(|effect| effect.kind == EffectKind::WriteHomeCache)
    );
    assert!(!engine.state.home.is_loading);
}
