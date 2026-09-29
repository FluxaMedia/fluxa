use super::*;
use serde_json::Value;

#[test]
fn nuvio_snapshots_are_scoped_to_account_and_profile() {
    let source = |profile: serde_json::Value| {
        serde_json::from_str::<Value>(
            &account_source_json(&json!({"profile": profile, "entity": "addons"}).to_string())
                .unwrap(),
        )
        .unwrap()
    };
    let nuvio =
        source(json!({"nuvioAccessToken": "t", "nuvioUserId": "u-1", "nuvioProfileIndex": 2}));
    assert_eq!(nuvio["backend"], "nuvio");
    assert_eq!(nuvio["snapshotKey"], "remote_addons_nuvio_u_1_2");
    assert_eq!(source(json!({"nuvioUserId": "u-1"}))["backend"], "local");
}

#[test]
fn active_profile_plan_returns_first_when_no_stored_id() {
    let result: Value = serde_json::from_str(
        &active_profile_plan_json(r#"{"profiles":[{"id":"p1"},{"id":"p2"}]}"#).unwrap(),
    )
    .unwrap();
    assert_eq!(result["activeId"], "p1");
    assert_eq!(result["shouldCreateDefault"], false);
}

#[test]
fn primary_profile_id_returns_first_profile() {
    assert_eq!(
        primary_profile_id_json(r#"[{"id":"p1"},{"id":"p2"}]"#),
        Some("p1".to_string())
    );
    assert_eq!(primary_profile_id_json(r#"[]"#), None);
}

#[test]
fn active_profile_plan_creates_default_when_profiles_empty() {
    let result: Value =
        serde_json::from_str(&active_profile_plan_json(r#"{"profiles":[]}"#).unwrap()).unwrap();
    assert_eq!(result["activeId"], "guest");
    assert_eq!(result["shouldCreateDefault"], true);
}

#[test]
fn active_profile_plan_selects_stored_id() {
    let result: Value = serde_json::from_str(
        &active_profile_plan_json(
            r#"{"profiles":[{"id":"p1"},{"id":"p2"}],"storedActiveId":"p2"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(result["activeId"], "p2");
}

#[test]
fn token_merge_plan_merges_trakt_tokens_into_profile() {
    let result: Value = serde_json::from_str(
        &token_merge_plan_json(
            r#"{"profile":{"id":"p1","email":"u@example.com"},"authResult":{"accessToken":"tok","refreshToken":"ref","expiresAt":999},"provider":"trakt"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(result["mergedProfile"]["traktAccessToken"], "tok");
    assert_eq!(result["mergedProfile"]["traktRefreshToken"], "ref");
    assert_eq!(result["mergedProfile"]["traktTokenExpiresAt"], 999);
}

#[test]
fn profile_sync_merge_plan_keeps_local_edits_and_applies_remote_changes() {
    let result: Value = serde_json::from_str(
        &profile_sync_merge_plan_json(
            r#"{"base":{"id":"p1","language":"en","cardLayout":"vertical"},"updated":{"id":"p1","language":"tr","cardLayout":"vertical"},"current":{"id":"p1","language":"en","cardLayout":"horizontal"}}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(result["language"], "tr");
    assert_eq!(result["cardLayout"], "horizontal");
}

#[test]
fn settings_migration_flattens_nested_external_accounts() {
    let result: Value = serde_json::from_str(
        &profile_settings_migration_plan_json(
            r#"{"raw":{"id":"p1","externalAccounts":{"traktAccessToken":"tok"}},"schemaVersion":1}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(result["migratedProfile"]["traktAccessToken"], "tok");
    assert!(
        result["appliedMigrations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m == "flatten_external_accounts")
    );
}

#[test]
fn settings_migration_keeps_nested_library_collections_when_top_level_is_empty() {
    let result: Value = serde_json::from_str(
        &profile_settings_migration_plan_json(
            r#"{"raw":{"id":"p1","libraryCollections":[],"homeFeedSettings":{"libraryCollections":[{"id":"c1","title":"Collection"}]}},"schemaVersion":1}"#,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        result["migratedProfile"]["libraryCollections"][0]["id"],
        "c1"
    );
}

#[test]
fn default_seed_produces_guest_profile_with_default_addon() {
    let result: Value = serde_json::from_str(&profile_default_seed_json("{}").unwrap()).unwrap();
    assert_eq!(result["id"], "guest");
    assert_eq!(result["isGuest"], true);
    let addons = result["localAddons"].as_array().unwrap();
    assert!(!addons.is_empty());
}

#[test]
fn settings_migration_preserves_an_explicitly_empty_addon_list() {
    let result: Value = serde_json::from_str(
        &profile_settings_migration_plan_json(
            r#"{"raw":{"id":"p1","localAddons":[]},"schemaVersion":2}"#,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        result["migratedProfile"]["localAddons"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );
}
