use super::helpers::{parse, str_field};
use serde_json::{Value, json};

fn map_catalog_source(source: &Value) -> Option<Value> {
    let addon_id = str_field(source, "addonId").unwrap_or("");
    Some(json!({
        "addonId": addon_id,
        "catalogId": str_field(source, "catalogId").unwrap_or(""),
        "type": str_field(source, "type").unwrap_or("movie"),
        "genre": str_field(source, "genre"),
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

fn map_folder(folder: &Value) -> Value {
    let mut out = folder.as_object().cloned().unwrap_or_default();
    out.insert(
        "id".into(),
        Value::String(
            folder
                .get("id")
                .map(value_to_display_string)
                .unwrap_or_default(),
        ),
    );
    out.insert(
        "title".into(),
        Value::String(
            folder
                .get("title")
                .map(value_to_display_string)
                .unwrap_or_default(),
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
        .cloned()
        .unwrap_or_default();
    let catalog_sources: Vec<Value> = if !sources.is_empty() {
        sources
            .iter()
            .filter(|s| str_field(s, "provider").unwrap_or("addon").to_lowercase() == "addon")
            .filter_map(map_catalog_source)
            .collect()
    } else {
        folder
            .get("catalogSources")
            .and_then(Value::as_array)
            .map(|list| list.iter().filter_map(map_catalog_source).collect())
            .unwrap_or_default()
    };
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

pub(crate) fn map_collections_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let collections = args.get("collections")?.as_array()?.clone();
    let mapped: Vec<Value> = collections
        .iter()
        .map(|c| {
            let mut out = c.as_object().cloned().unwrap_or_default();
            out.insert(
                "id".into(),
                Value::String(c.get("id").map(value_to_display_string).unwrap_or_default()),
            );
            out.insert(
                "title".into(),
                Value::String(
                    c.get("title")
                        .map(value_to_display_string)
                        .unwrap_or_default(),
                ),
            );
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
                "viewMode".into(),
                c.get("viewMode")
                    .filter(|v| v.is_string())
                    .cloned()
                    .unwrap_or_else(|| Value::String("ROWS".into())),
            );
            for field in ["showAllTab", "pinToTop"] {
                out.insert(
                    field.into(),
                    Value::Bool(c.get(field).and_then(Value::as_bool).unwrap_or(false)),
                );
            }
            let folders = c
                .get("folders")
                .and_then(Value::as_array)
                .map(|list| list.iter().map(map_folder).collect())
                .unwrap_or_default();
            out.insert("folders".into(), Value::Array(folders));
            Value::Object(out)
        })
        .collect();
    Some(Value::Array(mapped).to_string())
}
