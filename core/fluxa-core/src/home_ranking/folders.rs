use crate::search_plan::resolve_transport_url_json;
use serde_json::{Map, Value, json};
use std::collections::HashSet;

const DUPLICATE_FOLDER_PAGE_LIMIT: i64 = 3;

pub(crate) fn folder_page_state_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let state = request.get("state")?;
    let batch = request.get("batch")?;
    let batch_items = batch.get("items")?.as_array()?;
    let mut items = state.get("items")?.as_array()?.clone();
    if batch_items.is_empty() {
        return serde_json::to_string(&json!({
            "skip": state.get("skip"),
            "exhausted": true,
            "duplicateStreak": state.get("duplicateStreak"),
            "items": items,
        }))
        .ok();
    }
    let mut seen: HashSet<String> = items.iter().map(folder_item_key).collect();
    let new_items: Vec<Value> = batch_items
        .iter()
        .filter(|item| seen.insert(folder_item_key(item)))
        .cloned()
        .collect();
    let skip = state.get("skip").and_then(Value::as_i64).unwrap_or(0) + batch_items.len() as i64;
    let duplicate_streak = if new_items.is_empty() {
        state
            .get("duplicateStreak")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            + 1
    } else {
        0
    };
    items.extend(new_items);
    serde_json::to_string(&json!({
        "skip": skip,
        "exhausted": duplicate_streak >= DUPLICATE_FOLDER_PAGE_LIMIT,
        "duplicateStreak": duplicate_streak,
        "items": items,
    }))
    .ok()
}

pub(crate) fn folder_source_page_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let source = request.get("source")?;
    let skip = request
        .get("skip")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0);
    let provider = source.get("provider").and_then(Value::as_str);
    if matches!(provider, Some("trakt" | "tmdb")) {
        return serde_json::to_string(&json!({
            "kind": "remote", "page": skip / 50 + 1,
            "type": if source.get("mediaType").and_then(Value::as_str).is_some_and(|value| value.eq_ignore_ascii_case("TV")) { "series" } else { "movie" }
        })).ok();
    }
    let transport_url = source.get("transportUrl").and_then(Value::as_str)?;
    let content_type = source
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("movie");
    let mut extra = Map::new();
    if let Some(genre) = source
        .get("genre")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        extra.insert("genre".into(), json!(genre));
    }
    if skip > 0 {
        extra.insert("skip".into(), json!(skip));
    }
    serde_json::to_string(&json!({
        "kind": if transport_url == "tmdb://builtin" { "builtinTmdb" } else { "addon" },
        "type": content_type, "transportUrl": transport_url, "catalogId": source.get("catalogId"), "extra": extra
    })).ok()
}

pub(crate) fn merge_folder_sources_json(request_json: &str) -> Option<String> {
    let sources: Vec<Vec<Value>> = serde_json::from_str(request_json).ok()?;
    let mut seen = HashSet::new();
    let mut items = Vec::new();
    let max_len = sources.iter().map(Vec::len).max().unwrap_or(0);
    for index in 0..max_len {
        for source in &sources {
            if let Some(item) = source.get(index)
                && seen.insert(folder_item_key(item))
            {
                items.push(item.clone());
            }
        }
    }
    let mut groups: Vec<Value> = Vec::new();
    for item in &items {
        let content_type = item.get("type").and_then(Value::as_str).unwrap_or("");
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.get("type").and_then(Value::as_str) == Some(content_type))
        {
            group.get_mut("items")?.as_array_mut()?.push(item.clone());
        } else {
            groups.push(json!({"type": content_type, "items": [item]}));
        }
    }
    serde_json::to_string(&json!({"items": items, "groups": groups})).ok()
}

fn folder_item_key(item: &Value) -> String {
    format!(
        "{}:{}",
        item.get("type").and_then(Value::as_str).unwrap_or(""),
        item.get("id").and_then(Value::as_str).unwrap_or("")
    )
}

