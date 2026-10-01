use super::*;

pub(crate) fn core_home_card(item: &serde_json::Value) -> HomeCard {
    core_home_card_for_kind(item, HomeRowKind::Poster)
}

pub(crate) fn core_home_card_for_kind(item: &serde_json::Value, kind: HomeRowKind) -> HomeCard {
    let progress = item
        .get("resumeProgressPercent")
        .and_then(serde_json::Value::as_f64)
        .map(|value| value as f32 / 100.0)
        .or_else(|| {
            let offset = item.get("timeOffset").and_then(serde_json::Value::as_f64)?;
            let duration = item.get("duration").and_then(serde_json::Value::as_f64)?;
            (duration > 0.0).then_some((offset / duration) as f32)
        })
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    HomeCard {
        id: value_string(item, "id"),
        item_type: value_string(item, "type"),
        title: first_value_string(item, &["name", "title"])
            .unwrap_or_else(|| "Untitled".to_owned()),
        subtitle: first_value_string(item, &["episodeLabel", "lastEpisodeName", "releaseLabel"])
            .or_else(|| value_display(item, "year"))
            .unwrap_or_default(),
        progress,
        artwork_url: if matches!(kind, HomeRowKind::Continue) {
            first_value_string(
                item,
                &[
                    "continueWatchingBackground",
                    "continueWatchingPoster",
                    "lastEpisodeThumbnail",
                    "background",
                    "backgroundUrl",
                    "poster",
                    "posterUrl",
                    "backdrop",
                    "backdropUrl",
                    "artworkUrl",
                ],
            )
        } else {
            first_value_string(
                item,
                &[
                    "poster",
                    "posterUrl",
                    "resolvedPosterUrl",
                    "artworkUrl",
                    "background",
                    "backgroundUrl",
                    "backdrop",
                    "backdropUrl",
                ],
            )
        },
        collection_shape: if matches!(kind, HomeRowKind::Collection) {
            first_value_string(item, &["reason", "shape", "tileShape"])
        } else {
            None
        },
        hide_title: item
            .get("hideTitle")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        motion_url: if matches!(kind, HomeRowKind::Collection) {
            value_string(item, "focusGifUrl")
        } else {
            None
        },
        motion_enabled: item
            .get("focusGifEnabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        overlay: if matches!(kind, HomeRowKind::Collection) {
            Default::default()
        } else {
            poster_overlay::poster_facts(item)
        },
        logo_url: first_value_string(item, &["logo", "logoUrl", "clearLogo"]),
        backdrop_url: first_value_string(
            item,
            &["background", "backgroundUrl", "backdrop", "backdropUrl"],
        ),
        poster_shape: value_string(item, "posterShape"),
        raw: item.clone(),
        row_kind: kind,
        detail: String::new(),
        episodes: String::new(),
        upcoming: matches!(kind, HomeRowKind::Continue)
            && item
                .get("upcoming")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
    }
}

fn compact_duration(seconds: i64) -> String {
    let minutes = (seconds.max(0) + 59) / 60;
    match minutes {
        0..60 => format!("{minutes}m"),
        60..2880 if minutes % 60 == 0 => format!("{}h", minutes / 60),
        60..2880 => format!("{}h {}m", minutes / 60, minutes % 60),
        _ => format!("{}d", minutes / 1440),
    }
}

pub(crate) fn continue_detail(
    item: &serde_json::Value,
    language: &str,
    show_percent: bool,
) -> (String, String) {
    if item
        .get("upcoming")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        let airs_at = item.get("airsAt").and_then(serde_json::Value::as_i64);
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_millis() as i64)
            .unwrap_or(0);
        let airs = airs_at
            .map(|at| {
                localized("home.cw.airs_in", language)
                    .replace("%s", &compact_duration((at - now) / 1000))
            })
            .unwrap_or_default();
        return (airs, String::new());
    }
    let mut detail = String::new();
    let offset = item.get("timeOffset").and_then(serde_json::Value::as_f64);
    let duration = item.get("duration").and_then(serde_json::Value::as_f64);
    if let (Some(offset), Some(duration)) = (offset, duration)
        && offset > 0.0
        && duration > offset
    {
        let percent = (offset / duration * 100.0).round() as i64;
        detail = if show_percent {
            localized("home.cw.percent_watched", language)
                .replace("%s", &format!("{}%", percent.max(1)))
        } else {
            localized("home.cw.time_left", language)
                .replace("%s", &compact_duration((duration - offset) as i64))
        };
    }
    let episodes = match item.get("episodesLeft").and_then(serde_json::Value::as_i64) {
        Some(1) => localized("home.cw.episode_left", language),
        Some(count) if count > 1 => {
            localized("home.cw.episodes_left", language).replace("%s", &count.to_string())
        }
        _ => String::new(),
    };
    (detail, episodes)
}

pub(crate) fn value_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
}

pub(crate) fn value_display(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key).and_then(|raw| match raw {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    })
}

pub(crate) fn first_value_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| value_string(value, key))
}

pub(crate) fn hero_meta_line(value: &serde_json::Value, language: &str) -> String {
    let series = matches!(
        value_string(value, "type").as_deref(),
        Some("series" | "tv" | "show")
    )
    .then(|| localized("auto.series", language));
    let year = value
        .get("year")
        .and_then(serde_json::Value::as_i64)
        .map(|year| year.to_string())
        .or_else(|| value_string(value, "year"));
    let runtime = value_string(value, "runtime").or_else(|| {
        value
            .get("duration")
            .and_then(serde_json::Value::as_i64)
            .filter(|duration| *duration > 0)
            .map(|duration| format!("{}min", duration / 60))
    });
    let genres = value
        .get("genres")
        .and_then(serde_json::Value::as_array)
        .map(|genres| {
            genres
                .iter()
                .filter_map(serde_json::Value::as_str)
                .filter(|genre| !genre.eq_ignore_ascii_case("tv movie"))
                .take(2)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|genres| !genres.is_empty());
    [series, year, runtime, genres]
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}
