use super::{intro_db_submit_plan_json, skipdb_submit_plan_json, the_introdb_submit_plan_json};
use crate::services::publicmetadb::{publicmetadb_bearer, publicmetadb_skips_create_plan};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Input {
    provider: String,
    api_key: String,
    segment_type: String,
    start_ms: i64,
    end_ms: i64,
    #[serde(default)]
    imdb_id: Option<String>,
    #[serde(default)]
    tmdb_id: Option<i64>,
    #[serde(default)]
    media_type: String,
    #[serde(default)]
    season: Option<i64>,
    #[serde(default)]
    episode: Option<i64>,
    #[serde(default)]
    duration_ms: Option<i64>,
}

pub(crate) fn player_segments_submit_plan_json(input: &str) -> Option<String> {
    let input: Input = serde_json::from_str(input).ok()?;
    if input.end_ms <= input.start_ms || input.start_ms < 0 {
        return None;
    }
    let start_sec = input.start_ms as f64 / 1000.0;
    let end_sec = input.end_ms as f64 / 1000.0;
    match input.provider.as_str() {
        "introdb" => intro_db_submit_plan_json(
            &json!({
                "apiKey": input.api_key,
                "imdbId": input.imdb_id?,
                "season": input.season?,
                "episode": input.episode?,
                "segmentType": input.segment_type,
                "startSec": start_sec,
                "endSec": end_sec,
            })
            .to_string(),
        ),
        "skipdb" => skipdb_submit_plan_json(
            &json!({
                "apiKey": input.api_key,
                "imdbId": input.imdb_id?,
                "season": input.season?,
                "episode": input.episode?,
                "segmentType": input.segment_type,
                "startMs": input.start_ms,
                "endMs": input.end_ms,
            })
            .to_string(),
        ),
        "theintrodb" => the_introdb_submit_plan_json(
            &json!({
                "apiKey": input.api_key,
                "tmdbId": input.tmdb_id?,
                "mediaType": input.media_type,
                "segment": input.segment_type,
                "startSec": start_sec,
                "endSec": end_sec,
                "videoDurationMs": input.duration_ms,
                "imdbId": input.imdb_id,
                "season": input.season,
                "episode": input.episode,
            })
            .to_string(),
        ),
        "publicmetadb" => {
            let (start_key, end_key) = match input.segment_type.as_str() {
                "intro" => ("intro_start_ms", "intro_end_ms"),
                "outro" => ("credits_start_ms", "credits_end_ms"),
                _ => return None,
            };
            let plan = publicmetadb_skips_create_plan(
                &json!({
                    "tmdb_id": input.tmdb_id?,
                    "media_type": input.media_type,
                    "season": input.season,
                    "episode": input.episode,
                    start_key: input.start_ms,
                    end_key: input.end_ms,
                })
                .to_string(),
            )?;
            let mut plan: Value = serde_json::from_str(&plan).ok()?;
            plan["headers"] = json!({
                "Content-Type": "application/json",
                "Authorization": publicmetadb_bearer(&input.api_key),
            });
            if let Some(body) = plan.get("body").filter(|body| !body.is_string()) {
                plan["body"] = json!(body.to_string());
            }
            serde_json::to_string(&plan).ok()
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(provider: &str, segment_type: &str) -> Option<Value> {
        let input = json!({
            "provider": provider, "apiKey": "k", "segmentType": segment_type,
            "startMs": 1000, "endMs": 61000, "imdbId": "tt1", "tmdbId": 9,
            "mediaType": "tv", "season": 1, "episode": 2,
        });
        player_segments_submit_plan_json(&input.to_string())
            .map(|raw| serde_json::from_str(&raw).unwrap())
    }

    #[test]
    fn each_provider_builds_its_own_authenticated_post() {
        assert_eq!(
            plan("introdb", "intro").unwrap()["headers"]["X-API-Key"],
            "k"
        );
        assert_eq!(
            plan("theintrodb", "outro").unwrap()["headers"]["Authorization"],
            "Bearer k"
        );
        let pmdb = plan("publicmetadb", "outro").unwrap();
        assert!(pmdb["body"].as_str().unwrap().contains("credits_start_ms"));
    }

    #[test]
    fn empty_range_and_unsupported_type_are_rejected() {
        assert!(plan("publicmetadb", "recap").is_none());
        let backwards = json!({
            "provider": "introdb", "apiKey": "k", "segmentType": "intro",
            "startMs": 5000, "endMs": 5000, "imdbId": "tt1", "season": 1, "episode": 1,
        });
        assert!(player_segments_submit_plan_json(&backwards.to_string()).is_none());
    }
}
