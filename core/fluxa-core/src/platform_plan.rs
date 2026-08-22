mod collection_and_detail;
mod library_and_prefs;
mod playback_prepare;
mod resource_fetch;
mod resource_parse;

pub(crate) use collection_and_detail::{
    addon_collection_mutation_plan_json, detail_episode_plan_json, mark_seasons_action_plan_json,
    season_watched_plan_json,
};
pub(crate) use library_and_prefs::{
    apply_preference_update_json, library_local_state_plan_json, preferences_schema_json,
};
pub(crate) use playback_prepare::playback_prepare_plan_json;
pub(crate) use resource_fetch::{resource_fetch_execution_policy_json, resource_fetch_plan_json};
pub(crate) use resource_parse::{
    parse_and_plan_addon_resource_json, resource_kind_to_resource, resource_parse_plan_json,
};
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn parse_and_plan_addon_resource_matches_the_three_call_pipeline_for_discover() {
        let body = r#"{"metas":[{"id":"tt1","name":"A"},{"id":"tt2","name":"B"}]}"#;

        let combined = parse_and_plan_addon_resource_json(
            "catalog",
            "https://addon.example/catalog/movie/top.json",
            200,
            Some(body),
            "discover",
            None,
            None,
        );
        let combined: Value = serde_json::from_str(&combined).expect("combined result");
        assert_eq!(combined["kind"], "success");
        assert_eq!(
            combined["value"]["items"],
            json!([{"id":"tt1","name":"A"},{"id":"tt2","name":"B"}])
        );

        let step1 = crate::addon_resource::parse_addon_resource_result_json(
            "catalog",
            "https://addon.example/catalog/movie/top.json",
            200,
            Some(body),
        );
        let step1: Value = serde_json::from_str(&step1).expect("step1 result");
        let value_json = step1["valueJson"].as_str().expect("valueJson");
        let step2 = crate::addon_resource::wrap_addon_resource_response_value(
            "catalog",
            serde_json::from_str(value_json).unwrap(),
        );
        let step3 =
            resource_parse_plan_json(&json!({ "kind": "discover", "response": step2 }).to_string())
                .expect("step3 result");
        let step3: Value = serde_json::from_str(&step3).expect("step3 value");

        assert_eq!(combined["value"], step3);
    }

    #[test]
    fn parse_and_plan_addon_resource_reports_empty_without_crashing() {
        let combined = parse_and_plan_addon_resource_json(
            "catalog",
            "url",
            200,
            Some(r#"{"metas":[]}"#),
            "discover",
            None,
            None,
        );
        let combined: Value = serde_json::from_str(&combined).expect("combined result");
        assert_eq!(combined["kind"], "empty");
    }

    #[test]
    fn detail_episode_plan_picks_selected_episode_season_over_default() {
        let request = json!({
            "episodes": [
                { "id": "tt1:1:1", "season": 1 },
                { "id": "tt1:1:2", "season": 1 },
                { "id": "tt1:9:1", "season": 9 },
            ],
            "selectedEpisodeId": "tt1:9:1",
            "metaId": "tt1",
        });
        let plan = detail_episode_plan_json(&request.to_string())
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .expect("plan");

        assert_eq!(plan["seasonNumbers"], json!([1, 9]));
        assert_eq!(plan["selectedSeason"], 9);
        assert_eq!(plan["episodes"].as_array().unwrap().len(), 1);
        assert_eq!(plan["selectedEpisode"]["id"], "tt1:9:1");
        assert_eq!(plan["streamRequestId"], "tt1:9:1");
    }

    #[test]
    fn detail_episode_plan_falls_back_to_first_season_and_meta_id() {
        let request = json!({
            "episodes": [
                { "id": "tt1:2:1", "season": 2 },
                { "id": "tt1:3:1", "season": 3 },
            ],
            "metaId": "tt1",
        });
        let plan = detail_episode_plan_json(&request.to_string())
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .expect("plan");

        assert_eq!(plan["selectedSeason"], 2);
        assert_eq!(plan["selectedEpisode"]["id"], "tt1:2:1");
        // No selectedEpisodeId in the request, so streamRequestId falls back to the
        // first episode of the default season, not metaId.
        assert_eq!(plan["streamRequestId"], "tt1:2:1");
    }

    #[test]
    fn resource_fetch_plan_builds_catalog_page_url_with_genre_extra() {
        let request = json!({
            "kind": "catalogPage",
            "transportUrl": "https://addon.example/manifest.json",
            "contentType": "movie",
            "catalogId": "top",
            "genre": "action",
        });
        let plan = resource_fetch_plan_json(&request.to_string())
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .expect("plan");
        let requests = plan["requests"].as_array().unwrap();

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["kind"], "catalogPage");
        assert!(
            requests[0]["url"]
                .as_str()
                .unwrap()
                .contains("genre=action")
        );
    }

    #[test]
    fn resource_fetch_plan_skips_the_tmdb_builtin_pseudo_addon() {
        let request = json!({
            "kind": "metaDetail",
            "contentType": "series",
            "id": "tt1",
            "addons": [{
                "transportUrl": "tmdb://builtin",
                "name": "TMDB",
                "manifest": {
                    "resources": ["meta"],
                    "types": ["series"],
                },
            }],
        });
        let plan = resource_fetch_plan_json(&request.to_string())
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .expect("plan");
        let requests = plan["requests"].as_array().unwrap();

        assert!(
            requests.is_empty(),
            "the builtin TMDB pseudo-addon must never become a generic HTTP request"
        );
    }

    #[test]
    fn resource_fetch_plan_search_only_targets_catalogs_supporting_search() {
        let request = json!({
            "kind": "search",
            "query": "batman",
            "addons": [{
                "transportUrl": "https://addon.example/manifest.json",
                "name": "Addon One",
                "manifest": {
                    "catalogs": [
                        { "id": "top", "type": "movie", "name": "Top Movies", "extraSupported": ["search"] },
                        { "id": "noSearch", "type": "movie", "name": "No Search" },
                    ],
                },
            }],
        });
        let plan = resource_fetch_plan_json(&request.to_string())
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .expect("plan");
        let requests = plan["requests"].as_array().unwrap();

        assert_eq!(
            requests.len(),
            1,
            "catalog without search support must be excluded"
        );
        assert_eq!(requests[0]["catalogId"], "top");
        assert_eq!(requests[0]["categoryName"], "Addon One - Top Movies");
        assert!(
            requests[0]["url"]
                .as_str()
                .unwrap()
                .contains("search=batman")
        );
    }
}
