use super::*;
use serde_json::{Value, json};


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
fn parse_and_plan_addon_resource_returns_canonical_stream_payload() {
    let combined = parse_and_plan_addon_resource_json(
        "stream",
        "https://addon.example/stream/movie/tt1.json",
        200,
        Some(
            r#"{"streams":[{"name":"4K DV","title":"Legacy details","url":"https://video.example/4k"}]}"#,
        ),
        "streams",
        Some("Torrentio"),
        None,
    );
    let combined: Value = serde_json::from_str(&combined).expect("combined result");
    let streams: Value = serde_json::from_str(combined["valueJson"].as_str().unwrap())
        .expect("canonical stream payload");

    assert_eq!(streams[0]["name"], "4K DV");
    assert_eq!(streams[0]["title"], "Legacy details");
    assert_eq!(streams[0]["description"], "Legacy details");
    assert_eq!(streams[0]["addonName"], "Torrentio");
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
fn resource_fetch_plan_meta_detail_returns_executable_candidate_identity() {
    let request = json!({
        "kind": "metaDetail",
        "transportUrl": "https://preferred.example/manifest.json",
        "contentType": "movie",
        "id": "tt42",
        "addons": [{
            "name": "Preferred",
            "transportUrl": "https://preferred.example/manifest.json",
            "manifest": {
                "resources": ["meta"],
                "types": ["movie"],
            },
        }],
    });
    let plan = resource_fetch_plan_json(&request.to_string())
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("plan");
    let candidate = &plan["requests"][0];

    assert_eq!(
        candidate["transportUrl"],
        "https://preferred.example/manifest.json"
    );
    assert_eq!(candidate["contentType"], "movie");
    assert_eq!(candidate["id"], "tt42");
    assert_eq!(candidate["stopOnFirstResult"], true);
    assert!(
        candidate["url"]
            .as_str()
            .unwrap()
            .contains("/meta/movie/tt42.json")
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

#[test]
fn resource_fetch_execution_policy_marks_multiple_meta_candidates_as_priority_race() {
    let addons = ["first", "second"]
        .into_iter()
        .map(|name| {
            json!({
                "name": name,
                "transportUrl": format!("https://{name}.example/manifest.json"),
                "manifest": { "resources": ["meta"], "types": ["movie"] },
            })
        })
        .collect::<Vec<_>>();
    let request = json!({
        "kind": "metaDetail",
        "contentType": "movie",
        "id": "tt42",
        "addons": addons,
    });
    let policy = resource_fetch_execution_policy_json(&request.to_string())
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("policy");

    assert_eq!(policy["mode"], "race");
    assert_eq!(policy["concurrency"], 12);
    assert_eq!(policy["requests"][0]["addonName"], "first");
    assert_eq!(policy["requests"][1]["addonName"], "second");
    assert!(
        policy["requests"]
            .as_array()
            .unwrap()
            .iter()
            .all(|request| request["stopOnFirstResult"] == true)
    );
}

#[test]
fn resource_fetch_plan_home_excludes_catalogs_with_required_extras() {
    let request = json!({
        "kind": "home",
        "addons": [{
            "transportUrl": "https://addon.example/manifest.json",
            "manifest": {
                "catalogs": [
                    { "id": "top", "type": "movie", "name": "Top Movies" },
                    { "id": "genre", "type": "movie", "extra": [{ "name": "genre", "isRequired": true }] }
                ]
            }
        }]
    });
    let plan = resource_fetch_plan_json(&request.to_string())
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("plan");
    let requests = plan["requests"].as_array().unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["catalogId"], "top");
    assert_eq!(requests[0]["categoryName"], "Top Movies");
}
