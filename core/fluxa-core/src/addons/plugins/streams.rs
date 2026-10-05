use serde_json::{Map, Value, json};

pub fn plugin_scraper_streams(raw: &Value, scraper_name: &str) -> Vec<Value> {
    raw.as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| plugin_stream(item, scraper_name))
        .collect()
}

pub fn plugin_scraper_enabled_for(scraper: &Value, media_type: &str) -> bool {
    if scraper.get("enabled").and_then(Value::as_bool) == Some(false) {
        return false;
    }
    let wanted = match media_type {
        "series" | "tv" | "show" => "tv",
        _ => "movie",
    };
    scraper
        .get("supportedTypes")
        .and_then(Value::as_array)
        .is_none_or(|types| {
            types.is_empty() || types.iter().any(|kind| kind.as_str() == Some(wanted))
        })
}

fn plugin_stream(item: &Value, scraper_name: &str) -> Option<Value> {
    let text = |key: &str| {
        item.get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    };
    let url = text("url");
    let info_hash = text("infoHash");
    if url.is_none() && info_hash.is_none() {
        return None;
    }
    let title = text("title")
        .or_else(|| text("name"))
        .unwrap_or(scraper_name);
    let name = [
        text("name").unwrap_or(scraper_name),
        text("quality").unwrap_or(""),
    ]
    .iter()
    .filter(|part| !part.is_empty())
    .copied()
    .collect::<Vec<_>>()
    .join(" · ");
    let mut description = vec![title.to_owned()];
    let details = [text("size"), text("language"), text("provider")]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");
    if !details.is_empty() {
        description.push(details);
    }
    let mut stream = Map::new();
    stream.insert("name".into(), json!(name));
    stream.insert("title".into(), json!(title));
    stream.insert("description".into(), json!(description.join("\n")));
    stream.insert("addonName".into(), json!(scraper_name));
    if let Some(url) = url {
        stream.insert("url".into(), json!(url));
    }
    if let Some(info_hash) = info_hash {
        stream.insert("infoHash".into(), json!(info_hash));
    }
    let mut hints = Map::new();
    if let Some(headers) = item
        .get("headers")
        .and_then(Value::as_object)
        .filter(|headers| !headers.is_empty())
    {
        hints.insert("notWebReady".into(), json!(true));
        hints.insert("proxyHeaders".into(), json!({ "request": headers }));
    }
    if let Some(bytes) = text("size").and_then(parse_size) {
        hints.insert("videoSize".into(), json!(bytes));
    }
    if !hints.is_empty() {
        stream.insert("behaviorHints".into(), Value::Object(hints));
    }
    if let Some(seeders) = item.get("seeders").and_then(Value::as_i64) {
        stream.insert("seeders".into(), json!(seeders));
    }
    Some(Value::Object(stream))
}

fn parse_size(text: &str) -> Option<i64> {
    let text = text.trim();
    let split = text.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ','))?;
    let number: f64 = text[..split].replace(',', ".").parse().ok()?;
    let factor = match text[split..].trim().to_ascii_uppercase().as_str() {
        "B" => 1.0,
        "KB" | "KIB" => 1024.0,
        "MB" | "MIB" => 1024.0 * 1024.0,
        "GB" | "GIB" => 1024.0 * 1024.0 * 1024.0,
        "TB" | "TIB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((number * factor) as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scraper_results_without_a_playable_source_are_dropped() {
        let raw = json!([
            {"title": "A", "url": "https://x/a.mp4", "quality": "1080p", "size": "1.5 GB"},
            {"title": "B"}
        ]);
        let streams = plugin_scraper_streams(&raw, "Scraper");
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0]["name"], "Scraper · 1080p");
        assert_eq!(streams[0]["addonName"], "Scraper");
        assert_eq!(streams[0]["behaviorHints"]["videoSize"], 1610612736_i64);
    }

    #[test]
    fn scrapers_are_filtered_by_supported_type() {
        let scraper = json!({"enabled": true, "supportedTypes": ["movie"]});
        assert!(plugin_scraper_enabled_for(&scraper, "movie"));
        assert!(!plugin_scraper_enabled_for(&scraper, "series"));
        assert!(!plugin_scraper_enabled_for(
            &json!({"enabled": false}),
            "movie"
        ));
    }
}
