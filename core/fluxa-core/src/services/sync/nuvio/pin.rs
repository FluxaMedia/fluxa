use super::helpers::{parse, str_field};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const MAX_OFFLINE_FAILURES: i64 = 5;
const OFFLINE_LOCK_SECONDS: i64 = 300;

fn digest(profile_index: i64, salt: &str, pin: &str) -> String {
    Sha256::digest(format!("profile:{profile_index}:{salt}:{pin}"))
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn index(profile: &Value) -> i64 {
    profile
        .get("nuvioProfileIndex")
        .and_then(Value::as_i64)
        .filter(|index| *index > 0)
        .unwrap_or(1)
}

pub(crate) fn clear_cache_fields(fields: &mut serde_json::Map<String, Value>) -> bool {
    [
        "nuvioPinSalt",
        "nuvioPinDigest",
        "nuvioPinCachedAt",
        "nuvioPinFailures",
        "nuvioPinLockedUntil",
    ]
    .into_iter()
    .fold(false, |removed, key| {
        fields.remove(key).is_some() || removed
    })
}

fn clear_cache(profile: &mut Value) {
    if let Some(fields) = profile.as_object_mut() {
        clear_cache_fields(fields);
    }
}

pub(crate) fn cache_pin_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let mut profile = args.get("profile")?.clone();
    let salt = str_field(&args, "salt")?;
    let pin = str_field(&args, "pin")?;
    let digest = digest(index(&profile), salt, pin);
    let cached_at = str_field(&profile, "nuvioUpdatedAt")
        .unwrap_or_default()
        .to_owned();
    clear_cache(&mut profile);
    let fields = profile.as_object_mut()?;
    fields.insert("nuvioPinSalt".into(), json!(salt));
    fields.insert("nuvioPinDigest".into(), json!(digest));
    fields.insert("nuvioPinCachedAt".into(), json!(cached_at));
    Some(profile.to_string())
}

pub(crate) fn offline_check_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let mut profile = args.get("profile")?.clone();
    let pin = str_field(&args, "pin")?;
    let now = args.get("now").and_then(Value::as_i64).unwrap_or_default();
    let outcome = |name: &str, retry_after: i64, profile: &Value| {
        Some(json!({"outcome": name, "retryAfter": retry_after, "profile": profile}).to_string())
    };

    let locked_until = profile
        .get("nuvioPinLockedUntil")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    if locked_until > now {
        return outcome("locked", locked_until - now, &profile);
    }

    let cached = (
        str_field(&profile, "nuvioPinSalt"),
        str_field(&profile, "nuvioPinDigest"),
    );
    let (Some(salt), Some(expected)) = cached else {
        return outcome("unavailable", 0, &profile);
    };
    let stale = str_field(&profile, "nuvioPinCachedAt").unwrap_or_default()
        != str_field(&profile, "nuvioUpdatedAt").unwrap_or_default();
    if stale {
        clear_cache(&mut profile);
        return outcome("unavailable", 0, &profile);
    }

    if digest(index(&profile), salt, pin) == expected {
        let fields = profile.as_object_mut()?;
        fields.remove("nuvioPinFailures");
        fields.remove("nuvioPinLockedUntil");
        return outcome("unlocked", 0, &profile);
    }

    let failures = profile
        .get("nuvioPinFailures")
        .and_then(Value::as_i64)
        .unwrap_or_default()
        + 1;
    let fields = profile.as_object_mut()?;
    if failures >= MAX_OFFLINE_FAILURES {
        fields.remove("nuvioPinFailures");
        fields.insert(
            "nuvioPinLockedUntil".into(),
            json!(now + OFFLINE_LOCK_SECONDS),
        );
        return outcome("locked", OFFLINE_LOCK_SECONDS, &profile);
    }
    fields.insert("nuvioPinFailures".into(), json!(failures));
    outcome("incorrect", 0, &profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cached() -> Value {
        let profile = json!({"nuvioProfileIndex": 2, "nuvioUpdatedAt": "t1"});
        let request = json!({"profile": profile, "salt": "ab", "pin": "1234"});
        serde_json::from_str(&cache_pin_json(&request.to_string()).unwrap()).unwrap()
    }

    fn check(profile: &Value, pin: &str, now: i64) -> Value {
        let request = json!({"profile": profile, "pin": pin, "now": now});
        serde_json::from_str(&offline_check_json(&request.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn cached_pin_unlocks_offline() {
        assert_eq!(check(&cached(), "1234", 0)["outcome"], "unlocked");
    }

    #[test]
    fn fifth_wrong_pin_locks_for_five_minutes() {
        let mut profile = cached();
        for _ in 0..4 {
            let result = check(&profile, "0000", 100);
            assert_eq!(result["outcome"], "incorrect");
            profile = result["profile"].clone();
        }
        let result = check(&profile, "0000", 100);
        assert_eq!(result["outcome"], "locked");
        assert_eq!(result["retryAfter"], 300);
        let locked = result["profile"].clone();
        assert_eq!(check(&locked, "1234", 150)["outcome"], "locked");
        assert_eq!(check(&locked, "1234", 401)["outcome"], "unlocked");
    }

    #[test]
    fn cache_dies_when_the_remote_profile_changes() {
        let mut profile = cached();
        profile["nuvioUpdatedAt"] = json!("t2");
        let result = check(&profile, "1234", 0);
        assert_eq!(result["outcome"], "unavailable");
        assert!(result["profile"].get("nuvioPinDigest").is_none());
    }

    #[test]
    fn missing_cache_is_unavailable() {
        assert_eq!(check(&json!({}), "1234", 0)["outcome"], "unavailable");
    }
}
