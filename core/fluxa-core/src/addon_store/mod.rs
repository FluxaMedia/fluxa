mod ownership;
mod profile_sanitize;
mod repo_url;
mod search_policy;

pub(crate) use ownership::{
    effective_addons_owner_id_json, effective_plugins_owner_id_json, plugin_storage_fallback_json,
    profile_local_addons_key_json,
};
pub(crate) use profile_sanitize::{addon_profile_mutation_plan_json, sanitize_profile_json};
pub(crate) use repo_url::{
    addon_store_input_type, is_secure_remote_url, normalize_cloudstream_repo_url,
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
}
