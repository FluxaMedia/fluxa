use serde_json::Value;
use url::Url;

pub(crate) fn rating_page_url_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let text = |key: &str| {
        args.get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    };
    let url = rating_page_url(
        text("source")?,
        text("id").unwrap_or_default(),
        text("imdbId"),
        text("contentType").unwrap_or("movie"),
        text("title"),
    )?;
    serde_json::to_string(&url).ok()
}

pub(crate) fn rating_page_url(
    source: &str,
    id: &str,
    imdb_id: Option<&str>,
    content_type: &str,
    title: Option<&str>,
) -> Option<String> {
    let imdb = imdb_id
        .filter(|value| imdb_like(value))
        .or_else(|| imdb_like(id).then_some(id))
        .map(str::to_lowercase);
    let tmdb = id
        .strip_prefix("tmdb:")
        .filter(|value| !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()));
    let series = matches!(content_type, "series" | "tv" | "show" | "anime");
    let search = |base: &str, key: &str| {
        let title = title?;
        let mut url = Url::parse(base).ok()?;
        url.query_pairs_mut().append_pair(key, title);
        Some(url.to_string())
    };
    match source {
        "IMDb" => imdb.map(|imdb| format!("https://www.imdb.com/title/{imdb}/")),
        "TMDB" => tmdb.map(|tmdb| {
            let kind = if series { "tv" } else { "movie" };
            format!("https://www.themoviedb.org/{kind}/{tmdb}")
        }),
        "Trakt" => imdb.map(|imdb| format!("https://trakt.tv/search/imdb/{imdb}")),
        "Letterboxd" => imdb.map(|imdb| format!("https://letterboxd.com/imdb/{imdb}/")),
        "Rotten Tomatoes" | "Audience" => search("https://www.rottentomatoes.com/search", "search"),
        "Metacritic" | "Metacritic Users" => search("https://www.metacritic.com/search/", "query"),
        "MyAnimeList" => search("https://myanimelist.net/anime.php", "q"),
        _ => None,
    }
}

fn imdb_like(value: &str) -> bool {
    value
        .strip_prefix("tt")
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imdb_and_letterboxd_use_the_imdb_id() {
        let url = rating_page_url("IMDb", "tt0133093", None, "movie", None).unwrap();
        assert_eq!(url, "https://www.imdb.com/title/tt0133093/");
        let url = rating_page_url("Letterboxd", "tmdb:603", Some("tt0133093"), "movie", None);
        assert_eq!(url.unwrap(), "https://letterboxd.com/imdb/tt0133093/");
    }

    #[test]
    fn tmdb_page_follows_the_content_type() {
        let url = rating_page_url("TMDB", "tmdb:1399", None, "series", None).unwrap();
        assert_eq!(url, "https://www.themoviedb.org/tv/1399");
    }

    #[test]
    fn critic_sites_fall_back_to_a_title_search() {
        let url = rating_page_url("Rotten Tomatoes", "tt1", None, "movie", Some("The Matrix"));
        assert_eq!(
            url.unwrap(),
            "https://www.rottentomatoes.com/search?search=The+Matrix"
        );
        assert!(rating_page_url("Rotten Tomatoes", "tt1", None, "movie", None).is_none());
    }
}
