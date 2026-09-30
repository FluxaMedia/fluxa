use super::*;

pub(crate) fn poster_facts(item: &serde_json::Value) -> PosterFacts {
    PosterFacts {
        rating: rating(item),
        status: status(item, today()),
        caption: caption(item),
        imdb: item
            .get("imdb_id")
            .or_else(|| item.get("id"))
            .and_then(serde_json::Value::as_str)
            .map(|id| id.split(':').next().unwrap_or(id))
            .filter(|id| id.starts_with("tt"))
            .map(str::to_owned),
        tmdb: ["tmdb_id", "tmdbId", "moviedb_id"]
            .iter()
            .find_map(|key| item.get(*key).and_then(number))
            .map(|id| id as u64)
            .or_else(|| {
                item.get("id")
                    .and_then(serde_json::Value::as_str)?
                    .strip_prefix("tmdb:")?
                    .split(':')
                    .next()?
                    .parse()
                    .ok()
            }),
        title_key: item
            .get("name")
            .and_then(serde_json::Value::as_str)
            .zip(year(item))
            .map(|(name, year)| title_key(name, &year))
            .unwrap_or_default(),
    }
}

pub(super) fn number(value: &serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|text| text.trim().parse().ok()))
}

pub(super) fn rating(item: &serde_json::Value) -> Option<f32> {
    [
        "/imdbRating",
        "/mdblistRatings/imdb",
        "/meta/imdbRating",
        "/rating",
    ]
    .iter()
    .find_map(|path| item.pointer(path).and_then(number))
    .filter(|value| *value > 0.0 && *value <= 10.0)
    .map(|value| value as f32)
}

pub(super) fn caption(item: &serde_json::Value) -> String {
    let genre = item
        .get("genres")
        .and_then(serde_json::Value::as_array)
        .and_then(|genres| genres.iter().find_map(serde_json::Value::as_str));
    let year = year(item);
    genre
        .map(str::to_owned)
        .into_iter()
        .chain(year)
        .collect::<Vec<_>>()
        .join(" · ")
}

pub(super) fn year(item: &serde_json::Value) -> Option<String> {
    ["releaseInfo", "year", "released"].iter().find_map(|key| {
        let value = item.get(*key)?;
        let text = value
            .as_str()
            .map(str::to_owned)
            .or_else(|| value.as_i64().map(|year| year.to_string()))?;
        text.get(..4)
            .filter(|year| year.bytes().all(|b| b.is_ascii_digit()))
            .map(str::to_owned)
    })
}

pub(super) fn title_key(title: &str, year: &str) -> String {
    let mut key: String = title
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    key.push(':');
    key.push_str(year);
    key
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Graded {
    pub(super) score: Option<f32>,
    pub(super) certification: Option<String>,
    pub(super) oscar: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Enrichment {
    pub(super) trending_ids: HashMap<u64, u32>,
    pub(super) trending_titles: HashMap<String, u32>,
    pub(super) graded: HashMap<String, Graded>,
}

impl Enrichment {
    pub fn set_trending(&mut self, results: &[serde_json::Value]) {
        self.trending_ids.clear();
        self.trending_titles.clear();
        for (rank, item) in results.iter().enumerate() {
            let rank = rank as u32 + 1;
            if let Some(id) = item.get("id").and_then(serde_json::Value::as_u64) {
                self.trending_ids.entry(id).or_insert(rank);
            }
            let title = item
                .get("title")
                .or_else(|| item.get("name"))
                .and_then(serde_json::Value::as_str);
            let year = item
                .get("release_date")
                .or_else(|| item.get("first_air_date"))
                .and_then(serde_json::Value::as_str)
                .and_then(|date| date.get(..4));
            if let (Some(title), Some(year)) = (title, year) {
                self.trending_titles
                    .entry(title_key(title, year))
                    .or_insert(rank);
            }
        }
    }

    pub fn add_mdblist(&mut self, items: &[serde_json::Value]) {
        for item in items {
            let Some(imdb) = item
                .pointer("/ids/imdb")
                .or_else(|| item.get("imdbid"))
                .and_then(serde_json::Value::as_str)
            else {
                continue;
            };
            let score = item
                .get("score")
                .and_then(number)
                .filter(|score| *score > 0.0)
                .map(|score| (score / 10.0) as f32);
            let certification = item
                .get("certification")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned);
            let keywords: Vec<&str> = item
                .get("keywords")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|keyword| keyword.get("name")?.as_str())
                .collect();
            let oscar = if keywords.contains(&"best-picture-winner") {
                Some(true)
            } else if keywords.contains(&"best-picture-nominated") {
                Some(false)
            } else {
                None
            };
            self.graded.insert(
                imdb.to_owned(),
                Graded {
                    score,
                    certification,
                    oscar,
                },
            );
        }
    }

    pub fn has_graded(&self, imdb: &str) -> bool {
        self.graded.contains_key(imdb)
    }

    pub(super) fn trending(&self, facts: &PosterFacts) -> Option<u32> {
        facts
            .tmdb
            .and_then(|id| self.trending_ids.get(&id))
            .or_else(|| self.trending_titles.get(&facts.title_key))
            .copied()
    }

    pub(super) fn graded(&self, facts: &PosterFacts) -> Option<&Graded> {
        self.graded.get(facts.imdb.as_deref()?)
    }
}

