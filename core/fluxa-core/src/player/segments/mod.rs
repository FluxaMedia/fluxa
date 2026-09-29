#[cfg(test)]
use serde_json::{Value, json};

mod lookup;
mod parsers;
mod request_plans;
mod submit;

pub(crate) use lookup::player_segments_step_json;
pub(crate) use parsers::*;
pub(crate) use request_plans::*;
pub(crate) use submit::player_segments_submit_plan_json;

fn the_introdb_wire_type(canonical: &str) -> &'static str {
    match canonical {
        "outro" => "credits",
        "preview" => "preview",
        "recap" => "recap",
        _ => "intro",
    }
}

fn the_introdb_canonical_type(wire: &str) -> &'static str {
    match wire {
        "credits" => "outro",
        "preview" => "preview",
        "recap" => "recap",
        _ => "intro",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_publicmetadb_skips_response_including_credits_alias() {
        let response = json!({
            "items": [
                {
                    "id": "skip123",
                    "tmdb_id": 1399,
                    "media_type": "tv",
                    "season": 1,
                    "episode": 1,
                    "source": "streaming",
                    "intro_start_ms": 0,
                    "intro_end_ms": 62000,
                    "credits_start_ms": 3180000,
                    "credits_end_ms": 3240000
                }
            ],
            "total": 1
        })
        .to_string();
        let segments: Value =
            serde_json::from_str(&parse_publicmetadb_segments_json(&response).unwrap()).unwrap();
        let segments = segments.as_array().unwrap();
        assert!(
            segments
                .iter()
                .any(|s| s["type"] == "intro" && s["startTime"] == 0 && s["endTime"] == 62000)
        );
        assert!(
            segments.iter().any(|s| s["type"] == "outro"
                && s["startTime"] == 3180000
                && s["endTime"] == 3240000)
        );
    }

    #[test]
    fn parses_aniskip_camel_and_snake_case_payloads() {
        let response = json!({
            "results": [
                {
                    "skipType": "op",
                    "interval": { "startTime": 12.5, "endTime": 82.5 }
                },
                {
                    "skip_type": "ed",
                    "interval": { "start_time": 90000.0, "end_time": 93000.0 }
                }
            ]
        })
        .to_string();
        let segments: Value =
            serde_json::from_str(&parse_aniskip_results_json(&response).unwrap()).unwrap();
        assert_eq!(
            segments,
            json!([
                { "type": "intro", "startTime": 12500, "endTime": 82500 },
                { "type": "outro", "startTime": 90000, "endTime": 93000 }
            ])
        );
    }
}