pub(crate) fn build_home_collection_shelves_json(
    profile_json: &str,
    addons_json: &str,
) -> Option<String> {
    let profile: Value = serde_json::from_str(profile_json).ok()?;
    let collections =
        match profile.get("libraryCollections").and_then(Value::as_array) {
            Some(c) => c,
            None => return serde_json::to_string(
                &json!({ "pinnedShelves": [], "regularShelves": [], "hiddenFolderCategories": [] }),
            )
            .ok(),
        };

    let mut pinned: Vec<Value> = Vec::new();
    let mut regular: Vec<Value> = Vec::new();
    let mut hidden: Vec<Value> = Vec::new();

    for (ci, col) in collections.iter().enumerate() {
        let c = match col.as_object() {
            Some(o) => o,
            None => continue,
        };
        if !c.get("showOnHome").and_then(Value::as_bool).unwrap_or(true) {
            continue;
        }
        let folders = c
            .get("folders")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if folders.is_empty() {
            continue;
        }

        let mut tiles: Vec<Value> = Vec::new();

        for (fi, f) in folders.iter().enumerate() {
            let folder = match f.as_object() {
                Some(o) => o,
                None => continue,
            };
            let folder_title = folder
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if folder_title.is_empty() {
                continue;
            }
            let folder_id = folder
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("col{ci}_f{fi}"));

            let resolved = resolve_folder_catalog_sources(folder, addons_json);
            if !resolved.is_empty() {
                hidden.push(hidden_folder_category(
                    &folder_id,
                    &folder_title,
                    folder,
                    resolved,
                ));
            }
            tiles.push(folder_tile(&folder_id, &folder_title, folder));
        }

        if tiles.is_empty() {
            continue;
        }

        let shelf_id = c
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("col{ci}"));
        let shelf = json!({
            "id": shelf_id,
            "name": c.get("title").and_then(Value::as_str).unwrap_or(""),
            "type": "collection",
            "items": tiles,
            "canLoadMore": false,
            "focusGlowEnabled": c.get("focusGlowEnabled").and_then(Value::as_bool).unwrap_or(true),
        });

        if c.get("pinToTop").and_then(Value::as_bool).unwrap_or(false) {
            pinned.push(shelf);
        } else {
            regular.push(shelf);
        }
    }

    serde_json::to_string(&json!({
        "pinnedShelves": pinned,
        "regularShelves": regular,
        "hiddenFolderCategories": hidden,
    }))
    .ok()
}

pub(crate) fn resolve_folder_catalog_sources(
    folder: &Map<String, Value>,
    addons_json: &str,
) -> Vec<Value> {
    if let Some(sources) = folder
        .get("sources")
        .and_then(Value::as_array)
        .filter(|sources| !sources.is_empty())
    {
        let mut resolved: Vec<Value> = Vec::new();
        for source in sources {
            let provider = source
                .get("provider")
                .and_then(Value::as_str)
                .unwrap_or("addon")
                .to_ascii_lowercase();
            if (provider == "trakt" && source.get("traktListId").and_then(Value::as_i64).is_some())
                || (provider == "tmdb"
                    && source
                        .get("tmdbSourceType")
                        .and_then(Value::as_str)
                        .is_some())
            {
                resolved.push(source.clone());
                continue;
            }
            if provider == "trakt" || provider == "tmdb" {
                continue;
            }
            if source.get("catalogId").and_then(Value::as_str).is_none() {
                continue;
            }
            if let Some(t_url) = resolve_transport_url_json(&source.to_string(), addons_json)
                .and_then(|json| serde_json::from_str::<String>(&json).ok())
            {
                let catalog_id = source
                    .get("catalogId")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let content_type = source
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("movie");
                let mut entry =
                    json!({ "transportUrl": t_url, "catalogId": catalog_id, "type": content_type });
                if let Some(genre) = source
                    .get("genre")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|genre| !genre.is_empty() && !genre.eq_ignore_ascii_case("none"))
                    && let Some(fields) = entry.as_object_mut()
                {
                    fields.insert("genre".to_string(), Value::String(genre.to_string()));
                }
                resolved.push(entry);
            }
        }
        return resolved;
    }

    let mut resolved: Vec<Value> = Vec::new();
    if let Some(sources) = folder.get("catalogSources").and_then(Value::as_array) {
        for source in sources {
            if source.get("catalogId").and_then(Value::as_str).is_none() {
                continue;
            }
            if let Some(t_url) = resolve_transport_url_json(&source.to_string(), addons_json)
                .and_then(|json| serde_json::from_str::<String>(&json).ok())
            {
                let catalog_id = source
                    .get("catalogId")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let content_type = source
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("movie");
                let mut entry =
                    json!({ "transportUrl": t_url, "catalogId": catalog_id, "type": content_type });
                if let Some(genre) = source
                    .get("genre")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|genre| !genre.is_empty() && !genre.eq_ignore_ascii_case("none"))
                    && let Some(fields) = entry.as_object_mut()
                {
                    fields.insert("genre".to_string(), Value::String(genre.to_string()));
                }
                resolved.push(entry);
            }
        }
    }

    if resolved.is_empty()
        && let Some(catalog_id) = folder.get("catalogId").and_then(Value::as_str)
    {
        let src = json!({ "catalogId": catalog_id, "type": "movie" });
        if let Some(t_url) = resolve_transport_url_json(&src.to_string(), addons_json)
            .and_then(|json| serde_json::from_str::<String>(&json).ok())
        {
            let mut entry =
                json!({ "transportUrl": t_url, "catalogId": catalog_id, "type": "movie" });
            if let Some(g) = folder.get("genre").and_then(Value::as_str)
                && let Some(fields) = entry.as_object_mut()
            {
                fields.insert("genre".to_string(), Value::String(g.to_string()));
            }
            resolved.push(entry);
        }
    }
    resolved
}

