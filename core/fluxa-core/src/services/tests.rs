use super::anilist::anilist_user_id;
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
fn deep_links_route_to_oauth_or_detail() {
    let action = |url: &str| deep_link_action_json(&json!({"url": url}).to_string());
    let oauth = value(action("fluxa://oauth/simkl?code=abc"));
    assert_eq!(oauth["type"], "oauthCallback");
    let detail = value(action("app://play?id=tt1&type=series"));
    assert_eq!(
        (detail["id"].as_str(), detail["itemType"].as_str()),
        (Some("tt1"), Some("series"))
    );
    assert!(action("app://play?id=tt1&type=").is_none());
    assert!(action("https://example.com/?id=tt1&type=movie").is_none());
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
fn anilist_user_id_comes_from_the_token_subject() {
    let token = "h.eyJzdWIiOiI4MDkwNDMxIn0.s";
    assert_eq!(anilist_user_id(token), Some(8090431));
}

#[test]
fn profile_requests_hit_each_providers_user_endpoint() {
    let plan = |provider: &str| {
        let args = json!({
            "provider": provider,
            "token": "t",
            "clientId": "c",
            "command": {"type": "profile"},
        });
        value(provider_write_requests_json(&args.to_string()))[0].clone()
    };
    assert_eq!(plan("trakt")["url"], "https://api.trakt.tv/users/settings");
    assert!(
        plan("simkl")["url"]
            .as_str()
            .unwrap()
            .starts_with("https://api.simkl.com/users/settings")
    );
    assert_eq!(plan("mdblist")["url"], "https://api.mdblist.com/user");
    let anilist = plan("anilist");
    assert_eq!(anilist["url"], "https://graphql.anilist.co");
    assert!(
        anilist["body"]["query"]
            .as_str()
            .unwrap()
            .contains("Viewer")
    );
}

#[test]
fn profile_reads_handle_and_avatar_per_provider() {
    let parse = |provider: &str, body: Value| {
        value(provider_profile_json(
            &json!({"provider": provider, "status": 200, "body": body}).to_string(),
        ))
    };
    let trakt = parse(
        "trakt",
        json!({"user": {"username": "khooly", "name": "Kh", "images": {"avatar": {"full": "https://a/x.png"}}}}),
    );
    assert_eq!(
        (trakt["name"].as_str(), trakt["avatarUrl"].as_str()),
        (Some("khooly"), Some("https://a/x.png"))
    );
    let anilist = parse(
        "anilist",
        json!({"data": {"Viewer": {"name": "khooly", "avatar": {"large": "https://a/y.png"}}}}),
    );
    assert_eq!(anilist["avatarUrl"], "https://a/y.png");
    assert_eq!(
        parse("mdblist", json!({"username": "khooly"}))["avatarUrl"],
        Value::Null
    );
}

#[test]
fn profile_without_a_name_or_after_a_failed_request_is_none() {
    let args = |status: u64, body: Value| {
        json!({"provider": "simkl", "status": status, "body": body}).to_string()
    };
    assert!(provider_profile_json(&args(200, json!({"user": {"name": "  "}}))).is_none());
    assert!(provider_profile_json(&args(401, json!({"user": {"name": "khooly"}}))).is_none());
    assert!(provider_profile_json(&json!({"provider": "plex", "body": {}}).to_string()).is_none());
}
