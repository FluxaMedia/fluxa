use super::helpers::{
    
    usable_artwork
};
use serde_json::{Value, json};

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
                "resolvedArtworkUrl": resolved_artwork
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
