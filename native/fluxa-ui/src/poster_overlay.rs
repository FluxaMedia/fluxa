use egui::epaint::{Mesh, TextShape};
use egui::{Align2, Color32, FontId, Id, Painter, Pos2, Rect, Shape, Stroke, Vec2};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use super::{HomeCard, HomeRowKind, localized};

const NEW_EPISODE_DAYS: i64 = 7;
const NEW_RELEASE_DAYS: i64 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PosterStatus {
    Upcoming,
    NewSeason,
    NewEpisode,
    NewRelease,
}

impl PosterStatus {
    fn key(self) -> &'static str {
        match self {
            PosterStatus::Upcoming => "poster.badge.upcoming",
            PosterStatus::NewSeason => "poster.badge.new_season",
            PosterStatus::NewEpisode => "poster.badge.new_episode",
            PosterStatus::NewRelease => "poster.badge.new_release",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PosterFacts {
    pub rating: Option<f32>,
    pub status: Option<PosterStatus>,
    pub caption: String,
    pub imdb: Option<String>,
    pub tmdb: Option<u64>,
    pub title_key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Banner,
    Bar,
    Number,
    Minimal,
    Frosted,
    Sash,
}

impl Placement {
    fn parse(value: Option<&str>) -> Self {
        match value {
            Some("top_right") => Placement::TopRight,
            Some("bottom_left") => Placement::BottomLeft,
            Some("bottom_right") => Placement::BottomRight,
            Some("banner") => Placement::Banner,
            Some("bar") => Placement::Bar,
            Some("number") => Placement::Number,
            Some("minimal") => Placement::Minimal,
            Some("frosted") => Placement::Frosted,
            Some("sash") => Placement::Sash,
            _ => Placement::TopLeft,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PosterOverlays {
    pub rating: Option<Placement>,
    pub status: Option<Placement>,
    pub scale: f32,
    pub labels: [String; 4],
    pub watched: bool,
    pub progress: bool,
    pub saved: bool,
    pub fade: Fade,
    pub trending: bool,
    pub age: bool,
    pub mdblist_score: bool,
    pub trending_label: String,
    pub template: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fade {
    pub tint: bool,
    pub reach: f32,
    pub alpha: f32,
}

pub(super) fn setting_default(key: &str) -> Option<&'static serde_json::Value> {
    static DEFAULTS: OnceLock<serde_json::Value> = OnceLock::new();
    DEFAULTS
        .get_or_init(|| {
            serde_json::json!({
                "posterOverlaysEnabled": true,
                "posterRatingBadge": true,
                "posterRatingPosition": "bar",
                "posterStatusBadge": true,
                "posterStatusPosition": "sash",
                "posterBadgeSize": "default",
                "posterWatchedBadge": true,
                "posterProgressBar": true,
                "posterSavedBadge": false,
                "posterFadeTint": false,
                "posterFadeStrength": "high",
                "posterTrendingBadge": true,
                "posterAgeRating": true,
                "posterRatingSource": "imdb",
            })
        })
        .get(key)
}

impl super::SettingsModel {
    pub fn poster_overlays(&self) -> Option<PosterOverlays> {
        let template = self
            .str_value("posterUrlTemplate")
            .map(str::trim)
            .filter(|template| !template.is_empty())
            .map(str::to_owned);
        let enabled = template.is_none() && self.bool_value("posterOverlaysEnabled");
        if !enabled && template.is_none() {
            return None;
        }
        let placement = |toggle: &str, position: &str| {
            (enabled && self.bool_value(toggle)).then(|| Placement::parse(self.str_value(position)))
        };
        Some(PosterOverlays {
            rating: placement("posterRatingBadge", "posterRatingPosition"),
            status: placement("posterStatusBadge", "posterStatusPosition"),
            scale: match self.str_value("posterBadgeSize") {
                Some("small") => 0.8,
                Some("large") => 1.3,
                _ => 1.0,
            },
            labels: {
                let language = self.str_value("language").unwrap_or("en");
                [
                    PosterStatus::Upcoming,
                    PosterStatus::NewSeason,
                    PosterStatus::NewEpisode,
                    PosterStatus::NewRelease,
                ]
                .map(|status| localized(status.key(), language))
            },
            watched: enabled && self.bool_value("posterWatchedBadge"),
            progress: enabled && self.bool_value("posterProgressBar"),
            saved: enabled && self.bool_value("posterSavedBadge"),
            trending: enabled && self.bool_value("posterTrendingBadge"),
            age: enabled && self.bool_value("posterAgeRating"),
            mdblist_score: self.str_value("posterRatingSource") == Some("mdblist"),
            trending_label: localized(
                "poster.badge.trending",
                self.str_value("language").unwrap_or("en"),
            ),
            fade: Fade {
                tint: self.bool_value("posterFadeTint"),
                reach: match self.str_value("posterFadeStrength") {
                    Some("low") => 0.3,
                    Some("medium") => 0.4,
                    _ => 0.5,
                },
                alpha: match self.str_value("posterFadeStrength") {
                    Some("low") => 180.0,
                    Some("medium") => 210.0,
                    _ => 225.0,
                },
            },
            template,
        })
    }
}

fn overlays_id() -> Id {
    Id::new("fluxa-poster-overlays")
}

pub fn set_poster_overlays(context: &egui::Context, overlays: Option<PosterOverlays>) {
    context.data_mut(|data| match overlays {
        Some(overlays) => {
            let current = data.get_temp::<Arc<PosterOverlays>>(overlays_id());
            if current.as_deref() != Some(&overlays) {
                data.insert_temp(overlays_id(), Arc::new(overlays));
            }
        }
        None => data.remove::<Arc<PosterOverlays>>(overlays_id()),
    });
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Personal {
    pub watched: bool,
    pub saved: bool,
    pub progress: f32,
}

#[derive(Debug, Default, PartialEq)]
pub struct PersonalIndex(HashMap<String, Personal>);

pub(super) fn personal_index(library: &serde_json::Value) -> PersonalIndex {
    let mut index = HashMap::<String, Personal>::new();
    let ids = |key: &str| {
        library
            .get(key)
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|item| item.get("id").and_then(serde_json::Value::as_str))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    for id in ids("completed") {
        index.entry(id).or_default().watched = true;
    }
    for id in ids("watchlist").into_iter().chain(ids("liked")) {
        index.entry(id).or_default().saved = true;
    }
    if let Some(progress) = library
        .get("progress")
        .and_then(serde_json::Value::as_object)
    {
        for (key, entry) in progress {
            let offset = entry.get("timeOffset").and_then(number).unwrap_or(0.0);
            let duration = entry.get("duration").and_then(number).unwrap_or(0.0);
            if duration <= 0.0 {
                continue;
            }
            let id = entry
                .pointer("/meta/id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(key);
            index.entry(id.to_owned()).or_default().progress =
                (offset / duration).clamp(0.0, 1.0) as f32;
        }
    }
    PersonalIndex(index)
}

pub fn set_poster_personal(context: &egui::Context, index: Arc<PersonalIndex>) {
    context.data_mut(|data| {
        let current = data.get_temp::<Arc<PersonalIndex>>(personal_id());
        if !current.is_some_and(|current| Arc::ptr_eq(&current, &index)) {
            data.insert_temp(personal_id(), index);
        }
    });
}

fn personal_id() -> Id {
    Id::new("fluxa-poster-personal")
}

fn current(context: &egui::Context) -> Option<Arc<PosterOverlays>> {
    context.data(|data| data.get_temp::<Arc<PosterOverlays>>(overlays_id()))
}

thread_local! {
    static URLS: RefCell<(String, HashMap<String, Option<String>>)> = RefCell::default();
}

pub(super) fn custom_url(context: &egui::Context, card: &HomeCard) -> Option<String> {
    if card.row_kind == HomeRowKind::Collection {
        return None;
    }
    let overlays = current(context)?;
    let template = overlays.template.as_deref()?;
    let id = card.id.as_deref()?;
    URLS.with_borrow_mut(|(cached, urls)| {
        if cached != template {
            *cached = template.to_owned();
            urls.clear();
        }
        urls.entry(id.to_owned())
            .or_insert_with(|| {
                fill_template(
                    template,
                    id,
                    card.item_type.as_deref().unwrap_or_default(),
                    &card.raw,
                )
            })
            .clone()
    })
}

fn fill_template(
    template: &str,
    id: &str,
    item_type: &str,
    raw: &serde_json::Value,
) -> Option<String> {
    let field = |keys: &[&str]| {
        keys.iter().find_map(|key| {
            let value = raw.get(*key)?;
            value
                .as_str()
                .map(str::to_owned)
                .or_else(|| value.as_i64().map(|number| number.to_string()))
                .filter(|value| !value.is_empty())
        })
    };
    let prefixed = |prefix: &str| {
        id.strip_prefix(prefix)
            .map(|rest| rest.split(':').next().unwrap_or(rest).to_owned())
    };
    let lookup = |name: &str| match name {
        "imdb_id" => field(&["imdb_id", "imdbId"]).or_else(|| {
            id.starts_with("tt")
                .then(|| id.split(':').next().unwrap_or(id).to_owned())
        }),
        "tmdb_id" => field(&["tmdb_id", "tmdbId", "moviedb_id"]).or_else(|| prefixed("tmdb:")),
        "anilist_id" => prefixed("anilist:"),
        "kitsu_id" => prefixed("kitsu:"),
        "type" => Some(match item_type {
            "series" | "tv" | "show" => "series".to_owned(),
            _ => "movie".to_owned(),
        }),
        "id" => Some(id.to_owned()),
        _ => None,
    };
    let mut url = String::with_capacity(template.len() + 16);
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        url.push_str(&rest[..start]);
        let end = rest[start..].find('}')? + start;
        let name = &rest[start + 1..end];
        let (name, optional) = name
            .strip_suffix('?')
            .map_or((name, false), |name| (name, true));
        match lookup(name) {
            Some(value) => url.push_str(&value),
            None if optional => {}
            None => return None,
        }
        rest = &rest[end + 1..];
    }
    url.push_str(rest);
    Some(url)
}

pub(super) fn poster_facts(item: &serde_json::Value) -> PosterFacts {
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

fn number(value: &serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|text| text.trim().parse().ok()))
}

fn rating(item: &serde_json::Value) -> Option<f32> {
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

fn caption(item: &serde_json::Value) -> String {
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

fn year(item: &serde_json::Value) -> Option<String> {
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

fn title_key(title: &str, year: &str) -> String {
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
struct Graded {
    score: Option<f32>,
    certification: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Enrichment {
    trending_ids: HashMap<u64, u32>,
    trending_titles: HashMap<String, u32>,
    graded: HashMap<String, Graded>,
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
            self.graded.insert(
                imdb.to_owned(),
                Graded {
                    score,
                    certification,
                },
            );
        }
    }

    pub fn has_graded(&self, imdb: &str) -> bool {
        self.graded.contains_key(imdb)
    }

    fn trending(&self, facts: &PosterFacts) -> Option<u32> {
        facts
            .tmdb
            .and_then(|id| self.trending_ids.get(&id))
            .or_else(|| self.trending_titles.get(&facts.title_key))
            .copied()
    }

    fn graded(&self, facts: &PosterFacts) -> Option<&Graded> {
        self.graded.get(facts.imdb.as_deref()?)
    }
}

fn enrichment_id() -> Id {
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

fn status(item: &serde_json::Value, today: i64) -> Option<PosterStatus> {
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

fn series_status(videos: &[serde_json::Value], today: i64) -> Option<PosterStatus> {
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

fn day_number(date: &str) -> Option<i64> {
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

fn today() -> i64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|elapsed| (elapsed.as_secs() / 86_400) as i64)
        .unwrap_or(0)
}

pub(super) fn paint(
    painter: &Painter,
    rect: Rect,
    card: &HomeCard,
    radius: f32,
    tones: Option<[[u8; 3]; 2]>,
) {
    if card.row_kind == HomeRowKind::Collection {
        return;
    }
    let Some(overlays) = current(painter.ctx()) else {
        return;
    };
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    let facts = &card.overlay;
    let enrichment = painter
        .ctx()
        .data(|data| data.get_temp::<Arc<Enrichment>>(enrichment_id()));
    let graded = enrichment
        .as_deref()
        .and_then(|enrichment| enrichment.graded(facts));
    let rating = graded
        .and_then(|graded| graded.score)
        .filter(|_| overlays.mdblist_score)
        .or(facts.rating);
    let shade = match tones.filter(|_| overlays.fade.tint) {
        Some([_, [r, g, b]]) => {
            let dim = |c: u8| (c as f32 * 0.3) as u8;
            [dim(r), dim(g), dim(b)]
        }
        None => [0, 0, 0],
    };
    let vignette = |painter: &Painter| bottom_vignette(painter, rect, radius, overlays.fade, shade);
    let mut stacks = [0.0f32; 4];
    let personal = card
        .id
        .as_deref()
        .zip(
            painter
                .ctx()
                .data(|data| data.get_temp::<Arc<PersonalIndex>>(personal_id())),
        )
        .and_then(|(id, index)| index.0.get(id).copied())
        .unwrap_or_default();
    if overlays.progress
        && card.progress <= 0.0
        && !personal.watched
        && (0.02..0.95).contains(&personal.progress)
    {
        let height = (rect.height() * 0.018).max(3.0);
        let track = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - height), rect.max);
        painter.rect_filled(track, 0.0, Color32::from_black_alpha(160));
        painter.rect_filled(
            Rect::from_min_size(
                track.min,
                Vec2::new(track.width() * personal.progress, height),
            ),
            0.0,
            Color32::from_gray(235),
        );
    }
    let mark = (rect.width() * 0.13).clamp(14.0, 28.0) * overlays.scale;
    let inset = (rect.width() * 0.04).max(4.0);
    let mut corner = Pos2::new(rect.left() + inset, rect.top() + inset);
    if overlays.watched && personal.watched {
        watched_mark(&painter, Rect::from_min_size(corner, Vec2::splat(mark)));
        corner.x += mark + 3.0;
    }
    if overlays.saved && personal.saved {
        saved_mark(&painter, Rect::from_min_size(corner, Vec2::splat(mark)));
        corner.x += mark + 3.0;
    }
    if let Some(certification) = graded
        .and_then(|graded| graded.certification.as_deref())
        .filter(|_| overlays.age)
    {
        let size = mark * 0.5;
        let galley = painter.layout_no_wrap(
            certification.to_owned(),
            FontId::proportional(size),
            Color32::WHITE,
        );
        let badge = Rect::from_min_size(
            Pos2::new(corner.x, corner.y + (mark - size * 1.6) * 0.5),
            Vec2::new(galley.size().x + size * 0.7, size * 1.6),
        );
        painter.rect_filled(badge, 3.0, Color32::from_black_alpha(180));
        painter.rect_stroke(
            badge,
            3.0,
            Stroke::new(1.0, Color32::from_white_alpha(170)),
            egui::StrokeKind::Inside,
        );
        painter.galley(badge.center() - galley.size() * 0.5, galley, Color32::WHITE);
        corner.x += badge.width() + 3.0;
    }
    if corner.x > rect.left() + inset {
        stacks[Placement::TopLeft as usize] = mark + 3.0;
    }

    if rating.is_some() || !facts.caption.is_empty() {
        let reserved = match overlays.rating {
            Some(Placement::Bar) => {
                vignette(&painter);
                score_bar(&painter, rect, rating, &facts.caption, overlays.scale);
                0.2
            }
            Some(Placement::Number) => {
                vignette(&painter);
                score_number(&painter, rect, rating, &facts.caption, overlays.scale)
            }
            Some(Placement::Minimal) => {
                vignette(&painter);
                score_minimal(&painter, rect, rating, &facts.caption, overlays.scale)
            }
            Some(Placement::Frosted) => score_frosted(
                &painter,
                rect,
                rating,
                &facts.caption,
                overlays.scale,
                radius,
            ),
            _ => 0.0,
        };
        stacks[Placement::BottomLeft as usize] = rect.height() * reserved;
        stacks[Placement::BottomRight as usize] = rect.height() * reserved;
    }
    let trending = enrichment
        .as_deref()
        .and_then(|enrichment| enrichment.trending(facts))
        .filter(|_| overlays.trending);
    if let (Some(Placement::Sash), Some(status)) = (overlays.status, facts.status) {
        sash(
            &painter,
            rect,
            sash_colors(status),
            &overlays.labels[status as usize],
        );
        stacks[Placement::TopRight as usize] = rect.width() * 0.34;
    } else if let Some(rank) = trending {
        let label = format!("#{rank} {}", overlays.trending_label);
        sash(
            &painter,
            rect,
            (
                Color32::from_rgb(150, 30, 40),
                Color32::from_rgb(235, 90, 90),
            ),
            &label,
        );
        stacks[Placement::TopRight as usize] = rect.width() * 0.34;
    }

    let mut badges = Vec::with_capacity(2);
    if let (Some(placement), Some(value)) = (overlays.rating, rating) {
        let source = if rating == facts.rating {
            "IMDb"
        } else {
            "★"
        };
        badges.push((placement, format!("{source} {value:.1}")));
    }
    if let (Some(placement), Some(status)) = (overlays.status, facts.status) {
        badges.push((placement, overlays.labels[status as usize].clone()));
    }
    badges.retain(|(placement, _)| {
        !matches!(
            placement,
            Placement::Bar
                | Placement::Number
                | Placement::Minimal
                | Placement::Frosted
                | Placement::Sash
        )
    });
    badges.sort_by_key(|(placement, _)| *placement != Placement::Banner);

    let font_size = (rect.width() * 0.085).clamp(9.0, 15.0) * overlays.scale;
    let font = FontId::proportional(font_size);
    let pad = Vec2::new(font_size * 0.45, font_size * 0.2);
    let inset = (rect.width() * 0.04).max(4.0);
    for (placement, text) in badges {
        let galley = painter.layout_no_wrap(text, font.clone(), Color32::WHITE);
        if placement == Placement::Banner {
            let height = galley.size().y + pad.y * 4.0;
            let top = rect.bottom() - height - stacks[Placement::BottomLeft as usize];
            let banner = Rect::from_min_max(
                Pos2::new(rect.left(), top),
                Pos2::new(rect.right(), top + height),
            );
            let corner = if stacks[Placement::BottomLeft as usize] > 0.0 {
                0
            } else {
                radius as u8
            };
            painter.rect_filled(
                banner,
                egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: corner,
                    se: corner,
                },
                Color32::from_black_alpha(190),
            );
            painter.galley(
                banner.center() - galley.size() * 0.5,
                galley,
                Color32::WHITE,
            );
            stacks[Placement::BottomLeft as usize] += height;
            stacks[Placement::BottomRight as usize] += height;
            continue;
        }
        let size = galley.size() + pad * 2.0;
        let slot = placement as usize;
        let offset = stacks[slot];
        stacks[slot] += size.y + 3.0;
        let (x, y, anchor) = match placement {
            Placement::TopLeft => (
                rect.left() + inset,
                rect.top() + inset + offset,
                Align2::LEFT_TOP,
            ),
            Placement::TopRight => (
                rect.right() - inset,
                rect.top() + inset + offset,
                Align2::RIGHT_TOP,
            ),
            Placement::BottomLeft => (
                rect.left() + inset,
                rect.bottom() - inset - offset,
                Align2::LEFT_BOTTOM,
            ),
            _ => (
                rect.right() - inset,
                rect.bottom() - inset - offset,
                Align2::RIGHT_BOTTOM,
            ),
        };
        let badge = anchor.anchor_size(Pos2::new(x, y), size);
        painter.rect_filled(badge, size.y * 0.3, Color32::from_black_alpha(180));
        painter.rect_stroke(
            badge,
            size.y * 0.3,
            Stroke::new(1.0, Color32::from_white_alpha(28)),
            egui::StrokeKind::Inside,
        );
        painter.galley(badge.min + pad, galley, Color32::WHITE);
    }
}

fn watched_mark(painter: &Painter, rect: Rect) {
    painter.circle_filled(
        rect.center(),
        rect.width() * 0.5,
        Color32::from_black_alpha(190),
    );
    let at = |x: f32, y: f32| rect.min + rect.size() * Vec2::new(x, y);
    painter.line(
        vec![at(0.28, 0.52), at(0.44, 0.67), at(0.73, 0.36)],
        Stroke::new(rect.width() * 0.11, Color32::WHITE),
    );
}

fn saved_mark(painter: &Painter, rect: Rect) {
    painter.circle_filled(
        rect.center(),
        rect.width() * 0.5,
        Color32::from_black_alpha(190),
    );
    let at = |x: f32, y: f32| rect.min + rect.size() * Vec2::new(x, y);
    painter.add(Shape::convex_polygon(
        vec![
            at(0.34, 0.26),
            at(0.66, 0.26),
            at(0.66, 0.74),
            at(0.5, 0.62),
        ],
        Color32::WHITE,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![at(0.34, 0.26), at(0.5, 0.62), at(0.34, 0.74)],
        Color32::WHITE,
        Stroke::NONE,
    ));
}

fn bottom_vignette(painter: &Painter, rect: Rect, radius: f32, fade: Fade, [r, g, b]: [u8; 3]) {
    const STEPS: usize = 12;
    let solid = radius.min(rect.height() * 0.1);
    let top = rect.bottom() - rect.height() * fade.reach;
    let bottom = rect.bottom() - solid;
    let shade = |alpha: f32| Color32::from_rgba_unmultiplied(r, g, b, alpha as u8);
    let mut mesh = Mesh::default();
    for step in 0..=STEPS {
        let t = step as f32 / STEPS as f32;
        let y = top + (bottom - top) * t;
        let color = shade(fade.alpha * t.powf(1.5));
        mesh.colored_vertex(Pos2::new(rect.left(), y), color);
        mesh.colored_vertex(Pos2::new(rect.right(), y), color);
        if step > 0 {
            let base = (step as u32 - 1) * 2;
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 3, base + 2);
        }
    }
    painter.add(Shape::mesh(mesh));
    let corner = solid as u8;
    painter.rect_filled(
        Rect::from_min_max(Pos2::new(rect.left(), bottom), rect.max),
        egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: corner,
            se: corner,
        },
        shade(fade.alpha),
    );
}

fn score_colors(score: f32) -> (Color32, Color32) {
    let (left, right) = match score {
        s if s < 5.0 => ([255, 80, 80], [160, 40, 40]),
        s if s < 7.0 => ([255, 210, 90], [200, 150, 40]),
        s if s < 8.5 => ([120, 255, 160], [40, 170, 90]),
        _ => ([190, 140, 255], [186, 85, 211]),
    };
    let soft = |[r, g, b]: [u8; 3]| {
        let mix = |c: u8| (c as f32 * 0.9 + 255.0 * 0.1) as u8;
        Color32::from_rgba_unmultiplied(mix(r), mix(g), mix(b), 220)
    };
    (soft(left), soft(right))
}

fn score_bar(painter: &Painter, rect: Rect, rating: Option<f32>, caption: &str, scale: f32) {
    let width = rect.width();
    let bar_height = (rect.height() * 0.012 * scale).max(3.0);
    let side = width * 0.14;
    let bottom = rect.bottom() - rect.height() * 0.04;
    let track = Rect::from_min_max(
        Pos2::new(rect.left() + side, bottom - bar_height),
        Pos2::new(rect.right() - side, bottom),
    );
    if !caption.is_empty() {
        let size = (width * 0.08 * scale).clamp(8.0, 18.0);
        let galley = painter.layout_no_wrap(
            caption.to_owned(),
            FontId::proportional(size),
            Color32::from_gray(200),
        );
        let center = Pos2::new(rect.center().x, track.top() - size * 0.95);
        painter.galley(
            center - galley.size() * 0.5,
            galley,
            Color32::from_gray(200),
        );
    }
    let Some(score) = rating else {
        return;
    };
    let pill = bar_height * 0.5;
    painter.rect_filled(track, pill, Color32::from_white_alpha(45));
    let fill = Rect::from_min_size(
        track.min,
        Vec2::new(track.width() * score / 10.0, bar_height),
    );
    let (left, right) = score_colors(score);
    let fill_painter = painter.with_clip_rect(fill.intersect(painter.clip_rect()));
    let shape = Rect::from_min_size(fill.min, Vec2::new(fill.width() + pill, bar_height));
    fill_painter.rect_filled(shape, pill, left);
    let mut mesh = Mesh::default();
    let fade = |c: Color32| Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 0);
    mesh.colored_vertex(Pos2::new(fill.left() + pill, fill.top()), fade(right));
    mesh.colored_vertex(fill.right_top() + Vec2::X * pill, right);
    mesh.colored_vertex(fill.right_bottom() + Vec2::X * pill, right);
    mesh.colored_vertex(Pos2::new(fill.left() + pill, fill.bottom()), fade(right));
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    fill_painter.add(Shape::mesh(mesh));
    fill_painter.line_segment(
        [
            Pos2::new(fill.left() + pill, fill.top() + 1.0),
            Pos2::new(fill.right(), fill.top() + 1.0),
        ],
        Stroke::new(1.0, Color32::from_white_alpha(60)),
    );
}

fn score_number(
    painter: &Painter,
    rect: Rect,
    rating: Option<f32>,
    caption: &str,
    scale: f32,
) -> f32 {
    let width = rect.width();
    let left = rect.left() + width * 0.07;
    let mut bottom = rect.bottom() - rect.height() * 0.04;
    if !caption.is_empty() {
        let size = (width * 0.07 * scale).clamp(8.0, 16.0);
        let galley = painter.layout_no_wrap(
            caption.to_owned(),
            FontId::proportional(size),
            Color32::from_gray(200),
        );
        bottom -= galley.size().y;
        painter.galley(Pos2::new(left, bottom), galley, Color32::from_gray(200));
    }
    if let Some(score) = rating {
        let size = (width * 0.2 * scale).clamp(16.0, 56.0);
        let galley = painter.layout_no_wrap(
            format!("{score:.1}"),
            FontId::proportional(size),
            score_colors(score).0,
        );
        bottom -= galley.size().y * 0.9;
        painter.galley(Pos2::new(left, bottom), galley, Color32::WHITE);
    }
    (rect.bottom() - bottom) / rect.height() + 0.02
}

fn score_minimal(
    painter: &Painter,
    rect: Rect,
    rating: Option<f32>,
    caption: &str,
    scale: f32,
) -> f32 {
    let year = caption.rsplit(" · ").next().filter(|year| year.len() == 4);
    let text = rating
        .map(|score| format!("★ {score:.1}"))
        .into_iter()
        .chain(year.map(str::to_owned))
        .collect::<Vec<_>>()
        .join(" · ");
    if text.is_empty() {
        return 0.0;
    }
    let size = (rect.width() * 0.075 * scale).clamp(9.0, 18.0);
    let galley = painter.layout_no_wrap(text, FontId::proportional(size), Color32::from_gray(235));
    let top = rect.bottom() - rect.height() * 0.05 - galley.size().y;
    painter.galley(
        Pos2::new(rect.left() + rect.width() * 0.06, top),
        galley,
        Color32::from_gray(235),
    );
    (rect.bottom() - top) / rect.height() + 0.02
}

fn score_frosted(
    painter: &Painter,
    rect: Rect,
    rating: Option<f32>,
    caption: &str,
    scale: f32,
    radius: f32,
) -> f32 {
    let height = (rect.height() * 0.09 * scale).max(18.0);
    let bar = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - height), rect.max);
    let r = radius as u8;
    painter.rect_filled(
        bar,
        egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: r,
            se: r,
        },
        Color32::from_black_alpha(165),
    );
    painter.line_segment(
        [bar.left_top(), bar.right_top()],
        Stroke::new(1.0, Color32::from_white_alpha(35)),
    );
    let size = height * 0.45;
    let pad = rect.width() * 0.05;
    if let Some(score) = rating {
        let galley = painter.layout_no_wrap(
            format!("★ {score:.1}"),
            FontId::proportional(size),
            Color32::WHITE,
        );
        painter.galley(
            Pos2::new(bar.left() + pad, bar.center().y - galley.size().y * 0.5),
            galley,
            Color32::WHITE,
        );
    }
    if !caption.is_empty() {
        let galley = painter.layout_no_wrap(
            caption.to_owned(),
            FontId::proportional(size * 0.8),
            Color32::from_gray(190),
        );
        let pos = Pos2::new(
            bar.right() - pad - galley.size().x,
            bar.center().y - galley.size().y * 0.5,
        );
        painter.galley(pos, galley, Color32::from_gray(190));
    }
    height / rect.height()
}

fn sash_colors(status: PosterStatus) -> (Color32, Color32) {
    let ([r, g, b], [br, bg, bb]) = match status {
        PosterStatus::NewSeason => ([30, 130, 120], [100, 220, 210]),
        PosterStatus::NewEpisode => ([50, 110, 190], [160, 220, 255]),
        PosterStatus::Upcoming => ([170, 100, 20], [255, 190, 90]),
        PosterStatus::NewRelease => ([160, 130, 40], [212, 175, 55]),
    };
    (Color32::from_rgb(r, g, b), Color32::from_rgb(br, bg, bb))
}

fn sash(painter: &Painter, rect: Rect, (inner, border): (Color32, Color32), label: &str) {
    let width = rect.width();
    let length = width * 1.15;
    let height = width * 0.12;
    let center = Pos2::new(rect.right() - width * 0.162, rect.top() + width * 0.162);
    let along = Vec2::new(1.0, 1.0).normalized();
    let across = along.rot90();
    let band = |inset: f32| {
        let half = height * 0.5 - inset;
        vec![
            center - along * length * 0.5 - across * half,
            center + along * length * 0.5 - across * half,
            center + along * length * 0.5 + across * half,
            center - along * length * 0.5 + across * half,
        ]
    };
    let shadow: Vec<Pos2> = band(0.0)
        .into_iter()
        .map(|p| p + Vec2::splat(height * 0.08))
        .collect();
    painter.add(Shape::convex_polygon(
        shadow,
        Color32::from_black_alpha(90),
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(band(0.0), border, Stroke::NONE));
    painter.add(Shape::convex_polygon(
        band((height / 18.0).max(1.0)),
        inner,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        band(height * 0.12),
        Color32::from_rgba_unmultiplied(8, 8, 8, 245),
        Stroke::NONE,
    ));

    let size = (height * 0.4).min(height * 0.85 / (label.chars().count() as f32).powf(0.35));
    let text = label.to_uppercase();
    let font = FontId::proportional(size);
    let angle = std::f32::consts::FRAC_PI_4;
    for (offset, color) in [
        (Vec2::splat(size * 0.08), Color32::from_black_alpha(180)),
        (Vec2::ZERO, Color32::from_gray(225)),
    ] {
        let galley = painter.layout_no_wrap(text.clone(), font.clone(), color);
        let pos = center + offset - galley.size() * 0.5;
        painter.add(
            TextShape::new(pos, galley, color).with_angle_and_anchor(angle, Align2::CENTER_CENTER),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn postersplus_template_uses_tmdb_prefix_and_skips_missing_optional() {
        let template = "https://pp.example/poster?tmdb_id={tmdb_id}&type={type}&imdb={imdb_id?}";
        assert_eq!(
            fill_template(template, "tmdb:1399", "tv", &json!({})).as_deref(),
            Some("https://pp.example/poster?tmdb_id=1399&type=series&imdb=")
        );
        assert_eq!(
            fill_template(template, "tt0944947", "series", &json!({})),
            None
        );
    }

    #[test]
    fn day_number_matches_unix_epoch() {
        assert_eq!(day_number("1970-01-01"), Some(0));
        assert_eq!(day_number("2024-03-01T00:00:00.000Z"), Some(19_783));
    }

    #[test]
    fn second_season_premiere_within_a_month_is_new_season() {
        let today = day_number("2026-09-26").unwrap();
        let videos = json!([
            {"season": 1, "episode": 1, "released": "2024-01-01"},
            {"season": 2, "episode": 1, "released": "2026-09-10"},
            {"season": 2, "episode": 2, "released": "2026-10-01"},
        ]);
        let item = json!({ "videos": videos });
        assert_eq!(status(&item, today), Some(PosterStatus::NewSeason));
    }

    #[test]
    fn weekly_episode_on_old_season_is_new_episode() {
        let today = day_number("2026-09-26").unwrap();
        let item = json!({ "videos": [
            {"season": 1, "released": "2026-08-01"},
            {"season": 1, "released": "2026-09-24"},
        ]});
        assert_eq!(status(&item, today), Some(PosterStatus::NewEpisode));
    }

    #[test]
    fn future_movie_is_upcoming_and_string_rating_parses() {
        let today = day_number("2026-09-26").unwrap();
        let item = json!({ "released": "2026-12-18T00:00:00.000Z", "imdbRating": "7.4" });
        assert_eq!(status(&item, today), Some(PosterStatus::Upcoming));
        assert_eq!(rating(&item), Some(7.4));
    }

    #[test]
    fn progress_is_keyed_by_meta_id_and_completed_counts_as_watched() {
        let index = personal_index(&serde_json::json!({
            "completed": [{"id": "tt1"}],
            "watchlist": [{"id": "tt2"}],
            "progress": {"series:tt2": {"timeOffset": 30, "duration": 120, "meta": {"id": "tt2"}}},
        }));
        assert!(index.0["tt1"].watched);
        assert!(index.0["tt2"].saved);
        assert_eq!(index.0["tt2"].progress, 0.25);
    }

    #[test]
    fn imdb_only_card_matches_trending_by_title_and_year() {
        let facts = poster_facts(&serde_json::json!({
            "id": "tt0903747",
            "name": "Breaking Bad!",
            "releaseInfo": "2008–2013",
        }));
        let mut enrichment = Enrichment::default();
        enrichment.set_trending(&[
            serde_json::json!({"id": 1, "title": "Other", "release_date": "2024-01-01"}),
            serde_json::json!({"id": 1396, "name": "Breaking Bad", "first_air_date": "2008-01-20"}),
        ]);
        enrichment.add_mdblist(&[serde_json::json!({
            "ids": {"imdb": "tt0903747"}, "score": 95, "certification": "TV-MA",
        })]);
        assert_eq!(enrichment.trending(&facts), Some(2));
        let graded = enrichment.graded(&facts).unwrap();
        assert_eq!(graded.score, Some(9.5));
        assert_eq!(graded.certification.as_deref(), Some("TV-MA"));
    }

}
