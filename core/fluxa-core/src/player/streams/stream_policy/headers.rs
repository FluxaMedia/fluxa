use serde_json::Value;

pub fn selected_stream_headers(snapshot: &Value) -> Vec<(String, String)> {
    let player = &snapshot["player"];
    let Some(stream) = player["currentStreamIndex"]
        .as_u64()
        .and_then(|index| player["currentStreams"].get(index as usize))
    else {
        return Vec::new();
    };
    let hints = stream.get("behaviorHints");
    [
        stream.get("headers"),
        hints.and_then(|hints| hints.get("requestHeaders")),
        hints.and_then(|hints| hints.pointer("/proxyHeaders/request")),
    ]
    .into_iter()
    .flatten()
    .filter_map(Value::as_object)
    .flatten()
    .filter_map(|(name, value)| Some((name.clone(), value.as_str()?.to_owned())))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn proxy_headers_of_the_selected_stream_only() {
        let snapshot = json!({"player": {
            "currentStreamIndex": 1,
            "currentStreams": [
                {"behaviorHints": {"proxyHeaders": {"request": {"Referer": "https://a/"}}}},
                {"behaviorHints": {"proxyHeaders": {"request": {"Referer": "https://b/"}}}},
            ],
        }});
        assert_eq!(
            selected_stream_headers(&snapshot),
            vec![("Referer".to_owned(), "https://b/".to_owned())]
        );
    }

    #[test]
    fn missing_selection_gives_no_headers() {
        assert!(selected_stream_headers(&json!({"player": {}})).is_empty());
    }
}
