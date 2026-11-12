use super::{
    EffectEnvelope, EffectKind, EffectResultInput, HeadlessEngine, HomeState, Value,
    normalized_row, with_normalized_meta_trailers,
};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct HomePlan {
    feeds: Vec<Value>,
    keys: Vec<String>,
    fresh: Vec<Option<Option<Arc<Value>>>>,
    pinned: Vec<Arc<Value>>,
    regular: Vec<Arc<Value>>,
    hidden: Vec<Arc<Value>>,
    previous: Vec<Arc<Value>>,
    nuvio_items: Value,
    addons: Value,
    profile_id: String,
    language: String,
    ratings: HashMap<String, Value>,
    ratings_asked: Vec<String>,
    continue_pending: bool,
    saved: bool,
}

fn rows(plan: &Value, key: &str) -> Vec<Value> {
    plan.get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn shelves(plan: &Value, key: &str) -> Vec<Arc<Value>> {
    rows(plan, key).into_iter().map(normalized_row).collect()
}

pub(super) fn begin(
    engine: &mut HeadlessEngine,
    generation: u64,
    result: &EffectResultInput,
    effect_payload: &Value,
) -> Vec<EffectEnvelope> {
    let plan = result.value.get("plan").cloned().unwrap_or(Value::Null);
    let feeds = rows(&plan, "feeds");
    let keys = feeds
        .iter()
        .map(|feed| {
            feed.get("key")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        })
        .collect();
    let profile_id = effect_payload
        .get("profileId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let language = effect_payload
        .get("language")
        .and_then(Value::as_str)
        .unwrap_or("en")
        .to_owned();
    let home = &mut engine.state.home;
    let continue_pending = home.continue_pending;
    let state = HomePlan {
        fresh: vec![None; feeds.len()],
        keys,
        pinned: shelves(&plan, "pinnedShelves"),
        regular: shelves(&plan, "regularShelves"),
        hidden: shelves(&plan, "hiddenFolderCategories"),
        previous: home.categories.clone(),
        nuvio_items: plan.get("nuvioItems").cloned().unwrap_or(Value::Null),
        addons: plan.get("addons").cloned().unwrap_or(Value::Null),
        profile_id: profile_id.clone(),
        language: language.clone(),
        continue_pending,
        feeds: feeds.clone(),
        ..HomePlan::default()
    };
    home.metadata_feeds = Value::Array(feeds.clone());
    home.pending_feeds = feeds.len() as u32;
    home.is_stale = false;
    home.is_loading = true;
    home.error = Value::Null;
    home.plan = Some(state);
    let mut effects: Vec<EffectEnvelope> = feeds
        .iter()
        .enumerate()
        .map(|(index, feed)| {
            engine.effect(
                EffectKind::FetchHomeFeed,
                generation,
                json!({
                    "index": index,
                    "feed": feed,
                    "profileId": profile_id,
                    "language": language,
                }),
            )
        })
        .collect();
    assemble(engine);
    effects.extend(follow_ups(engine, generation));
    effects
}

pub(super) fn feed_done(
    engine: &mut HeadlessEngine,
    generation: u64,
    result: &EffectResultInput,
    effect_payload: &Value,
) -> Vec<EffectEnvelope> {
    if generation != engine.state.home.generation {
        return vec![];
    }
    let index = effect_payload
        .get("index")
        .and_then(Value::as_u64)
        .unwrap_or(u64::MAX) as usize;
    let row = result
        .status
        .is_ok()
        .then(|| {
            result
                .value
                .get("row")
                .filter(|row| !row.is_null())
                .cloned()
                .map(normalized_row)
        })
        .flatten();
    let Some(plan) = engine.state.home.plan.as_mut() else {
        return vec![];
    };
    if plan.fresh.get(index).is_none_or(Option::is_some) {
        return vec![];
    }
    plan.fresh[index] = Some(row);
    engine.state.home.pending_feeds = engine.state.home.pending_feeds.saturating_sub(1);
    assemble(engine);
    follow_ups(engine, generation)
}

pub(super) fn continue_done(
    engine: &mut HeadlessEngine,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    let generation = engine.state.home.generation;
    engine.state.home.continue_pending = false;
    let Some(plan) = engine.state.home.plan.as_mut() else {
        return vec![];
    };
    if !std::mem::take(&mut plan.continue_pending) {
        return vec![];
    }
    let _ = result;
    follow_ups(engine, generation)
}

pub(super) fn ratings_done(
    engine: &mut HeadlessEngine,
    generation: u64,
    result: &EffectResultInput,
    effect_payload: &Value,
) -> Vec<EffectEnvelope> {
    if generation != engine.state.home.generation {
        return vec![];
    }
    let id = effect_payload
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let ratings = result
        .status
        .is_ok()
        .then(|| result.value.get("ratings").cloned())
        .flatten()
        .filter(|ratings| !ratings.is_null());
    let Some(plan) = engine.state.home.plan.as_mut() else {
        return vec![];
    };
    if let Some(ratings) = ratings {
        plan.ratings.insert(id, ratings);
    }
    assemble(engine);
    vec![]
}

fn assemble(engine: &mut HeadlessEngine) {
    let home = &mut engine.state.home;
    let Some(plan) = home.plan.as_ref() else {
        return;
    };
    let mut categories = plan.pinned.clone();
    for (index, key) in plan.keys.iter().enumerate() {
        match &plan.fresh[index] {
            Some(Some(row)) => categories.push(Arc::clone(row)),
            Some(None) => {}
            None => categories.extend(
                plan.previous
                    .iter()
                    .find(|row| row.get("id").and_then(Value::as_str) == Some(key.as_str()))
                    .cloned(),
            ),
        }
    }
    categories.extend(plan.regular.iter().cloned());
    categories.extend(plan.hidden.iter().cloned());
    if !plan.nuvio_items.is_null() {
        let input = NuvioLayoutInput {
            categories: &categories,
            addons: &plan.addons,
            items: &plan.nuvio_items,
        };
        if let Some(Value::Array(ordered)) = serde_json::to_string(&input)
            .ok()
            .and_then(|raw| crate::services::nuvio::home_layout_json(&raw))
            .and_then(|raw| serde_json::from_str(&raw).ok())
        {
            let reordered = ordered
                .into_iter()
                .map(|row| {
                    categories
                        .iter()
                        .find(|known| ***known == row)
                        .cloned()
                        .unwrap_or_else(|| normalized_row(row))
                })
                .collect();
            categories = reordered;
        }
    }
    let mut billboard = categories
        .iter()
        .filter(|category| category.get("type").and_then(Value::as_str) != Some("collection"))
        .find_map(|category| {
            category
                .get("items")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .cloned()
        });
    if let Some(Value::Object(item)) = billboard.as_mut()
        && let Some(ratings) = item
            .get("id")
            .and_then(Value::as_str)
            .and_then(|id| plan.ratings.get(id))
    {
        item.insert("mdblistRatings".to_owned(), ratings.clone());
    }
    home.categories = categories;
    home.billboard = billboard
        .map(with_normalized_meta_trailers)
        .unwrap_or(Value::Null);
}

#[derive(Serialize)]
struct NuvioLayoutInput<'a> {
    categories: &'a [Arc<Value>],
    addons: &'a Value,
    items: &'a Value,
}

fn follow_ups(engine: &mut HeadlessEngine, generation: u64) -> Vec<EffectEnvelope> {
    let mut effects = Vec::new();
    let billboard = engine.state.home.billboard.as_object().and_then(|item| {
        Some((
            item.get("type")?.as_str()?.to_owned(),
            item.get("id")?.as_str()?.to_owned(),
        ))
    });
    if let Some((content_type, id)) = billboard
        && let Some(plan) = engine.state.home.plan.as_mut()
        && !plan.ratings_asked.contains(&id)
    {
        plan.ratings_asked.push(id.clone());
        let profile_id = plan.profile_id.clone();
        effects.push(engine.effect(
            EffectKind::FetchHomeHeroRatings,
            generation,
            json!({"profileId": profile_id, "contentType": content_type, "id": id}),
        ));
    }
    let home: &HomeState = &engine.state.home;
    let Some(plan) = home.plan.as_ref() else {
        return effects;
    };
    if home.pending_feeds > 0 || plan.continue_pending || plan.saved {
        return effects;
    }
    let bootstrap = json!({
        "categories": home.categories,
        "continueWatching": home.continue_watching,
        "metadataFeeds": home.metadata_feeds,
        "billboard": home.billboard,
    });
    let keep = !home.categories.is_empty() || plan.feeds.is_empty();
    let payload = json!({
        "profileId": plan.profile_id,
        "language": plan.language,
        "bootstrap": bootstrap,
    });
    if let Some(plan) = engine.state.home.plan.as_mut() {
        plan.saved = true;
    }
    engine.state.home.is_loading = false;
    if keep {
        effects.push(engine.effect(EffectKind::WriteHomeCache, generation, payload));
    }
    effects
}
