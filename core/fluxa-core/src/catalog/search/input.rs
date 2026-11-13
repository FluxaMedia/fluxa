use serde_json::{Value, json};

pub(crate) fn search_input_action_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<Value>(request_json).ok()?;
    let query = request.get("query")?.as_str()?;
    search_input_action(query).map(|action| action.to_string())
}

pub(crate) fn search_input_action(query: &str) -> Option<Value> {
    let query = query.trim();
    if query.is_empty() || query.contains(char::is_whitespace) {
        return None;
    }
    if is_imdb_id(query) {
        return Some(detail(&query.to_lowercase(), "movie"));
    }
    let url = url::Url::parse(query).ok()?;
    match url.scheme() {
        "stremio" | "fluxa" | "fluxaapple" | "app" => {
            Some(json!({"type": "openLink", "url": query}))
        }
        "http" | "https" => web_link(query, &url),
        _ => None,
    }
}

fn web_link(link: &str, url: &url::Url) -> Option<Value> {
    let host = url.host_str()?.trim_start_matches("www.");
    let segments: Vec<&str> = url
        .path_segments()?
        .filter(|segment| !segment.is_empty())
        .collect();
    if host == "imdb.com" || host.ends_with(".imdb.com") {
        let id = match segments.as_slice() {
            ["title", id, ..] => id,
            _ => return None,
        };
        return is_imdb_id(id).then(|| detail(&id.to_lowercase(), "movie"));
    }
    if host == "themoviedb.org" {
        let [kind, slug, ..] = segments.as_slice() else {
            return None;
        };
        let item_type = match *kind {
            "movie" => "movie",
            "tv" => "series",
            _ => return None,
        };
        let digits: String = slug.chars().take_while(char::is_ascii_digit).collect();
        return (!digits.is_empty()).then(|| detail(&format!("tmdb:{digits}"), item_type));
    }
    if segments.last() == Some(&"manifest.json") {
        return Some(json!({
            "type": "coreCommand",
            "command": {"type": "addonInstallRequested", "transportUrl": link, "forceRefresh": true},
        }));
    }
    let title = segments
        .last()
        .map(|name| crate::player::stream_policy::percent_decode_component(name))
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| host.to_owned());
    Some(json!({"type": "openFile", "uri": link, "title": title}))
}

fn is_imdb_id(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.strip_prefix("tt").is_some_and(|digits| {
        (7..=9).contains(&digits.len()) && digits.bytes().all(|b| b.is_ascii_digit())
    })
}

fn detail(id: &str, item_type: &str) -> Value {
    json!({"type": "detail", "id": id, "itemType": item_type})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_titles_are_left_to_search() {
        assert!(search_input_action("the matrix").is_none());
        assert!(search_input_action("tt12").is_none());
        assert!(search_input_action("matrix").is_none());
    }

    #[test]
    fn imdb_ids_and_links_open_detail() {
        let id = search_input_action("TT0133093").unwrap();
        assert_eq!(id["id"], "tt0133093");
        let link = search_input_action("https://www.imdb.com/title/tt0133093/?ref_=x").unwrap();
        assert_eq!(link["id"], "tt0133093");
    }

    #[test]
    fn tmdb_links_carry_their_type() {
        let show =
            search_input_action("https://www.themoviedb.org/tv/1399-game-of-thrones").unwrap();
        assert_eq!(show["id"], "tmdb:1399");
        assert_eq!(show["itemType"], "series");
    }

    #[test]
    fn other_web_links_play_and_manifests_install() {
        let play = search_input_action("https://example.com/video/clip%20one.mp4").unwrap();
        assert_eq!(play["type"], "openFile");
        assert_eq!(play["title"], "clip one.mp4");
        let addon = search_input_action("https://example.com/manifest.json").unwrap();
        assert_eq!(addon["type"], "coreCommand");
    }
}
