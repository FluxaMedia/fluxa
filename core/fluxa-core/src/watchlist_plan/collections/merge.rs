use serde_json::{Value, json};

pub(crate) fn collection_merge_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let existing = args.get("existing")?.as_array()?;
    let incoming = args.get("incoming")?.as_array()?;
    let mut merged = existing.clone();
    let mut ids: std::collections::HashSet<&str> = existing
        .iter()
        .filter_map(|item| item.get("id").and_then(Value::as_str))
        .collect();
    merged.extend(
        incoming
            .iter()
            .filter(|item| {
                item.get("id")
                    .and_then(Value::as_str)
                    .is_some_and(|id| ids.insert(id))
            })
            .cloned(),
    );
    serde_json::to_string(&merged).ok()
}

pub(crate) fn collection_folder_items_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let folder = args.get("folder")?;
    let categories = args.get("categories")?.as_array()?;
    let remote_sources = folder
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|source| {
            matches!(
                source.get("provider").and_then(Value::as_str),
                Some("trakt" | "tmdb")
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let modern: Vec<Value> = folder
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|source| source.get("provider").and_then(Value::as_str) == Some("addon"))
        .cloned()
        .collect();
    let fallback = folder
        .get("catalogSources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let sources = if !modern.is_empty() {
        modern
    } else if !fallback.is_empty() {
        fallback
    } else {
        folder.get("catalogId").or_else(|| folder.get("catalog_id")).and_then(Value::as_str)
            .map(|id| vec![json!({"catalogId": id, "type": folder.get("type").or_else(|| folder.get("catalogType"))})]).unwrap_or_default()
    };
    let mut groups: Vec<Value> = Vec::new();
    for source in sources {
        let catalog_id = source
            .get("catalogId")
            .and_then(Value::as_str)
            .unwrap_or("");
        let Some(category) = categories.iter().find(|category| {
            category.get("id").and_then(Value::as_str) == Some(catalog_id)
                || category.get("catalogId").and_then(Value::as_str) == Some(catalog_id)
        }) else {
            continue;
        };
        let content_type = source.get("type").and_then(Value::as_str).unwrap_or("");
        let genre = source
            .get("genre")
            .or_else(|| folder.get("genre"))
            .and_then(Value::as_str);
        let selected: Vec<Value> = category
            .get("items")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|item| {
                genre.is_none_or(|target| {
                    item.get("genres")
                        .and_then(Value::as_array)
                        .is_some_and(|genres| {
                            genres
                                .iter()
                                .filter_map(Value::as_str)
                                .any(|value| value.eq_ignore_ascii_case(target))
                        })
                })
            })
            .cloned()
            .collect();
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.get("type").and_then(Value::as_str) == Some(content_type))
        {
            group.get_mut("items")?.as_array_mut()?.extend(selected);
        } else {
            groups.push(json!({"type": content_type, "items": selected}));
        }
    }
    let items: Vec<Value> = groups
        .iter()
        .flat_map(|group| {
            group
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .cloned()
        })
        .collect();
    serde_json::to_string(
        &json!({"items": items, "groups": groups, "remoteSources": remote_sources}),
    )
    .ok()
}

fn source_key(source: &Value) -> Option<String> {
    match source.get("provider").and_then(Value::as_str) {
        Some("addon") => source.get("catalogId").and_then(Value::as_str).map(str::to_string),
        Some("trakt") => source.get("traktListId").map(|v| v.to_string()),
        Some("tmdb") => source
            .get("tmdbId")
            .map(|v| v.to_string())
            .or_else(|| source.get("tmdbSourceType").and_then(Value::as_str).map(str::to_string)),
        _ => source.get("catalogId").and_then(Value::as_str).map(str::to_string),
    }
}

fn source_title(source: &Value, fallback: &str) -> String {
    source
        .get("title")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

fn round_robin_merge(lists: &[Vec<Value>]) -> Vec<Value> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut merged = Vec::new();
    let max_len = lists.iter().map(Vec::len).max().unwrap_or(0);
    for i in 0..max_len {
        for list in lists {
            let Some(item) = list.get(i) else { continue };
            let Some(id) = item.get("id").and_then(Value::as_str) else { continue };
            if seen.insert(id.to_string()) {
                merged.push(item.clone());
            }
        }
    }
    merged
}

