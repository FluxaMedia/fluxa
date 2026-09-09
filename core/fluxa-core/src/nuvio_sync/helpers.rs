use serde_json::Value;

pub(crate) fn parse(args_json: &str) -> Option<Value> {
    serde_json::from_str(args_json).ok()
}

pub(crate) fn str_field<'a>(value: &'a Value, name: &str) -> Option<&'a str> {
    value.get(name).and_then(Value::as_str)
}

pub(crate) fn canonical_content_type(value: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "series" | "show" | "tv" | "anime" => "series",
        _ => "movie",
    }
}

pub(crate) fn iso_from_ms(ms: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms)
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        .unwrap_or_default()
}
