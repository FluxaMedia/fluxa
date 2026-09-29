use serde_json::{Value, json};

pub(crate) fn oauth_request_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let service = request.get("service")?.as_str()?;
    let operation = request.get("operation")?.as_str()?;
    let client_id = request
        .get("clientId")
        .and_then(Value::as_str)
        .unwrap_or("");
    let client_secret = request
        .get("clientSecret")
        .and_then(Value::as_str)
        .unwrap_or("");
    let code = request.get("code").and_then(Value::as_str).unwrap_or("");
    let code_verifier = request
        .get("codeVerifier")
        .and_then(Value::as_str)
        .unwrap_or("");
    let refresh_token = request
        .get("refreshToken")
        .and_then(Value::as_str)
        .unwrap_or("");
    let (url, body) = match (service, operation) {
        ("trakt", "device_start") => (
            "https://api.trakt.tv/oauth/device/code",
            json!({"client_id": client_id}),
        ),
        ("trakt", "device_poll") => (
            "https://api.trakt.tv/oauth/device/token",
            json!({"code": code, "client_id": client_id}),
        ),
        ("trakt", "exchange") => (
            "https://api.trakt.tv/oauth/token",
            json!({"code": code, "client_id": client_id, "client_secret": client_secret, "redirect_uri": "fluxa://oauth/trakt", "grant_type": "authorization_code"}),
        ),
        ("trakt", "refresh") => (
            "https://api.trakt.tv/oauth/token",
            json!({"refresh_token": refresh_token, "client_id": client_id, "client_secret": client_secret, "redirect_uri": "fluxa://oauth/trakt", "grant_type": "refresh_token"}),
        ),
        ("anilist", "exchange") => (
            "https://anilist.co/api/v2/oauth/token",
            json!({"grant_type": "authorization_code", "client_id": client_id, "client_secret": client_secret, "redirect_uri": "fluxa://oauth/anilist", "code": code}),
        ),
        ("simkl", "exchange") => (
            "https://api.simkl.com/oauth/token",
            json!({"code": code, "client_id": client_id, "code_verifier": code_verifier, "redirect_uri": "fluxa://oauth/simkl", "grant_type": "authorization_code"}),
        ),
        ("mdblist", "device_start") => (
            "https://api.mdblist.com/oauth/device-authorization/",
            json!({"client_id": client_id, "scope": "write"}),
        ),
        ("mdblist", "device_poll") => (
            "https://api.mdblist.com/oauth/token/",
            json!({"grant_type": "urn:ietf:params:oauth:grant-type:device_code", "device_code": code, "client_id": client_id}),
        ),
        ("mdblist", "exchange") => (
            "https://api.mdblist.com/oauth/token/",
            json!({"grant_type": "authorization_code", "code": code, "client_id": client_id, "client_secret": client_secret, "redirect_uri": "fluxa://oauth/mdblist", "code_verifier": code_verifier}),
        ),
        ("mdblist", "refresh") => (
            "https://api.mdblist.com/oauth/token/",
            json!({"grant_type": "refresh_token", "refresh_token": refresh_token, "client_id": client_id, "client_secret": client_secret}),
        ),
        _ => return None,
    };
    serde_json::to_string(&json!({"url": url, "body": body})).ok()
}

pub(crate) fn oauth_response_outcome(service: &str, operation: &str, status: u16) -> &'static str {
    if (200..300).contains(&status) {
        return "success";
    }
    if service == "trakt" && operation == "device_poll" {
        return if status == 400 || status == 429 {
            "pending"
        } else {
            "expired"
        };
    }
    "error"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_provider_requests_and_poll_outcomes() {
        let plan: Value = serde_json::from_str(&oauth_request_plan_json(r#"{"service":"trakt","operation":"exchange","clientId":"id","clientSecret":"secret","code":"code"}"#).unwrap()).unwrap();
        assert_eq!(plan["url"], "https://api.trakt.tv/oauth/token");
        assert_eq!(plan["body"]["redirect_uri"], "fluxa://oauth/trakt");
        assert_eq!(
            oauth_response_outcome("trakt", "device_poll", 429),
            "pending"
        );
        assert_eq!(
            oauth_response_outcome("trakt", "device_poll", 410),
            "expired"
        );
    }

    #[test]
    fn plans_mdblist_device_flow_and_refresh() {
        let plan: Value = serde_json::from_str(
            &oauth_request_plan_json(
                r#"{"service":"mdblist","operation":"device_start","clientId":"id"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            plan["url"],
            "https://api.mdblist.com/oauth/device-authorization/"
        );
        assert_eq!(plan["body"]["scope"], "write");

        let poll: Value = serde_json::from_str(&oauth_request_plan_json(r#"{"service":"mdblist","operation":"device_poll","clientId":"id","code":"devcode"}"#).unwrap()).unwrap();
        assert_eq!(poll["body"]["device_code"], "devcode");
        assert_eq!(
            poll["body"]["grant_type"],
            "urn:ietf:params:oauth:grant-type:device_code"
        );

        let refresh: Value = serde_json::from_str(&oauth_request_plan_json(r#"{"service":"mdblist","operation":"refresh","clientId":"id","clientSecret":"secret","refreshToken":"tok"}"#).unwrap()).unwrap();
        assert_eq!(refresh["url"], "https://api.mdblist.com/oauth/token/");
        assert_eq!(refresh["body"]["refresh_token"], "tok");
    }
}
