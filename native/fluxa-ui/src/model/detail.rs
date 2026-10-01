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
    pub crew: Vec<DetailCastMember>,
    pub season_posters: Vec<(i64, String)>,
    pub season_selector: SeasonSelector,
    pub episode_layout: Option<EpisodeLayout>,
    pub show_episode_descriptions: bool,
    pub blur_unwatched: bool,
    pub hide_spoilers: bool,
    pub info_lines: Vec<String>,
    pub resume: Option<HomeCard>,
    pub row_scroll_offsets: [f32; 4],
    pub selected_season: Option<i64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeasonSelector {
    #[default]
    Posters,
    Chips,
    Dropdown,
}

impl SeasonSelector {
    pub fn from_setting(value: Option<&str>) -> Self {
        match value {
            Some("chips") => Self::Chips,
            Some("dropdown" | "compact") => Self::Dropdown,
            _ => Self::Posters,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpisodeLayout {
    Cards,
    List,
    Grid,
    Numbers,
}

impl EpisodeLayout {
    pub fn from_setting(value: Option<&str>) -> Option<Self> {
        match value {
            Some("cards") => Some(Self::Cards),
            Some("list") => Some(Self::List),
            Some("grid") => Some(Self::Grid),
            Some("numbers") => Some(Self::Numbers),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DetailEpisode {
    pub id: String,
    pub season: i64,
    pub number: i64,
    pub title: String,
    pub overview: String,
    pub thumbnail: Option<String>,
    pub watched: bool,
    pub progress: f32,
}

pub struct EpisodeView<'a> {
    pub title: &'a str,
    pub overview: Option<&'a str>,
    pub hide_still: bool,
}

#[derive(Clone, Debug, Default)]
pub struct DetailCastMember {
    pub name: String,
    pub role: Option<String>,
    pub photo: Option<String>,
}

impl DetailModel {
    pub fn season_selector_mode(&self) -> SeasonSelector {
        match self.season_selector {
            SeasonSelector::Posters if self.season_posters.is_empty() => SeasonSelector::Chips,
            mode => mode,
        }
    }

    pub fn is_series(&self) -> bool {
        matches!(self.content_type.as_str(), "series" | "tv" | "show") || !self.episodes.is_empty()
    }

    pub fn current_season(&self) -> i64 {
        let seasons = self.seasons();
        self.selected_season
            .filter(|season| seasons.contains(season))
            .or_else(|| seasons.iter().copied().find(|season| *season > 0))
            .or_else(|| seasons.first().copied())
            .unwrap_or(1)
    }

    pub fn episode_view<'a>(&self, episode: &'a DetailEpisode) -> EpisodeView<'a> {
        let unwatched = !episode.watched;
        let hide_info = self.hide_spoilers && unwatched;
        EpisodeView {
            title: if hide_info { "" } else { &episode.title },
            overview: (self.show_episode_descriptions && !hide_info)
                .then_some(episode.overview.as_str()),
            hide_still: (self.blur_unwatched || self.hide_spoilers) && unwatched,
        }
    }

    pub fn episode_layout_for(&self, compact: bool) -> EpisodeLayout {
        self.episode_layout.unwrap_or(if compact {
            EpisodeLayout::List
        } else {
            EpisodeLayout::Cards
        })
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

pub(crate) fn detail_episodes(
    meta: &serde_json::Value,
    library: &serde_json::Value,
) -> Vec<DetailEpisode> {
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
                        id: id.clone(),
                        season,
                        number: episode,
                        title: first_value_string(video, &["title", "name"]).unwrap_or_default(),
                        overview: first_value_string(video, &["overview", "description"])
                            .unwrap_or_default(),
                        thumbnail: first_value_string(video, &["thumbnail", "still", "image"]),
                        watched: library
                            .pointer(&format!(
                                "/watched/{}",
                                id.replace('~', "~0").replace('/', "~1")
                            ))
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false),
                        progress: episode_progress(library, &id),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    episodes.sort_by_key(|episode| (episode.season, episode.number));
    episodes
}

fn episode_progress(library: &serde_json::Value, id: &str) -> f32 {
    let entry = library
        .get("progress")
        .and_then(|progress| progress.get(id));
    let number = |key: &str| entry?.get(key)?.as_f64();
    match (number("timeOffset"), number("duration")) {
        (Some(offset), Some(duration)) if duration > 0.0 => {
            (offset / duration).clamp(0.0, 1.0) as f32
        }
        _ => 0.0,
    }
}

fn cast_member(item: &serde_json::Value) -> DetailCastMember {
    match item {
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
    }
}

fn app_extras(meta: &serde_json::Value) -> Option<&serde_json::Value> {
    meta.get("app_extras").or_else(|| meta.get("appExtras"))
}

pub(crate) fn detail_crew(meta: &serde_json::Value, language: &str) -> Vec<DetailCastMember> {
    let Some(extras) = app_extras(meta) else {
        return Vec::new();
    };
    let groups = [
        (["directors", "director"], "detail.director"),
        (["writers", "writer"], "detail.writer"),
        (["producers", "producer"], "detail.producer"),
    ];
    let mut crew = Vec::<DetailCastMember>::new();
    for (keys, label) in groups {
        let people = keys
            .iter()
            .filter_map(|key| extras.get(*key))
            .filter_map(serde_json::Value::as_array)
            .flatten();
        for item in people {
            let mut member = cast_member(item);
            member.role = Some(localized(label, language));
            if !member.name.is_empty()
                && !crew.iter().any(|known| {
                    known.name.eq_ignore_ascii_case(&member.name) && known.role == member.role
                })
            {
                crew.push(member);
            }
        }
    }
    crew.truncate(12);
    crew
}

pub(crate) fn detail_season_posters(meta: &serde_json::Value) -> Vec<(i64, String)> {
    let mut posters = app_extras(meta)
        .and_then(|extras| extras.get("seasonPosters"))
        .and_then(serde_json::Value::as_object)
        .map(|posters| {
            posters
                .iter()
                .filter_map(|(season, url)| {
                    let url = url.as_str().filter(|url| !url.is_empty())?;
                    Some((season.parse::<i64>().ok()?, url.to_owned()))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    posters.sort_by_key(|(season, _)| if *season == 0 { i64::MAX } else { *season });
    posters
}

pub(crate) fn detail_certification(meta: &serde_json::Value) -> Option<String> {
    let extras = app_extras(meta)?;
    first_value_string(extras, &["certificationLocal", "certification"])
        .filter(|value| !value.trim().is_empty())
}

fn release_date_lines(meta: &serde_json::Value, language: &str) -> Vec<String> {
    let Some(raw) = app_extras(meta).and_then(|extras| extras.get("releaseDates")) else {
        return Vec::new();
    };
    let results = raw
        .get("results")
        .unwrap_or(raw)
        .as_array()
        .cloned()
        .unwrap_or_default();
    let region = language
        .split(['-', '_'])
        .nth(1)
        .map(str::to_uppercase)
        .unwrap_or_else(|| "US".to_owned());
    let in_region = |code: &str| {
        results.iter().find(|entry| {
            value_string(entry, "iso_3166_1").is_some_and(|value| value.eq_ignore_ascii_case(code))
        })
    };
    let Some(entry) = in_region(&region).or_else(|| in_region("US")) else {
        return Vec::new();
    };
    let dates = entry
        .get("release_dates")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    [
        (3, "detail.release_theatrical"),
        (4, "detail.release_digital"),
        (5, "detail.release_physical"),
    ]
    .into_iter()
    .filter_map(|(kind, label)| {
        let date = dates
            .iter()
            .filter(|item| item.get("type").and_then(serde_json::Value::as_i64) == Some(kind))
            .filter_map(|item| value_string(item, "release_date"))
            .filter(|date| date.len() >= 10)
            .map(|date| date[..10].to_owned())
            .min()?;
        Some(format!("{} {date}", localized(label, language)))
    })
    .collect()
}

fn streaming_line(meta: &serde_json::Value, language: &str) -> Option<String> {
    let providers = app_extras(meta)?
        .get("watchProviders")?
        .as_array()?
        .iter()
        .filter_map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .or_else(|| first_value_string(item, &["name", "title"]))
        })
        .fold(Vec::<String>::new(), |mut names, name| {
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
            names
        });
    (!providers.is_empty()).then(|| {
        format!(
            "{}: {}",
            localized("detail.watch_on", language),
            providers.join(", ")
        )
    })
}

pub(crate) fn detail_info_lines(meta: &serde_json::Value, language: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let releases = release_date_lines(meta, language);
    if !releases.is_empty() {
        lines.push(releases.join("  ·  "));
    }
    lines.extend(streaming_line(meta, language));
    lines
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
        let member = cast_member(item);
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
    let episodes = detail_episodes(meta, library);
    let language = snapshot_language(snapshot);
    let mut facts = detail_facts(meta, &episodes, &language);
    if let Some(certification) = detail_certification(meta) {
        facts.insert(facts.len().min(1), certification);
    }
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
        facts,
        cast: detail_cast(meta),
        crew: detail_crew(meta, &language),
        season_posters: detail_season_posters(meta),
        season_selector: SeasonSelector::from_setting(
            snapshot
                .pointer("/settings/values/detailSeasonSelectorMode")
                .and_then(serde_json::Value::as_str),
        ),
        episode_layout: EpisodeLayout::from_setting(
            snapshot
                .pointer("/settings/values/episodeCardsLayout")
                .and_then(serde_json::Value::as_str),
        ),
        show_episode_descriptions: snapshot
            .pointer("/settings/values/detailShowEpisodeDescriptions")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        blur_unwatched: snapshot
            .pointer("/settings/values/blurUnwatchedEpisodes")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        hide_spoilers: snapshot
            .pointer("/settings/values/spoilerHideEpisodeInfo")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        info_lines: detail_info_lines(meta, &language),
        resume: None,
        episodes,
        row_scroll_offsets: [0.0; 4],
        selected_season: None,
    }
}
