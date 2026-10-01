use super::text::form_decode;
use serde_json::{Map, Value};

pub(crate) fn parse_extra_args_json(extra: &str) -> Option<String> {
    let mut map = Map::new();
    for part in extra.split('&') {
        let key = part.split_once('=').map(|(key, _)| key).unwrap_or(part);
        if key.is_empty() {
            continue;
        }
        let value = part.split_once('=').map(|(_, value)| value).unwrap_or("");
        map.insert(form_decode(key), Value::String(form_decode(value)));
    }
    serde_json::to_string(&Value::Object(map)).ok()
}
