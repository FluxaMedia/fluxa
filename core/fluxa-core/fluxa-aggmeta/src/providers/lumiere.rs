use serde_json::Value;

use crate::search::encode_component;
use crate::types::ContentType;

pub const LIMIT: usize = 60;

pub fn list_url(base: &str, list: &str, genre: &str, kind: ContentType) -> String {
    let media = if kind == ContentType::Movie {
        "movies"
    } else {
        "series"
    };
    let mut url = format!(
        "{}/lists/{}?type={media}&limit={LIMIT}",
        base.trim_end_matches('/'),
        encode_component(list)
    );
    if !genre.is_empty() {
        url.push_str(&format!("&genres={}", encode_component(genre)));
    }
    url
}

pub fn ids(body: &Value) -> Vec<String> {
    body.pointer("/items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("tconst")?.as_str())
        .filter(|id| id.starts_with("tt"))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_url_carries_type_and_genre() {
        let url = list_url(
            "http://host:8000/",
            "popular",
            "action",
            ContentType::Series,
        );
        assert_eq!(
            url,
            "http://host:8000/lists/popular?type=series&limit=60&genres=action"
        );
    }

    #[test]
    fn only_imdb_ids_are_kept() {
        let body = serde_json::json!({"items": [{"tconst": "tt1"}, {"tconst": "nm2"}, {}]});
        assert_eq!(ids(&body), ["tt1"]);
    }
}
