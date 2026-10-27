use crate::runtime::core_error::{CoreError, LogAndDiscard};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct Anime4kRequest {
    #[serde(default)]
    tier: String,
    #[serde(default)]
    mode: String,
}

fn profile(tier: &str, mode: &str) -> String {
    let tier = match tier {
        "anime4k_s" => "s",
        "anime4k_l" => "l",
        _ => "m",
    };
    let mode = match mode {
        "b" | "c" | "aa" | "bb" | "ca" => mode,
        _ => "a",
    };
    format!("enhance-anime-{tier}-{mode}")
}

pub(crate) fn anime4k_profile_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<Anime4kRequest>(request_json)
        .map_err(|error| CoreError::BadInput {
            context: "anime4k_profile_json",
            detail: error.to_string(),
        })
        .log_discard()?;
    Some(json!(profile(&request.tier, &request.mode)).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_and_mode_map_to_fork_profile() {
        assert_eq!(profile("anime4k_s", "aa"), "enhance-anime-s-aa");
        assert_eq!(profile("anime4k_l", "c"), "enhance-anime-l-c");
    }

    #[test]
    fn unknown_values_fall_back_to_medium_a() {
        assert_eq!(profile("", ""), "enhance-anime-m-a");
    }
}
