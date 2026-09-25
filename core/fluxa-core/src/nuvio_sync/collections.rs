use super::helpers::{parse, str_field};
use serde_json::{Value, json};

fn map_catalog_source(source: &Value) -> Option<Value> {
    let catalog_id = str_field(source, "catalogId")?.trim();
    let content_type = str_field(source, "type")?.trim();
    if catalog_id.is_empty() || content_type.is_empty() {
        return None;
    }
    let addon_id = str_field(source, "addonId").unwrap_or("");
    Some(json!({
        "addonId": addon_id,
        "catalogId": catalog_id,
        "type": content_type,
        "genre": str_field(source, "genre"),
        "displayName": str_field(source, "title"),
    }))
}

fn map_folder_source(source: &Value) -> Option<Value> {
    let provider = str_field(source, "provider")
        .unwrap_or("addon")
        .to_lowercase();
    let mut out = source.as_object().cloned().unwrap_or_default();
    match provider.as_str() {
        "trakt" => {
            source.get("traktListId").and_then(Value::as_i64)?;
            out.insert("provider".into(), Value::String("trakt".into()));
            for field in ["title", "mediaType", "sortBy", "sortHow"] {
                if !source.get(field).map(Value::is_string).unwrap_or(false) {
                    out.remove(field);
                }
            }
        }
        "tmdb" => {
            str_field(source, "tmdbSourceType")?;
            out.insert("provider".into(), Value::String("tmdb".into()));
            for field in ["title", "mediaType", "sortBy", "sortHow"] {
                if !source.get(field).map(Value::is_string).unwrap_or(false) {
                    out.remove(field);
                }
            }
            if !source
                .get("tmdbId")
                .map(|v| v.is_i64() || v.is_u64())
                .unwrap_or(false)
            {
                out.remove("tmdbId");
            }
            let filters_ok = source
                .get("filters")
                .map(|v| v.is_object())
                .unwrap_or(false);
            if !filters_ok {
                out.remove("filters");
            }
        }
        "addon" => {
            str_field(source, "addonId")?;
            str_field(source, "type")?;
            str_field(source, "catalogId")?;
            out.insert("provider".into(), Value::String("addon".into()));
            if !source.get("genre").map(Value::is_string).unwrap_or(false) {
                out.remove("genre");
            }
        }
        _ => return None,
    }
    Some(Value::Object(out))
}

fn normalize_tile_shape(value: Option<&str>) -> String {
    let raw = value.unwrap_or("poster").to_lowercase();
    if raw == "landscape" {
        "wide".to_string()
    } else {
        raw
    }
}

fn map_folder(folder: &Value, collection_id: &str, folder_index: usize) -> Value {
    let mut out = folder.as_object().cloned().unwrap_or_default();
    let fallback_id = format!("{collection_id}_folder_{folder_index}");
    let fallback_title = format!("Folder {}", folder_index + 1);
    out.insert(
        "id".into(),
        Value::String(
            folder
                .get("id")
                .map(value_to_display_string)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(fallback_id),
        ),
    );
    out.insert(
        "title".into(),
        Value::String(
            folder
                .get("title")
                .map(value_to_display_string)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(fallback_title),
        ),
    );
    for field in [
        "coverImageUrl",
        "coverEmoji",
        "focusGifUrl",
        "titleLogoUrl",
        "heroBackdropUrl",
        "heroVideoUrl",
    ] {
        if !folder.get(field).map(Value::is_string).unwrap_or(false) {
            out.remove(field);
        }
    }
    if let Some(image_url) = str_field(folder, "coverImageUrl")
        .or_else(|| str_field(folder, "imageUrl"))
        .filter(|value| !value.trim().is_empty())
    {
        out.insert("imageUrl".into(), Value::String(image_url.to_string()));
    }
    out.insert(
        "focusGifEnabled".into(),
        Value::Bool(folder.get("focusGifEnabled") != Some(&Value::Bool(false))),
    );
    out.insert(
        "shape".into(),
        Value::String(normalize_tile_shape(str_field(folder, "tileShape"))),
    );
    out.insert(
        "hideTitle".into(),
        Value::Bool(
            folder
                .get("hideTitle")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        ),
    );

    let sources = folder
        .get("sources")
        .and_then(Value::as_array)
        .filter(|sources| !sources.is_empty())
        .cloned()
        .or_else(|| {
            folder
                .get("catalogSources")
                .and_then(Value::as_array)
                .cloned()
        })
        .unwrap_or_default();
    let catalog_sources: Vec<Value> = sources
        .iter()
        .filter(|source| {
            str_field(source, "provider")
                .unwrap_or("addon")
                .eq_ignore_ascii_case("addon")
        })
        .filter_map(map_catalog_source)
        .collect();
    out.insert("catalogSources".into(), Value::Array(catalog_sources));
    out.insert(
        "sources".into(),
        Value::Array(sources.iter().filter_map(map_folder_source).collect()),
    );
    Value::Object(out)
}

fn value_to_display_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn fallback_collection_id(title: &str, index: usize, profile_index: Option<i64>) -> String {
    let slug = title
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    let slug = if slug.is_empty() {
        index.to_string()
    } else {
        slug
    };
    profile_index
        .map(|profile| format!("nuvio_{profile}_{slug}"))
        .unwrap_or_else(|| format!("nuvio_collection_{index}"))
}

