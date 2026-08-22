use serde_json::{Value, json};

pub(crate) fn trakt_related_lookup_slug(lookup_json: &str, want_type: &str) -> Option<String> {
    let lookup: Vec<Value> = serde_json::from_str(lookup_json).ok()?;
    lookup
        .first()?
        .get(want_type)?
        .get("ids")?
        .get("slug")?
        .as_str()
        .map(|s| s.to_string())
}

pub(crate) fn trakt_related_items_to_metas_json(
    related_json: &str,
    content_type: &str,
) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(related_json).ok()?;
    let metas: Vec<Value> = items
        .iter()
        .filter_map(|item| {
            let ids = item.get("ids")?;
            let id = ids
                .get("imdb")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .or_else(|| {
                    ids.get("tmdb")
                        .and_then(Value::as_i64)
                        .map(|t| format!("tmdb:{t}"))
                })?;
            let name = item.get("title").and_then(Value::as_str)?;
            let mut meta = json!({ "id": id, "type": content_type, "name": name });
            if let Some(year) = item.get("year").and_then(Value::as_i64) {
                meta.as_object_mut()?
                    .insert("releaseInfo".to_string(), json!(year.to_string()));
            }
            Some(meta)
        })
        .collect();
    if metas.is_empty() {
        return None;
    }
    serde_json::to_string(&metas).ok()
}

pub(crate) fn simkl_lookup_id_for_type(lookup_json: &str, want_type: &str) -> Option<i64> {
    let lookup: Vec<Value> = serde_json::from_str(lookup_json).ok()?;
    lookup
        .iter()
        .find(|item| item.get("type").and_then(Value::as_str) == Some(want_type))
        .and_then(|item| item.get("ids")?.get("simkl")?.as_i64())
}

pub(crate) fn simkl_recommendation_candidates_json(detail_json: &str) -> Option<String> {
    let detail: Value = serde_json::from_str(detail_json).ok()?;
    let recs = detail.get("users_recommendations")?.as_array()?;
    let candidates: Vec<Value> = recs.iter().take(15).cloned().collect();
    serde_json::to_string(&candidates).ok()
}

pub(crate) fn simkl_poster_url(path: &str) -> String {
    format!("https://wsrv.nl/?url=https://simkl.in/posters/{path}_c.webp&q=90")
}

pub(crate) fn simkl_recommendation_to_meta_json(
    rec_json: &str,
    resolved_imdb: &str,
) -> Option<String> {
    let rec: Value = serde_json::from_str(rec_json).ok()?;
    let title = rec.get("title").and_then(Value::as_str)?;
    let type_str = rec.get("type").and_then(Value::as_str).unwrap_or("movie");
    let meta_type = if type_str == "tv" { "series" } else { "movie" };
    let mut meta = json!({ "id": resolved_imdb, "type": meta_type, "name": title });
    if let Some(poster) = rec.get("poster").and_then(Value::as_str) {
        meta.as_object_mut()?
            .insert("poster".to_string(), json!(simkl_poster_url(poster)));
    }
    if let Some(year) = rec.get("year").and_then(Value::as_i64) {
        meta.as_object_mut()?
            .insert("releaseInfo".to_string(), json!(year.to_string()));
    }
    serde_json::to_string(&meta).ok()
}
