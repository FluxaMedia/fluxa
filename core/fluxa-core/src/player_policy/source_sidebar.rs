use crate::core_error::{CoreError, LogAndDiscard};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceSidebarRequest {
    #[serde(default)]
    streams: Vec<Value>,
    #[serde(default)]
    current_stream_index: i32,
    #[serde(default)]
    available_addons: Vec<String>,
    #[serde(default)]
    selected_addon: Option<String>,
}

/// Build the source sidebar option state: which streams to show and which is selected.
pub fn player_source_sidebar_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<SourceSidebarRequest>(request_json)
        .map_err(|e| CoreError::BadInput {
            context: "player_source_sidebar_plan_json",
            detail: e.to_string(),
        })
        .log_discard()?;
    let current_index = request.current_stream_index.clamp(0, i32::MAX);
    let streams_by_addon: std::collections::BTreeMap<String, Vec<(usize, &Value)>> = request
        .streams
        .iter()
        .enumerate()
        .fold(std::collections::BTreeMap::new(), |mut acc, (i, stream)| {
            let addon_name = stream
                .get("addonName")
                .and_then(Value::as_str)
                .unwrap_or("Unknown")
                .to_string();
            acc.entry(addon_name).or_default().push((i, stream));
            acc
        });

    let groups: Vec<Value> = streams_by_addon
        .into_iter()
        .map(|(addon_name, streams)| {
            let entries: Vec<Value> = streams
                .iter()
                .map(|(idx, stream)| {
                    json!({
                        "index": idx,
                        "isSelected": *idx == current_index as usize,
                        "title": stream.get("title").cloned().unwrap_or_else(|| json!("")),
                        "name": stream.get("name").cloned().unwrap_or_else(|| json!("")),
                        "quality": stream.get("quality").cloned().unwrap_or(Value::Null)
                    })
                })
                .collect();
            json!({
                "addonName": addon_name,
                "streams": entries,
                "isSelected": entries.iter().any(|e| e["isSelected"].as_bool().unwrap_or(false))
            })
        })
        .collect();

    serde_json::to_string(&json!({
        "groups": groups,
        "currentStreamIndex": current_index,
        "availableAddons": request.available_addons,
        "selectedAddon": request.selected_addon
    }))
    .ok()
}
