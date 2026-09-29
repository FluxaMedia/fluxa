use super::anilist::{anilist_snapshot, anilist_user_id};
use super::*;

fn value(json: Option<String>) -> Value {
    serde_json::from_str(&json.unwrap()).unwrap()
}

#[test]
fn trakt_calendar_plan_covers_the_whole_month() {
    let plan = value(trakt_calendar_plan_json(
        &json!({"token": "t", "clientId": "c", "year": 2026, "month": 2}).to_string(),
    ));
    assert_eq!(
        plan[0]["url"],
        "https://api.trakt.tv/calendars/my/shows/2026-02-01/28?extended=full,images"
    );
    assert_eq!(plan[1]["key"], "movies");
}

#[test]
fn trakt_snapshot_dedupes_playback_and_derives_completed_from_progress() {
    let show = |imdb: &str, title: &str| json!({"title": title, "ids": {"imdb": imdb, "trakt": 1}});
    let episode = |n: i64| json!({"season": 1, "number": n, "title": "e", "ids": {"trakt": n}});
    let responses = json!({
        "playback": [
            {"id": 1, "type": "episode", "progress": 30.0, "paused_at": "2026-01-01T00:00:00.000Z", "show": show("tt1", "A"), "episode": episode(1)},
            {"id": 2, "type": "episode", "progress": 40.0, "paused_at": "2026-02-01T00:00:00.000Z", "show": show("tt1", "A"), "episode": episode(2)}
        ],
        "progress": [
            {"show": show("tt2", "Done"), "progress": {"aired": 3, "completed": 3, "last_watched_at": "2026-01-01T00:00:00.000Z"}},
            {"show": show("tt3", "Partial"), "progress": {"aired": 3, "completed": 1, "last_watched_at": "2026-01-01T00:00:00.000Z",
                "next_episode": {"season": 1, "number": 2, "title": "n", "ids": {"trakt": 9}, "first_aired": "2026-01-02T00:00:00.000Z"}}},
            {"show": show("tt4", "Future"), "progress": {"aired": 3, "completed": 1, "last_watched_at": "2026-01-01T00:00:00.000Z",
                "next_episode": {"season": 1, "number": 2, "title": "n", "ids": {"trakt": 10}, "first_aired": "2030-01-02T00:00:00.000Z"}}}
        ],
        "hidden_dropped": [{"show": show("tt5", "Dropped")}]
    });
    let snapshot = value(provider_library_snapshot_json(
        &json!({"provider": "trakt", "responses": responses, "nowSeconds": 1_800_000_000})
            .to_string(),
    ));
    let ids = |list: &str| -> Vec<String> {
        snapshot[list]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["id"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(ids("completed"), ["tt2"]);
    assert_eq!(ids("dropped"), ["tt5"]);
    assert_eq!(ids("continueWatching"), ["tt1", "tt3"]);
    assert_eq!(snapshot["continueWatching"][0]["lastEpisodeNumber"], 2);
}

#[test]
fn simkl_authorize_url_carries_an_s256_challenge() {
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let out = value(provider_authorize_url_json(
        &json!({"provider": "simkl", "clientId": "cid", "codeVerifier": verifier, "state": "st"})
            .to_string(),
    ));
    let url = url::Url::parse(out["url"].as_str().unwrap()).unwrap();
    let param = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .unwrap()
            .1
            .into_owned()
    };
    assert_eq!(
        param("code_challenge"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
    assert_eq!(param("redirect_uri"), "fluxa://oauth/simkl");
    assert_eq!(param("state"), "st");
}

#[test]
fn oauth_callback_yields_code_and_state() {
    let callback = value(provider_auth_callback_json(
        r#"{"url":"fluxa://oauth/simkl?code=abc&state=st"}"#,
    ));
    assert_eq!(callback["provider"], "simkl");
    assert_eq!(callback["code"], "abc");
    assert_eq!(callback["state"], "st");
    assert!(provider_auth_callback_json(r#"{"url":"https://evil/x?code=abc"}"#).is_none());
    let denied = value(provider_auth_callback_json(
        r#"{"url":"fluxa://oauth/simkl?error=access_denied"}"#,
    ));
    assert!(denied["code"].is_null());
}

#[test]
fn simkl_code_exchange_posts_the_verifier() {
    let plan = value(provider_auth_request_json(
        r#"{"provider":"simkl","operation":"exchange","clientId":"cid","code":"abc","codeVerifier":"ver"}"#,
    ));
    assert!(
        plan["url"]
            .as_str()
            .unwrap()
            .starts_with("https://api.simkl.com/oauth2/token")
    );
    assert_eq!(plan["body"]["grant_type"], "authorization_code");
    assert_eq!(plan["body"]["code_verifier"], "ver");
    assert_eq!(plan["body"]["redirect_uri"], "fluxa://oauth/simkl");
}

#[test]
fn mdblist_single_episode_is_not_the_whole_show() {
    let plan = value(provider_write_requests_json(
        &json!({"provider":"mdblist","token":"t","command":{"type":"markWatched","seriesId":"tt1","videoIds":["tt1:2:5"],"watched":true}})
            .to_string(),
    ));
    assert_eq!(plan[0]["url"], "https://api.mdblist.com/sync/watched");
    assert_eq!(
        plan[0]["body"]["shows"][0]["seasons"][0]["episodes"][0]["number"],
        5
    );
    assert!(plan[0]["body"].get("movies").is_none());
}

#[test]
fn mdblist_paused_movie_resumes_from_its_string_progress() {
    let responses = json!({"playback": [{
        "progress": "30.00",
        "paused_at": "2026-01-01T00:00:00.000Z",
        "type": "movie",
        "movie": {"title": "M", "ids": {"imdb": "tt9"}},
        "episode": null,
        "show": null,
    }]});
    let snapshot = value(provider_library_snapshot_json(
        &json!({"provider": "mdblist", "responses": responses}).to_string(),
    ));
    assert_eq!(snapshot["continueWatching"][0]["id"], "tt9");
    assert_eq!(snapshot["continueWatching"][0]["lastVideoId"], "tt9");
    assert_eq!(
        snapshot["continueWatching"][0]["resumeProgressPercent"],
        30.0
    );
}

#[test]
fn mdblist_partly_watched_show_is_not_completed() {
    let show = |imdb: &str, total: i64| json!({"show": {"title": imdb, "ids": {"imdb": imdb, "tmdb": 9}, "total_aired_episodes": total}});
    let episode = |imdb: &str, n: i64| json!({"episode": {"season": 1, "number": n, "show": {"ids": {"imdb": imdb}}}});
    let responses = json!({
        "watched": {
            "movies": [{"movie": {"title": "M", "ids": {"imdb": "tt9"}}}],
            "shows": [show("tt1", 2), show("tt2", 3)],
            "episodes": [episode("tt1", 1), episode("tt1", 2), episode("tt2", 1)],
        },
        "upnext": {"items": [{
            "show": {"title": "B", "ids": {"tmdb": 9}},
            "next_episode": {"season": 1, "episode": 2, "title": "n"},
            "last_watched_at": "2026-01-01T00:00:00.000Z",
        }]},
    });
    let snapshot = value(provider_library_snapshot_json(
        &json!({"provider": "mdblist", "responses": responses}).to_string(),
    ));
    let completed: Vec<&str> = snapshot["completed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    assert_eq!(completed, ["tt9", "tt1"]);
    assert_eq!(snapshot["watched"]["tt2:1:1"], true);
    assert!(snapshot["watched"].get("tt2:1:2").is_none());
    assert_eq!(snapshot["continueWatching"][0]["lastVideoId"], "tt2:1:2");
}

#[test]
fn simkl_allow_rewatch_is_sent_only_for_explicit_rewatches() {
    let request = |command: Value| {
        value(provider_write_requests_json(
            &json!({"provider":"simkl","token":"t","clientId":"c","command":command}).to_string(),
        ))
    };
    let plain =
        request(json!({"type":"markWatched","seriesId":"tt1","videoIds":["tt1"],"watched":true}));
    assert!(!plain[0]["url"].as_str().unwrap().contains("allow_rewatch"));
    let rewatch = request(
        json!({"type":"markWatched","seriesId":"tt1","videoIds":["tt1"],"watched":true,"rewatch":true}),
    );
    assert!(
        rewatch[0]["url"]
            .as_str()
            .unwrap()
            .contains("allow_rewatch=yes")
    );
    let unwatch = request(
        json!({"type":"markWatched","seriesId":"tt1","videoIds":["tt1"],"watched":false,"rewatch":true}),
    );
    assert!(
        !unwatch[0]["url"]
            .as_str()
            .unwrap()
            .contains("allow_rewatch")
    );
}

#[test]
fn simkl_scrobble_uses_anime_ids_directly() {
    let plan = value(provider_scrobble_request_json(
        r#"{"provider":"simkl","accessToken":"t","action":"start","itemId":"kitsu:46474:5","metaType":"series","progress":12.5}"#,
    ));
    assert_eq!(plan["body"]["show"]["ids"], json!({"kitsu": 46474}));
    assert_eq!(plan["body"]["episode"], json!({"season": 1, "number": 5}));
}

#[test]
fn simkl_device_poll_stays_pending_until_token_arrives() {
    let pending = value(provider_auth_outcome_json(
        r#"{"provider":"simkl","operation":"poll","status":400,"body":{"error":"authorization_pending"}}"#,
    ));
    assert_eq!(pending["state"], "pending");
    let slow = value(provider_auth_outcome_json(
        r#"{"provider":"simkl","operation":"poll","status":400,"body":{"error":"slow_down"}}"#,
    ));
    assert_eq!(slow["state"], "slow_down");
    let expired = value(provider_auth_outcome_json(
        r#"{"provider":"simkl","operation":"poll","status":400,"body":{"error":"expired_token"}}"#,
    ));
    assert_eq!(expired["state"], "error");
    let done = value(provider_auth_outcome_json(
        r#"{"provider":"simkl","operation":"poll","status":200,"nowSeconds":100,"body":{"access_token":"tok","refresh_token":"ref","expires_in":604800}}"#,
    ));
    assert_eq!(done["auth"]["accessToken"], "tok");
    assert_eq!(done["auth"]["refreshToken"], "ref");
    assert_eq!(done["auth"]["expiresAt"], 604900);
}

#[test]
fn simkl_sign_in_uses_the_v2_endpoints_and_asks_for_write_scope() {
    let start = value(provider_auth_request_json(
        r#"{"provider":"simkl","operation":"start","clientId":"cid"}"#,
    ));
    assert!(
        start["url"]
            .as_str()
            .unwrap()
            .starts_with("https://api.simkl.com/oauth2/device")
    );
    assert_eq!(start["body"]["scope"], "media:read media:write");
    let poll = value(provider_auth_request_json(
        r#"{"provider":"simkl","operation":"poll","clientId":"cid","code":"dev"}"#,
    ));
    assert!(poll["url"].as_str().unwrap().contains("/oauth2/token"));
    assert_eq!(poll["body"]["device_code"], "dev");
    let refresh = value(provider_auth_request_json(
        r#"{"provider":"simkl","operation":"refresh","clientId":"cid","refreshToken":"ref"}"#,
    ));
    assert_eq!(refresh["body"]["grant_type"], "refresh_token");
    assert_eq!(refresh["body"]["refresh_token"], "ref");
}

#[test]
fn mdblist_device_sign_in_posts_form_fields() {
    let start = value(provider_auth_request_json(
        r#"{"provider":"mdblist","operation":"start","clientId":"cid"}"#,
    ));
    assert_eq!(
        start["url"],
        "https://api.mdblist.com/oauth/device-authorization/"
    );
    assert_eq!(
        start["headers"]["Content-Type"],
        "application/x-www-form-urlencoded"
    );
    assert_eq!(start["body"]["client_id"], "cid");
    assert_eq!(start["body"]["scope"], "write");
    let poll = value(provider_auth_request_json(
        r#"{"provider":"mdblist","operation":"poll","clientId":"cid","code":"dev"}"#,
    ));
    assert_eq!(poll["body"]["device_code"], "dev");
}

#[test]
fn mdblist_poll_outcomes() {
    let outcome = |status: u16, body: Value| {
        value(provider_auth_outcome_json(
            &json!({"provider": "mdblist", "operation": "poll", "status": status, "body": body, "nowSeconds": 100})
                .to_string(),
        ))
    };
    assert_eq!(
        outcome(400, json!({"error": "authorization_pending"}))["state"],
        "pending"
    );
    assert_eq!(
        outcome(400, json!({"error": "slow_down"}))["state"],
        "slow_down"
    );
    let done = outcome(
        200,
        json!({"access_token": "a", "refresh_token": "r", "expires_in": 60}),
    );
    assert_eq!(done["auth"]["accessToken"], "a");
    assert_eq!(done["auth"]["expiresAt"], 160);
}

#[test]
fn mdblist_bearer_token_replaces_the_api_key() {
    let requests = value(provider_library_requests_json(
        r#"{"provider":"mdblist","token":"tok","apiKey":"k"}"#,
    ));
    let first = &requests[0];
    assert_eq!(first["headers"]["Authorization"], "Bearer tok");
    assert!(!first["url"].as_str().unwrap().contains("apikey="));
    let legacy = value(provider_library_requests_json(
        r#"{"provider":"mdblist","apiKey":"k"}"#,
    ));
    assert!(legacy[0]["url"].as_str().unwrap().contains("apikey=k"));
}

#[test]
fn trakt_watched_movies_count_as_completed_and_watched() {
    let snapshot = value(provider_library_snapshot_json(
        &json!({
            "provider": "trakt",
            "responses": {
                "watched_movies": [{"movie": {"title": "Heat", "ids": {"imdb": "tt0113277"}}}],
                "playback": [{"progress": 40.0, "movie": {"title": "Alien", "ids": {"imdb": "tt0078748"}}}],
            }
        })
        .to_string(),
    ));
    assert_eq!(snapshot["completed"][0]["id"], "tt0113277");
    assert_eq!(snapshot["watched"]["tt0113277"], true);
    assert_eq!(snapshot["continueWatching"][0]["id"], "tt0078748");
}

#[test]
fn simkl_requests_identify_the_app() {
    let plan = value(provider_scrobble_request_json(
        r#"{"provider":"simkl","action":"start","itemId":"tt0078748","metaType":"movie","progress":1}"#,
    ));
    let url = plan["url"].as_str().unwrap();
    assert!(url.contains("app-name=fluxa&app-version="));
    assert!(
        plan["headers"]["User-Agent"]
            .as_str()
            .unwrap()
            .starts_with("fluxa/")
    );
}

#[test]
fn mdblist_requests_carry_the_api_key() {
    let requests = value(provider_library_requests_json(
        r#"{"provider":"mdblist","apiKey":"k"}"#,
    ));
    assert!(
        requests
            .as_array()
            .unwrap()
            .iter()
            .all(|request| request["url"].as_str().unwrap().contains("apikey=k"))
    );
}

#[test]
fn episode_scrobble_targets_the_show() {
    let plan = value(provider_scrobble_request_json(
        r#"{"provider":"trakt","action":"pause","itemId":"tt0903747:2:3","metaType":"series","progress":50}"#,
    ));
    assert_eq!(plan["body"]["show"]["ids"]["imdb"], "tt0903747");
    assert_eq!(plan["body"]["episode"]["number"], 3);
}

#[test]
fn anilist_user_id_comes_from_the_token_subject() {
    let token = "h.eyJzdWIiOiI4MDkwNDMxIn0.s";
    assert_eq!(anilist_user_id(token), Some(8090431));
}

#[test]
fn anilist_paused_entries_land_in_on_hold() {
    let entry = |status: &str, id: i64| json!({"status": status, "progress": 1, "updatedAt": 10, "media": {"id": id, "title": {"romaji": "x"}}});
    let list = json!({"isCustomList": false, "entries": [entry("PAUSED", 1), entry("DROPPED", 2)]});
    let responses = json!({"list_1": {"data": {"MediaListCollection": {"lists": [list]}}}});
    let snapshot = anilist_snapshot(&responses, 0);
    assert_eq!(snapshot["onHold"][0]["id"], "anilist:1");
    assert_eq!(snapshot["dropped"][0]["id"], "anilist:2");
}
