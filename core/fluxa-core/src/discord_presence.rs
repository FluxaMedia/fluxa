use serde_json::{json, Value};

pub(crate) fn snapshot_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let title = args.get("title")?.as_str()?.trim();
    if title.is_empty() {
        return None;
    }
    let episode_title = args
        .get("episodeTitle")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let season = args.get("season").and_then(Value::as_u64);
    let episode = args.get("episode").and_then(Value::as_u64);
    let position_ms = args.get("positionMs").and_then(Value::as_u64).unwrap_or(0);
    let duration_ms = args.get("durationMs").and_then(Value::as_u64).unwrap_or(0);
    let is_playing = args
        .get("isPlaying")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let is_buffering = args
        .get("isBuffering")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let status = if is_buffering {
        "Buffering"
    } else if is_playing {
        "Watching"
    } else {
        "Paused"
    };
    let episode_code = match (season, episode) {
        (Some(season), Some(episode)) => Some(format!("S{season}, E{episode}")),
        (Some(season), None) => Some(format!("S{season}")),
        _ => None,
    };
    let episode_line = match (episode_code, episode_title) {
        (Some(code), Some(name)) => format!("{code}: {name}"),
        (Some(code), None) => code,
        (None, Some(name)) => name.to_owned(),
        (None, None) => String::new(),
    };
    let artwork_url = args
        .get("artworkUrl")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    serde_json::to_string(&json!({
        "title": title,
        "episodeLine": episode_line,
        "status": status,
        "positionMs": position_ms,
        "durationMs": duration_ms,
        "artworkUrl": artwork_url,
    }))
    .ok()
}

#[cfg(test)]
mod tests {
    use super::snapshot_json;
    use serde_json::Value;

    #[test]
    fn formats_episode_presence_payload() {
        let value: Value = serde_json::from_str(
            &snapshot_json(
                r#"{"title":"Rick and Morty","episodeTitle":"Lawnmower Dog","season":1,"episode":2,"positionMs":12000,"durationMs":130000,"isPlaying":true,"artworkUrl":"https://example.test/episode.jpg"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(value["title"], "Rick and Morty");
        assert_eq!(value["episodeLine"], "S1, E2: Lawnmower Dog");
        assert_eq!(value["status"], "Watching");
        assert_eq!(value["artworkUrl"], "https://example.test/episode.jpg");
    }

    #[test]
    fn paused_payload_does_not_advance_position() {
        let value: Value = serde_json::from_str(
            &snapshot_json(
                r#"{"title":"Movie","positionMs":12000,"durationMs":130000,"isPlaying":false}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(value["status"], "Paused");
        assert_eq!(value["positionMs"], 12000);
    }
}
