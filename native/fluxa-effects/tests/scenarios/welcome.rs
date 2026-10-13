use super::support::Scenario;
use serde_json::{Value, json};
use std::sync::mpsc::Receiver;
use std::time::Duration;

const HOST: &str = "api.nuvio.tv";

fn receive(rx: Receiver<Result<Value, String>>) -> Result<Value, String> {
    rx.recv_timeout(Duration::from_secs(20)).expect("reply")
}

fn session() -> Value {
    json!({
        "access_token": "access-1",
        "refresh_token": "refresh-1",
        "expires_in": 3600,
        "user": {"id": "u-1", "email": "me@example.com"},
    })
}

fn serve_profiles(scenario: &Scenario) {
    scenario.world.respond(
        HOST,
        "POST",
        "/rest/v1/rpc/sync_pull_profiles",
        200,
        json!([{"profile_index": 1, "name": "Khooly"}]),
    );
    scenario.world.respond(
        HOST,
        "POST",
        "/rest/v1/rpc/get_avatar_catalog",
        200,
        json!([]),
    );
}

#[test]
fn password_sign_in_creates_a_profile_named_from_the_nuvio_account() {
    let scenario = Scenario::start();
    scenario
        .world
        .respond(HOST, "POST", "/auth/v1/token", 200, session());
    serve_profiles(&scenario);
    let app = scenario.app();

    let profile = receive(
        app.session
            .executor()
            .nuvio_sign_in("me@example.com".into(), "secret".into()),
    )
    .unwrap();

    assert_eq!(profile["nuvioAccessToken"], "access-1");
    assert_eq!(profile["nuvioUserId"], "u-1");
    assert_eq!(profile["name"], "Khooly");
    let call = &scenario.world.calls(HOST, "POST", "/auth/v1/token")[0];
    assert_eq!(call.query_param("grant_type").as_deref(), Some("password"));
    assert_eq!(call.json()["email"], "me@example.com");
}

#[test]
fn wrong_password_comes_back_as_a_translation_key() {
    let scenario = Scenario::start();
    scenario.world.respond(
        HOST,
        "POST",
        "/auth/v1/token",
        400,
        json!({"error_description": "Invalid login credentials"}),
    );
    let app = scenario.app();

    let outcome = receive(
        app.session
            .executor()
            .nuvio_sign_in("me@example.com".into(), "nope".into()),
    );

    assert_eq!(outcome.unwrap_err(), "auth.error.invalid_credentials");
}

#[test]
fn tv_pairing_waits_for_approval_then_signs_in() {
    let scenario = Scenario::start();
    scenario.world.respond(
        HOST,
        "POST",
        "/rest/v1/rpc/start_device_login_session",
        404,
        json!({"code": "PGRST202", "message": "Could not find the function public.start_device_login_session"}),
    );
    scenario.world.respond(
        HOST,
        "POST",
        "/rest/v1/rpc/start_tv_login_session",
        200,
        json!([{
            "code": "ABCD",
            "web_url": "https://nuvio.tv/tv-login?code=ABCD",
            "expires_at": "2099-01-01T00:00:00Z",
            "poll_interval_seconds": 2,
        }]),
    );
    scenario.world.respond(
        HOST,
        "POST",
        "/rest/v1/rpc/poll_tv_login_session",
        200,
        json!([{"approved": false, "status": "pending"}]),
    );
    scenario.world.respond(
        HOST,
        "POST",
        "/functions/v1/tv-logins-exchange",
        200,
        session(),
    );
    serve_profiles(&scenario);
    let app = scenario.app();
    let executor = app.session.executor();

    let started = receive(executor.nuvio_tv_start()).unwrap();
    assert_eq!(started["code"], "ABCD");
    assert_eq!(started["webUrl"], "https://nuvio.tv/tv-login?code=ABCD");
    let nonce = started["nonce"].as_str().unwrap().to_owned();

    let waiting = receive(executor.nuvio_tv_poll("ABCD".into(), nonce.clone())).unwrap();
    assert_eq!(waiting["state"], "pending");

    scenario.world.respond(
        HOST,
        "POST",
        "/rest/v1/rpc/poll_tv_login_session",
        200,
        json!([{"approved": true}]),
    );
    let done = receive(executor.nuvio_tv_poll("ABCD".into(), nonce.clone())).unwrap();
    assert_eq!(done["state"], "success");
    assert_eq!(done["profile"]["nuvioAccessToken"], "access-1");
    let exchange = &scenario
        .world
        .calls(HOST, "POST", "/functions/v1/tv-logins-exchange")[0];
    assert_eq!(exchange.json()["device_nonce"], nonce);
}

#[test]
fn device_login_is_preferred_and_ended_sessions_are_reported() {
    let scenario = Scenario::start();
    scenario.world.respond(
        HOST,
        "POST",
        "/rest/v1/rpc/start_device_login_session",
        200,
        json!([{
            "device_code": "DEV-1",
            "user_code": "WXYZ",
            "verification_uri": "https://nuvio.tv/link",
            "verification_uri_complete": "https://nuvio.tv/link?code=WXYZ",
            "expires_at": "2099-01-01T00:00:00Z",
            "poll_interval_seconds": 3,
        }]),
    );
    scenario.world.respond(
        HOST,
        "POST",
        "/rest/v1/rpc/poll_tv_login_session",
        200,
        json!([{"status": "expired"}]),
    );
    let app = scenario.app();
    let executor = app.session.executor();

    let started = receive(executor.nuvio_tv_start()).unwrap();
    assert_eq!(started["code"], "DEV-1");
    assert_eq!(started["webUrl"], "https://nuvio.tv/link?code=WXYZ");

    let nonce = started["nonce"].as_str().unwrap().to_owned();
    let ended = receive(executor.nuvio_tv_poll("DEV-1".into(), nonce)).unwrap();
    assert_eq!(ended["state"], "ended");
}
