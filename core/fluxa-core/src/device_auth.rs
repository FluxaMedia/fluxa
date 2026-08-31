use serde_json::{json, Value};

const DEFAULT_POLL_INTERVAL_MS: i64 = 5_000;
const MAX_POLL_INTERVAL_MS: i64 = 60_000;

fn object(input: &str) -> Option<serde_json::Map<String, Value>> {
    serde_json::from_str::<Value>(input).ok()?.as_object().cloned()
}

fn provider(value: Option<&Value>) -> Option<&'static str> {
    match value?.as_str()? {
        "nuvio" => Some("nuvio"),
        "stremio" => Some("stremio"),
        "fluxa" => Some("fluxa"),
        _ => None,
    }
}

pub(crate) fn start_json(input: &str) -> Option<String> {
    let args = object(input)?;
    let provider = provider(args.get("provider"))?;
    let session_id = args.get("sessionId")?.as_str()?.trim();
    let verification_uri = args.get("verificationUri")?.as_str()?.trim();
    let user_code = args.get("userCode")?.as_str()?.trim();
    let now_ms = args.get("nowMs").and_then(Value::as_i64)?;
    let expires_at_ms = args.get("expiresAtMs").and_then(Value::as_i64)?;
    if session_id.is_empty() || user_code.is_empty() || !verification_uri.starts_with("https://") || expires_at_ms <= now_ms {
        return None;
    }
    let poll_interval_ms = args.get("pollIntervalMs").and_then(Value::as_i64).unwrap_or(DEFAULT_POLL_INTERVAL_MS).clamp(1_000, MAX_POLL_INTERVAL_MS);
    Some(json!({
        "provider": provider,
        "sessionId": session_id,
        "status": "pending",
        "verificationUri": verification_uri,
        "userCode": user_code,
        "startedAtMs": now_ms,
        "expiresAtMs": expires_at_ms,
        "pollIntervalMs": poll_interval_ms,
        "nextPollAtMs": now_ms,
        "attempt": 0
    }).to_string())
}

pub(crate) fn transition_json(input: &str) -> Option<String> {
    let args = object(input)?;
    let mut state = args.get("state")?.as_object()?.clone();
    let now_ms = args.get("nowMs").and_then(Value::as_i64)?;
    let outcome = args.get("outcome")?.as_str()?;
    if state.get("status")?.as_str()? != "pending" {
        return Some(Value::Object(state).to_string());
    }
    if now_ms >= state.get("expiresAtMs")?.as_i64()? {
        state.insert("status".into(), json!("expired"));
        return Some(Value::Object(state).to_string());
    }
    match outcome {
        "pending" => {
            let interval = state.get("pollIntervalMs")?.as_i64()?.clamp(1_000, MAX_POLL_INTERVAL_MS);
            let attempt = state.get("attempt").and_then(Value::as_i64).unwrap_or(0) + 1;
            state.insert("attempt".into(), json!(attempt));
            state.insert("nextPollAtMs".into(), json!(now_ms + interval));
        }
        "slow_down" => {
            let interval = (state.get("pollIntervalMs")?.as_i64()? * 2).clamp(1_000, MAX_POLL_INTERVAL_MS);
            state.insert("pollIntervalMs".into(), json!(interval));
            state.insert("nextPollAtMs".into(), json!(now_ms + interval));
        }
        "authorized" | "denied" | "error" => {
            state.insert("status".into(), json!(outcome));
        }
        _ => return None,
    }
    Some(Value::Object(state).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_provider_and_expires() {
        let state = start_json(r#"{"provider":"fluxa","sessionId":"s","verificationUri":"https://example.test/tv","userCode":"ABCD","nowMs":100,"expiresAtMs":5000}"#).unwrap();
        assert!(state.contains(r#""provider":"fluxa""#));
        let expired = transition_json(&format!(r#"{{"state":{},"nowMs":5000,"outcome":"pending"}}"#, state)).unwrap();
        assert!(expired.contains(r#""status":"expired""#));
    }

    #[test]
    fn slows_down_polling_without_finishing() {
        let state = start_json(r#"{"provider":"nuvio","sessionId":"s","verificationUri":"https://example.test/tv","userCode":"ABCD","nowMs":100,"expiresAtMs":5000,"pollIntervalMs":2000}"#).unwrap();
        let next = transition_json(&format!(r#"{{"state":{},"nowMs":200,"outcome":"slow_down"}}"#, state)).unwrap();
        assert!(next.contains(r#""pollIntervalMs":4000"#));
        assert!(next.contains(r#""status":"pending""#));
    }
}
