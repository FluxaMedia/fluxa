use super::helpers::{build_url, extract_query, plan};
use super::media_ratings::LIST_ITEMS_QUERY_KEYS;
use serde_json::{Value, json};

fn list_items_path(list_ref: &Value) -> Option<String> {
    if let Some(id) = list_ref.get("listId").and_then(Value::as_i64) {
        return Some(format!("/lists/{id}/items"));
    }
    if let Some(id) = list_ref.get("externalListId").and_then(Value::as_i64) {
        return Some(format!("/external/lists/{id}/items"));
    }
    if let Some(slug) = list_ref.get("officialSlug").and_then(Value::as_str) {
        return Some(format!("/lists/official/{slug}/items"));
    }
    if let Some(token) = list_ref.get("shareToken").and_then(Value::as_str) {
        return Some(format!("/lists/share/{token}/items"));
    }
    if let Some(section) = list_ref.get("recommendedSection").and_then(Value::as_str) {
        return Some(format!("/lists/recommended/{section}/items"));
    }
    let username = list_ref.get("username").and_then(Value::as_str)?;
    let list_name = list_ref.get("listName").and_then(Value::as_str)?;
    match list_ref.get("mediaType").and_then(Value::as_str) {
        Some(media_type) => Some(format!("/lists/{username}/{list_name}/items/{media_type}")),
        None => Some(format!("/lists/{username}/{list_name}/items")),
    }
}

pub(crate) fn mdblist_list_items_url(list_ref_json: &str, query_json: &str) -> Option<String> {
    let list_ref: Value = serde_json::from_str(list_ref_json).ok()?;
    let path = list_items_path(&list_ref)?;
    let query: Value = serde_json::from_str(query_json).unwrap_or(json!({}));
    Some(build_url(
        &path,
        &extract_query(&query, LIST_ITEMS_QUERY_KEYS),
    ))
}

pub(crate) fn mdblist_list_item_to_meta_json(item_json: &str, media_type: &str) -> Option<String> {
    let item: Value = serde_json::from_str(item_json).ok()?;
    let ids = item.get("ids");
    let imdb_id = ids
        .and_then(|ids| ids.get("imdb"))
        .and_then(Value::as_str)
        .or_else(|| item.get("imdb_id").and_then(Value::as_str))
        .filter(|value| !value.is_empty());
    let tmdb_id = ids.and_then(|ids| ids.get("tmdb")).and_then(Value::as_i64);
    let id = if let Some(imdb_id) = imdb_id {
        imdb_id.to_string()
    } else if let Some(tmdb_id) = tmdb_id {
        format!("tmdb:{tmdb_id}")
    } else {
        return None;
    };
    let content_type = if media_type == "show" {
        "series"
    } else {
        "movie"
    };
    let title = item.get("title").and_then(Value::as_str).unwrap_or("");
    serde_json::to_string(&json!({
        "id": id,
        "type": content_type,
        "name": title,
        "releaseInfo": item.get("release_year"),
        "mdblistId": item.get("id"),
    }))
    .ok()
}

pub(crate) fn mdblist_list_items_response_to_metas_json(response_json: &str) -> Option<String> {
    let response: Value = serde_json::from_str(response_json).ok()?;
    let mut metas = Vec::new();
    for (key, media_type) in [("movies", "movie"), ("shows", "show")] {
        let Some(items) = response.get(key).and_then(Value::as_array) else {
            continue;
        };
        for item in items {
            let item_json = serde_json::to_string(item).ok()?;
            if let Some(meta_json) = mdblist_list_item_to_meta_json(&item_json, media_type)
                && let Ok(meta) = serde_json::from_str::<Value>(&meta_json)
            {
                metas.push(meta);
            }
        }
    }
    serde_json::to_string(&metas).ok()
}

pub(crate) fn mdblist_list_info_url(listid: i64) -> String {
    build_url(&format!("/lists/{listid}"), &[])
}

pub(crate) fn mdblist_list_by_name_url(username: &str, list_name: &str) -> String {
    build_url(&format!("/lists/{username}/{list_name}"), &[])
}

pub(crate) fn mdblist_list_update_plan(
    listid: i64,
    name: Option<&str>,
    private: Option<bool>,
) -> Option<String> {
    let mut body = serde_json::Map::new();
    if let Some(name) = name {
        body.insert("name".to_string(), json!(name));
    }
    if let Some(private) = private {
        body.insert("private".to_string(), json!(private));
    }
    if body.is_empty() {
        return None;
    }
    plan(
        "PUT",
        build_url(&format!("/lists/{listid}"), &[]),
        Some(Value::Object(body)),
    )
}

pub(crate) fn mdblist_list_delete_plan(listid: i64) -> Option<String> {
    plan("DELETE", build_url(&format!("/lists/{listid}"), &[]), None)
}

