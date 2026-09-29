pub(crate) fn plugin_content_id(
    video_id: &str,
    season: Option<i64>,
    episode: Option<i64>,
) -> String {
    let trimmed = video_id.trim();
    if trimmed.is_empty() {
        return video_id.to_string();
    }

    let without_prefix = trimmed
        .strip_prefix("tmdb:")
        .or_else(|| trimmed.strip_prefix("tmdb/"))
        .unwrap_or(trimmed);
    let without_episode_suffix = match (season, episode) {
        (Some(season), Some(episode)) => without_prefix
            .strip_suffix(&format!(":{season}:{episode}"))
            .unwrap_or(without_prefix),
        _ => without_prefix,
    };

    without_episode_suffix
        .split('/')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(trimmed)
        .to_string()
}

pub(crate) fn plugin_content_type(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "tv" | "series" | "show" | "tvshow" => "tv".to_string(),
        "movie" | "film" => "movie".to_string(),
        value => value.to_string(),
    }
}

pub(crate) fn candidate_content_types(value: &str) -> Vec<String> {
    let aliases = match value.trim().to_ascii_lowercase().as_str() {
        "series" | "show" | "tv" | "anime" => ["series", "show", "tv", "anime"].as_slice(),
        "movie" | "film" => ["movie"].as_slice(),
        _ => [].as_slice(),
    };
    std::iter::once(value)
        .chain(aliases.iter().copied())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .fold(Vec::new(), |mut values, value| {
            if !values.iter().any(|existing| existing == &value) {
                values.push(value);
            }
            values
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_content_id_removes_provider_and_episode_parts() {
        assert_eq!(
            plugin_content_id("tmdb:60625:3:7", Some(3), Some(7)),
            "60625"
        );
        assert_eq!(
            plugin_content_id("tmdb/60625/anything", None, None),
            "60625"
        );
        assert_eq!(
            plugin_content_id("tt2861424:1:3", Some(1), Some(3)),
            "tt2861424"
        );
    }

    #[test]
    fn plugin_content_type_maps_supported_aliases() {
        assert_eq!(plugin_content_type("series"), "tv");
        assert_eq!(plugin_content_type("show"), "tv");
        assert_eq!(plugin_content_type("film"), "movie");
        assert_eq!(plugin_content_type("anime"), "anime");
    }

    #[test]
    fn candidate_content_types_preserves_the_original_then_adds_aliases() {
        assert_eq!(
            candidate_content_types(" show "),
            vec!["show", "series", "tv", "anime"]
        );
        assert_eq!(candidate_content_types("film"), vec!["film", "movie"]);
        assert_eq!(candidate_content_types(""), Vec::<String>::new());
    }
}
