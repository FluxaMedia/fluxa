mod ownership;
mod entries;
mod profile_sanitize;
mod repo_url;
mod search_policy;

pub(crate) use ownership::{
    effective_addons_owner_id_json, effective_plugins_owner_id_json, plugin_storage_fallback_json,
    profile_local_addons_key_json,
};
pub(crate) use entries::addon_store_entries_plan_json;
pub(crate) use profile_sanitize::{addon_profile_mutation_plan_json, sanitize_profile_json};
pub(crate) use repo_url::{
    addon_store_input_type, is_secure_remote_url, normalize_cloudstream_repo_input,
    normalize_cloudstream_repo_url,
    normalize_plugin_repository_url, same_plugin_repository_url,
};
pub(crate) use search_policy::{
    addon_store_search_policy_json, extract_addon_manifest_url, filter_enabled_addons_json,
};

#[cfg(test)]
mod tests {
    use super::ownership::effective_shared_owner_id;
    use super::*;
    use serde_json::Value;

    #[test]
    fn detects_manifest_before_generic_https_repo_rule() {
        assert_eq!(
            "stremio_manifest",
            addon_store_input_type("https://addon.example/manifest.json")
        );
        assert_eq!(
            "cloudstream_repo",
            addon_store_input_type("cloudstreamrepo://example.com/repo.json")
        );
        assert_eq!("search_query", addon_store_input_type("cinemeta"));
    }

