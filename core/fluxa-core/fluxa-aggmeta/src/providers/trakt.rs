use serde_json::Value;

use super::{nonempty, text};
use crate::catalog::ProviderPage;
use crate::service::HttpRequest;
use crate::tmdb_catalog::PAGE_SIZE;
use crate::types::{ContentType, Meta};

const API_BASE: &str = "https://api.trakt.tv";

pub fn request(kind: ContentType, list: &str, client_id: &str, page: u32) -> HttpRequest {
    let media = if kind == ContentType::Movie {
        "movies"
    } else {
        "shows"
    };
    HttpRequest {
        url: format!("{API_BASE}/{media}/{list}?page={page}&limit={PAGE_SIZE}&extended=full"),
        post_body: None,
        headers: vec![
            ("trakt-api-version".into(), "2".into()),
            ("trakt-api-key".into(), client_id.into()),
            ("Accept".into(), "application/json".into()),
        ],
    }
}

fn meta(kind: ContentType, entry: &Value) -> Option<Meta> {
    let item = entry
        .get("movie")
        .or_else(|| entry.get("show"))
        .unwrap_or(entry);
    let imdb = item
        .pointer("/ids/imdb")
        .and_then(Value::as_str)
        .filter(|id| id.starts_with("tt"));
    let tmdb = item.pointer("/ids/tmdb").and_then(Value::as_u64);
    let id = match (imdb, tmdb) {
        (Some(imdb), _) => imdb.to_string(),
        (None, Some(tmdb)) => format!("tmdb:{tmdb}"),
        _ => return None,
    };
    let year = item
        .get("year")
        .and_then(Value::as_u64)
        .map(|y| y.to_string())
        .unwrap_or_default();
    let rating = item
        .get("rating")
        .and_then(Value::as_f64)
        .filter(|r| *r > 0.0)
        .map(|r| format!("{r:.1}"))
        .unwrap_or_default();
    Some(Meta {
        id,
        kind: kind.as_str().into(),
        name: nonempty(text(item, "title"))?,
        description: text(item, "overview").to_owned(),
        release_info: year.clone(),
        year,
        imdb_rating: rating,
        imdb_id: imdb.map(String::from),
        p_tmdb: tmdb,
        ..Meta::default()
    })
}

pub fn parse(kind: ContentType, body: &Value) -> ProviderPage<Meta> {
    let rows = body.as_array().map_or(0, Vec::len);
    let items = body
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| meta(kind, entry))
        .collect();
    ProviderPage {
        items,
        has_next_page: rows >= PAGE_SIZE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn wrapped_and_plain_entries_both_parse() {
        let body = json!([
            {"watchers": 9, "movie": {"title": "One", "year": 2020, "ids": {"imdb": "tt1", "tmdb": 5}}},
            {"title": "Two", "ids": {"tmdb": 6}},
            {"title": "No ids", "ids": {}}
        ]);
        let page = parse(ContentType::Movie, &body);
        let ids: Vec<_> = page.items.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["tt1", "tmdb:6"]);
        assert!(!page.has_next_page);
    }

    #[test]
    fn request_carries_the_client_id() {
        let req = request(ContentType::Series, "trending", "abc", 2);
        assert!(req.url.contains("/shows/trending?page=2"), "{}", req.url);
        assert!(
            req.headers
                .iter()
                .any(|(k, v)| k == "trakt-api-key" && v == "abc")
        );
    }
}
