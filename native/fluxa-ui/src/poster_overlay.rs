use egui::epaint::{Mesh, TextShape};
use egui::{Align2, Color32, FontId, Id, Painter, Pos2, Rect, Shape, Stroke, TextureId, Vec2};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use super::{HomeCard, HomeRowKind, localized};

mod enrichment;
mod landscape;
mod personal;
mod scores;
mod template;
pub use enrichment::*;
pub use landscape::*;
pub use personal::*;
pub use scores::*;
pub use template::*;

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
    pub quality: bool,
    pub age: bool,
    pub awards: bool,
    pub mdblist_score: bool,
    pub trending_label: String,
    pub award_labels: [String; 2],
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
                "posterQualityBadges": true,
                "posterAgeRating": true,
                "posterAwardBadge": true,
                "posterRatingSource": "imdb",
            })
        })
        .get(key)
}

impl super::SettingsModel {
    pub fn poster_landscape(&self) -> bool {
        self.bool_value("posterLandscapeMode")
    }

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
            quality: enabled && self.bool_value("posterQualityBadges"),
            age: enabled && self.bool_value("posterAgeRating"),
            awards: enabled && self.bool_value("posterAwardBadge"),
            mdblist_score: self.str_value("posterRatingSource") == Some("mdblist"),
            trending_label: localized(
                "poster.badge.trending",
                self.str_value("language").unwrap_or("en"),
            ),
            award_labels: ["poster.badge.oscar_winner", "poster.badge.oscar_nominee"]
                .map(|key| localized(key, self.str_value("language").unwrap_or("en"))),
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

static LANDSCAPE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_poster_landscape(on: bool) {
    LANDSCAPE.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub(super) fn landscape() -> bool {
    LANDSCAPE.load(std::sync::atomic::Ordering::Relaxed)
}

fn logos_id() -> Id {
    Id::new("fluxa-poster-logos")
}

pub fn set_rating_logos(context: &egui::Context, logos: Vec<(&'static str, TextureId, Vec2)>) {
    context.data_mut(|data| data.insert_temp(logos_id(), Arc::new(logos)));
}

fn rating_logo(context: &egui::Context, name: &str) -> Option<(TextureId, Vec2)> {
    context
        .data(|data| data.get_temp::<Arc<Vec<(&'static str, TextureId, Vec2)>>>(logos_id()))?
        .iter()
        .find(|(key, ..)| *key == name)
        .map(|(_, texture, aspect)| (*texture, *aspect))
}

fn draw_logo(
    painter: &Painter,
    logo: Option<(TextureId, Vec2)>,
    left_center: Pos2,
    height: f32,
) -> f32 {
    let Some((texture, aspect)) = logo else {
        return 0.0;
    };
    let size = aspect * height;
    painter.image(
        texture,
        Rect::from_min_size(left_center - Vec2::new(0.0, height * 0.5), size),
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
    size.x
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
    } else if let Some(won) = enrichment
        .as_deref()
        .and_then(|enrichment| enrichment.graded(facts)?.oscar)
        .filter(|_| overlays.awards)
    {
        let colors = if won {
            (
                Color32::from_rgb(150, 115, 30),
                Color32::from_rgb(235, 200, 90),
            )
        } else {
            (
                Color32::from_rgb(70, 70, 75),
                Color32::from_rgb(190, 190, 195),
            )
        };
        sash(
            &painter,
            rect,
            colors,
            &overlays.award_labels[usize::from(!won)],
        );
        stacks[Placement::TopRight as usize] = rect.width() * 0.34;
    }

    if overlays.quality && personal.quality.contains(&true) {
        let size = (rect.width() * 0.06).clamp(8.0, 13.0) * overlays.scale;
        let top = rect.top() + inset + stacks[Placement::TopRight as usize];
        let bottom = quality_badges(&painter, rect.right() - inset, top, size, personal.quality);
        stacks[Placement::TopRight as usize] = bottom - rect.top() - inset;
    }

    let mut badges = Vec::with_capacity(2);
    if let (Some(placement), Some(value)) = (overlays.rating, rating) {
        let (source, name) = if rating == facts.rating {
            ("IMDb", "imdb")
        } else {
            ("★", "mdblist")
        };
        let logo = rating_logo(painter.ctx(), name);
        let text = match logo {
            Some(_) => format!("{value:.1}"),
            None => format!("{source} {value:.1}"),
        };
        badges.push((placement, text, logo));
    }
    if let (Some(placement), Some(status)) = (overlays.status, facts.status) {
        badges.push((placement, overlays.labels[status as usize].clone(), None));
    }
    badges.retain(|(placement, ..)| {
        !matches!(
            placement,
            Placement::Bar
                | Placement::Number
                | Placement::Minimal
                | Placement::Frosted
                | Placement::Sash
        )
    });
    badges.sort_by_key(|(placement, ..)| *placement != Placement::Banner);

    let font_size = (rect.width() * 0.085).clamp(9.0, 15.0) * overlays.scale;
    let font = FontId::proportional(font_size);
    let pad = Vec2::new(font_size * 0.45, font_size * 0.2);
    let inset = (rect.width() * 0.04).max(4.0);
    for (placement, text, logo) in badges {
        let galley = painter.layout_no_wrap(text, font.clone(), Color32::WHITE);
        let logo_width = logo.map_or(0.0, |(_, aspect)| aspect.x * galley.size().y + pad.x * 0.8);
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
        let size = galley.size() + pad * 2.0 + Vec2::new(logo_width, 0.0);
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
        let height = galley.size().y;
        draw_logo(
            &painter,
            logo,
            Pos2::new(badge.left() + pad.x, badge.center().y),
            height,
        );
        painter.galley(
            badge.min + pad + Vec2::new(logo_width, 0.0),
            galley,
            Color32::WHITE,
        );
    }
}

fn quality_badges(painter: &Painter, right: f32, mut y: f32, size: f32, quality: [bool; 4]) -> f32 {
    for label in QUALITY_LABELS
        .iter()
        .zip(quality)
        .filter_map(|(label, on)| on.then_some(*label))
    {
        let galley =
            painter.layout_no_wrap(label.to_owned(), FontId::proportional(size), Color32::WHITE);
        let badge = Rect::from_min_size(
            Pos2::new(right - galley.size().x - size * 0.8, y),
            Vec2::new(galley.size().x + size * 0.8, size * 1.5),
        );
        painter.rect_filled(badge, 3.0, Color32::from_black_alpha(190));
        painter.rect_stroke(
            badge,
            3.0,
            Stroke::new(1.0, Color32::from_white_alpha(60)),
            egui::StrokeKind::Inside,
        );
        painter.galley(badge.center() - galley.size() * 0.5, galley, Color32::WHITE);
        y += badge.height() + 3.0;
    }
    y
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn postersplus_template_uses_tmdb_prefix_and_skips_missing_optional() {
        let template = "https://pp.example/poster?tmdb_id={tmdb_id}&type={type}&imdb={imdb_id?}";
        assert_eq!(
            fill_template(template, "tmdb:1399", "tv", "poster", &json!({})).as_deref(),
            Some("https://pp.example/poster?tmdb_id=1399&type=series&imdb=")
        );
        assert_eq!(
            fill_template(template, "tt0944947", "series", "poster", &json!({})),
            None
        );
    }

    #[test]
    fn shape_placeholder_follows_card_shape() {
        let template = "https://pp.example/{shape}/{id}.jpg";
        assert_eq!(
            fill_template(template, "tt1", "movie", "landscape", &json!({})).as_deref(),
            Some("https://pp.example/landscape/tt1.jpg")
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
    fn progress_follows_continue_watching_and_watched_follows_library() {
        let index = personal_index(&serde_json::json!({
            "completed": [{"id": "tt1"}],
            "watched": {"tt3": true},
            "watchlist": [{"id": "tt2"}],
            "continueWatching": [{"id": "tt2", "resumeProgressPercent": 25.0}],
            "progress": {"movie:tt4": {"timeOffset": 30, "duration": 120, "meta": {"id": "tt4"}}},
        }));
        assert!(index.0["tt1"].watched);
        assert!(index.0["tt3"].watched);
        assert!(index.0["tt2"].saved);
        assert_eq!(index.0["tt2"].progress, 0.25);
        assert_eq!(index.0["tt4"].progress, 0.0);
    }

    #[test]
    fn best_picture_keywords_mark_oscar_win_over_nomination() {
        let mut enrichment = Enrichment::default();
        enrichment.add_mdblist(&[
            serde_json::json!({"ids": {"imdb": "tt1"}, "keywords": [{"name": "best-picture-nominated"}, {"name": "best-picture-winner"}]}),
            serde_json::json!({"ids": {"imdb": "tt2"}, "keywords": [{"name": "best-picture-nominated"}]}),
        ]);
        assert_eq!(enrichment.graded["tt1"].oscar, Some(true));
        assert_eq!(enrichment.graded["tt2"].oscar, Some(false));
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

    #[test]
    fn dolby_vision_release_is_not_also_tagged_hdr() {
        assert_eq!(
            stream_quality("Movie.2023.2160p.DV.HEVC.REMUX.mkv"),
            [true, true, false, true]
        );
        assert_eq!(
            stream_quality("Show S01E01 1080p HDR10"),
            [false, false, true, false]
        );
    }
}
