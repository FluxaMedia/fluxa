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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Banner,
    Bar,
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
    pub template: Option<String>,
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
    let year = ["releaseInfo", "year", "released"].iter().find_map(|key| {
        let value = item.get(*key)?;
        let text = value
            .as_str()
            .map(str::to_owned)
            .or_else(|| value.as_i64().map(|year| year.to_string()))?;
        text.get(..4)
            .filter(|year| year.bytes().all(|b| b.is_ascii_digit()))
            .map(str::to_owned)
    });
    genre
        .map(str::to_owned)
        .into_iter()
        .chain(year)
        .collect::<Vec<_>>()
        .join(" · ")
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

pub(super) fn paint(painter: &Painter, rect: Rect, card: &HomeCard, radius: f32) {
    if card.row_kind == HomeRowKind::Collection {
        return;
    }
    let Some(overlays) = current(painter.ctx()) else {
        return;
    };
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    let facts = &card.overlay;
    let mut stacks = [0.0f32; 4];

    if overlays.rating == Some(Placement::Bar)
        && (facts.rating.is_some() || !facts.caption.is_empty())
    {
        bottom_vignette(&painter, rect, radius);
        score_bar(&painter, rect, facts, overlays.scale);
        stacks[Placement::BottomLeft as usize] = rect.height() * 0.2;
        stacks[Placement::BottomRight as usize] = rect.height() * 0.2;
    }
    if let (Some(Placement::Sash), Some(status)) = (overlays.status, facts.status) {
        sash(
            &painter,
            rect,
            status,
            &overlays.labels[status as usize],
        );
        stacks[Placement::TopRight as usize] = rect.width() * 0.34;
    }

    let mut badges = Vec::with_capacity(2);
    if let (Some(placement), Some(value)) = (overlays.rating, facts.rating) {
        badges.push((placement, format!("IMDb {value:.1}")));
    }
    if let (Some(placement), Some(status)) = (overlays.status, facts.status) {
        badges.push((placement, overlays.labels[status as usize].clone()));
    }
    badges.retain(|(placement, _)| !matches!(placement, Placement::Bar | Placement::Sash));
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

fn bottom_vignette(painter: &Painter, rect: Rect, radius: f32) {
    const STEPS: usize = 12;
    let max_alpha = 225.0;
    let solid = radius.min(rect.height() * 0.1);
    let top = rect.bottom() - rect.height() * 0.5;
    let bottom = rect.bottom() - solid;
    let mut mesh = Mesh::default();
    for step in 0..=STEPS {
        let t = step as f32 / STEPS as f32;
        let y = top + (bottom - top) * t;
        let color = Color32::from_black_alpha((max_alpha * t.powf(1.5)) as u8);
        mesh.colored_vertex(Pos2::new(rect.left(), y), color);
        mesh.colored_vertex(Pos2::new(rect.right(), y), color);
        if step > 0 {
            let base = (step as u32 - 1) * 2;
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 3, base + 2);
        }
    }
    painter.add(Shape::mesh(mesh));
    let r = solid as u8;
    painter.rect_filled(
        Rect::from_min_max(Pos2::new(rect.left(), bottom), rect.max),
        egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: r,
            se: r,
        },
        Color32::from_black_alpha(max_alpha as u8),
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

fn score_bar(painter: &Painter, rect: Rect, facts: &PosterFacts, scale: f32) {
    let width = rect.width();
    let bar_height = (rect.height() * 0.012 * scale).max(3.0);
    let side = width * 0.14;
    let bottom = rect.bottom() - rect.height() * 0.04;
    let track = Rect::from_min_max(
        Pos2::new(rect.left() + side, bottom - bar_height),
        Pos2::new(rect.right() - side, bottom),
    );
    if !facts.caption.is_empty() {
        let size = (width * 0.08 * scale).clamp(8.0, 18.0);
        let galley = painter.layout_no_wrap(
            facts.caption.clone(),
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
    let Some(score) = facts.rating else {
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

fn sash_colors(status: PosterStatus) -> (Color32, Color32) {
    let ([r, g, b], [br, bg, bb]) = match status {
        PosterStatus::NewSeason => ([30, 130, 120], [100, 220, 210]),
        PosterStatus::NewEpisode => ([50, 110, 190], [160, 220, 255]),
        PosterStatus::Upcoming => ([170, 100, 20], [255, 190, 90]),
        PosterStatus::NewRelease => ([160, 130, 40], [212, 175, 55]),
    };
    (Color32::from_rgb(r, g, b), Color32::from_rgb(br, bg, bb))
}

fn sash(painter: &Painter, rect: Rect, status: PosterStatus, label: &str) {
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
    let (inner, border) = sash_colors(status);
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
}
