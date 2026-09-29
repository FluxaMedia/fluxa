use serde_json::{Map, Value, json};
use url::form_urlencoded;

mod jellyfin;
mod plex;
pub(crate) mod routes;
#[cfg(test)]
mod tests;

const PRODUCT: &str = "Fluxa";
const TICKS_PER_MS: i64 = 10_000;

pub(crate) struct Server {
    kind: String,
    base: String,
    token: String,
    user_id: String,
    device_id: String,
    key: String,
}

impl Server {
    fn from(args: &Value) -> Self {
        let server = args.get("server").unwrap_or(&Value::Null);
        let text = |name: &str| str_field(server, name).to_owned();
        let kind = str_field(args, "kind").to_owned();
        let mut base = text("baseUrl").trim_end_matches('/').to_owned();
        if kind == "emby" && !base.ends_with("/emby") {
            base.push_str("/emby");
        }
        Server {
            kind,
            base,
            token: text("token"),
            user_id: text("userId"),
            device_id: text("deviceId"),
            key: text("key"),
        }
    }

    fn item_id(&self, raw: &str) -> String {
        format!("ms:{}:{raw}", self.key)
    }

    fn media_source(&self, raw: &str) -> Value {
        json!({"kind": self.kind, "key": self.key, "itemId": raw})
    }
}

fn str_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn int_field(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

fn float_field(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn list<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn query(pairs: &[(&str, String)]) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (name, value) in pairs {
        serializer.append_pair(name, value);
    }
    serializer.finish()
}

fn plan(method: &str, url: String, headers: Vec<(&str, String)>, body: Value) -> Value {
    let headers: Map<String, Value> = headers
        .into_iter()
        .filter(|(_, value)| !value.is_empty())
        .map(|(name, value)| (name.to_owned(), json!(value)))
        .collect();
    json!({"method": method, "url": url, "headers": headers, "body": body})
}

fn put(map: &mut Map<String, Value>, key: &str, value: Option<Value>) {
    if let Some(value) = value.filter(|value| !value.is_null()) {
        map.insert(key.to_owned(), value);
    }
}

fn non_empty(text: &str) -> Option<Value> {
    (!text.is_empty()).then(|| json!(text))
}

fn size_text(bytes: i64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1e9)
    } else {
        format!("{} MB", bytes / 1_000_000)
    }
}

fn stream_title(parts: &[String]) -> String {
    parts
        .iter()
        .filter(|part| !part.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" · ")
}

fn param_i64(params: &Value, key: &str, default: i64) -> i64 {
    int_field(params, key).unwrap_or(default)
}

fn item_kinds(params: &Value) -> Option<&'static str> {
    match str_field(params, "kind") {
        "movie" => Some("Movie"),
        "series" => Some("Series"),
        _ => None,
    }
}

fn external_id(id: &str) -> Option<(&'static str, &str)> {
    if id.starts_with("tt") {
        Some(("imdb", id))
    } else {
        id.strip_prefix("tmdb:").map(|tmdb| ("tmdb", tmdb))
    }
}

pub(crate) fn request_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let server = Server::from(&args);
    let operation = str_field(&args, "operation");
    let params = args.get("params").unwrap_or(&Value::Null);
    let plan = match server.kind.as_str() {
        "jellyfin" | "emby" => jellyfin::request(&server, operation, params)?,
        "plex" => plex::request(&server, operation, params)?,
        _ => return None,
    };
    serde_json::to_string(&plan).ok()
}

pub(crate) fn parse_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let server = Server::from(&args);
    let operation = str_field(&args, "operation");
    let params = args.get("params").unwrap_or(&Value::Null);
    let status = args.get("status").and_then(Value::as_u64).unwrap_or(0);
    let body = match args.get("body") {
        Some(Value::String(text)) => serde_json::from_str(text).unwrap_or(Value::Null),
        Some(body) => body.clone(),
        None => Value::Null,
    };
    let ok = (200..300).contains(&status);
    let parsed = match server.kind.as_str() {
        "jellyfin" | "emby" => jellyfin::parse(&server, operation, params, ok, &body),
        "plex" => plex::parse(&server, operation, params, ok, &body),
        _ => return None,
    };
    let mut parsed = parsed;
    if let Some(object) = parsed.as_object_mut() {
        object.insert("status".into(), json!(status));
    }
    serde_json::to_string(&parsed).ok()
}
