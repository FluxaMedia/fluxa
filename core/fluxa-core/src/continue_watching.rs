use crate::library_state::{
    build_continue_watching_from_progress_json, normalized_continue_watching_source,
};
use serde_json::{Value, json};

mod nuvio;

pub(crate) fn continue_watching_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let source = normalized_continue_watching_source(args.get("source").and_then(Value::as_str));
    match source {
        "nuvio" => nuvio::continue_watching_json(args_json),
        "local" => build_continue_watching_from_progress_json(
            &args.get("progress").cloned().unwrap_or_else(|| json!({})).to_string(),
        ),
        provider => provider_rows_json(&args, provider),
    }
}

fn provider_rows_json(args: &Value, provider: &str) -> Option<String> {
    let rows: Vec<Value> = args
        .get("providerWatching")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter(|row| row_source(row).is_none_or(|row_source| row_source == provider))
                .map(|row| tag_source(row, provider))
                .collect()
        })
        .unwrap_or_default();
    serde_json::to_string(&rows).ok()
}

fn row_source(row: &Value) -> Option<&str> {
    row.get("source")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|source| !source.is_empty())
}

fn tag_source(row: &Value, provider: &str) -> Value {
    let mut row = row.clone();
    if let Some(object) = row.as_object_mut() {
        object.insert("source".into(), Value::String(provider.to_string()));
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_provider_row_from_another_service_is_left_out() {
        let args = json!({
            "source": "trakt",
            "providerWatching": [
                { "id": "tt1", "source": "trakt" },
                { "id": "tt2", "source": "simkl" },
                { "id": "tt3" },
            ],
        });
        let rows: Vec<Value> =
            serde_json::from_str(&continue_watching_json(&args.to_string()).unwrap()).unwrap();
        let ids: Vec<&str> = rows
            .iter()
            .filter_map(|row| row.get("id").and_then(Value::as_str))
            .collect();
        assert_eq!(ids, ["tt1", "tt3"]);
        assert!(rows.iter().all(|row| row["source"] == "trakt"));
    }

    #[test]
    fn local_falls_back_to_the_progress_map() {
        let args = json!({
            "source": "local",
            "progress": {
                "tt1": {
                    "meta": { "id": "tt1", "name": "Show", "type": "series" },
                    "timeOffset": 300,
                    "duration": 2400,
                    "lastVideoId": "tt1:1:2",
                    "savedAt": "2026-01-01T00:00:00Z",
                },
            },
        });
        let rows: Vec<Value> =
            serde_json::from_str(&continue_watching_json(&args.to_string()).unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["lastVideoId"], "tt1:1:2");
    }
}
