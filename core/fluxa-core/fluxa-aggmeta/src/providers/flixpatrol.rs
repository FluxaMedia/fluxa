use serde_json::Value;

use super::nonempty;
use crate::catalog::ProviderPage;
use crate::tmdb_catalog::PAGE_SIZE;
use crate::types::{ContentType, Meta};

pub fn region_url(base: &str, region: &str) -> String {
    let file = if region == "world" { "global" } else { region };
    format!("{}/{file}.json", base.trim_end_matches('/'))
}

pub fn parse(kind: ContentType, chart: &str, body: &Value, page: u32) -> ProviderPage<Meta> {
    let category = if kind == ContentType::Movie {
        "movies"
    } else {
        "series"
    };
    let charts = body
        .get("charts")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let find = |id: String| {
        charts
            .iter()
            .find(|c| c.get("catalog_id").and_then(Value::as_str) == Some(id.as_str()))
    };
    let selected = find(format!("{chart}.{category}")).or_else(|| find(format!("{chart}.overall")));
    let media = if kind == ContentType::Movie {
        "movie"
    } else {
        "tv"
    };
    let entries: Vec<Meta> = selected
        .and_then(|c| c.get("entries")?.as_array())
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let tmdb = entry.get("tmdb").filter(|t| t.is_object())?;
            if tmdb.get("media_type").and_then(Value::as_str) != Some(media) {
                return None;
            }
            let id = tmdb.get("id").and_then(Value::as_u64)?;
            let year = tmdb
                .get("release_date")
                .and_then(Value::as_str)
                .and_then(|date| date.get(..4))
                .unwrap_or_default()
                .to_owned();
            Some(Meta {
                id: format!("tmdb:{id}"),
                kind: kind.as_str().into(),
                name: nonempty(entry.get("title").and_then(Value::as_str).unwrap_or(""))?,
                release_info: year.clone(),
                year,
                p_tmdb: Some(id),
                ..Meta::default()
            })
        })
        .collect();
    let start = (page.max(1) as usize - 1) * PAGE_SIZE;
    ProviderPage {
        has_next_page: entries.len() > start + PAGE_SIZE,
        items: entries.into_iter().skip(start).take(PAGE_SIZE).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_entries_follow_the_media_type() {
        let body = serde_json::json!({"charts": [{
            "catalog_id": "netflix.movies",
            "entries": [
                {"rank": 1, "title": "Film", "tmdb": {"id": 5, "media_type": "movie", "release_date": "2024-01-02"}},
                {"rank": 2, "title": "Show", "tmdb": {"id": 6, "media_type": "tv", "release_date": "2023-01-02"}},
                {"rank": 3, "title": "Unmatched", "tmdb": null}
            ]
        }]});
        let page = parse(ContentType::Movie, "netflix", &body, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].id, "tmdb:5");
        assert_eq!(page.items[0].year, "2024");
        assert_eq!(
            region_url("https://example.com/c/", "world"),
            "https://example.com/c/global.json"
        );
    }
}
