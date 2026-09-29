use super::helpers::parse;
use super::progress_sync::*;
use serde_json::{Value, json};

pub(crate) fn provider_library_snapshot_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let library = args.get("library")?.as_array()?.clone();
    let progress = super::delta_state::projection(args.get("progress")?.as_array()?);
    let watchlist: Value = serde_json::from_str(&library_to_watchlist_json(
        &json!({"library": library.clone()}).to_string(),
    )?)
    .ok()?;
    let watching: Value = serde_json::from_str(&resolve_continue_watching_json(
        &json!({"progress": progress, "addonMetas": {}}).to_string(),
    )?)
    .ok()?;
    let library_by_id: std::collections::HashMap<&str, &Value> = library
        .iter()
        .filter_map(|item| Some((item.get("content_id")?.as_str()?, item)))
        .collect();
    let metas = args.get("metas").and_then(Value::as_object);
    let watching = watching
        .as_array()?
        .iter()
        .map(|entry| {
            let mut out = entry.as_object().cloned().unwrap_or_default();
            if let Some(content_id) = entry.get("content_id").and_then(Value::as_str) {
                if let Some(meta) = metas.and_then(|metas| metas.get(content_id)) {
                    for field in [
                        "name",
                        "poster",
                        "background",
                        "description",
                        "releaseInfo",
                        "genres",
                    ] {
                        if let Some(value) = meta.get(field).filter(|value| !value.is_null()) {
                            out.insert(field.to_owned(), value.clone());
                        }
                    }
                    if let Some(video) = episode_video(meta, entry) {
                        let text = |field| {
                            video
                                .get(field)
                                .and_then(Value::as_str)
                                .filter(|value| !value.trim().is_empty())
                        };
                        if let Some(thumbnail) = text("thumbnail") {
                            out.insert("lastEpisodeThumbnail".into(), json!(thumbnail));
                        }
                        if let Some(name) = text("name").or_else(|| text("title")) {
                            out.insert("lastEpisodeName".into(), json!(name));
                        }
                    }
                }
                if let Some(item) = library_by_id.get(content_id) {
                    for (target, source) in [
                        ("name", "name"),
                        ("poster", "poster"),
                        ("background", "background"),
                        ("description", "description"),
                        ("releaseInfo", "release_info"),
                        ("genres", "genres"),
                    ] {
                        if let Some(value) = item.get(source).filter(|value| !value.is_null()) {
                            out.insert(target.to_owned(), value.clone());
                        }
                    }
                }
                out.insert("id".into(), json!(content_id));
            }
            if let Some(content_type) = entry.get("content_type") {
                out.insert("type".into(), content_type.clone());
            }
            out.insert(
                "timeOffset".into(),
                json!(
                    entry
                        .get("position")
                        .and_then(Value::as_i64)
                        .unwrap_or_default()
                        / 1000
                ),
            );
            out.insert(
                "duration".into(),
                json!(
                    entry
                        .get("duration")
                        .and_then(Value::as_i64)
                        .unwrap_or_default()
                        / 1000
                ),
            );
            if let Some(video_id) = entry.get("video_id") {
                out.insert("videoId".into(), video_id.clone());
                out.insert("lastVideoId".into(), video_id.clone());
            }
            if let Some(season) = entry.get("season").filter(|value| !value.is_null()) {
                out.insert("lastEpisodeSeason".into(), season.clone());
            }
            if let Some(episode) = entry.get("episode").filter(|value| !value.is_null()) {
                out.insert("lastEpisodeNumber".into(), episode.clone());
            }
            Value::Object(out)
        })
        .collect::<Vec<_>>();
    let watched = args
        .get("watched")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let id = item.get("content_id")?.as_str()?.trim();
            if id.is_empty() {
                return None;
            }
            let key = match (
                item.get("season").and_then(Value::as_i64),
                item.get("episode").and_then(Value::as_i64),
            ) {
                (Some(season), Some(episode)) => format!("{id}:{season}:{episode}"),
                _ => id.to_owned(),
            };
            Some((key, Value::Bool(true)))
        })
        .collect::<serde_json::Map<_, _>>();
    serde_json::to_string(&json!({
        "watchlist": watchlist,
        "continueWatching": watching,
        "liked": [],
        "watched": watched,
        "dropped": [],
        "completed": [],
    }))
    .ok()
}
