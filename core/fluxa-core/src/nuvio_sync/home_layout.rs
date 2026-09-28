use serde_json::{Value, json};
use std::collections::HashMap;

fn text<'a>(value: &'a Value, field: &str) -> &'a str {
    value.get(field).and_then(Value::as_str).unwrap_or_default()
}

pub(crate) fn home_layout_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let categories = args.get("categories")?.as_array()?;
    let manifest_ids: HashMap<&str, &str> = args
        .get("addons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|addon| {
            Some((
                addon.get("transportUrl")?.as_str()?,
                addon.get("manifest")?.get("id")?.as_str()?,
            ))
        })
        .collect();
    let mut items: Vec<&Value> = args
        .get("items")
        .and_then(Value::as_array)
        .map(|items| items.iter().collect())
        .unwrap_or_default();
    items.sort_by_key(|item| item.get("order").and_then(Value::as_i64).unwrap_or(i64::MAX));
    let ranks: HashMap<String, (usize, bool, &str)> = items
        .iter()
        .enumerate()
        .map(|(rank, item)| {
            let key = if item.get("is_collection").and_then(Value::as_bool) == Some(true) {
                format!("c:{}", text(item, "collection_id"))
            } else {
                format!(
                    "a:{}:{}:{}",
                    text(item, "addon_id"),
                    text(item, "type"),
                    text(item, "catalog_id")
                )
            };
            let enabled = item.get("enabled").and_then(Value::as_bool) != Some(false);
            (key, (rank, enabled, text(item, "custom_title")))
        })
        .collect();

    let mut ranked = Vec::new();
    let mut rest = Vec::new();
    for category in categories {
        let key = if text(category, "type") == "collection" {
            format!("c:{}", text(category, "id"))
        } else {
            let addon = manifest_ids
                .get(text(category, "transportUrl"))
                .copied()
                .unwrap_or_default();
            format!(
                "a:{addon}:{}:{}",
                text(category, "type"),
                text(category, "catalogId")
            )
        };
        match ranks.get(&key) {
            Some((_, false, _)) => {}
            Some((rank, true, title)) => {
                let mut category = category.clone();
                if !title.trim().is_empty() {
                    category["name"] = json!(title);
                    category["semanticName"] = json!(title);
                }
                ranked.push((*rank, category));
            }
            None => rest.push(category.clone()),
        }
    }
    ranked.sort_by_key(|(rank, _)| *rank);
    let mut ordered: Vec<Value> = ranked.into_iter().map(|(_, category)| category).collect();
    ordered.extend(rest);
    serde_json::to_string(&ordered).ok()
}
