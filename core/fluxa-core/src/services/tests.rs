use super::anilist::{anilist_user_id};
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

