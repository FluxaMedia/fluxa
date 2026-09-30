use super::helpers::parse;
use serde_json::{Value, json};

pub(crate) fn addon_reconciliation_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let current = args.get("current")?.as_array()?;
    let desired = args.get("desired")?.as_array()?;
    let user_id = args.get("userId").and_then(Value::as_str).unwrap_or("");
    let profile_id = args
        .get("profileId")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let desired_by_url: std::collections::BTreeMap<String, Value> = desired.iter().enumerate().filter_map(|(index, addon)| {
        let url = addon.get("url")?.as_str()?.trim();
        (!url.is_empty()).then(|| (url.to_string(), json!({"url":url,"name":addon.get("name").and_then(Value::as_str),"enabled":addon.get("enabled").and_then(Value::as_bool).unwrap_or(true),"sort_order":addon.get("sort_order").and_then(Value::as_i64).unwrap_or(index as i64)})))
    }).collect();
    let delete_ids = current
        .iter()
        .filter_map(|addon| {
            let url = addon.get("url")?.as_str()?;
            (!desired_by_url.contains_key(url))
                .then(|| addon.get("id").cloned())
                .flatten()
        })
        .collect::<Vec<_>>();
    let updates = current
        .iter()
        .filter_map(|addon| {
            let url = addon.get("url")?.as_str()?;
            Some(json!({"id":addon.get("id")?,"payload":desired_by_url.get(url)?}))
        })
        .collect::<Vec<_>>();
    let creates = desired_by_url
        .iter()
        .filter(|(url, _)| {
            !current
                .iter()
                .any(|addon| addon.get("url").and_then(Value::as_str) == Some(url.as_str()))
        })
        .map(|(_, payload)| {
            let mut payload = payload.as_object().cloned().unwrap_or_default();
            payload.insert("user_id".into(), Value::String(user_id.to_string()));
            payload.insert("profile_id".into(), json!(profile_id));
            Value::Object(payload)
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&json!({"deleteIds":delete_ids,"updates":updates,"creates":creates})).ok()
}

#[expect(
    clippy::indexing_slicing,
    reason = "index is obtained from the current remote vector before replacement"
)]
pub(crate) fn library_mutation_plan_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let mut remote = args.get("remote")?.as_array()?.clone();
    let item = args.get("item")?;
    let command = args.get("command")?.as_str()?;
    let id = item.get("id").and_then(Value::as_str).unwrap_or("");
    let content_type = item.get("type").and_then(Value::as_str).unwrap_or("movie");
    if id.is_empty() {
        return None;
    }
    let existing = remote.iter().position(|entry| {
        entry.get("content_id").and_then(Value::as_str) == Some(id)
            && entry.get("content_type").and_then(Value::as_str) == Some(content_type)
    });
    if command == "remove" {
        if let Some(index) = existing {
            remote.remove(index);
        }
    } else {
        let entry = json!({
            "content_id": id, "content_type": content_type, "name": item.get("name").and_then(Value::as_str).unwrap_or(""),
            "poster": item.get("poster"), "poster_shape": "poster", "background": item.get("background"), "description": item.get("description"),
            "release_info": item.get("releaseInfo"), "imdb_rating": item.get("imdbRating"),
            "genres": item.get("genres").and_then(Value::as_array).map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>()).unwrap_or_default(),
            "addon_base_url": Value::Null, "added_at": args.get("nowMs"),
        });
        if let Some(index) = existing {
            let mut merged = remote[index].as_object().cloned().unwrap_or_default();
            merged.extend(entry.as_object()?.clone());
            remote[index] = Value::Object(merged);
        } else {
            remote.push(entry);
        }
    }
    serde_json::to_string(&remote).ok()
}
