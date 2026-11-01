use serde_json::Value;

use super::{nonempty, text};
use crate::catalog::ProviderPage;
use crate::search::encode_component;
use crate::tmdb_catalog::PAGE_SIZE;
use crate::types::{ContentType, Meta};

const API_BASE: &str = "https://api.mdblist.com";

pub fn items_url(list: &str, key: &str, page: u32) -> String {
    let offset = (page.max(1) as usize - 1) * PAGE_SIZE;
    format!(
        "{API_BASE}/lists/{}/items?apikey={}&limit={PAGE_SIZE}&offset={offset}",
        encode_component(list),
        encode_component(key)
    )
}

fn meta(kind: ContentType, item: &Value) -> Option<Meta> {
    let imdb = item
        .pointer("/ids/imdb")
        .and_then(Value::as_str)
        .or_else(|| item.get("imdb_id").and_then(Value::as_str))
        .filter(|id| id.starts_with("tt"));
    let tmdb = item.pointer("/ids/tmdb").and_then(Value::as_u64);
    let id = match (imdb, tmdb) {
        (Some(imdb), _) => imdb.to_string(),
        (None, Some(tmdb)) => format!("tmdb:{tmdb}"),
        _ => return None,
    };
    let name = nonempty(text(item, "title"))?;
    let year = item
        .get("release_year")
        .and_then(Value::as_u64)
        .map(|y| y.to_string())
        .unwrap_or_default();
    Some(Meta {
        id,
        kind: kind.as_str().into(),
        name,
        release_info: year.clone(),
        year,
        poster: nonempty(text(item, "poster")),
        imdb_id: imdb.map(String::from),
        p_tmdb: tmdb,
        ..Meta::default()
    })
}

pub fn parse_items(kind: ContentType, body: &Value) -> ProviderPage<Meta> {
    let count = |key: &str| body.get(key).and_then(Value::as_array).map_or(0, Vec::len);
    let key = if kind == ContentType::Movie {
        "movies"
    } else {
        "shows"
    };
    let items = body
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| meta(kind, item))
        .collect();
    ProviderPage {
        items,
        has_next_page: count("movies") + count("shows") >= PAGE_SIZE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn url_pages_by_offset_and_encodes_the_key() {
        let url = items_url("123", "a b", 3);
        assert!(url.contains("/lists/123/items"), "{url}");
        assert!(url.contains("apikey=a%20b"), "{url}");
        assert!(url.contains("offset=40"), "{url}");
    }

    #[test]
    fn each_kind_reads_its_own_array_and_prefers_imdb_ids() {
        let body = json!({
            "movies": [
                {"title": "Movie One", "release_year": 2020, "ids": {"imdb": "tt1", "tmdb": 11}},
                {"title": "No ids"}
            ],
            "shows": [{"title": "Show One", "ids": {"tmdb": 22}}]
        });
        let movies = parse_items(ContentType::Movie, &body);
        assert_eq!(movies.items.len(), 1);
        assert_eq!(movies.items[0].id, "tt1");
        let shows = parse_items(ContentType::Series, &body);
        assert_eq!(shows.items[0].id, "tmdb:22");
        assert!(!shows.has_next_page);
    }
}
