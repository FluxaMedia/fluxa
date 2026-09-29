use super::{
    stream_discovery_cache_prefix, stream_discovery_execution_policy_json,
    stream_discovery_plan_json,
};
use serde_json::Value;

#[test]
fn stream_discovery_plan_filters_by_manifest_without_reordering_addons() {
    let plan = stream_discovery_plan_json(
        r#"{
            "type":"series",
            "id":"tt1:2:7",
            "language":"en",
            "preferFastStart":true,
            "addonRequestTimeoutMs":1000,
            "fastAddonRequestTimeoutMs":3000,
            "cloudstreamTimeoutMs":5000,
            "cs3PluginNames":["cs"],
            "cs3SearchQuery":"Show",
            "cs3OriginalName":"Original Show",
            "cs3Year":2024,
            "addons":[
                {"transportUrl":"https://a/manifest.json","manifest":{"id":"a","name":"A","resources":["stream"],"types":["movie"]}},
                {"transportUrl":"https://b/manifest.json","manifest":{"id":"b","name":"B","resources":["stream"],"types":["series"]}},
                {"transportUrl":"https://c/manifest.json","manifest":{"id":"c","name":"C","resources":[{"name":"stream","types":["series"]}],"types":["series"]}}
            ]
        }"#,
    )
    .expect("plan");
    let plan: Value = serde_json::from_str(&plan).expect("plan json");
    let addon_requests = plan["addonRequests"].as_array().expect("addon requests");

    assert_eq!(addon_requests.len(), 2);
    assert_eq!(addon_requests[0]["addonName"].as_str(), Some("B"));
    assert_eq!(addon_requests[1]["addonName"].as_str(), Some("C"));
    assert_eq!(addon_requests[0]["timeoutMs"].as_i64(), Some(3000));
    assert_eq!(plan["cloudstreamRequest"]["season"].as_i64(), Some(2));
    assert_eq!(plan["cloudstreamRequest"]["episode"].as_i64(), Some(7));
}

#[test]
fn stream_discovery_execution_policy_owns_cache_and_progress_rules() {
    let policy = stream_discovery_execution_policy_json(
        r#"{
            "type":"movie",
            "id":"tt1",
            "language":"tr",
            "addonRequestTimeoutMs":1000,
            "fastAddonRequestTimeoutMs":3000,
            "cloudstreamTimeoutMs":5000,
            "maxConcurrentAddonRequests":64,
            "addons":[
                {"transportUrl":"https://a/manifest.json","manifest":{"id":"a","name":"A","resources":["stream"],"types":["movie"]}},
                {"transportUrl":"https://b/manifest.json","manifest":{"id":"b","name":"B","resources":["stream"],"types":["movie"]}}
            ],
            "cs3PluginNames":["cs"],
            "cs3SearchQuery":"Movie"
        }"#,
    )
    .expect("policy");
    let policy: Value = serde_json::from_str(&policy).expect("policy json");
    let addon_requests = policy["addonRequests"].as_array().expect("addon requests");

    assert_eq!(policy["cacheLookupPrefix"].as_str(), Some("movie|tt1|tr"));
    assert_eq!(policy["maxConcurrentAddonRequests"].as_i64(), Some(64));
    assert_eq!(policy["cacheWriteMinimumResultCount"].as_i64(), Some(1));
    assert_eq!(policy["emitCachedResult"].as_bool(), Some(true));
    assert_eq!(policy["emitPartialNonEmptyResults"].as_bool(), Some(true));
    assert_eq!(addon_requests[0]["addonName"].as_str(), Some("A"));
    assert_eq!(addon_requests[1]["addonName"].as_str(), Some("B"));
    assert!(policy["cloudstreamRequest"].is_object());
}

#[test]
fn stream_discovery_execution_policy_auto_concurrency_uses_addon_count() {
    let policy = stream_discovery_execution_policy_json(
        r#"{
            "type":"movie",
            "id":"tt2",
            "language":"en",
            "addonRequestTimeoutMs":1000,
            "fastAddonRequestTimeoutMs":3000,
            "cloudstreamTimeoutMs":5000,
            "maxConcurrentAddonRequests":0,
            "addons":[
                {"transportUrl":"https://a/manifest.json","manifest":{"id":"a","name":"A","resources":["stream"],"types":["movie"]}},
                {"transportUrl":"https://b/manifest.json","manifest":{"id":"b","name":"B","resources":["stream"],"types":["movie"]}},
                {"transportUrl":"https://c/manifest.json","manifest":{"id":"c","name":"C","resources":["stream"],"types":["movie"]}}
            ],
            "cs3PluginNames":[]
        }"#,
    )
    .expect("policy");
    let policy: Value = serde_json::from_str(&policy).expect("policy json");
    // 3 addons, auto mode → concurrency = min(3, 8) = 3
    assert_eq!(policy["maxConcurrentAddonRequests"].as_i64(), Some(3));
}

#[test]
fn stream_discovery_cache_prefix_is_owned_by_core() {
    assert_eq!(
        stream_discovery_cache_prefix("series", "tt1:1:2", "en"),
        "series|tt1:1:2|en"
    );
}
