use serde_json::{Value, json};

fn video_key(video: &Value) -> (i64, i64) {
    let season = video.get("season").and_then(Value::as_i64).unwrap_or(0);
    let episode = video
        .get("episode")
        .or_else(|| video.get("number"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    (season, episode)
}

fn release_ms(video: &Value) -> Option<i64> {
    let released = video.get("released").and_then(Value::as_str)?;
    chrono::DateTime::parse_from_rfc3339(released)
        .ok()
        .map(|date| date.timestamp_millis())
}

pub(crate) fn continue_watching_episode_status(item: &Value, videos: &[Value], now_ms: i64) -> Value {
    let regular: Vec<&Value> = videos.iter().filter(|v| video_key(v).0 > 0).collect();
    let video_id = item.get("lastVideoId").and_then(Value::as_str);
    let current = match (
        item.get("lastEpisodeSeason").and_then(Value::as_i64),
        item.get("lastEpisodeNumber").and_then(Value::as_i64),
    ) {
        (Some(season), Some(number)) => Some((season, number)),
        _ => video_id.and_then(|id| {
            regular
                .iter()
                .find(|v| v.get("id").and_then(Value::as_str) == Some(id))
                .map(|v| video_key(v))
        }),
    };
    let Some(current) = current else {
        return json!({"episodesLeft": null, "upcoming": false, "airsAt": null});
    };
    let from = (current.0, current.1 - 1);
    let mut remaining: Vec<&Value> = regular
        .into_iter()
        .filter(|v| video_key(v) > from)
        .collect();
    remaining.sort_by_key(|v| video_key(v));
    let released = remaining
        .iter()
        .filter(|v| release_ms(v).is_none_or(|at| at <= now_ms))
        .count();
    let next = remaining.first().and_then(|v| release_ms(v));
    let upcoming = released == 0 && next.is_some_and(|at| at > now_ms);
    json!({
        "episodesLeft": released,
        "upcoming": upcoming,
        "airsAt": if upcoming { json!(next) } else { Value::Null },
    })
}

pub(crate) fn continue_watching_episode_status_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let videos = args.get("videos")?.as_array()?;
    let now_ms = args.get("nowMs")?.as_i64()?;
    serde_json::to_string(&continue_watching_episode_status(args.get("item")?, videos, now_ms)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn videos() -> Vec<Value> {
        vec![
            json!({"id": "t:1:1", "season": 1, "episode": 1, "released": "2026-01-01T00:00:00Z"}),
            json!({"id": "t:1:2", "season": 1, "episode": 2, "released": "2026-01-08T00:00:00Z"}),
            json!({"id": "t:1:3", "season": 1, "episode": 3, "released": "2026-01-15T00:00:00Z"}),
            json!({"id": "t:1:4", "season": 1, "episode": 4, "released": "2099-01-01T00:00:00Z"}),
        ]
    }

    #[test]
    fn counts_released_episodes_including_the_current_one() {
        let item = json!({"lastEpisodeSeason": 1, "lastEpisodeNumber": 2, "timeOffset": 300});
        let status = continue_watching_episode_status(&item, &videos(), 1_800_000_000_000);
        assert_eq!(status["episodesLeft"], 2);
        assert_eq!(status["upcoming"], false);
    }

    #[test]
    fn unreleased_next_episode_is_upcoming() {
        let item = json!({"lastEpisodeSeason": 1, "lastEpisodeNumber": 4, "continueWatchingBadge": "upNext"});
        let status = continue_watching_episode_status(&item, &videos(), 1_800_000_000_000);
        assert_eq!(status["upcoming"], true);
        assert_eq!(status["episodesLeft"], 0);
    }
}
