use serde_json::{Value, json};

pub(crate) fn welcome_required(request: &Value) -> bool {
    let has_profiles = request
        .get("profiles")
        .and_then(Value::as_array)
        .is_some_and(|profiles| !profiles.is_empty());
    let offline = request
        .get("offlineChosen")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    !has_profiles && !offline
}

pub(crate) fn nuvio_session_profile_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let session = request.get("session")?;
    let token = session
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|token| !token.is_empty())?;
    let id = request.get("id").and_then(Value::as_str)?;
    let now = request
        .get("nowSeconds")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let expires_in = session
        .get("expires_in")
        .and_then(Value::as_i64)
        .unwrap_or(3600);
    let claims = token_claims(token);
    let email = request
        .get("email")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|email| !email.is_empty())
        .or_else(|| session.pointer("/user/email").and_then(Value::as_str))
        .or_else(|| claims.get("email").and_then(Value::as_str))
        .unwrap_or_default()
        .trim();
    let user_id = session
        .pointer("/user/id")
        .or_else(|| request.get("userId"))
        .or_else(|| claims.get("sub"))
        .cloned()
        .unwrap_or(Value::Null);
    Some(
        json!({
            "id": id,
            "name": "",
            "email": email,
            "nuvioAccessToken": token,
            "nuvioRefreshToken": session.get("refresh_token").cloned().unwrap_or(Value::Null),
            "nuvioTokenExpiresAt": now + expires_in,
            "nuvioUserId": user_id,
            "nuvioEmail": email,
            "nuvioProfileIndex": 1,
        })
        .to_string(),
    )
}

fn token_claims(token: &str) -> Value {
    use base64::Engine;
    token
        .split('.')
        .nth(1)
        .and_then(|payload| {
            base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(payload.trim_end_matches('='))
                .ok()
        })
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or(Value::Null)
}

pub(crate) fn nuvio_auth_error_key(request: &Value) -> &'static str {
    let status = request
        .get("status")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let text = request
        .get("body")
        .map(|body| match body {
            Value::String(text) => text.to_lowercase(),
            other => other.to_string().to_lowercase(),
        })
        .unwrap_or_default();
    if text.contains("email_not_confirmed") || text.contains("not confirmed") {
        "auth.error.email_not_confirmed"
    } else if status == 429 || text.contains("rate limit") || text.contains("too many") {
        "auth.error.rate_limited"
    } else if status >= 500 {
        "auth.error.server"
    } else if status == 0 {
        "auth.error.network"
    } else if status == 400 || status == 401 {
        "auth.error.invalid_credentials"
    } else {
        "auth.error.unknown"
    }
}