pub(crate) fn map_collections_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let collections = args.get("collections")?.as_array()?.clone();
    let profile_index = args.get("profileIndex").and_then(Value::as_i64);
    let mapped: Vec<Value> = collections
        .iter()
        .enumerate()
        .map(|(collection_index, c)| {
            let mut out = c.as_object().cloned().unwrap_or_default();
            let fallback_title = format!("Collection {}", collection_index + 1);
            let title = c
                .get("title")
                .map(value_to_display_string)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(fallback_title);
            let fallback_id = fallback_collection_id(&title, collection_index, profile_index);
            out.insert(
                "id".into(),
                Value::String(
                    c.get("id")
                        .map(value_to_display_string)
                        .filter(|value| !value.trim().is_empty())
                        .unwrap_or(fallback_id),
                ),
            );
            out.insert("title".into(), Value::String(title));
            match c.get("backdropImageUrl").filter(|v| v.is_string()) {
                Some(url) => {
                    out.insert("imageUrl".into(), url.clone());
                    out.insert("backdropImageUrl".into(), url.clone());
                }
                None => {
                    out.remove("imageUrl");
                    out.remove("backdropImageUrl");
                }
            }
            out.insert("showOnHome".into(), Value::Bool(true));
            out.insert(
                "focusGlowEnabled".into(),
                Value::Bool(
                    c.get("focusGlowEnabled")
                        .and_then(Value::as_bool)
                        .unwrap_or(true),
                ),
            );
            out.insert(
                "viewMode".into(),
                c.get("viewMode")
                    .filter(|v| v.is_string())
                    .cloned()
                    .unwrap_or_else(|| Value::String("FOLLOW_LAYOUT".into())),
            );
            out.insert(
                "showAllTab".into(),
                Value::Bool(c.get("showAllTab").and_then(Value::as_bool).unwrap_or(true)),
            );
            out.insert(
                "pinToTop".into(),
                Value::Bool(c.get("pinToTop").and_then(Value::as_bool).unwrap_or(false)),
            );
            let folders = c
                .get("folders")
                .and_then(Value::as_array)
                .map(|list| {
                    let collection_id = out.get("id").and_then(Value::as_str).unwrap_or("");
                    list.iter()
                        .enumerate()
                        .map(|(folder_index, folder)| {
                            map_folder(folder, collection_id, folder_index)
                        })
                        .collect()
                })
                .unwrap_or_default();
            out.insert("folders".into(), Value::Array(folders));
            Value::Object(out)
        })
        .collect();
    Some(Value::Array(mapped).to_string())
}

#[cfg(test)]
mod tests {
    use super::map_collections_json;
    use serde_json::{Value, json};

    #[test]
    fn maps_nuvio_collections_to_the_canonical_profile_shape() {
        let result: Value = serde_json::from_str(
            &map_collections_json(
                &json!({
                    "collections": [{
                        "id": "c1",
                        "title": "Action",
                        "folders": [{
                            "id": "f1",
                            "title": "Movies",
                            "coverImageUrl": "https://image.example/cover.jpg",
                            "sources": [
                                { "provider": "addon", "addonId": "addon.example", "catalogId": "top", "type": "movie" },
                                { "provider": "trakt", "traktListId": 42, "mediaType": "MOVIE" }
                            ]
                        }]
                    }]
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        let collection = &result[0];
        assert_eq!(collection["showOnHome"], true);
        assert_eq!(collection["viewMode"], "FOLLOW_LAYOUT");
        assert_eq!(collection["showAllTab"], true);
        assert_eq!(collection["focusGlowEnabled"], true);
        let folder = &collection["folders"][0];
        assert_eq!(folder["imageUrl"], "https://image.example/cover.jpg");
        assert_eq!(folder["shape"], "poster");
        assert_eq!(folder["catalogSources"].as_array().unwrap().len(), 1);
        assert_eq!(folder["sources"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn generates_stable_ids_and_titles_for_incomplete_collections() {
        let result: Value = serde_json::from_str(
            &map_collections_json(
                &json!({
                    "collections": [{
                        "folders": [{}]
                    }]
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(result[0]["id"], "nuvio_collection_0");
        assert_eq!(result[0]["title"], "Collection 1");
        assert_eq!(result[0]["folders"][0]["id"], "nuvio_collection_0_folder_0");
        assert_eq!(result[0]["folders"][0]["title"], "Folder 1");
    }

    #[test]
    fn preserves_profile_scoped_fallback_collection_ids() {
        let result: Value = serde_json::from_str(
            &map_collections_json(
                &json!({
                    "profileIndex": 7,
                    "collections": [{"title": "My Action List"}]
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(result[0]["id"], "nuvio_7_my_action_list");
    }

    #[test]
    fn falls_back_to_catalog_sources_when_modern_sources_are_empty() {
        let result: Value = serde_json::from_str(
            &map_collections_json(
                &json!({
                    "collections": [{
                        "id": "c1",
                        "title": "Action",
                        "folders": [{
                            "id": "f1",
                            "title": "Movies",
                            "sources": [],
                            "catalogSources": [{"catalogId": "top", "type": "movie"}]
                        }]
                    }]
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            result[0]["folders"][0]["catalogSources"][0]["catalogId"],
            "top"
        );
    }
}
