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

#[test]
fn welcome_is_skipped_once_a_profile_exists_or_offline_was_chosen() {
    assert!(welcome_required(&json!({"profiles": []})));
    assert!(!welcome_required(&json!({"profiles": [{"id": "p1"}]})));
    assert!(!welcome_required(
        &json!({"profiles": [], "offlineChosen": true})
    ));
}

#[test]
fn nuvio_sign_in_profile_keeps_tokens_and_leaves_the_name_for_sync() {
    let profile: Value = serde_json::from_str(
        &nuvio_session_profile_json(
            &json!({
                "id": "p1",
                "email": " me@example.com ",
                "nowSeconds": 1000,
                "session": {
                    "access_token": "a",
                    "refresh_token": "r",
                    "expires_in": 60,
                    "user": {"id": "u-1"},
                },
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(profile["nuvioTokenExpiresAt"], 1060);
    assert_eq!(profile["nuvioUserId"], "u-1");
    assert_eq!(profile["nuvioEmail"], "me@example.com");
    assert_eq!(profile["name"], "");
}

#[test]
fn qr_login_without_user_object_takes_identity_from_the_token() {
    let payload = "eyJzdWIiOiJ1LTIiLCJlbWFpbCI6InFyQGV4YW1wbGUuY29tIn0";
    let profile: Value = serde_json::from_str(
        &nuvio_session_profile_json(
            &json!({
                "id": "p1",
                "email": "",
                "nowSeconds": 1000,
                "session": {"access_token": format!("h.{payload}.s")},
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(profile["nuvioUserId"], "u-2");
    assert_eq!(profile["nuvioEmail"], "qr@example.com");
}

#[test]
fn nuvio_auth_errors_map_to_translation_keys() {
    let key = |status, body: &str| nuvio_auth_error_key(&json!({"status": status, "body": body}));
    assert_eq!(key(400, "Invalid login credentials"), "auth.error.invalid_credentials");
    assert_eq!(key(400, "email_not_confirmed"), "auth.error.email_not_confirmed");
    assert_eq!(key(429, ""), "auth.error.rate_limited");
    assert_eq!(key(503, ""), "auth.error.server");
    assert_eq!(key(0, ""), "auth.error.network");
}
