use super::*;

#[derive(Clone, Debug, Default)]
pub struct DetailModel {
    pub item: serde_json::Value,
    pub id: String,
    pub content_type: String,
    pub is_loading: bool,
    pub is_loading_streams: bool,
    pub title: String,
    pub meta_line: String,
    pub description: String,
    pub poster_url: Option<String>,
    pub background_url: Option<String>,
    pub in_watchlist: bool,
    pub completed: bool,
    pub dropped: bool,
    pub favorite: bool,
    pub language: String,
    pub ratings: Vec<(String, String)>,
    pub genres: Vec<String>,
    pub trailers: Vec<String>,
    pub trailer: Option<egui::TextureId>,
    pub error: Option<String>,
    pub streams_error: Option<String>,
    pub similar: Vec<HomeCard>,
    pub logo_url: Option<String>,
    pub facts: Vec<String>,
    pub episodes: Vec<DetailEpisode>,
    pub cast: Vec<DetailCastMember>,
    pub resume: Option<HomeCard>,
    pub row_scroll_offsets: [f32; 4],
    pub selected_season: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct DetailEpisode {
    pub id: String,
    pub season: i64,
    pub number: i64,
    pub title: String,
    pub overview: String,
    pub thumbnail: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct DetailCastMember {
    pub name: String,
    pub role: Option<String>,
    pub photo: Option<String>,
}

impl DetailModel {
    pub fn is_series(&self) -> bool {
        matches!(self.content_type.as_str(), "series" | "tv" | "show") || !self.episodes.is_empty()
    }

    pub fn seasons(&self) -> Vec<i64> {
        let mut seasons = self
            .episodes
            .iter()
            .map(|episode| episode.season)
            .collect::<Vec<_>>();
        seasons.sort_unstable_by_key(|season| if *season == 0 { i64::MAX } else { *season });
        seasons.dedup();
        seasons
    }
}

pub(crate) fn detail_episodes(meta: &serde_json::Value) -> Vec<DetailEpisode> {
    let mut episodes = meta
        .get("videos")
        .and_then(serde_json::Value::as_array)
        .map(|videos| {
            videos
                .iter()
                .filter_map(|video| {
                    let id = value_string(video, "id")?;
                    let number = |key: &str| video.get(key).and_then(serde_json::Value::as_i64);
                    let season = number("season").unwrap_or(1);
                    let episode = number("episode").or_else(|| number("number")).unwrap_or(0);
                    Some(DetailEpisode {
                        id,
                        season,
                        number: episode,
                        title: first_value_string(video, &["title", "name"]).unwrap_or_default(),
                        overview: first_value_string(video, &["overview", "description"])
                            .unwrap_or_default(),
                        thumbnail: first_value_string(video, &["thumbnail", "still", "image"]),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    episodes.sort_by_key(|episode| (episode.season, episode.number));
    episodes
}

pub(crate) fn detail_cast(meta: &serde_json::Value) -> Vec<DetailCastMember> {
    let mut cast = Vec::<DetailCastMember>::new();
    let sources = [
        meta.get("cast"),
        meta.pointer("/app_extras/cast"),
        meta.pointer("/appExtras/cast"),
    ];
    for item in sources
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_array)
        .flatten()
    {
        let member = match item {
            serde_json::Value::String(name) => DetailCastMember {
                name: name.trim().to_owned(),
                ..DetailCastMember::default()
            },
            _ => DetailCastMember {
                name: first_value_string(item, &["name", "fullName", "actor"]).unwrap_or_default(),
                role: first_value_string(item, &["character", "role", "as"]),
                photo: first_value_string(
                    item,
                    &[
                        "profilePath",
                        "profile_path",
                        "photo",
                        "profile",
                        "image",
                        "img",
                    ],
                )
                .map(|photo| {
                    if photo.starts_with('/') {
                        format!("https://image.tmdb.org/t/p/w185{photo}")
                    } else {
                        photo
                    }
                }),
            },
        };
        if !member.name.is_empty()
            && !cast
                .iter()
                .any(|known| known.name.eq_ignore_ascii_case(&member.name))
        {
            cast.push(member);
        }
    }
    cast.truncate(20);
    cast
}

pub(crate) fn detail_facts(
    meta: &serde_json::Value,
    episodes: &[DetailEpisode],
    language: &str,
) -> Vec<String> {
    let mut facts = Vec::new();
    if let Some(release) = first_value_string(meta, &["releaseInfo", "year"]).or_else(|| {
        meta.get("year")
            .and_then(serde_json::Value::as_i64)
            .map(|year| year.to_string())
    }) {
        facts.push(release);
    }
    let mut seasons = episodes
        .iter()
        .map(|episode| episode.season)
        .filter(|season| *season > 0)
        .collect::<Vec<_>>();
    seasons.dedup();
    if !seasons.is_empty() {
        facts.push(format!(
            "{} {}",
            seasons.len(),
            localized("auto.seasons", language)
        ));
    }
    if let Some(runtime) = value_string(meta, "runtime") {
        facts.push(runtime);
    }
    facts
}

pub fn detail_model_from_core_snapshot(snapshot: &serde_json::Value) -> DetailModel {
    let detail = snapshot.get("detail").unwrap_or(&serde_json::Value::Null);
    let meta = detail.get("meta").unwrap_or(&serde_json::Value::Null);
    let id = value_string(detail, "id").unwrap_or_default();
    let library = snapshot.get("library").unwrap_or(&serde_json::Value::Null);
    let library_has_item = |key: &str| {
        library
            .get(key)
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    first_value_string(item, &["id", "contentId", "metaId"]).as_deref()
                        == Some(id.as_str())
                })
            })
    };
    let mut ratings = Vec::new();
    let mdblist = detail
        .get("mdblistRatings")
        .and_then(serde_json::Value::as_object);
    let rating_sources = [
        ("imdb", "IMDb", 1),
        ("tmdb", "TMDB", 0),
        ("trakt", "Trakt", 0),
        ("tomatoes", "Rotten Tomatoes", 0),
        ("popcorn", "Audience", 0),
        ("metacritic", "Metacritic", 2),
        ("metacriticuser", "Metacritic Users", 1),
        ("letterboxd", "Letterboxd", 1),
        ("myanimelist", "MyAnimeList", 1),
    ];
    if let Some(mdblist) = mdblist {
        for (key, label, precision) in rating_sources {
            if let Some(value) = mdblist.get(key).and_then(serde_json::Value::as_f64) {
                let value = match precision {
                    0 => format!("{}%", value.round() as i64),
                    1 => format!("{value:.1}"),
                    _ => format!("{}", value.round() as i64),
                };
                ratings.push((label.to_owned(), value));
            }
        }
    }
    if ratings.is_empty() {
        if let Some(value) = meta
            .get("imdbRating")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<f64>().ok())
        {
            ratings.push(("IMDb".to_owned(), format!("{value:.1}")));
        }
        if let Some(value) = detail
            .pointer("/omdbRatings/rottenTomatoes")
            .and_then(serde_json::Value::as_str)
        {
            ratings.push(("Rotten Tomatoes".to_owned(), value.to_owned()));
        }
        if let Some(value) = detail
            .pointer("/omdbRatings/metascore")
            .and_then(serde_json::Value::as_str)
        {
            ratings.push(("Metacritic".to_owned(), value.to_owned()));
        }
    }
    let episodes = detail_episodes(meta);
    DetailModel {
        item: meta.clone(),
        id: id.clone(),
        content_type: value_string(detail, "contentType").unwrap_or_else(|| "movie".to_owned()),
        language: snapshot_language(snapshot),
        is_loading: detail
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        is_loading_streams: detail
            .get("isLoadingStreams")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        title: first_value_string(meta, &["name", "title"])
            .unwrap_or_else(|| "Title details".to_owned()),
        meta_line: hero_meta_line(meta, &snapshot_language(snapshot)),
        description: first_value_string(meta, &["description", "overview", "plot"])
            .unwrap_or_else(|| "No description available.".to_owned()),
        poster_url: first_value_string(
            meta,
            &["poster", "posterUrl", "resolvedPosterUrl", "background"],
        ),
        background_url: first_value_string(
            meta,
            &[
                "backdrop",
                "backdropUrl",
                "background",
                "backgroundUrl",
                "fanart",
            ],
        ),
        in_watchlist: detail
            .get("isInWatchlist")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_else(|| library_has_item("watchlist")),
        completed: library_has_item("completed"),
        dropped: library_has_item("dropped"),
        favorite: library_has_item("liked"),
        ratings,
        genres: meta
            .get("genres")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        item.as_str()
                            .map(ToOwned::to_owned)
                            .or_else(|| first_value_string(item, &["name", "title"]))
                    })
                    .take(6)
                    .collect()
            })
            .unwrap_or_default(),
        trailers: trailer_urls(detail),
        trailer: None,
        error: detail
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        streams_error: detail
            .get("streamsError")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        similar: detail
            .get("similarItems")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().take(16).map(core_home_card).collect())
            .unwrap_or_default(),
        logo_url: detail
            .pointer("/fanartArtwork/hdLogo")
            .and_then(serde_json::Value::as_str)
            .filter(|url| !url.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| value_string(meta, "logo")),
        facts: detail_facts(meta, &episodes, &snapshot_language(snapshot)),
        cast: detail_cast(meta),
        resume: None,
        episodes,
        row_scroll_offsets: [0.0; 4],
        selected_season: None,
    }
}