    #[test]
    fn plans_search_url_and_cache_use() {
        let json = addon_store_search_policy_json(
            r#"{"query":"Game of Thrones","nowMillis":2000,"cachedAtMillis":1500,"ttlMillis":1000}"#,
        )
        .unwrap();
        assert!(json.contains(r#""normalizedQuery":"game of thrones""#));
        assert!(
            json.contains(r#""url":"https://stremio-addons.net/addons?query=game+of+thrones""#)
        );
        assert!(json.contains(r#""useCache":true"#));
    }

    #[test]
    fn extracts_escaped_manifest_url_from_detail_page() {
        assert_eq!(
            Some("https://addon.example/root/manifest.json?x=1&y=2".to_string()),
            extract_addon_manifest_url(
                r#"<script>"https://addon.example\/root\/manifest.json?x=1\u0026y=2"</script>"#,
            )
        );
    }

    #[test]
    fn plugin_repository_url_policy_normalizes_and_requires_https() {
        assert_eq!(
            normalize_plugin_repository_url("cloudstream://example.com/repo.json"),
            "https://example.com/repo.json"
        );
        assert_eq!(
            normalize_plugin_repository_url("http://example.com/repo.json"),
            "https://example.com/repo.json"
        );
        assert!(is_secure_remote_url("https://example.com/repo.json"));
        assert!(!is_secure_remote_url("http://example.com/repo.json"));
        assert!(same_plugin_repository_url(
            "http://example.com/repo.json/",
            "https://EXAMPLE.com/repo.json"
        ));
    }

    #[test]
    fn cloudstream_repository_input_expands_github_shortcuts() {
        assert_eq!(
            normalize_cloudstream_repo_input("owner/repository"),
            "https://raw.githubusercontent.com/owner/repository/builds"
        );
        assert_eq!(
            normalize_cloudstream_repo_input("github.com/owner/repository"),
            "https://github.com/owner/repository"
        );
        assert_eq!(
            normalize_cloudstream_repo_input("cloudstream://example.test/repo.json"),
            "https://example.test/repo.json"
        );
    }

    #[test]
    fn sanitize_profile_merges_and_deduplicates_local_addons() {
        let sanitized = sanitize_profile_json(
            r#"{"id":"p1","email":"u@example.com","localAddons":["http://a.example/manifest.json"],"disabledLocalAddons":["https://b.example/manifest.json","https://missing.example/manifest.json"],"language":"tr"}"#,
            r#"["https://a.example/manifest.json","https://b.example/manifest.json"]"#,
            true,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("profile");

        assert_eq!(
            sanitized
                .get("localAddons")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
        assert_eq!(
            sanitized
                .get("disabledLocalAddons")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(
            sanitized
                .get("appearanceSettings")
                .and_then(|value| value.get("language"))
                .and_then(Value::as_str),
            Some("tr")
        );
    }

    #[test]
    fn effective_owner_id_falls_back_to_primary_only_when_flag_set() {
        let profiles = r#"[{"id":"p1"},{"id":"p2","usesPrimaryAddons":true},{"id":"p3"}]"#;
        assert_eq!(
            effective_shared_owner_id(profiles, "p1", "usesPrimaryAddons"),
            Some("p1".to_string())
        );
        assert_eq!(
            effective_shared_owner_id(profiles, "p2", "usesPrimaryAddons"),
            Some("p1".to_string())
        );
        assert_eq!(
            effective_shared_owner_id(profiles, "p3", "usesPrimaryAddons"),
            Some("p3".to_string())
        );
    }

    #[test]
    fn plugin_storage_fallback_prefers_scoped_over_legacy() {
        let result = plugin_storage_fallback_json(
            r#"{"scopedRepositoryUrls":["https://a.example/repo.json"],"legacyRepositoryUrls":["https://legacy.example/repo.json"],"scopedScraperOverrides":null,"legacyScraperOverrides":{"s1":true}}"#,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("result");

        assert_eq!(
            result["repositoryUrls"],
            serde_json::json!(["https://a.example/repo.json"])
        );
        assert_eq!(result["scraperOverrides"], serde_json::json!({"s1": true}));
    }

    #[test]
    fn addon_profile_mutation_plan_owns_install_disable_remove_and_order() {
        let profile = r#"{
            "id":"p1",
            "localAddons":["https://a.example/manifest.json","https://b.example/manifest.json"],
            "disabledLocalAddons":[]
        }"#;
        let disabled = addon_profile_mutation_plan_json(&format!(
            r#"{{"profile":{profile},"command":"disable","addonKey":"http://a.example"}}"#
        ))
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("disabled profile");
        assert_eq!(disabled["disabledLocalAddons"][0], "https://a.example/manifest.json");

        let moved = addon_profile_mutation_plan_json(&format!(
            r#"{{"profile":{},"command":"move","addonKey":"https://b.example","direction":-1}}"#,
            disabled
        ))
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("moved profile");
        assert_eq!(moved["localAddons"][0], "https://b.example/manifest.json");

        let removed = addon_profile_mutation_plan_json(&format!(
            r#"{{"profile":{},"command":"remove","addonKey":"https://a.example"}}"#,
            moved
        ))
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("removed profile");
        assert_eq!(removed["localAddons"].as_array().map(Vec::len), Some(1));
        assert!(removed["disabledLocalAddons"].as_array().unwrap().is_empty());
    }

    #[test]
    fn sanitize_profile_syncs_home_feed_settings_from_top_level_fields() {
        let sanitized = sanitize_profile_json(
            r#"{"id":"p1","email":"u@example.com","localAddons":["https://a.example/manifest.json"],"libraryCollections":[{"id":"new","title":"New"}],"homeFeedSettings":{"libraryCollections":[{"id":"old","title":"Old"}],"homeFeedToggles":["old"]},"homeFeedToggles":[]}"#,
            r#"[]"#,
            false,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("profile");

        assert_eq!(
            sanitized["homeFeedSettings"]["libraryCollections"][0]["id"],
            "new"
        );
        assert_eq!(
            sanitized["homeFeedSettings"]["homeFeedToggles"]
                .as_array()
                .map(Vec::len),
            Some(0)
        );
    }

    #[test]
    fn addon_entries_plan_merges_repository_and_local_state() {
        let result = addon_store_entries_plan_json(
            r#"{"repositoryAddons":[{"transportUrl":"https://a.example/manifest.json","manifest":{"name":"A","description":"Desc","configurable":true},"isManaged":true,"isEnabled":true}],"localUrls":["https://a.example","https://b.example/manifest.json"],"disabledKeys":["https://a.example"],"isNuvioProfile":false,"localLoaded":true,"refreshingUrl":"https://b.example"}"#,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("entries");

        assert_eq!(result.as_array().map(Vec::len), Some(2));
        assert_eq!(result[0]["name"], "A");
        assert_eq!(result[0]["isEnabled"], false);
        assert_eq!(result[0]["canRemove"], true);
        assert_eq!(result[1]["isRefreshing"], true);
        assert_eq!(result[1]["name"], "B");
    }
}
