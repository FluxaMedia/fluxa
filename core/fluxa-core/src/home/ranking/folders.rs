use crate::catalog::search::resolve_transport_url_json;
use serde_json::{Map, Value, json};

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
                let source_categories = folder_source_categories(&folder_id, &resolved, addons_json);
                let mut category = hidden_folder_category(&folder_id, &folder_title, folder, resolved);
                if let Some(fields) = category.as_object_mut() {
                    fields.insert(
                        "viewMode".to_owned(),
                        c.get("viewMode").cloned().unwrap_or_else(|| json!("TABBED_GRID")),
                    );
                    fields.insert(
                        "showAllTab".to_owned(),
                        c.get("showAllTab").cloned().unwrap_or(Value::Bool(true)),
                    );
                    fields.insert("sourceCount".to_owned(), json!(source_categories.len()));
                }
                hidden.push(category);
                hidden.extend(source_categories);
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

fn folder_source_categories(folder_id: &str, resolved: &[Value], addons_json: &str) -> Vec<Value> {
    let addons: Value = serde_json::from_str(addons_json).unwrap_or(Value::Null);
    resolved
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let mut category = json!({
                "id": format!("{folder_id}#{index}"),
                "type": "collection_folder_source",
                "folderId": folder_id,
                "sourceIndex": index,
                "items": [],
                "canLoadMore": true,
            });
            let fields = category.as_object_mut().expect("object literal");
            let provider = source.get("provider").and_then(Value::as_str);
            if matches!(provider, Some("trakt" | "tmdb")) {
                fields.insert("remoteSource".to_owned(), source.clone());
                fields.insert(
                    "name".to_owned(),
                    source.get("title").cloned().unwrap_or_else(|| json!(provider)),
                );
                fields.insert("contentType".to_owned(), source.get("mediaType").cloned().unwrap_or(json!("movie")));
                return category;
            }
            let transport_url = source.get("transportUrl").and_then(Value::as_str).unwrap_or("");
            let catalog_id = source.get("catalogId").and_then(Value::as_str).unwrap_or("");
            let content_type = source.get("type").and_then(Value::as_str).unwrap_or("movie");
            let name = addons
                .as_array()
                .into_iter()
                .flatten()
                .filter(|addon| addon.get("transportUrl").and_then(Value::as_str) == Some(transport_url))
                .filter_map(|addon| addon.pointer("/manifest/catalogs").and_then(Value::as_array))
                .flatten()
                .find(|catalog| {
                    catalog.get("id").and_then(Value::as_str) == Some(catalog_id)
                        && catalog.get("type").and_then(Value::as_str) == Some(content_type)
                })
                .and_then(|catalog| catalog.get("name").and_then(Value::as_str))
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(capitalize)
                .unwrap_or_else(|| type_label(content_type));
            let genre = source
                .get("genre")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|genre| !genre.is_empty() && !genre.eq_ignore_ascii_case("none"));
            let name = match genre {
                Some(genre) => format!("{name} · {genre}"),
                None => name,
            };
            fields.insert("name".to_owned(), json!(name));
            fields.insert("transportUrl".to_owned(), json!(transport_url));
            fields.insert("catalogId".to_owned(), json!(catalog_id));
            fields.insert("contentType".to_owned(), json!(content_type));
            if let Some(genre) = source.get("genre") {
                fields.insert("genre".to_owned(), genre.clone());
            }
            category
        })
        .collect()
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
    if let Some(sources) = folder
        .get("catalogSources")
        .or_else(|| folder.get("sources"))
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
            Value::Bool(
                folder
                    .get("hideTitle")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            ),
        );
        fields.insert(
            "focusGifEnabled".to_string(),
            Value::Bool(
                folder
                    .get("focusGifEnabled")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
            ),
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
        )
        .unwrap();
        let tile = &result["pinnedShelves"][0]["items"][0];
        assert_eq!(tile["id"], "collections.streaming.prime-video");
        assert_eq!(
            tile["collectionSources"][0]["catalogId"],
            "tmdb.discover.movie.streaming.prime-video"
        );
        assert_eq!(
            result["hiddenFolderCategories"][0]["catalogSources"][0]["transportUrl"],
            "https://aiometadata.elfhosted.com/stremio/configured/manifest.json"
        );
    }

    #[test]
    fn folder_sources_become_individual_categories() {
        let profile = serde_json::json!({
            "libraryCollections": [{
                "id": "c1",
                "title": "Mix",
                "viewMode": "ROWS",
                "showAllTab": false,
                "folders": [{
                    "id": "c1.f1",
                    "title": "Folder",
                    "catalogSources": [
                        {"addonId": "a", "catalogId": "top", "type": "movie"},
                        {"addonId": "a", "catalogId": "top", "type": "series"}
                    ]
                }]
            }]
        });
        let addons = serde_json::json!([{
            "transportUrl": "https://addon.test/manifest.json",
            "manifest": {
                "id": "a",
                "catalogs": [
                    { "id": "top", "type": "movie", "name": "Top Movies" },
                    { "id": "top", "type": "series", "name": "Top Series" }
                ]
            }
        }]);
        let result: Value = serde_json::from_str(
            &build_home_collection_shelves_json(&profile.to_string(), &addons.to_string()).unwrap(),
        )
        .unwrap();
        let hidden = result["hiddenFolderCategories"].as_array().unwrap();
        let sources: Vec<&Value> = hidden
            .iter()
            .filter(|category| category["type"] == "collection_folder_source")
            .collect();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[1]["id"], "c1.f1#1");
        assert_eq!(sources[1]["name"], "Top Series");
        assert_eq!(sources[1]["folderId"], "c1.f1");
        assert_eq!(hidden[0]["viewMode"], "ROWS");
        assert_eq!(hidden[0]["showAllTab"], false);
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn type_label(content_type: &str) -> String {
    match content_type.to_ascii_lowercase().as_str() {
        "movie" => "Movies".to_owned(),
        "series" => "Series".to_owned(),
        other => capitalize(other),
    }
}
