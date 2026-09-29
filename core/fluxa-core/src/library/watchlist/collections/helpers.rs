use serde_json::Value;

pub(super) fn timestamp(item: &Value, key: &str) -> i64 {
    item.get(key)
        .and_then(|value| {
            value
                .as_i64()
                .map(|value| {
                    if value < 10_000_000_000 {
                        value * 1_000
                    } else {
                        value
                    }
                })
                .or_else(|| {
                    value
                        .as_str()
                        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                        .map(|value| value.timestamp_millis())
                })
        })
        .unwrap_or(0)
}
