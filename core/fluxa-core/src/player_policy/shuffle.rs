use serde_json::{Value, json};

pub(crate) fn shuffle_episode_pick_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let ids = |key: &str| -> Vec<&str> {
        request
            .get(key)
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default()
    };
    let episodes = ids("episodeIds");
    let played = ids("playedIds");
    let current = request.get("currentId").and_then(Value::as_str);
    let seed = request.get("seed").and_then(Value::as_u64).unwrap_or(0);
    let fresh: Vec<&str> = episodes
        .iter()
        .copied()
        .filter(|id| !played.contains(id) && Some(*id) != current)
        .collect();
    let restarted = fresh.is_empty();
    let pool = if restarted {
        episodes
            .iter()
            .copied()
            .filter(|id| Some(*id) != current)
            .collect()
    } else {
        fresh
    };
    let pick = *pool.get(mix(seed) as usize % pool.len().max(1))?;
    Some(json!({"videoId": pick, "restarted": restarted}).to_string())
}

fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pick(request: Value) -> Value {
        serde_json::from_str(&shuffle_episode_pick_json(&request.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn skips_episodes_already_played_this_session() {
        for seed in 0..50 {
            let result = pick(json!({
                "episodeIds": ["a", "b", "c"],
                "playedIds": ["a", "b"],
                "seed": seed,
            }));
            assert_eq!(result["videoId"], "c");
        }
    }

    #[test]
    fn starts_over_without_repeating_the_current_episode() {
        for seed in 0..50 {
            let result = pick(json!({
                "episodeIds": ["a", "b"],
                "playedIds": ["a", "b"],
                "currentId": "b",
                "seed": seed,
            }));
            assert_eq!(result["videoId"], "a");
            assert_eq!(result["restarted"], true);
        }
    }

    #[test]
    fn single_episode_series_has_nothing_after_it() {
        assert!(
            shuffle_episode_pick_json(
                &json!({"episodeIds": ["a"], "playedIds": ["a"], "currentId": "a"}).to_string()
            )
            .is_none()
        );
    }
}
