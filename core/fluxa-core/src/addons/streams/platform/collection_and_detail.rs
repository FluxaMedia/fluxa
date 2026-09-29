use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddonCollectionMutationRequest {
    #[serde(default)]
    existing: Vec<Value>,
    #[serde(default)]
    incoming: Vec<Value>,
    #[serde(default)]
    remove_key: Option<String>,
}

pub(crate) fn addon_collection_mutation_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<AddonCollectionMutationRequest>(request_json).ok()?;
    let mut addons = request.existing;
    if let Some(remove_key) = request.remove_key.as_deref() {
        addons.retain(|addon| addon_key(addon) != remove_key);
    }
    for incoming in request.incoming {
        let key = addon_key(&incoming);
        if key.is_empty() {
            continue;
        }
        if let Some(existing) = addons.iter_mut().find(|addon| addon_key(addon) == key) {
            *existing = incoming;
        } else {
            addons.push(incoming);
        }
    }
    serde_json::to_string(&json!({ "addons": addons })).ok()
}

fn addon_key(addon: &Value) -> String {
    addon
        .get("transportUrl")
        .or_else(|| addon.get("id"))
        .or_else(|| {
            addon
                .get("manifest")
                .and_then(|manifest| manifest.get("id"))
        })
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetailEpisodePlanRequest {
    #[serde(default)]
    episodes: Vec<Value>,
    #[serde(default)]
    selected_season: Option<i64>,
    #[serde(default)]
    selected_episode_id: Option<String>,
    #[serde(default)]
    meta_id: Option<String>,
}

pub(crate) fn detail_episode_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<DetailEpisodePlanRequest>(request_json).ok()?;
    let mut seasons = request
        .episodes
        .iter()
        .filter_map(|episode| episode.get("season").and_then(Value::as_i64).or(Some(1)))
        .collect::<Vec<_>>();
    seasons.sort_unstable();
    seasons.dedup();
    // Search for the target episode across ALL episodes before season filtering,
    // so that a lastVideoId from a later season (e.g. S9 when default would be S1) is found.
    let target_episode = request.selected_episode_id.as_deref().and_then(|id| {
        request
            .episodes
            .iter()
            .find(|ep| ep.get("id").and_then(Value::as_str) == Some(id))
            .cloned()
    });
    let selected_season = target_episode
        .as_ref()
        .and_then(|ep| ep.get("season").and_then(Value::as_i64))
        .or_else(|| {
            request
                .selected_season
                .filter(|season| seasons.contains(season))
        })
        .or_else(|| seasons.first().copied())
        .unwrap_or(1);
    let episodes = request
        .episodes
        .into_iter()
        .filter(|episode| {
            episode.get("season").and_then(Value::as_i64).unwrap_or(1) == selected_season
        })
        .collect::<Vec<_>>();
    let selected_episode = target_episode
        .filter(|ep| ep.get("season").and_then(Value::as_i64).unwrap_or(1) == selected_season)
        .or_else(|| episodes.first().cloned());
    serde_json::to_string(&json!({
        "seasonNumbers": seasons,
        "selectedSeason": selected_season,
        "episodes": episodes,
        "selectedEpisode": selected_episode,
        "streamRequestId": selected_episode
            .as_ref()
            .and_then(|episode| episode.get("id").and_then(Value::as_str))
            .or(request.meta_id.as_deref())
    }))
    .ok()
}
pub(crate) fn season_watched_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let episodes = request.get("episodes")?.as_array()?;
    let watched = request.get("watchedMap")?.as_object()?;
    let seasons = request.get("seasonNumbers")?.as_array()?;
    let mut result = serde_json::Map::new();
    for season in seasons.iter().filter_map(Value::as_i64) {
        let matching: Vec<&Value> = episodes
            .iter()
            .filter(|episode| episode.get("season").and_then(Value::as_i64).unwrap_or(1) == season)
            .collect();
        if !matching.is_empty() {
            result.insert(
                season.to_string(),
                Value::Bool(matching.iter().all(|episode| {
                    episode
                        .get("id")
                        .and_then(Value::as_str)
                        .and_then(|id| watched.get(id))
                        .and_then(Value::as_bool)
                        == Some(true)
                })),
            );
        }
    }
    serde_json::to_string(&result).ok()
}
pub(crate) fn mark_seasons_action_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let selected: std::collections::HashSet<i64> = request
        .get("seasons")?
        .as_array()?
        .iter()
        .filter_map(Value::as_i64)
        .collect();
    let watched = request
        .get("watched")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let now_ms = request.get("nowMs").and_then(Value::as_i64).unwrap_or(0);
    let episodes: Vec<&Value> = request
        .get("episodes")?
        .as_array()?
        .iter()
        .filter(|episode| {
            selected.contains(&episode.get("season").and_then(Value::as_i64).unwrap_or(1))
        })
        .filter(|episode| {
            !watched
                || episode
                    .get("released")
                    .and_then(Value::as_str)
                    .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                    .is_none_or(|released| released.timestamp_millis() <= now_ms)
        })
        .collect();
    if episodes.is_empty() {
        return None;
    }
    let meta = request.get("meta")?;
    serde_json::to_string(&json!({
        "type": "markWatchedRequested",
        "seriesId": meta.get("id"),
        "videoIds": episodes.iter().filter_map(|episode| episode.get("id")).collect::<Vec<_>>(),
        "watched": watched,
        "meta": meta,
        "episodes": episodes.iter().map(|episode| json!({
            "id": episode.get("id"),
            "name": episode.get("name").or_else(|| episode.get("title")),
            "season": episode.get("season"),
            "number": episode.get("episode").or_else(|| episode.get("number")),
            "thumbnail": episode.get("thumbnail"),
        })).collect::<Vec<_>>(),
    }))
    .ok()
}
