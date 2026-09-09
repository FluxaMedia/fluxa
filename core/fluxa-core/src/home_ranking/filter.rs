use serde_json::{Value, json};

pub(crate) fn filter_home_categories_json(categories_json: &str, filter: &str) -> Option<String> {
    let categories = serde_json::from_str::<Vec<Value>>(categories_json).ok()?;
    let filtered = categories
        .into_iter()
        .filter_map(|mut category| {
            let id = category.get("id").and_then(Value::as_str).unwrap_or_default();
            let category_type = category
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let original_items = category
                .get("items")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let action_row = matches!(id, "continue_watching" | "upcoming" | "library");
            let items = if filter == "all" {
                original_items.clone()
            } else {
                let matching: Vec<Value> = original_items
                    .iter()
                    .filter(|item| item_matches_filter(item, filter))
                    .cloned()
                    .collect();
                if action_row && matching.is_empty() {
                    original_items.clone()
                } else {
                    matching
                }
            };
            if items.is_empty() && category_type != "collection_folder" {
                return None;
            }
            if items != original_items {
                category["items"] = json!(items);
            }
            Some(category)
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&filtered).ok()
}

fn item_matches_filter(item: &Value, filter: &str) -> bool {
    match filter {
        "movie" => item.get("type").and_then(Value::as_str) == Some("movie"),
        "series" => matches!(
            item.get("type").and_then(Value::as_str),
            Some("series" | "tv" | "anime")
        ),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::filter_home_categories_json;
    use serde_json::Value;

    #[test]
    fn filters_catalogs_and_keeps_action_row_fallback() {
        let result: Vec<Value> = serde_json::from_str(
            &filter_home_categories_json(
                r#"[{"id":"continue_watching","type":"mixed","items":[{"id":"m","type":"movie"}]},{"id":"catalog","type":"mixed","items":[{"id":"m","type":"movie"},{"id":"s","type":"series"}]}]"#,
                "series",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result[0]["items"][0]["id"], "m");
        assert_eq!(result[1]["items"][0]["id"], "s");
    }

    #[test]
    fn drops_empty_catalogs_but_keeps_empty_collection_folders() {
        let result: Vec<Value> = serde_json::from_str(
            &filter_home_categories_json(
                r#"[{"id":"catalog","type":"mixed","items":[{"id":"m","type":"movie"}]},{"id":"folder","type":"collection_folder","items":[]}]"#,
                "series",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["id"], "folder");
    }
}
