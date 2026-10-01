use super::text::{json_string_list, parse_string_list};
use serde_json::Value;

pub(crate) fn cs3_metadata_feed_options_json(descriptors_json: &str) -> Option<String> {
    let descriptors = serde_json::from_str::<Vec<Value>>(descriptors_json).ok()?;
    let options = descriptors
        .into_iter()
        .filter_map(|descriptor| {
            let plugin_name = descriptor.get("pluginName")?.as_str()?;
            let catalog_name = descriptor.get("catalogName")?.as_str()?;
            let catalog_index = descriptor.get("catalogIndex")?.as_i64()? as i32;
            let key = super::text::cs3_catalog_feed_key(plugin_name, catalog_name, catalog_index);
            Some(serde_json::json!({
                "key": key,
                "label": format!("{catalog_name} - {plugin_name}"),
                "transportUrl": format!("cs3://{key}"),
                "type": "all",
                "id": key,
            }))
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&options).ok()
}

pub(crate) fn effective_metadata_feed_selection_json(
    selected_keys_json: &str,
    available_keys_json: &str,
) -> Option<String> {
    let selected = serde_json::from_str::<Option<Vec<String>>>(selected_keys_json)
        .ok()
        .flatten()?;
    let available = parse_string_list(available_keys_json);
    let filtered = selected
        .into_iter()
        .filter(|key| available.contains(key))
        .collect::<Vec<_>>();
    json_string_list(&filtered)
}

pub(crate) fn ordered_metadata_feed_keys(
    option_keys_json: &str,
    order_json: &str,
) -> Option<String> {
    let option_keys = parse_string_list(option_keys_json);
    let order = parse_string_list(order_json);
    let mut output = Vec::<String>::new();
    for key in order {
        if option_keys.contains(&key) && !output.contains(&key) {
            output.push(key);
        }
    }
    for key in option_keys {
        if !output.contains(&key) {
            output.push(key);
        }
    }
    json_string_list(&output)
}
