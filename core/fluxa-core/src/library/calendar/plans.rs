use super::helpers::{
    CalendarItemInput, calendar_item_detail_score, calendar_item_identity,
    resolve_calendar_artwork, usable_artwork,
};
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) fn calendar_visibility_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let items = request.get("items")?.as_array()?;
    let today_iso = request
        .get("todayIso")
        .and_then(Value::as_str)
        .unwrap_or("");
    let completed = request.get("completedItems")?.as_array()?;
    let show_completed = request.get("showCompleted").and_then(Value::as_bool) == Some(true);
    let visible: Vec<&Value> = items
        .iter()
        .filter(|item| {
            if show_completed {
                return true;
            }
            if !today_iso.is_empty()
                && item
                    .get("dateIso")
                    .and_then(Value::as_str)
                    .is_some_and(|date| date.get(..10).unwrap_or(date) >= today_iso)
            {
                return true;
            }
            let ids: Vec<&str> = ["contentId", "seriesId", "id"]
                .iter()
                .filter_map(|key| item.get(*key).and_then(Value::as_str))
                .collect();
            let name = item
                .get("title")
                .or_else(|| item.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("");
            !completed.iter().any(|entry| {
                let completed_id = entry.get("id").and_then(Value::as_str).unwrap_or("");
                (!completed_id.is_empty()
                    && ids.iter().any(|id| {
                        *id == completed_id || id.starts_with(&format!("{completed_id}:"))
                    }))
                    || (!name.is_empty()
                        && entry
                            .get("name")
                            .and_then(Value::as_str)
                            .is_some_and(|value| value.eq_ignore_ascii_case(name)))
            })
        })
        .collect();
    let mut seen = std::collections::HashMap::new();
    let mut unique = Vec::new();
    for item in visible {
        let key = calendar_item_identity(item);
        let score = calendar_item_detail_score(item);
        if let Some((index, current_score)) = seen.get_mut(&key) {
            if score > *current_score
                && let Some(slot) = unique.get_mut(*index)
            {
                *slot = item;
                *current_score = score;
            }
        } else {
            seen.insert(key, (unique.len(), score));
            unique.push(item);
        }
    }
    serde_json::to_string(&unique).ok()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CandidatePlanRequest {
    #[serde(default)]
    groups: Vec<Vec<Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentPlanRequest {
    #[serde(default)]
    items: Vec<CalendarItemInput>,
    month_prefix: String,
    #[serde(default)]
    watched_video_ids: std::collections::HashSet<String>,
}

pub(crate) fn calendar_content_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<ContentPlanRequest>(request_json).ok()?;
    let prefix = request.month_prefix.trim();
    if prefix.is_empty() {
        return serde_json::to_string(&json!([])).ok();
    }
    let mut seen = std::collections::HashSet::new();
    let mut filtered: Vec<&CalendarItemInput> = request
        .items
        .iter()
        .filter(|item| {
            let meta_id = if item.meta_id.trim().is_empty() {
                item.meta.get("id").and_then(Value::as_str).unwrap_or("")
            } else {
                &item.meta_id
            };
            if !request.watched_video_ids.is_empty()
                && let (Some(season), Some(episode)) = (item.season_number, item.episode_number)
                && request
                    .watched_video_ids
                    .contains(&format!("{meta_id}:{season}:{episode}"))
            {
                return false;
            }
            item.date_iso.starts_with(prefix)
                && !meta_id.trim().is_empty()
                && seen.insert(format!(
                    "{}:{}:{}",
                    item.date_iso,
                    meta_id,
                    item.subtitle.as_deref().unwrap_or("")
                ))
        })
        .collect();
    filtered.sort_by(|a, b| {
        a.date_iso
            .cmp(&b.date_iso)
            .then_with(|| a.title.cmp(&b.title))
    });
    let out: Vec<Value> = filtered
        .iter()
        .map(|item| {
            let meta_id = if item.meta_id.trim().is_empty() {
                item.meta.get("id").and_then(Value::as_str).unwrap_or("")
            } else {
                &item.meta_id
            };
            let meta_type = if item.meta_type.trim().is_empty() {
                item.meta.get("type").and_then(Value::as_str).unwrap_or("")
            } else {
                &item.meta_type
            };
            json!({
                "dateIso": item.date_iso,
                "metaId": meta_id,
                "metaType": meta_type,
                "title": item.title,
                "subtitle": item.subtitle,
                "seasonNumber": item.season_number,
                "episodeNumber": item.episode_number,
                "episodeTitle": item.episode_title,
                "artworkUrl": item.artwork_url,
                "meta": item.meta,
                "poster": item.poster,
                "episodePoster": item.episode_poster,
                "resolvedArtworkUrl": resolve_calendar_artwork(item)
            })
        })
        .collect();
    serde_json::to_string(&out).ok()
}

pub(crate) fn desktop_calendar_read_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let prefix = request
        .get("monthPrefix")
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut seen = std::collections::HashSet::new();
    let local_items: Vec<Value> = request
        .get("libraryItems")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("series"))
        .filter_map(|item| {
            let date_iso = item.get("nextEpisodeAirDate")?.as_str()?;
            if !prefix.is_empty() && !date_iso.starts_with(prefix) {
                return None;
            }
            let id = item.get("id")?.as_str()?;
            let key = format!("{}:{}", id, date_iso.get(..10).unwrap_or(date_iso));
            if !seen.insert(key.clone()) {
                return None;
            }
            let episode_poster = item.get("nextEpisodePoster").and_then(Value::as_str);
            let series_poster = item.get("poster").and_then(Value::as_str);
            let resolved_artwork = [episode_poster, series_poster]
                .into_iter()
                .find_map(usable_artwork)
                .map(str::to_string);
            Some(json!({
                "id": key,
                "title": item.get("name"),
                "name": item.get("name"),
                "dateIso": date_iso,
                "poster": item.get("nextEpisodePoster").or_else(|| item.get("poster")),
                "seriesPoster": item.get("poster"),
                "episodePoster": item.get("nextEpisodePoster"),
                "seasonNumber": item.get("nextEpisodeSeason"),
                "episodeNumber": item.get("nextEpisodeNumber"),
                "episodeTitle": item.get("nextEpisodeTitle"),
                "contentId": id,
                "seriesId": id,
                "metaType": item.get("type"),
                "resolvedArtworkUrl": resolved_artwork,
            }))
        })
        .collect();
    let external_items: Vec<&Value> = request
        .get("externalItems")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| {
            prefix.is_empty()
                || item
                    .get("dateIso")
                    .and_then(Value::as_str)
                    .is_some_and(|date| date.starts_with(prefix))
        })
        .collect();
    serde_json::to_string(&json!({"items": request.get("plannedItems").and_then(Value::as_array).cloned().unwrap_or_default(), "localItems": local_items, "externalItems": external_items})).ok()
}

fn meta_artwork_score(item: &Value) -> usize {
    [
        "continueWatchingPoster",
        "continueWatchingBackground",
        "poster",
        "background",
    ]
    .iter()
    .filter(|key| {
        item.get(**key)
            .and_then(Value::as_str)
            .and_then(|value| usable_artwork(Some(value)))
            .is_some()
    })
    .count()
}

pub(crate) fn calendar_candidate_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<CandidatePlanRequest>(request_json).ok()?;
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut candidates: Vec<Value> = Vec::new();
    for item in request.groups.into_iter().flatten() {
        let id = item
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let content_type = item
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if id.is_empty() || content_type == "catalog_folder" {
            continue;
        }
        let key = format!("{content_type}:{id}");
        match seen.get(&key) {
            Some(&index) => {
                if meta_artwork_score(&item) > meta_artwork_score(&candidates[index]) {
                    candidates[index] = item;
                }
            }
            None => {
                seen.insert(key, candidates.len());
                candidates.push(item);
            }
        }
    }
    serde_json::to_string(&candidates).ok()
}
