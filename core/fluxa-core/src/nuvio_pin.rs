//! Shared Nuvio profile-PIN rules.
//!
//! The Nuvio service owns the authoritative PIN and failed-attempt lockout.
//! Clients use this module for the protocol-compatible local verification
//! cache, so every Fluxa client uses the same digest and invalidation rules.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn valid_pin(pin: &str) -> bool {
    pin.len() == 4 && pin.bytes().all(|byte| byte.is_ascii_digit())
}

fn hash(profile_index: i64, salt: &str, pin: &str) -> String {
    let input = format!("profile:{profile_index}:{salt}:{pin}");
    Sha256::digest(input.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.bytes()
        .zip(right.bytes())
        .fold(0u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

fn args(args_json: &str) -> Option<Value> {
    serde_json::from_str(args_json).ok()
}

/// Returns the digest used by NuvioMobile:
/// SHA-256("profile:<profileIndex>:<salt>:<pin>").
pub(crate) fn pin_hash_json(args_json: &str) -> Option<String> {
    let args = args(args_json)?;
    let profile_index = args.get("profileIndex").and_then(Value::as_i64)?;
    let salt = args.get("salt").and_then(Value::as_str)?;
    let pin = args.get("pin").and_then(Value::as_str)?;
    if profile_index < 1 || salt.is_empty() || !valid_pin(pin) {
        return None;
    }
    Some(hash(profile_index, salt, pin))
}

/// Builds the portable local cache payload after a successful online verify.
pub(crate) fn cache_payload_json(args_json: &str) -> Option<String> {
    let args = args(args_json)?;
    let profile_index = args.get("profileIndex").and_then(Value::as_i64)?;
    let salt = args.get("salt").and_then(Value::as_str)?;
    let pin = args.get("pin").and_then(Value::as_str)?;
    let profile_updated_at = args
        .get("profileUpdatedAt")
        .and_then(Value::as_str)
        .unwrap_or("");
    if profile_index < 1 || salt.is_empty() || !valid_pin(pin) {
        return None;
    }
    Some(
        json!({
            "salt": salt,
            "digest": hash(profile_index, salt, pin),
            "profileUpdatedAt": profile_updated_at,
        })
        .to_string(),
    )
}

/// Verifies a cached PIN using NuvioMobile's offline rules.
///
/// `reason` is intentionally machine-readable; UI layers localize it.
pub(crate) fn verify_cached_json(args_json: &str) -> Option<String> {
    let args = args(args_json)?;
    let profile_index = args.get("profileIndex").and_then(Value::as_i64)?;
    let pin = args.get("pin").and_then(Value::as_str)?;
    let pin_enabled = args
        .get("pinEnabled")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let profile_updated_at = args
        .get("profileUpdatedAt")
        .and_then(Value::as_str)
        .unwrap_or("");

    if !pin_enabled {
        return Some(json!({"unlocked": true, "reason": "disabled"}).to_string());
    }
    if profile_index < 1 || !valid_pin(pin) {
        return Some(json!({"unlocked": false, "reason": "incorrect"}).to_string());
    }

    let cache = match args.get("cache") {
        Some(Value::String(raw)) => serde_json::from_str::<Value>(raw).ok(),
        Some(value) => Some(value.clone()),
        None => None,
    };
    let cache = match cache.and_then(|value| value.as_object().cloned()) {
        Some(cache) => cache,
        None => {
            return Some(json!({"unlocked": false, "reason": "requires_online"}).to_string());
        }
    };
    let salt = cache.get("salt").and_then(Value::as_str).unwrap_or("");
    let digest = cache.get("digest").and_then(Value::as_str).unwrap_or("");
    let cached_updated_at = cache
        .get("profileUpdatedAt")
        .and_then(Value::as_str)
        .unwrap_or("");
    if salt.is_empty() || digest.is_empty() {
        return Some(json!({"unlocked": false, "reason": "requires_online"}).to_string());
    }
    if !cached_updated_at.is_empty()
        && !profile_updated_at.is_empty()
        && cached_updated_at != profile_updated_at
    {
        return Some(json!({"unlocked": false, "reason": "profile_changed"}).to_string());
    }

    let unlocked = constant_time_eq(&hash(profile_index, salt, pin), digest);
    Some(
        json!({
            "unlocked": unlocked,
            "reason": if unlocked { "ok" } else { "incorrect" },
        })
        .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_nuvio_mobile_digest_contract() {
        assert_eq!(
            pin_hash_json(r#"{"profileIndex":2,"salt":"abc","pin":"1234"}"#),
            Some("aed6477b00b4e1e94d7898284647ff276c17a5f4211511d0cac93106f0689d9b".into())
        );
    }

    #[test]
    fn cache_requires_online_when_missing_or_profile_changed() {
        let missing = verify_cached_json(r#"{"profileIndex":1,"pin":"1234","pinEnabled":true}"#).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&missing).unwrap()["reason"], "requires_online");

        let cache = cache_payload_json(r#"{"profileIndex":1,"salt":"s","pin":"1234","profileUpdatedAt":"a"}"#).unwrap();
        let changed = verify_cached_json(&format!(r#"{{"profileIndex":1,"pin":"1234","profileUpdatedAt":"b","cache":{cache}}}"#)).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&changed).unwrap()["reason"], "profile_changed");
    }

    #[test]
    fn cached_pin_verifies_and_wrong_pin_fails() {
        let cache = cache_payload_json(r#"{"profileIndex":1,"salt":"s","pin":"1234","profileUpdatedAt":"a"}"#).unwrap();
        let ok = verify_cached_json(&format!(r#"{{"profileIndex":1,"pin":"1234","profileUpdatedAt":"a","cache":{cache}}}"#)).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&ok).unwrap()["unlocked"], true);
        let bad = verify_cached_json(&format!(r#"{{"profileIndex":1,"pin":"0000","profileUpdatedAt":"a","cache":{cache}}}"#)).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&bad).unwrap()["reason"], "incorrect");
    }
}
