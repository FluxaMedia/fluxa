use serde_json::Value;

use super::{nonempty, text};
use crate::catalog::ProviderPage;
use crate::service::HttpRequest;
use crate::tmdb_catalog::PAGE_SIZE;
use crate::types::Meta;

pub fn request(user: &str, status: u8) -> HttpRequest {
    HttpRequest {
        url: format!(
            "https://myanimelist.net/animelist/{}/load.json?status={status}&offset=0",
            crate::search::encode_component(user)
        ),
        post_body: None,
        headers: vec![("Accept".into(), "application/json".into())],
    }
}

fn full_size(path: &str) -> String {
    let mut out = path.split('?').next().unwrap_or(path).to_owned();
    if let Some(start) = out.find("/r/") {
        if let Some(end) = out[start + 3..].find('/') {
            out.replace_range(start..start + 3 + end, "");
        }
    }
    out
}

pub fn parse(body: &Value, page: u32) -> ProviderPage<Meta> {
    let all = body.as_array().cloned().unwrap_or_default();
    let start = (page.max(1) as usize - 1) * PAGE_SIZE;
    let items = all
        .iter()
        .skip(start)
        .take(PAGE_SIZE)
        .filter_map(|item| {
            let id = item.get("anime_id").and_then(Value::as_u64)?;
            let name = nonempty(text(item, "anime_title_eng"))
                .or_else(|| nonempty(text(item, "anime_title")))?;
            Some(Meta {
                id: format!("mal:{id}"),
                kind: "series".into(),
                name,
                poster: nonempty(text(item, "anime_image_path")).map(|p| full_size(&p)),
                p_mal: Some(id),
                ..Meta::default()
            })
        })
        .collect();
    ProviderPage {
        items,
        has_next_page: all.len() > start + PAGE_SIZE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn entries_are_paged_and_posters_enlarged() {
        let body = json!([{
            "anime_id": 7, "anime_title": "Example Anime", "anime_title_eng": "",
            "anime_image_path": "https://cdn.example/r/96x136/images/anime/1/2.jpg?s=abc"
        }]);
        let page = parse(&body, 1);
        assert_eq!(page.items[0].id, "mal:7");
        assert_eq!(page.items[0].name, "Example Anime");
        assert_eq!(
            page.items[0].poster.as_deref(),
            Some("https://cdn.example/images/anime/1/2.jpg")
        );
        assert!(!page.has_next_page);
    }
}
