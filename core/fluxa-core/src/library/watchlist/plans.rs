use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TogglePlanRequest {
    item: Value,
    #[serde(default)]
    is_currently_in_watchlist: bool,
    #[serde(default)]
    profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProgressMergeRequest {
    existing: Value,
    incoming: Value,
}

pub(crate) fn watchlist_toggle_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<TogglePlanRequest>(request_json).ok()?;
    let is_in_watchlist = request.is_currently_in_watchlist;
    let should_add = !is_in_watchlist;
    let item_id = request
        .item
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    serde_json::to_string(&json!({
        "command": if should_add { "add" } else { "remove" },
        "itemId": item_id,
        "optimisticIsInWatchlist": should_add,
        "profileId": request.profile_id
    }))
    .ok()
}

pub(crate) fn playback_progress_merge_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<ProgressMergeRequest>(request_json).ok()?;
    let existing = &request.existing;
    let incoming = &request.incoming;

    let existing_video_id = existing.get("lastVideoId").and_then(Value::as_str);
    let incoming_video_id = incoming.get("lastVideoId").and_then(Value::as_str);
    let video_changed = incoming_video_id.is_some() && incoming_video_id != existing_video_id;

    let id_parts = incoming_video_id.and_then(|id| {
        let mut parts = id.rsplitn(3, ':');
        let episode = parts.next()?.parse::<i64>().ok()?;
        let season = parts.next()?.parse::<i64>().ok()?;
        parts.next()?;
        Some((season, episode))
    });

    let resolve_field = |key: &str| -> Value {
        let incoming_value = incoming.get(key).filter(|v| !v.is_null()).cloned();
        let episode_field = matches!(
            key,
            "lastEpisodeName"
                | "lastEpisodeSeason"
                | "lastEpisodeNumber"
                | "lastEpisodeThumbnail"
                | "lastStreamUrl"
                | "lastStreamTitle"
                | "lastStreamIndex"
                | "lastBingeGroup"
        );
        if video_changed && episode_field {
            return incoming_value
                .or_else(|| match (key, id_parts) {
                    ("lastEpisodeSeason", Some((season, _))) => Some(json!(season)),
                    ("lastEpisodeNumber", Some((_, episode))) => Some(json!(episode)),
                    _ => None,
                })
                .unwrap_or(Value::Null);
        }
        incoming_value
            .or_else(|| existing.get(key).cloned())
            .unwrap_or(Value::Null)
    };

    serde_json::to_string(&json!({
        "lastVideoId": resolve_field("lastVideoId"),
        "timeOffset": incoming.get("timeOffset").cloned().unwrap_or(Value::Null),
        "duration": incoming.get("duration").cloned().unwrap_or(Value::Null),
        "lastStreamIndex": resolve_field("lastStreamIndex"),
        "lastEpisodeName": resolve_field("lastEpisodeName"),
        "lastEpisodeSeason": resolve_field("lastEpisodeSeason"),
        "lastEpisodeNumber": resolve_field("lastEpisodeNumber"),
        "lastEpisodeThumbnail": resolve_field("lastEpisodeThumbnail"),
        "lastStreamUrl": resolve_field("lastStreamUrl"),
        "lastStreamTitle": resolve_field("lastStreamTitle"),
        "lastBingeGroup": resolve_field("lastBingeGroup"),
        "continueWatchingPoster": resolve_field("continueWatchingPoster"),
        "continueWatchingBackground": resolve_field("continueWatchingBackground"),
        "lastAudioLanguage": resolve_field("lastAudioLanguage"),
        "lastSubtitleLanguage": resolve_field("lastSubtitleLanguage"),
        "videoChanged": video_changed
    }))
    .ok()
}

#[cfg(test)]
mod progress_merge_tests {
    use super::playback_progress_merge_plan_json;
    use serde_json::Value;

    #[test]
    fn changed_episode_drops_stale_season_and_number() {
        let plan = playback_progress_merge_plan_json(
            r#"{"existing":{"lastVideoId":"tt1:2:2","lastEpisodeSeason":2,"lastEpisodeNumber":2,"lastEpisodeName":"Old"},"incoming":{"lastVideoId":"tt1:1:1","timeOffset":10,"duration":100}}"#,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .unwrap();
        assert_eq!(plan["lastEpisodeSeason"], 1);
        assert_eq!(plan["lastEpisodeNumber"], 1);
        assert!(plan["lastEpisodeName"].is_null());
    }
}
