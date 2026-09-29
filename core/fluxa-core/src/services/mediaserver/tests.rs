use super::*;

fn server(kind: &str) -> Value {
    json!({"baseUrl": "https://m.example/", "token": "tok", "userId": "u1", "deviceId": "dev", "key": "srv"})
        .as_object()
        .map(|server| json!({"kind": kind, "server": server}))
        .unwrap()
}

fn request(kind: &str, operation: &str, params: Value) -> Value {
    let mut args = server(kind);
    args["operation"] = json!(operation);
    args["params"] = params;
    serde_json::from_str(&request_json(&args.to_string()).expect("plan")).unwrap()
}

fn parse(kind: &str, operation: &str, body: Value) -> Value {
    let mut args = server(kind);
    args["operation"] = json!(operation);
    args["status"] = json!(200);
    args["body"] = body;
    serde_json::from_str(&parse_json(&args.to_string()).expect("parsed")).unwrap()
}

#[test]
fn jellyfin_sends_token_in_authorization_and_emby_in_its_own_header() {
    let jellyfin = request("jellyfin", "libraries", json!({}));
    assert_eq!(jellyfin["url"], "https://m.example/Users/u1/Views");
    assert!(
        jellyfin["headers"]["Authorization"]
            .as_str()
            .unwrap()
            .contains("Token=\"tok\"")
    );
    let emby = request("emby", "libraries", json!({}));
    assert_eq!(emby["headers"]["X-Emby-Token"], "tok");
    assert!(emby["headers"].get("Authorization").is_none());
}

#[test]
fn jellyfin_login_has_no_token_and_parses_auth() {
    let mut args = server("jellyfin");
    args["server"]["token"] = json!("");
    args["operation"] = json!("login");
    args["params"] = json!({"username": "a", "password": "b"});
    let plan: Value = serde_json::from_str(&request_json(&args.to_string()).unwrap()).unwrap();
    assert_eq!(plan["body"]["Pw"], "b");
    assert!(
        !plan["headers"]["Authorization"]
            .as_str()
            .unwrap()
            .contains("Token=")
    );
    let parsed = parse(
        "jellyfin",
        "login",
        json!({"AccessToken": "t", "ServerId": "s", "User": {"Id": "u", "Name": "a"}}),
    );
    assert_eq!(parsed["auth"]["userId"], "u");
}

#[test]
fn lookup_by_imdb_uses_provider_id_filter() {
    let plan = request(
        "jellyfin",
        "lookup",
        json!({"externalId": "tt42", "kind": "movie"}),
    );
    let url = plan["url"].as_str().unwrap();
    assert!(url.contains("AnyProviderIdEquals=imdb.tt42"));
    assert!(url.contains("IncludeItemTypes=Movie"));
}

#[test]
fn jellyfin_library_items_become_metas_with_external_ids() {
    let parsed = parse(
        "jellyfin",
        "catalog",
        json!({"Items": [{
            "Id": "i1", "Type": "Movie", "Name": "Film", "ProductionYear": 2020,
            "ImageTags": {"Primary": "pt"}, "ProviderIds": {"Imdb": "tt9"},
            "RunTimeTicks": 72_000_000_000i64
        }]}),
    );
    let meta = &parsed["metas"][0];
    assert_eq!(meta["id"], "ms:srv:i1");
    assert_eq!(meta["type"], "movie");
    assert_eq!(meta["imdb_id"], "tt9");
    assert_eq!(meta["runtime"], "120 min");
    assert!(
        meta["poster"]
            .as_str()
            .unwrap()
            .contains("/Items/i1/Images/Primary?tag=pt")
    );
}

