use super::*;

pub(super) const ENRICHMENT_FIELD_GROUPS: &[(&str, &[&str])] = &[
    ("artwork", &["logo", "poster", "background"]),
    ("description", &["description", "tagline"]),
    ("genresKeywords", &["genres", "keywords"]),
    ("castCrew", &["cast", "director", "createdBy"]),
    ("network", &["network"]),
    ("ratings", &["imdbRating", "certification"]),
    ("collection", &["collection"]),
    (
        "statusSchedule",
        &["status", "nextEpisodeToAir", "lastEpisodeToAir"],
    ),
    (
        "originTitles",
        &[
            "originalLanguage",
            "productionCountries",
            "alternativeTitles",
        ],
    ),
    ("watchProviders", &["watchProviders"]),
];

pub(crate) fn merge_tmdb_enrichment_json(
    base_json: &str,
    tmdb_json: &str,
    flags_json: &str,
) -> Option<String> {
    let mut base: Value = serde_json::from_str(base_json).ok()?;
    let tmdb: Value = serde_json::from_str(tmdb_json).ok()?;
    let flags: Value = serde_json::from_str(flags_json).ok()?;

    for (flag, fields) in ENRICHMENT_FIELD_GROUPS {
        if flags.get(flag).and_then(Value::as_bool) != Some(true) {
            continue;
        }
        for field in *fields {
            if let Some(value) = tmdb.get(*field).filter(|v| !v.is_null()) {
                base[*field] = value.clone();
            }
        }
    }

    if flags.get("episodeStills").and_then(Value::as_bool) == Some(true) {
        if let Some(tmdb_videos) = tmdb.get("videos").and_then(Value::as_array) {
            if let Some(base_videos) = base.get_mut("videos").and_then(Value::as_array_mut) {
                for video in base_videos.iter_mut() {
                    let season = video.get("season").and_then(Value::as_i64);
                    let episode = video.get("episode").and_then(Value::as_i64);
                    let Some(matched) = tmdb_videos.iter().find(|v| {
                        v.get("season").and_then(Value::as_i64) == season
                            && v.get("episode").and_then(Value::as_i64) == episode
                    }) else {
                        continue;
                    };
                    if let Some(thumbnail) = matched.get("thumbnail").filter(|v| !v.is_null()) {
                        video["thumbnail"] = thumbnail.clone();
                    }
                }
            }
        }
    }

    serde_json::to_string(&base).ok()
}