pub(crate) fn collection_folder_tabs_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let folder = args.get("folder")?;
    let categories = args.get("categories")?.as_array()?;
    let remote_items = args.get("remoteItems").and_then(Value::as_object);
    let show_all_tab = args.get("showAllTab").and_then(Value::as_bool).unwrap_or(false);

    let sources: Vec<Value> = folder
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .cloned()
        .collect();

    let mut tabs: Vec<Value> = Vec::new();
    for source in &sources {
        let provider = source.get("provider").and_then(Value::as_str).unwrap_or("addon");
        let Some(key) = source_key(source) else { continue };

        let items: Vec<Value> = if provider == "addon" {
            let genre = source.get("genre").and_then(Value::as_str);
            let Some(category) = categories.iter().find(|category| {
                category.get("id").and_then(Value::as_str) == Some(key.as_str())
                    || category.get("catalogId").and_then(Value::as_str) == Some(key.as_str())
            }) else {
                continue;
            };
            category
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|item| {
                    genre.is_none_or(|target| {
                        item.get("genres")
                            .and_then(Value::as_array)
                            .is_some_and(|genres| {
                                genres
                                    .iter()
                                    .filter_map(Value::as_str)
                                    .any(|value| value.eq_ignore_ascii_case(target))
                            })
                    })
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            remote_items
                .and_then(|map| map.get(&key))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        };

        tabs.push(json!({
            "id": key,
            "title": source_title(source, folder.get("title").and_then(Value::as_str).unwrap_or("")),
            "type": source.get("type").and_then(Value::as_str).unwrap_or("mixed"),
            "items": items,
            "source": source,
        }));
    }

    if show_all_tab && tabs.len() > 1 {
        let lists: Vec<Vec<Value>> = tabs
            .iter()
            .map(|tab| tab.get("items").and_then(Value::as_array).cloned().unwrap_or_default())
            .collect();
        let all_items = round_robin_merge(&lists);
        tabs.insert(0, json!({ "id": "all", "title": "All", "type": "mixed", "items": all_items }));
    }

    serde_json::to_string(&json!({ "tabs": tabs })).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn category(id: &str, items: &[&str]) -> Value {
        json!({
            "id": id,
            "items": items.iter().map(|item_id| json!({ "id": item_id })).collect::<Vec<_>>(),
        })
    }

    #[test]
    fn single_source_folder_has_no_all_tab() {
        let args = json!({
            "folder": { "title": "Marvel", "sources": [{ "provider": "addon", "catalogId": "cat1", "type": "movie" }] },
            "categories": [category("cat1", &["a", "b"])],
            "showAllTab": true,
        });
        let result: Value = serde_json::from_str(&collection_folder_tabs_plan_json(&args.to_string()).unwrap()).unwrap();
        let tabs = result["tabs"].as_array().unwrap();
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0]["id"], "cat1");
    }

    #[test]
    fn multi_source_with_show_all_tab_round_robins_and_dedupes() {
        let args = json!({
            "folder": { "title": "Marvel", "sources": [
                { "provider": "addon", "catalogId": "cat1", "type": "movie" },
                { "provider": "addon", "catalogId": "cat2", "type": "movie" },
            ] },
            "categories": [category("cat1", &["a", "b"]), category("cat2", &["b", "c"])],
            "showAllTab": true,
        });
        let result: Value = serde_json::from_str(&collection_folder_tabs_plan_json(&args.to_string()).unwrap()).unwrap();
        let tabs = result["tabs"].as_array().unwrap();
        assert_eq!(tabs.len(), 3);
        assert_eq!(tabs[0]["id"], "all");
        let all_ids: Vec<&str> = tabs[0]["items"].as_array().unwrap().iter().map(|item| item["id"].as_str().unwrap()).collect();
        assert_eq!(all_ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn multi_source_without_show_all_tab_has_no_synthetic_tab() {
        let args = json!({
            "folder": { "title": "Marvel", "sources": [
                { "provider": "addon", "catalogId": "cat1", "type": "movie" },
                { "provider": "addon", "catalogId": "cat2", "type": "movie" },
            ] },
            "categories": [category("cat1", &["a"]), category("cat2", &["b"])],
            "showAllTab": false,
        });
        let result: Value = serde_json::from_str(&collection_folder_tabs_plan_json(&args.to_string()).unwrap()).unwrap();
        let tabs = result["tabs"].as_array().unwrap();
        assert_eq!(tabs.len(), 2);
        assert!(tabs.iter().all(|tab| tab["id"] != "all"));
    }
}
