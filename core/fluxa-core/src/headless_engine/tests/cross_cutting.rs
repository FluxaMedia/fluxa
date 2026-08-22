use super::super::*;
use serde_json::Value;

#[test]
fn detail_player_sync_auth_settings_calendar_and_offline_are_core_actions() {
    let handle = create_headless_engine(r#"{"profile":{"activeProfileId":"p1"}}"#);

    let season: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"detailSeasonRequested","seriesId":"tt1","season":2}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(season["effects"][0]["type"], "fetchSeasonEpisodes");

    let subtitles: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"subtitleLoadRequested","stream":{"url":"http://a"},"contentType":"movie","id":"tt1","extraArgs":"videoHash=abc"}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(subtitles["effects"][0]["type"], "fetchSubtitles");
    assert_eq!(
        subtitles["effects"][0]["payload"]["extraArgs"],
        "videoHash=abc"
    );

    let sync: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"externalSyncRequested","provider":"trakt","language":"tr"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(sync["effects"][0]["type"], "runExternalSync");
    assert_eq!(sync["effects"][0]["payload"]["profileId"], "p1");

    let auth: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"authFlowRequested","provider":"trakt","mode":"deviceCode"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(auth["effects"][0]["type"], "runAuthFlow");

    let settings: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"settingsChanged","key":"language","value":"tr"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(settings["state"]["settings"]["values"]["language"], "tr");
    assert_eq!(settings["effects"][0]["type"], "writeSettings");

    let calendar: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"calendarMonthRequested","year":2026,"month":20}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(calendar["effects"][0]["type"], "readCalendarMonth");
    assert_eq!(calendar["effects"][0]["payload"]["month"], 12);

    let offline: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"offlineDownloadRequested","meta":{"id":"tt1"},"stream":{"url":"http://a"},"videoId":"tt1"}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(offline["effects"][0]["type"], "enqueueOfflineDownload");
    assert_eq!(offline["effects"][0]["payload"]["meta"]["id"], "tt1");
    assert!(destroy_headless_engine(handle));
}