fn hidden_folder_category(
    folder_id: &str,
    folder_title: &str,
    folder: &Map<String, Value>,
    resolved: Vec<Value>,
) -> Value {
    let mut hcat = json!({
        "id": folder_id,
        "name": folder_title,
        "type": "collection_folder",
        "items": [],
        "catalogSources": resolved,
        "canLoadMore": false,
    });
    if let Some(g) = folder.get("genre").and_then(Value::as_str)
        && let Some(fields) = hcat.as_object_mut()
    {
        fields.insert("addonGenre".to_string(), Value::String(g.to_string()));
    }
    hcat
}

fn folder_tile(folder_id: &str, folder_title: &str, folder: &Map<String, Value>) -> Value {
    let img_url = folder
        .get("coverImageUrl")
        .and_then(Value::as_str)
        .or_else(|| folder.get("imageUrl").and_then(Value::as_str))
        .unwrap_or("");
    let bg_url = folder
        .get("heroBackdropUrl")
        .and_then(Value::as_str)
        .unwrap_or(img_url);
    let mut tile = json!({
        "id": folder_id,
        "type": "catalog_folder",
        "name": folder_title,
        "poster": if img_url.is_empty() { Value::Null } else { Value::String(img_url.to_string()) },
        "background": if bg_url.is_empty() { Value::Null } else { Value::String(bg_url.to_string()) },
        "reason": folder
            .get("shape")
            .or_else(|| folder.get("tileShape"))
            .and_then(Value::as_str)
            .unwrap_or("poster"),
    });
    if let Some(sources) = folder.get("catalogSources").or_else(|| folder.get("sources"))
        && let Some(fields) = tile.as_object_mut()
    {
        fields.insert("collectionSources".to_string(), sources.clone());
    }
    if let Some(logo) = folder.get("titleLogoUrl").and_then(Value::as_str)
        && let Some(fields) = tile.as_object_mut()
    {
        fields.insert("logo".to_string(), Value::String(logo.to_string()));
    }
    if let Some(info) = folder.get("catalogTitle").and_then(Value::as_str)
        && let Some(fields) = tile.as_object_mut()
    {
        fields.insert("releaseInfo".to_string(), Value::String(info.to_string()));
    }
    if let Some(gif) = folder.get("focusGifUrl").and_then(Value::as_str)
        && let Some(fields) = tile.as_object_mut()
    {
        fields.insert("focusGifUrl".to_string(), Value::String(gif.to_string()));
    }
    if let Some(emoji) = folder.get("coverEmoji").and_then(Value::as_str)
        && let Some(fields) = tile.as_object_mut()
    {
        fields.insert("coverEmoji".to_string(), Value::String(emoji.to_string()));
    }
    if let Some(fields) = tile.as_object_mut() {
        fields.insert(
            "hideTitle".to_string(),
            Value::Bool(folder.get("hideTitle").and_then(Value::as_bool).unwrap_or(false)),
        );
        fields.insert(
            "focusGifEnabled".to_string(),
            Value::Bool(folder.get("focusGifEnabled").and_then(Value::as_bool).unwrap_or(true)),
        );
    }
    tile
}

#[cfg(test)]
mod tests {
    use super::build_home_collection_shelves_json;
    use serde_json::Value;

    #[test]
    fn prime_video_collection_keeps_raw_and_resolved_sources() {
        let profile = serde_json::json!({
            "libraryCollections": [{
                "id": "collections.streaming",
                "title": "Streaming",
                "pinToTop": true,
                "folders": [{
                    "id": "collections.streaming.prime-video",
                    "title": "Prime Video",
                    "catalogSources": [{
                        "addonId": "aio-metadata",
                        "catalogId": "tmdb.discover.movie.streaming.prime-video",
                        "type": "movie",
                        "genre": "None"
                    }]
                }]
            }]
        });
        let addons = serde_json::json!([{
            "transportUrl": "https://aiometadata.elfhosted.com/stremio/configured/manifest.json",
            "manifest": {
                "id": "aio-metadata",
                "catalogs": [{ "id": "tmdb.top", "type": "movie" }]
            }
        }]);

        let result: Value = serde_json::from_str(
            &build_home_collection_shelves_json(&profile.to_string(), &addons.to_string()).unwrap(),
        ).unwrap();
        let tile = &result["pinnedShelves"][0]["items"][0];
        assert_eq!(tile["id"], "collections.streaming.prime-video");
        assert_eq!(tile["collectionSources"][0]["catalogId"], "tmdb.discover.movie.streaming.prime-video");
        assert_eq!(result["hiddenFolderCategories"][0]["catalogSources"][0]["transportUrl"], "https://aiometadata.elfhosted.com/stremio/configured/manifest.json");
    }
}
