use super::support::Scenario;
use super::{simkl, trakt};
use serde_json::{Value, json};

fn guest() -> Value {
    json!({"id": "p1", "name": "Me"})
}

fn exchange(provider: &str, profile: Value) -> Value {
    json!({"type": "authExchangeRequested", "provider": provider, "code": "dc", "profile": profile})
}

#[test]
fn trakt_device_flow_signs_the_profile_in() {
    let scenario = Scenario::start();
    scenario.world.respond(trakt::HOST, "POST", "/oauth/device/code", 200, json!({
        "device_code": "dc", "user_code": "ABCD",
        "verification_url": "https://trakt.tv/activate", "expires_in": 600, "interval": 5
    }));
    scenario.world.respond(trakt::HOST, "POST", "/oauth/device/token", 200, json!({
        "access_token": "new", "refresh_token": "ref", "expires_in": 7200, "created_at": 1_700_000_000
    }));
    let app = scenario.app_with_profile(guest());

    app.dispatch(json!({"type": "authFlowRequested", "provider": "trakt", "mode": "device"}));
    assert_eq!(app.at("/auth/result/state"), "pending");
    assert_eq!(app.at("/auth/result/device/userCode"), "ABCD");

    app.dispatch(exchange("trakt", guest()));
    assert_eq!(app.at("/auth/result/state"), "success");
    assert_eq!(app.at("/profile/active/traktAccessToken"), "new");
    assert_eq!(app.at("/profile/active/traktRefreshToken"), "ref");
    assert_eq!(app.at("/profile/active/traktTokenExpiresAt"), 1_700_007_200);
    let poll = &scenario.world.calls(trakt::HOST, "POST", "/oauth/device/token")[0];
    assert_eq!(poll.json()["client_id"], "test-trakt");
    assert_eq!(poll.json()["code"], "dc");
}

#[test]
fn trakt_refresh_swaps_in_the_new_tokens() {
    let scenario = Scenario::start();
    scenario.world.respond(trakt::HOST, "POST", "/oauth/token", 200, json!({
        "access_token": "fresh", "refresh_token": "ref2", "expires_in": 7200, "created_at": 1_700_000_000
    }));
    let profile = json!({"id": "p1", "name": "Me", "traktAccessToken": "old", "traktRefreshToken": "ref"});
    let app = scenario.app_with_profile(profile.clone());

    app.dispatch(json!({"type": "authRefreshRequested", "provider": "trakt", "profile": profile}));

    let call = &scenario.world.calls(trakt::HOST, "POST", "/oauth/token")[0];
    assert_eq!(call.json()["refresh_token"], "ref");
    assert_eq!(app.at("/profile/active/traktAccessToken"), "fresh");
    assert_eq!(app.at("/profile/active/traktRefreshToken"), "ref2");
}

#[test]
fn rejected_refresh_keeps_the_old_token_and_reports_an_error() {
    let scenario = Scenario::start();
    scenario.world.respond(trakt::HOST, "POST", "/oauth/token", 401, json!({"error": "invalid_grant"}));
    let profile = json!({"id": "p1", "name": "Me", "traktAccessToken": "old", "traktRefreshToken": "ref"});
    let app = scenario.app_with_profile(profile.clone());

    app.dispatch(json!({"type": "authRefreshRequested", "provider": "trakt", "profile": profile}));

    assert_ne!(app.at("/auth/error"), Value::Null);
    assert_ne!(app.at("/profile/active/traktAccessToken"), "fresh");
}

#[test]
fn simkl_device_flow_signs_the_profile_in() {
    let scenario = Scenario::start();
    scenario.world.respond(simkl::HOST, "POST", "/oauth2/device", 200, json!({
        "device_code": "dc", "user_code": "WXYZ",
        "verification_uri": "https://simkl.com/pin", "expires_in": 600, "interval": 5
    }));
    scenario.world.respond(simkl::HOST, "POST", "/oauth2/token", 200, json!({
        "access_token": "stok", "refresh_token": "sref", "expires_in": 3600
    }));
    let app = scenario.app_with_profile(guest());

    app.dispatch(json!({"type": "authFlowRequested", "provider": "simkl", "mode": "device"}));
    assert_eq!(app.at("/auth/result/device/verificationUrl"), "https://simkl.com/pin");

    app.dispatch(exchange("simkl", guest()));
    assert_eq!(app.at("/profile/active/simklAccessToken"), "stok");
    assert!(scenario.world.unmatched().is_empty(), "{:?}", scenario.world.unmatched());
}

#[test]
fn mdblist_device_flow_signs_the_profile_in() {
    let scenario = Scenario::start();
    scenario.world.respond("api.mdblist.com", "POST", "/oauth/device-authorization/", 200, json!({
        "device_code": "dc", "user_code": "MDB1",
        "verification_uri": "https://mdblist.com/activate", "expires_in": 600, "interval": 5
    }));
    scenario.world.respond("api.mdblist.com", "POST", "/oauth/token/", 200, json!({
        "access_token": "mtok", "refresh_token": "mref", "expires_in": 3600
    }));
    let app = scenario.app_with_profile(guest());

    app.dispatch(json!({"type": "authFlowRequested", "provider": "mdblist", "mode": "device"}));
    app.dispatch(exchange("mdblist", guest()));

    assert_eq!(app.at("/profile/active/mdblistAccessToken"), "mtok");
}

#[test]
fn browser_flow_hands_back_an_authorize_url_without_touching_the_network() {
    let scenario = Scenario::start();
    let app = scenario.app_with_profile(guest());

    for (provider, host) in [("anilist", "anilist.co"), ("simkl", "simkl.com")] {
        app.dispatch(json!({"type": "authFlowRequested", "provider": provider, "mode": "pkce"}));
        assert_eq!(app.at("/auth/result/state"), "redirect");
        let url = app.at("/auth/result/url");
        let url = url.as_str().unwrap();
        assert!(url.starts_with(&format!("https://{host}/")), "{url}");
        assert!(url.contains(&format!("client_id=test-{provider}")), "{url}");
        assert!(url.contains("redirect_uri=fluxa%3A%2F%2Foauth%2F"), "{url}");
    }
    assert!(scenario.world.requests("api.simkl.com").is_empty());
}
