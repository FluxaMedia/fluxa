use super::*;

fn parse(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

#[test]
fn direct_calls_match_the_string_route() {
    let item = json!({"id": "tt1", "type": "movie", "name": "A", "poster": "p", "background": "b"});
    let cases = [
        (
            "homeHeroPlan",
            json!({"categories": [{"id": "c", "type": "movie", "items": [item]}], "prefs": {}}),
        ),
        (
            "normalizeLibraryDocument",
            json!({"watchlist": [item], "progress": []}),
        ),
        (
            "buildContinueWatchingFromProgress",
            json!({"tt1": {"timeOffset": 10, "duration": 100, "meta": item}}),
        ),
        (
            "mergeSearchSources",
            json!([{"id": "s", "name": "S", "items": [item]}]),
        ),
        (
            "mergeDiscoverSources",
            json!({"sources": [{"type": "movie", "items": [item, item]}]}),
        ),
    ];
    for (method, args) in cases {
        let direct = call_direct(method, &args).unwrap();
        let routed = route(method, &args.to_string()).ok().unwrap();
        assert_eq!(direct, routed, "{method}");
    }
}

#[test]
fn unknown_method_reports_kind_and_name() {
    let env = parse(&core_invoke("nope.doesNotExist", "{}"));
    assert_eq!(env["ok"], json!(false));
    assert_eq!(env["error"]["kind"], json!("unknown_method"));
    assert_eq!(env["error"]["method"], json!("nope.doesNotExist"));
}

#[test]
fn invalid_args_distinguished_from_empty_result() {
    let bad_json = parse(&core_invoke("identity", "{ not json"));
    assert_eq!(bad_json["error"]["kind"], json!("invalid_args"));

    let missing_field = parse(&core_invoke("identity", "{}"));
    assert_eq!(missing_field["error"]["kind"], json!("invalid_args"));
}

#[test]
fn plugin_stream_results_accept_a_top_level_array() {
    let env = parse(&core_invoke("pluginStreamResultsToStreams", "[]"));
    assert_eq!(env["ok"], json!(true));
    assert_eq!(env["value"][0]["pluginUnavailable"], json!(true));
}

#[test]
fn stateless_helper_returns_ok_value() {
    let env = parse(&core_invoke("parseVideoId", r#"{"id":"tt123:1:2"}"#));
    assert_eq!(env["ok"], json!(true));
    assert_eq!(env["value"]["imdb"], json!("tt123"));
    assert_eq!(env["value"]["isEpisode"], json!(true));
}

#[test]
fn new_sync_and_detection_methods_are_routed() {
    let detect = parse(&core_invoke(
        "detectAnimePlayback",
        r#"{"meta":{"genres":["Anime"]},"episode":null,"stream":null,"addons":[]}"#,
    ));
    assert_eq!(detect["ok"], json!(true));
    assert_eq!(detect["value"]["confidence"], json!(65));

    let sync = parse(&core_invoke(
        "anilistEntriesToSync",
        r#"{"entries":[],"nowMs":0}"#,
    ));
    assert_eq!(sync["ok"], json!(true));
    assert_eq!(sync["value"]["watchlist"], json!([]));

    let merged = parse(&core_invoke(
        "mergeLibraryItemsById",
        r#"{"local":[],"incoming":[{"id":"a"}]}"#,
    ));
    assert_eq!(merged["value"][0]["id"], json!("a"));

    let plan = parse(&core_invoke(
        "tmdbPeopleRequestPlan",
        r#"{"meta":{"id":"tt123","type":"movie"},"apiKey":"k","language":"en"}"#,
    ));
    assert_eq!(
        plan["value"]["findUrl"],
        json!(
            "https://api.themoviedb.org/3/find/tt123?api_key=k&language=en-US&external_source=imdb_id"
        )
    );

    let images = parse(&core_invoke(
        "tmdbPeopleImagesFromCredits",
        r#"{"credits":{"cast":[{"name":"Jane Doe","profile_path":"/x.jpg"}]},"links":[{"name":"jane  doe"}]}"#,
    ));
    assert_eq!(
        images["value"]["jane  doe"],
        json!("https://image.tmdb.org/t/p/w185/x.jpg")
    );

    let upcoming = parse(&core_invoke(
        "releaseDateUpcoming",
        r#"{"released":"2026-09-09T00:00:00Z","todayIso":"2026-09-08"}"#,
    ));
    assert_eq!(upcoming["ok"], json!(true));
    assert_eq!(upcoming["value"], json!(true));

    let recent = parse(&core_invoke(
        "releaseDateRecentlyReleased",
        r#"{"released":"2026-09-01","todayIso":"2026-09-08","windowDays":7}"#,
    ));
    assert_eq!(recent["ok"], json!(true));
    assert_eq!(recent["value"], json!(true));
}

#[test]
fn trailer_selection_methods_are_routed() {
    let video_ids = parse(&core_invoke(
        "trailerYoutubeVideoIds",
        r#"{"urls":["https://youtu.be/abcdefghijk","https://youtube.com/watch?v=abcdefghijk"]}"#,
    ));
    assert_eq!(video_ids["ok"], json!(true));
    assert_eq!(video_ids["value"], json!(["abcdefghijk"]));

    let direct = parse(&core_invoke(
        "trailerDirectSelection",
        r#"{"maxHeight":1080,"trailers":[{"title":"4K","url":"https://video.example/4k.m3u8"},{"title":"1080p","url":"https://video.example/1080.mp4"}]}"#,
    ));
    assert_eq!(direct["ok"], json!(true));
    assert_eq!(
        direct["value"]["url"],
        json!("https://video.example/1080.mp4")
    );
}

#[test]
fn engine_roundtrips_through_the_funnel() {
    let created = parse(&core_invoke("engine.create", "{}"));
    let h = created["value"].as_i64().unwrap();
    assert!(h > 0);

    let snap = parse(&core_invoke("engine.snapshot", &h.to_string()));
    assert_eq!(snap["ok"], json!(true));

    let destroyed = parse(&core_invoke("engine.destroy", &h.to_string()));
    assert_eq!(destroyed["ok"], json!(true));
    assert_eq!(destroyed["value"], json!(true));
}

#[test]
fn lifecycle_create_rejects_malformed_initial_state() {
    for method in ["engine.create", "app.create"] {
        let result = parse(&core_invoke(method, "{ malformed"));
        assert_eq!(result["ok"], json!(false));
        assert_eq!(result["error"]["kind"], json!("invalid_args"));
    }
}

#[test]
fn app_delta_keeps_the_same_wire_value_without_reparsing() {
    let created = parse(&core_invoke("app.create", "{}"));
    let handle = created["value"].as_i64().unwrap();
    let delta = parse(&core_invoke(
        "app.dispatchDelta",
        &format!(r#"{{"handle":{handle},"action":{{"type":"setHomeLoading","value":true}}}}"#),
    ));
    assert_eq!(delta["ok"], json!(true));
    assert_eq!(delta["value"]["patch"]["home"]["isLoading"], json!(true));
    assert_eq!(
        parse(&core_invoke("app.destroy", &handle.to_string()))["value"],
        json!(true)
    );
}

#[test]
fn calendar_plan_methods_route_and_compute() {
    let candidates = parse(&core_invoke(
        "calendarSeasonCandidates",
        r#"{"seasonsCount":10,"lastVideoId":"tt1:2:3"}"#,
    ));
    assert_eq!(candidates["ok"], json!(true));
    assert_eq!(candidates["value"], json!([2, 3, 10]));

    let rows = parse(&core_invoke(
        "calendarWidgetRows",
        r#"{"items":[{"dateIso":"2026-07-18","title":"Show","seasonNumber":1,"episodeNumber":2}],"maxRows":4}"#,
    ));
    assert_eq!(rows["value"][0]["episodeText"], json!("S1E2"));

    let content = parse(&core_invoke(
        "calendarContentPlan",
        r#"{"items":[{"dateIso":"2026-07-18","metaId":"tt1","title":"Show"}],"monthPrefix":"2026-07"}"#,
    ));
    assert_eq!(content["value"][0]["metaId"], json!("tt1"));

    let notifications = parse(&core_invoke(
        "calendarNotificationContent",
        r#"{"items":[{"dateIso":"2026-07-18","metaId":"tt1","metaType":"series","title":"Show","seasonNumber":1,"episodeNumber":1}],"todayIso":"2026-07-18","alreadyNotifiedKeys":[]}"#,
    ));
    assert_eq!(
        notifications["value"]["items"][0]["titleKey"],
        json!("notification.new_season_released")
    );
    assert_eq!(notifications["value"]["keys"].as_array().unwrap().len(), 1);

    let released = parse(&core_invoke(
        "calendarReleaseDetection",
        r#"{"items":[{"dateIso":"2026-07-18"},{"dateIso":"2026-07-19"}],"todayIso":"2026-07-18"}"#,
    ));
    assert_eq!(released["value"].as_array().unwrap().len(), 1);
}

#[test]
fn newly_routed_modules_compute() {
    let input_type = parse(&core_invoke(
        "addonStoreInputType",
        r#"{"input":"https://example.com/addon/manifest.json"}"#,
    ));
    assert_eq!(input_type["value"], json!("stremio_manifest"));

    let secure = parse(&core_invoke(
        "isSecureRemoteUrl",
        r#"{"url":"http://example.com"}"#,
    ));
    assert_eq!(secure["value"], json!(false));

    let same = parse(&core_invoke(
        "samePluginRepositoryUrl",
        r#"{"left":"https://Example.com/repo/","right":"http://example.com/repo"}"#,
    ));
    assert_eq!(same["value"], json!(true));

    let buffer = parse(&core_invoke("safePlayerBufferCacheMb", r#"{"value":50}"#));
    assert_eq!(buffer["value"], json!(100));

    let dv_mode = parse(&core_invoke(
        "safeDolbyVisionFallbackMode",
        r#"{"mode":"dv8"}"#,
    ));
    assert_eq!(dv_mode["value"], json!("dv8"));

    let source_mode = parse(&core_invoke(
        "safeStreamSourceSelectionMode",
        r#"{"mode":"regex"}"#,
    ));
    assert_eq!(source_mode["value"], json!("regex"));

    let policy = parse(&core_invoke("directPlaybackPolicy", "{}"));
    assert_eq!(policy["value"]["metaDetailTimeoutMs"], json!(3500));

    let prefix = parse(&core_invoke(
        "streamDiscoveryCachePrefix",
        r#"{"contentType":"movie","id":"tt1","language":"en"}"#,
    ));
    assert_eq!(prefix["value"], json!("movie|tt1|en"));
}

#[test]
fn gap_filled_routes_compute() {
    let bearer = parse(&core_invoke("traktBearer", r#"{"token":"abc"}"#));
    assert_eq!(bearer["value"], json!("Bearer abc"));

    let has_client = parse(&core_invoke("traktHasClient", r#"{"apiKey":""}"#));
    assert_eq!(has_client["value"], json!(false));

    let expires_at = parse(&core_invoke(
        "traktTokenExpiresAt",
        r#"{"createdAtSeconds":1000,"expiresInSeconds":7200}"#,
    ));
    assert_eq!(expires_at["value"], json!(1000 + 6900));

    let show_id = parse(&core_invoke(
        "traktShowIdFromEpisodeId",
        r#"{"videoId":"tt1:2:3"}"#,
    ));
    assert_eq!(show_id["value"], json!("tt1"));

    let episode_matches = parse(&core_invoke(
        "episodeTextMatches",
        r#"{"text":"Show S01E02","season":1,"episode":2}"#,
    ));
    assert_eq!(episode_matches["value"], json!(true));

    let stream_matches = parse(&core_invoke(
        "streamMatchesEpisode",
        r#"{"videoId":"tt1:1:2","title":"","name":"","description":"","filename":"Show.S01E02.mkv","effectiveFilename":""}"#,
    ));
    assert_eq!(stream_matches["value"], json!(true));

    let content_type = parse(&core_invoke("normalizeContentType", r#"{"value":"tv"}"#));
    assert_eq!(content_type["value"], json!("series"));

    let feed_part = parse(&core_invoke("stableFeedPart", r#"{"value":"Foo Bar!"}"#));
    assert_eq!(feed_part["value"], json!("foo_bar"));

    let base = parse(&core_invoke(
        "baseUrl",
        r#"{"url":"https://example.com/addon/manifest.json"}"#,
    ));
    assert_eq!(base["value"], json!("https://example.com/addon/"));

    let progress = parse(&core_invoke(
        "playerProgressPercent",
        r#"{"positionMs":50,"durationMs":100}"#,
    ));
    assert_eq!(progress["value"], json!(50.0));

    let should_save = parse(&core_invoke(
        "playerShouldSaveOnDispose",
        r#"{"positionMs":6000}"#,
    ));
    assert_eq!(should_save["value"], json!(true));

    let category_json =
        r#"{\"id\":\"a\",\"name\":\"A\",\"type\":\"movie\",\"items\":[{\"id\":\"tt1\"}]}"#;
    let overlap = parse(&core_invoke(
        "homeOverlapRatio",
        &format!(r#"{{"firstJson":"{category_json}","secondJson":"{category_json}"}}"#),
    ));
    assert_eq!(overlap["value"], json!(1.0));

    let select = parse(&core_invoke(
        "selectStreamIndex",
        r#"{"streamsJson":"[]","currentVideoId":"tt1","initialStreamIndex":0,"sourceSelectionMode":"manual"}"#,
    ));
    assert_eq!(select["value"], json!(-1));

    let ids = parse(&core_invoke(
        "streamRequestIds",
        r#"{"contentType":"movie","id":"tt1"}"#,
    ));
    assert_eq!(ids["value"], json!(["tt1"]));
}

#[test]
fn last_gap_filled_routes_compute() {
    let locator = parse(&core_invoke(
        "parseEpisodeLocator",
        r#"{"input":"tt1:2:3"}"#,
    ));
    assert_eq!(locator["value"]["baseId"], json!("tt1"));
    assert_eq!(locator["value"]["season"], json!(2));
    assert_eq!(locator["value"]["episode"], json!(3));

    let no_locator = parse(&core_invoke("parseEpisodeLocator", r#"{"input":"nope"}"#));
    assert_eq!(no_locator["value"], Value::Null);

    let audio = parse(&core_invoke(
        "resolvePreferredAudioLanguage",
        r#"{"lastAudioLanguage":null,"preferredAudioLanguage":"en","originalLanguage":"ja"}"#,
    ));
    assert_eq!(audio["value"], json!("ja"));

    let subtitle_match = parse(&core_invoke(
        "subtitleLanguageMatches",
        r#"{"label":"english","language":null,"preferredLanguage":"en"}"#,
    ));
    assert_eq!(subtitle_match["value"], json!(true));

    let toggled = parse(&core_invoke(
        "toggleMetadataFeed",
        r#"{"selectedKeys":"[]","availableKeys":"[\"a\"]","key":"a"}"#,
    ));
    assert_eq!(toggled["value"], json!(["a"]));

    let manifest_request = json!({
        "body": json!({"resources": ["catalog"], "types": ["movie"]}).to_string(),
        "transportUrl": "https://example.com/manifest.json",
        "unknownName": "Unknown Addon"
    });
    let manifest = parse(&core_invoke("parseManifest", &manifest_request.to_string()));
    assert_eq!(manifest["ok"], json!(true));
    assert_eq!(
        manifest["value"]["manifest"]["name"],
        json!("Unknown Addon")
    );
}

// Renaming or removing a routed method must show up as a diff in this fixture.
#[test]
fn every_known_core_invoke_method_still_routes() {
    let fixture_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/wire/core_invoke_methods.txt");
    let methods = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("missing fixture {fixture_path:?}"));
    let methods: Vec<&str> = methods
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert!(!methods.is_empty(), "fixture list must not be empty");

    for method in methods {
        let result = parse(&core_invoke(method, "{}"));
        let kind = result["error"]["kind"].as_str().unwrap_or("");
        assert_ne!(
            kind, "unknown_method",
            "{method} no longer routes anywhere — renamed or removed?"
        );
    }
}
