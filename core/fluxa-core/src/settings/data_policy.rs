use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CacheEntryPolicyInput {
    key: String,
    stored_at_millis: i64,
    ttl_millis: i64,
    now_millis: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CacheTrimPolicyInput {
    #[serde(default)]
    entries: Vec<CacheTrimEntry>,
    max_entries: i64,
    now_millis: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CacheTrimEntry {
    key: String,
    expires_at_millis: i64,
    stored_at_millis: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataFailurePolicyInput {
    operation: String,
    kind: String,
    message: Option<String>,
    throwable_class: Option<String>,
    reason: Option<String>,
    status_code: Option<i64>,
}

pub(crate) fn cache_entry_policy_json(request_json: &str) -> Option<String> {
    let request: CacheEntryPolicyInput = serde_json::from_str(request_json).ok()?;
    let ttl = request.ttl_millis.max(1);
    let expires_at_millis = saturating_add(request.stored_at_millis, ttl);
    serde_json::to_string(&json!({
        "key": request.key,
        "storedAtMillis": request.stored_at_millis,
        "expiresAtMillis": expires_at_millis,
        "isExpired": expires_at_millis <= request.now_millis
    }))
    .ok()
}

pub(crate) fn cache_trim_policy_json(request_json: &str) -> Option<String> {
    let request: CacheTrimPolicyInput = serde_json::from_str(request_json).ok()?;
    let max_entries = request.max_entries.max(1) as usize;
    let expired_keys = request
        .entries
        .iter()
        .filter(|entry| entry.expires_at_millis <= request.now_millis)
        .map(|entry| entry.key.clone())
        .collect::<Vec<_>>();
    let mut live_entries = request
        .entries
        .into_iter()
        .filter(|entry| entry.expires_at_millis > request.now_millis)
        .collect::<Vec<_>>();
    live_entries.sort_by(|a, b| {
        a.stored_at_millis
            .cmp(&b.stored_at_millis)
            .then_with(|| a.key.cmp(&b.key))
    });
    let overflow = live_entries.len().saturating_sub(max_entries);
    let evicted_keys = live_entries
        .iter()
        .take(overflow)
        .map(|entry| entry.key.clone())
        .collect::<Vec<_>>();

    serde_json::to_string(&json!({
        "expiredKeys": expired_keys,
        "evictedKeys": evicted_keys
    }))
    .ok()
}

pub(crate) fn data_failure_policy_json(request_json: &str) -> Option<String> {
    let request: DataFailurePolicyInput = serde_json::from_str(request_json).ok()?;
    let kind = normalize_failure_kind(&request.kind);
    let message = match kind {
        "auth_unavailable" => "auth_unavailable".to_string(),
        "unsupported" => request
            .reason
            .or(request.message)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "unsupported".to_string()),
        "network_error" => request
            .message
            .or(request.throwable_class)
            .filter(|value| !value.trim().is_empty())
            .or_else(|| request.status_code.map(|code| format!("http_{code}")))
            .unwrap_or_else(|| "network_error".to_string()),
        "parse_error" => request
            .message
            .or(request.throwable_class)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "parse_error".to_string()),
        _ => request
            .message
            .or(request.throwable_class)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "unknown".to_string()),
    };
    let retryable = matches!(kind, "network_error")
        && !matches!(request.status_code, Some(400 | 401 | 403 | 404));
    let stale_fallback_allowed = matches!(kind, "network_error" | "parse_error");

    serde_json::to_string(&json!({
        "operation": request.operation,
        "kind": kind,
        "message": message,
        "retryable": retryable,
        "staleFallbackAllowed": stale_fallback_allowed
    }))
    .ok()
}

fn normalize_failure_kind(kind: &str) -> &'static str {
    match kind.trim() {
        "auth" | "auth_unavailable" | "AuthUnavailable" => "auth_unavailable",
        "network" | "network_error" | "NetworkError" => "network_error",
        "parse" | "parse_error" | "ParseError" => "parse_error",
        "unsupported" | "Unsupported" => "unsupported",
        _ => "unknown",
    }
}

fn saturating_add(left: i64, right: i64) -> i64 {
    left.checked_add(right).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn cache_entry_policy_clamps_ttl_and_reports_expiry() {
        let value: Value = serde_json::from_str(
            &cache_entry_policy_json(
                r#"{"key":"k","storedAtMillis":1000,"ttlMillis":0,"nowMillis":1001}"#,
            )
            .expect("policy"),
        )
        .expect("json");

        assert_eq!(value["expiresAtMillis"].as_i64(), Some(1001));
        assert_eq!(value["isExpired"].as_bool(), Some(true));
    }

    #[test]
    fn cache_trim_policy_removes_expired_before_oldest_overflow() {
        let value: Value = serde_json::from_str(
            &cache_trim_policy_json(
                r#"{
                  "maxEntries":2,
                  "nowMillis":50,
                  "entries":[
                    {"key":"expired","expiresAtMillis":49,"storedAtMillis":1},
                    {"key":"old","expiresAtMillis":100,"storedAtMillis":2},
                    {"key":"newer","expiresAtMillis":100,"storedAtMillis":3},
                    {"key":"newest","expiresAtMillis":100,"storedAtMillis":4}
                  ]
                }"#,
            )
            .expect("policy"),
        )
        .expect("json");

        assert_eq!(value["expiredKeys"][0].as_str(), Some("expired"));
        assert_eq!(value["evictedKeys"][0].as_str(), Some("old"));
    }

    #[test]
    fn data_failure_policy_keeps_stable_messages_and_retry_flags() {
        let auth: Value = serde_json::from_str(
            &data_failure_policy_json(r#"{"operation":"trakt","kind":"AuthUnavailable"}"#)
                .expect("policy"),
        )
        .expect("json");
        let network_404: Value = serde_json::from_str(
            &data_failure_policy_json(
                r#"{"operation":"catalog","kind":"NetworkError","statusCode":404}"#,
            )
            .expect("policy"),
        )
        .expect("json");

        assert_eq!(auth["message"].as_str(), Some("auth_unavailable"));
        assert_eq!(network_404["retryable"].as_bool(), Some(false));
        assert_eq!(network_404["staleFallbackAllowed"].as_bool(), Some(true));
    }
}