#[test]
fn jellyfin_streams_offer_direct_play_transcode_and_text_subtitles() {
    let parsed = parse(
        "jellyfin",
        "streams",
        json!({"Id": "i1", "MediaSources": [{
            "Id": "ms1", "Name": "Film", "Size": 4_500_000_000i64, "Path": "/media/Film.mkv",
            "MediaStreams": [
                {"Type": "Video", "Height": 1080, "Codec": "hevc"},
                {"Type": "Audio", "Codec": "eac3", "ChannelLayout": "5.1"},
                {"Type": "Subtitle", "Index": 3, "Language": "eng", "IsTextSubtitleStream": true},
                {"Type": "Subtitle", "Index": 4, "Language": "jpn", "IsTextSubtitleStream": false}
            ]
        }]}),
    );
    let streams = parsed["streams"].as_array().unwrap();
    assert_eq!(streams.len(), 2);
    assert!(
        streams[0]["url"]
            .as_str()
            .unwrap()
            .contains("/Videos/i1/stream?static=true")
    );
    assert_eq!(streams[0]["title"], "1080p · HEVC · eac3 5.1 · 4.5 GB");
    assert_eq!(streams[0]["behaviorHints"]["filename"], "Film.mkv");
    assert_eq!(streams[0]["subtitles"].as_array().unwrap().len(), 1);
    assert!(streams[1]["url"].as_str().unwrap().contains("master.m3u8"));
}

#[test]
fn jellyfin_episodes_carry_season_and_episode_numbers() {
    let parsed = parse(
        "jellyfin",
        "episodes",
        json!({"Items": [{"Id": "e1", "Name": "Pilot", "ParentIndexNumber": 1, "IndexNumber": 2}]}),
    );
    assert_eq!(parsed["videos"][0]["season"], 1);
    assert_eq!(parsed["videos"][0]["episode"], 2);
    assert_eq!(parsed["videos"][0]["id"], "ms:srv:e1");
}

#[test]
fn playback_progress_converts_milliseconds_to_ticks() {
    let plan = request(
        "jellyfin",
        "progress",
        json!({"action": "pause", "itemId": "i1", "mediaSourceId": "m", "positionMs": 1500}),
    );
    assert_eq!(plan["url"], "https://m.example/Sessions/Playing/Progress");
    assert_eq!(plan["body"]["PositionTicks"], 15_000_000);
    assert_eq!(plan["body"]["IsPaused"], true);
    let stop = request(
        "jellyfin",
        "progress",
        json!({"action": "stop", "itemId": "i1"}),
    );
    assert!(
        stop["url"]
            .as_str()
            .unwrap()
            .ends_with("/Sessions/Playing/Stopped")
    );
}

#[test]
fn unwatching_uses_delete_on_jellyfin_and_unscrobble_on_plex() {
    let jellyfin = request(
        "jellyfin",
        "markWatched",
        json!({"itemId": "i1", "watched": false}),
    );
    assert_eq!(jellyfin["method"], "DELETE");
    let plex = request(
        "plex",
        "markWatched",
        json!({"itemId": "9", "watched": false}),
    );
    assert!(plex["url"].as_str().unwrap().contains("/:/unscrobble?"));
}

#[test]
fn plex_pin_flow_and_server_discovery() {
    let start = parse("plex", "pinStart", json!({"id": 7, "code": "ABCD"}));
    assert!(
        start["pin"]["authUrl"]
            .as_str()
            .unwrap()
            .contains("code=ABCD")
    );
    let pending = parse("plex", "pinCheck", json!({"authToken": null}));
    assert_eq!(pending["state"], "pending");
    let done = parse("plex", "pinCheck", json!({"authToken": "xyz"}));
    assert_eq!(done["auth"]["accessToken"], "xyz");
    let servers = parse(
        "plex",
        "servers",
        json!([
            {"name": "Home", "provides": "server", "clientIdentifier": "c1", "owned": true,
             "accessToken": "st", "connections": [{"uri": "http://192.168.1.2:32400", "local": true, "relay": false}]},
            {"name": "Phone", "provides": "client", "clientIdentifier": "c2", "connections": []}
        ]),
    );
    assert_eq!(servers["servers"].as_array().unwrap().len(), 1);
    assert_eq!(servers["servers"][0]["connections"][0]["local"], true);
}

#[test]
fn plex_lookup_drops_items_whose_guid_does_not_match() {
    let mut args = server("plex");
    args["operation"] = json!("lookup");
    args["params"] = json!({"externalId": "tt1"});
    args["status"] = json!(200);
    args["body"] = json!({"MediaContainer": {"Metadata": [
        {"ratingKey": "1", "type": "movie", "title": "A", "Guid": [{"id": "imdb://tt1"}]},
        {"ratingKey": "2", "type": "movie", "title": "B", "Guid": [{"id": "imdb://tt2"}]}
    ]}});
    let parsed: Value = serde_json::from_str(&parse_json(&args.to_string()).unwrap()).unwrap();
    let metas = parsed["metas"].as_array().unwrap();
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0]["id"], "ms:srv:1");
    assert_eq!(metas[0]["imdb_id"], "tt1");
}

