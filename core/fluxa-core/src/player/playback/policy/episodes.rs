use serde_json::Value;

fn position(video: &Value) -> (i64, i64) {
    let number = |key: &str| video.get(key).and_then(Value::as_i64);
    (
        number("season").unwrap_or(0),
        number("episode").or_else(|| number("number")).unwrap_or(0),
    )
}

pub(crate) fn player_episodes_json(input: &str) -> Option<String> {
    let input: Value = serde_json::from_str(input).ok()?;
    let mut videos: Vec<&Value> = input
        .get("videos")?
        .as_array()?
        .iter()
        .filter(|video| position(video).0 > 0)
        .collect();
    videos.sort_by_key(|video| position(video));
    serde_json::to_string(&videos).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn specials_are_dropped_and_numbers_are_the_fallback() {
        let input = json!({"videos": [
            {"id": "c", "season": 2, "number": 1},
            {"id": "sp", "season": 0, "episode": 1},
            {"id": "b", "season": 1, "episode": 2},
            {"id": "a", "season": 1, "episode": 1},
        ]});
        let sorted: Vec<Value> =
            serde_json::from_str(&player_episodes_json(&input.to_string()).unwrap()).unwrap();
        let ids: Vec<_> = sorted.iter().map(|video| video["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["a", "b", "c"]);
    }
}