pub(super) fn enrichment_id() -> Id {
    Id::new("fluxa-poster-enrichment")
}

pub fn set_poster_enrichment(context: &egui::Context, enrichment: Arc<Enrichment>) {
    context.data_mut(|data| {
        let current = data.get_temp::<Arc<Enrichment>>(enrichment_id());
        if !current.is_some_and(|current| Arc::ptr_eq(&current, &enrichment)) {
            data.insert_temp(enrichment_id(), enrichment);
        }
    });
}

pub(super) fn status(item: &serde_json::Value, today: i64) -> Option<PosterStatus> {
    let videos = item
        .get("videos")
        .or_else(|| item.pointer("/meta/videos"))
        .and_then(serde_json::Value::as_array);
    if let Some(videos) = videos.filter(|videos| !videos.is_empty()) {
        return series_status(videos, today);
    }
    let released = ["released", "releaseDate", "firstAirDate"]
        .iter()
        .find_map(|key| item.get(*key)?.as_str().and_then(day_number))?;
    if released > today {
        Some(PosterStatus::Upcoming)
    } else if today - released <= NEW_RELEASE_DAYS {
        Some(PosterStatus::NewRelease)
    } else {
        None
    }
}

pub(super) fn series_status(videos: &[serde_json::Value], today: i64) -> Option<PosterStatus> {
    let episodes = videos.iter().filter_map(|video| {
        let season = video.get("season").and_then(number)? as i64;
        let aired = video
            .get("released")
            .or_else(|| video.get("firstAired"))
            .and_then(serde_json::Value::as_str)
            .and_then(day_number)?;
        (season > 0).then_some((season, aired))
    });
    let (mut latest_season, mut season_start, mut last_aired) = (0, i64::MAX, i64::MIN);
    let mut any_upcoming = false;
    for (season, aired) in episodes {
        if aired > today {
            any_upcoming = true;
            continue;
        }
        last_aired = last_aired.max(aired);
        if season > latest_season {
            latest_season = season;
            season_start = aired;
        } else if season == latest_season {
            season_start = season_start.min(aired);
        }
    }
    if latest_season == 0 {
        return any_upcoming.then_some(PosterStatus::Upcoming);
    }
    if latest_season > 1 && today - season_start <= NEW_RELEASE_DAYS {
        Some(PosterStatus::NewSeason)
    } else if today - last_aired <= NEW_EPISODE_DAYS {
        Some(PosterStatus::NewEpisode)
    } else {
        None
    }
}

pub(super) fn day_number(date: &str) -> Option<i64> {
    let mut parts = date.get(..10)?.splitn(3, '-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}

pub(super) fn today() -> i64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|elapsed| (elapsed.as_secs() / 86_400) as i64)
        .unwrap_or(0)
}