pub(crate) fn mdblist_list_create_plan(name: &str, private: Option<bool>) -> Option<String> {
    if name.trim().is_empty() {
        return None;
    }
    let mut body = serde_json::Map::new();
    body.insert("name".to_string(), json!(name));
    if let Some(private) = private {
        body.insert("private".to_string(), json!(private));
    }
    plan(
        "POST",
        build_url("/lists/user/add", &[]),
        Some(Value::Object(body)),
    )
}

pub(crate) fn mdblist_list_changes_url(listid: i64) -> String {
    build_url(&format!("/lists/{listid}/changes"), &[])
}

pub(crate) fn mdblist_list_like_plan(listid: i64, liked: bool) -> Option<String> {
    let method = if liked { "PUT" } else { "DELETE" };
    plan(
        method,
        build_url(&format!("/lists/{listid}/like"), &[]),
        None,
    )
}

fn ids_from_meta(meta: &Value) -> Option<Value> {
    let imdb = meta
        .get("imdbId")
        .or_else(|| meta.get("imdb"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty());
    let tmdb = meta
        .get("tmdbId")
        .or_else(|| meta.get("tmdb"))
        .and_then(Value::as_i64);
    let mut ids = serde_json::Map::new();
    if let Some(imdb) = imdb {
        ids.insert("imdb".to_string(), json!(imdb));
    }
    if let Some(tmdb) = tmdb {
        ids.insert("tmdb".to_string(), json!(tmdb));
    }
    if ids.is_empty() {
        return None;
    }
    Some(Value::Object(ids))
}

pub(crate) fn items_by_type_body(items_json: &str) -> Option<Value> {
    let items: Value = serde_json::from_str(items_json).ok()?;
    let mut movies = Vec::new();
    let mut shows = Vec::new();
    for item in items.as_array()? {
        let ids = ids_from_meta(item)?;
        let bucket = if item.get("type").and_then(Value::as_str) == Some("series") {
            &mut shows
        } else {
            &mut movies
        };
        let mut entry = serde_json::Map::new();
        entry.insert("ids".to_string(), ids);
        if let Some(rating) = item.get("rating") {
            entry.insert("rating".to_string(), rating.clone());
        }
        bucket.push(Value::Object(entry));
    }
    let mut body = serde_json::Map::new();
    if !movies.is_empty() {
        body.insert("movies".to_string(), Value::Array(movies));
    }
    if !shows.is_empty() {
        body.insert("shows".to_string(), Value::Array(shows));
    }
    if body.is_empty() {
        return None;
    }
    Some(Value::Object(body))
}

pub(crate) fn mdblist_list_items_mutate_plan(
    listid: i64,
    action: &str,
    items_json: &str,
) -> Option<String> {
    let body = items_by_type_body(items_json)?;
    plan(
        "POST",
        build_url(&format!("/lists/{listid}/items/{action}"), &[]),
        Some(body),
    )
}

pub(crate) fn mdblist_list_membership_url(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let params = extract_query(&args, &["mediatype", "tmdb", "imdb", "trakt", "tvdb"]);
    if !params.iter().any(|(key, _)| key == "mediatype") {
        return None;
    }
    Some(build_url("/lists/user/membership", &params))
}

pub(crate) fn mdblist_lists_search_url(query: &str) -> Option<String> {
    if query.trim().is_empty() {
        return None;
    }
    Some(build_url(
        "/lists/search",
        &[("query".to_string(), query.to_string())],
    ))
}

pub(crate) fn mdblist_lists_curated_url(args_json: &str) -> String {
    let args: Value = serde_json::from_str(args_json).unwrap_or(json!({}));
    build_url(
        "/lists/curated",
        &extract_query(&args, &["limit", "offset", "append_to_response"]),
    )
}

pub(crate) fn mdblist_lists_top_url(args_json: &str) -> String {
    let args: Value = serde_json::from_str(args_json).unwrap_or(json!({}));
    build_url(
        "/lists/top",
        &extract_query(&args, &["limit", "offset", "append_to_response"]),
    )
}

pub(crate) fn mdblist_lists_liked_url(args_json: &str) -> String {
    let args: Value = serde_json::from_str(args_json).unwrap_or(json!({}));
    build_url(
        "/lists/liked",
        &extract_query(&args, &["limit", "offset", "append_to_response"]),
    )
}

pub(crate) fn mdblist_lists_official_url() -> String {
    build_url("/lists/official", &[])
}

pub(crate) fn mdblist_lists_recommended_url(section: Option<&str>) -> String {
    match section.filter(|value| !value.is_empty()) {
        Some(section) => build_url(&format!("/lists/recommended/{section}"), &[]),
        None => build_url("/lists/recommended", &[]),
    }
}

pub(crate) fn mdblist_lists_user_url(user_ref: Option<&str>, args_json: &str) -> String {
    let args: Value = serde_json::from_str(args_json).unwrap_or(json!({}));
    let params = extract_query(&args, &["sort", "unified", "append_to_response"]);
    match user_ref.filter(|value| !value.is_empty()) {
        Some(user_ref) => build_url(&format!("/lists/user/{user_ref}"), &params),
        None => build_url("/lists/user", &params),
    }
}
