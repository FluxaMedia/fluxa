use super::*;

#[test]
fn reducer_updates_home_search_state_without_reordering_payloads() {
    let handle = create_app_core_state(
        r#"{"home":{"categories":[{"id":"c1"}]},"homeSearch":{"searchHistory":[{"id":"tt1"}]}}"#,
    );

    let snapshot = app_core_dispatch_json(
        handle,
        r#"{"type":"setSearchResults","value":[{"id":"tt2"},{"id":"tt1"}]}"#,
    )
    .unwrap();
    let value: Value = serde_json::from_str(&snapshot).unwrap();

    assert_eq!(
        value["homeSearch"]["searchResults"],
        json!([{"id":"tt2"},{"id":"tt1"}])
    );
    assert_eq!(value["home"]["categories"], json!([{"id":"c1"}]));
    assert_eq!(value["homeSearch"]["searchHistory"], json!([{"id":"tt1"}]));
    assert!(destroy_app_core_state(handle));
}

#[test]
fn reducer_owns_home_shell_state() {
    let handle = create_app_core_state("{}");

    let snapshot = app_core_dispatch_json(
        handle,
        r#"{"type":"setHomeCategories","value":[{"id":"continue_watching"},{"id":"popular"}]}"#,
    )
    .unwrap();
    let value: Value = serde_json::from_str(&snapshot).unwrap();
    assert_eq!(
        value["home"]["categories"],
        json!([{"id":"continue_watching"},{"id":"popular"}])
    );

    let snapshot = app_core_dispatch_json(
        handle,
        r#"{"type":"setHomeCurrentFilter","value":"movies"}"#,
    )
    .unwrap();
    let value: Value = serde_json::from_str(&snapshot).unwrap();
    assert_eq!(value["home"]["currentFilter"], json!("movies"));

    let snapshot =
        app_core_dispatch_json(handle, r#"{"type":"setHomeLoading","value":true}"#).unwrap();
    let value: Value = serde_json::from_str(&snapshot).unwrap();
    assert_eq!(value["home"]["isLoading"], json!(true));
    assert_eq!(value["home"]["currentFilter"], json!("movies"));
    assert!(destroy_app_core_state(handle));
}

#[test]
fn reducer_owns_home_feature_state_branches() {
    let handle = create_app_core_state("{}");

    let snapshot =
        app_core_dispatch_json(handle, r#"{"type":"setBillboardIndex","value":2}"#).unwrap();
    let value: Value = serde_json::from_str(&snapshot).unwrap();
    assert_eq!(value["billboard"]["index"], json!(2));

    let snapshot = app_core_dispatch_json(
        handle,
        r#"{"type":"setDiscoverResults","value":[{"id":"tt1"},{"id":"tt2"}]}"#,
    )
    .unwrap();
    let value: Value = serde_json::from_str(&snapshot).unwrap();
    assert_eq!(
        value["discover"]["results"],
        json!([{"id":"tt1"},{"id":"tt2"}])
    );

    let snapshot = app_core_dispatch_json(
        handle,
        r#"{"type":"setCalendarItems","value":[{"title":"Episode"}]}"#,
    )
    .unwrap();
    let value: Value = serde_json::from_str(&snapshot).unwrap();
    assert_eq!(value["calendar"]["items"], json!([{"title":"Episode"}]));

    let snapshot = app_core_dispatch_json(
        handle,
        r#"{"type":"setLibraryUiState","value":{"isLoading":false,"lastLoadedProfileKey":"profile"}}"#,
    )
    .unwrap();
    let value: Value = serde_json::from_str(&snapshot).unwrap();
    assert_eq!(
        value["library"]["uiState"]["lastLoadedProfileKey"],
        json!("profile")
    );
    assert!(destroy_app_core_state(handle));
}

#[test]
fn reducer_resets_player_episode_state_like_kotlin_state_holder() {
    let handle = create_app_core_state(
        r#"{"player":{"currentStreamIndex":3,"lastSavedPosition":9200,"playbackEnded":true,"hasStartedPlaying":true,"isVideoRendered":true,"isBuffering":false}}"#,
    );

    let snapshot = app_core_dispatch_json(
        handle,
        r#"{"type":"playerResetForEpisode","videoId":"tt123:1:2"}"#,
    )
    .unwrap();
    let value: Value = serde_json::from_str(&snapshot).unwrap();

    assert_eq!(value["player"]["currentVideoId"], json!("tt123:1:2"));
    assert_eq!(value["player"]["currentStreamIndex"], json!(0));
    assert_eq!(value["player"]["lastSavedPosition"], json!(0));
    assert_eq!(value["player"]["shouldApplyInitialProgress"], json!(false));
    assert_eq!(value["player"]["playbackEnded"], json!(false));
    assert_eq!(value["player"]["hasStartedPlaying"], json!(false));
    assert_eq!(value["player"]["isVideoRendered"], json!(false));
    assert_eq!(value["player"]["isBuffering"], json!(true));
    assert!(destroy_app_core_state(handle));
}

#[test]
fn primitive_player_updates_avoid_json_dispatch() {
    let handle = create_app_core_state("{}");
    assert!(app_core_update_player(
        handle, 12_345, 2, false, true, true, true
    ));
    let value: Value = serde_json::from_str(&app_core_state_json(handle).unwrap()).unwrap();
    assert_eq!(value["player"]["lastSavedPosition"], json!(12_345));
    assert_eq!(value["player"]["isBuffering"], json!(false));
    assert_eq!(value["player"]["currentStreamIndex"], json!(2));
    assert_eq!(value["player"]["playbackEnded"], json!(true));
    assert_eq!(value["player"]["isVideoRendered"], json!(true));
    assert_eq!(value["player"]["hasStartedPlaying"], json!(true));
    assert!(destroy_app_core_state(handle));
}

#[test]
fn delta_dispatch_returns_the_small_action_payload() {
    let handle = create_app_core_state("{}");
    let delta =
        app_core_dispatch_delta_json(handle, r#"{"type":"setHomeLoading","value":true}"#).unwrap();
    assert_eq!(delta, r#"{"patch":{"home":{"isLoading":true}}}"#);
    assert!(destroy_app_core_state(handle));
}