#[test]
fn plex_streams_use_part_key_with_token_and_external_subtitles_only() {
    let parsed = parse(
        "plex",
        "streams",
        json!({"MediaContainer": {"Metadata": [{"ratingKey": "5", "Media": [{
            "videoResolution": "1080", "videoCodec": "h264", "audioCodec": "aac", "audioChannels": 2,
            "Part": [{"key": "/library/parts/9/file.mkv", "size": 2_000_000_000i64, "file": "/d/file.mkv",
                "Stream": [
                    {"streamType": 3, "id": 1, "languageCode": "eng", "key": "/library/streams/1"},
                    {"streamType": 3, "id": 2, "languageCode": "fra"}
                ]}]
        }]}]}}),
    );
    let streams = parsed["streams"].as_array().unwrap();
    assert_eq!(
        streams[0]["url"],
        "https://m.example/library/parts/9/file.mkv?X-Plex-Token=tok"
    );
    assert_eq!(streams[0]["title"], "1080p · H264 · aac 2ch · 2.0 GB");
    assert_eq!(streams[0]["subtitles"].as_array().unwrap().len(), 1);
    assert!(streams[1]["url"].as_str().unwrap().contains("protocol=hls"));
}

#[test]
fn plex_libraries_skip_music_and_map_show_to_series() {
    let parsed = parse(
        "plex",
        "libraries",
        json!({"MediaContainer": {"Directory": [
            {"key": "1", "type": "movie", "title": "Movies"},
            {"key": "2", "type": "show", "title": "TV"},
            {"key": "3", "type": "artist", "title": "Music"}
        ]}}),
    );
    let catalogs = parsed["catalogs"].as_array().unwrap();
    assert_eq!(catalogs.len(), 2);
    assert_eq!(catalogs[1]["type"], "series");
}

#[test]
fn failed_responses_report_an_error_instead_of_empty_results() {
    let mut args = server("jellyfin");
    args["operation"] = json!("catalog");
    args["status"] = json!(401);
    args["body"] = Value::Null;
    let parsed: Value = serde_json::from_str(&parse_json(&args.to_string()).unwrap()).unwrap();
    assert_eq!(parsed["error"], true);
    assert_eq!(parsed["status"], 401);
}

#[test]
fn episodes_in_rows_open_their_series() {
    let parsed = parse(
        "jellyfin",
        "resume",
        json!({"Items": [{
            "Id": "e1", "Type": "Episode", "Name": "Pilot", "SeriesName": "Show", "SeriesId": "s1",
            "SeriesPrimaryImageTag": "st", "ParentIndexNumber": 2, "IndexNumber": 3,
            "UserData": {"PlaybackPositionTicks": 600_000_000i64}
        }]}),
    );
    let meta = &parsed["metas"][0];
    assert_eq!(meta["id"], "ms:srv:s1");
    assert_eq!(meta["lastVideoId"], "ms:srv:e1");
    assert_eq!(meta["name"], "Show");
    assert_eq!(meta["timeOffset"], 60_000);
    assert!(
        meta["poster"]
            .as_str()
            .unwrap()
            .contains("/Items/s1/Images/Primary?tag=st")
    );
}

#[test]
fn jellyfin_lookup_drops_items_the_server_returned_for_an_unmatched_id() {
    let mut args = server("jellyfin");
    args["operation"] = json!("lookup");
    args["params"] = json!({"externalId": "tt1"});
    args["status"] = json!(200);
    args["body"] = json!({"Items": [
        {"Id": "a", "Type": "Movie", "Name": "A", "ProviderIds": {}},
        {"Id": "b", "Type": "Movie", "Name": "B", "ProviderIds": {"Imdb": "tt1"}}
    ]});
    let parsed: Value = serde_json::from_str(&parse_json(&args.to_string()).unwrap()).unwrap();
    let metas = parsed["metas"].as_array().unwrap();
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0]["id"], "ms:srv:b");
}
