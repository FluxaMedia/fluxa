use serde_json::Value;

use super::{nonempty, text};
use crate::catalog::ProviderPage;
use crate::service::HttpRequest;
use crate::tmdb_catalog::PAGE_SIZE;
use crate::types::{ContentType, Meta};

pub fn request(kind: ContentType, status: &str, token: &str, client_id: &str) -> HttpRequest {
    let segment = if kind == ContentType::Movie {
        "movies"
    } else {
        "shows"
    };
    HttpRequest {
        url: format!("https://api.simkl.com/sync/all-items/{segment}/{status}"),
        post_body: None,
        headers: vec![
            ("Authorization".into(), format!("Bearer {token}")),
            ("simkl-api-key".into(), client_id.to_owned()),
        ],
    }
}

fn meta(kind: ContentType, row: &Value) -> Option<Meta> {
    let item = row.get("movie").or_else(|| row.get("show"))?;
    let imdb = item
        .pointer("/ids/imdb")
        .and_then(Value::as_str)
        .filter(|id| id.starts_with("tt"));
    let tmdb = item
        .pointer("/ids/tmdb")
        .and_then(|id| id.as_u64().or_else(|| id.as_str()?.parse().ok()));
    let id = match (imdb, tmdb) {
        (Some(imdb), _) => imdb.to_owned(),
        (None, Some(tmdb)) => format!("tmdb:{tmdb}"),
        _ => return None,
    };
    let year = item
        .get("year")
        .and_then(Value::as_u64)
        .map(|year| year.to_string())
        .unwrap_or_default();
    Some(Meta {
        id,
        kind: kind.as_str().into(),
        name: nonempty(text(item, "title"))?,
        release_info: year.clone(),
        year,
        poster: nonempty(text(item, "poster"))
            .map(|path| format!("https://simkl.in/posters/{path}_m.webp")),
        imdb_id: imdb.map(String::from),
        p_tmdb: tmdb,
        ..Meta::default()
    })
}

pub fn parse(kind: ContentType, body: &Value, page: u32) -> ProviderPage<Meta> {
    let key = if kind == ContentType::Movie {
        "movies"
    } else {
        "shows"
    };
    let rows = body
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let start = (page.max(1) as usize - 1) * PAGE_SIZE;
    ProviderPage {
        items: rows
            .iter()
            .skip(start)
            .take(PAGE_SIZE)
            .filter_map(|row| meta(kind, row))
            .collect(),
        has_next_page: rows.len() > start + PAGE_SIZE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_without_a_usable_id_are_dropped() {
        let body = serde_json::json!({"shows": [
            {"show": {"title": "A", "year": 2020, "poster": "12/345", "ids": {"simkl": 1, "imdb": "tt1"}}},
            {"show": {"title": "B", "ids": {"simkl": 2, "tmdb": "99"}}},
            {"show": {"title": "C", "ids": {"simkl": 3}}}
        ]});
        let page = parse(ContentType::Series, &body, 1);
        let ids: Vec<_> = page.items.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["tt1", "tmdb:99"]);
        assert_eq!(
            page.items[0].poster.as_deref(),
            Some("https://simkl.in/posters/12/345_m.webp")
        );
    }
}
