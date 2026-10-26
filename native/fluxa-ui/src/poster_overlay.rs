use egui::epaint::{Mesh, TextShape};
use egui::{Align2, Color32, FontId, Id, Painter, Pos2, Rect, Shape, Stroke, TextureId, Vec2};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use super::{HomeCard, HomeRowKind, components, ink, localized, stroke, text_primary, tone};

mod enrichment;
mod landscape;
mod meta;
mod personal;
mod sashes;
mod scores;
mod template;
mod top_numbers;
pub use enrichment::*;
pub use landscape::*;
pub use meta::*;
pub use personal::*;
pub use sashes::*;
pub use scores::*;
pub use template::*;
pub use top_numbers::*;

const NEW_EPISODE_DAYS: i64 = 7;
const NEW_RELEASE_DAYS: i64 = 30;
const NEW_PREMIERE_DAYS: i64 = 14;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PosterStatus {
    Upcoming,
    NewSeason,
    NewEpisode,
    NewRelease,
    Premiere,
    Finale,
    Ended,
    Cancelled,
    MiniSeries,
}

impl PosterStatus {
    fn slug(self) -> &'static str {
        match self {
            PosterStatus::Upcoming => "upcoming",
            PosterStatus::NewSeason => "new_season",
            PosterStatus::NewEpisode => "new_episode",
            PosterStatus::NewRelease => "new_release",
            PosterStatus::Premiere => "premiere",
            PosterStatus::Finale => "finale",
            PosterStatus::Ended => "ended",
            PosterStatus::Cancelled => "cancelled",
            PosterStatus::MiniSeries => "mini_series",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Structure {
    pub seasons: u32,
    pub episodes: u32,
    pub pending: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PosterFacts {
    pub rating: Option<f32>,
    pub status: Option<PosterStatus>,
    pub caption: String,
    pub genre: String,
    pub year: String,
    pub imdb: Option<String>,
    pub tmdb: Option<u64>,
    pub title_key: String,
    pub series: bool,
    pub structure: Structure,
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
    Band,
    Clean,
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
            Some("band") => Placement::Band,
            Some("clean") => Placement::Clean,
            _ => Placement::TopLeft,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PosterOverlays {
    pub rating: Option<Placement>,
    pub status: Option<Placement>,
    pub scale: f32,
    pub labels: Vec<(&'static str, String)>,
    pub language: String,
    pub sources: SashSources,
    pub priority: SashPriority,
    pub sash_style: SashStyle,
    pub notch_look: NotchLook,
    pub meta_order: MetaOrder,
    pub score_scale: ScoreScale,
    pub top_fade: f32,
    pub mute_unreleased: bool,
    pub watched: bool,
    pub progress: bool,
    pub saved: bool,
    pub fade: Fade,
    pub textless: bool,
    pub age_country: Option<String>,
    pub quality: bool,
    pub age: bool,
    pub rating_source: RatingSource,
    pub weights: Weights,
    pub badge_count: usize,
    pub template: Option<String>,
}

impl PosterOverlays {
    pub(super) fn label(&self, slug: &str) -> String {
        self.labels
            .iter()
            .find(|(key, _)| *key == slug)
            .map(|(_, text)| text.clone())
            .unwrap_or_default()
    }
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
                "posterOverlaysEnabled": false,
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
                "posterSashStyle": "diagonal",
                "posterNotchStyle": "frosted",
                "posterSashPriority": "status_first",
                "posterTextless": false,
                "posterAgeCountry": "auto",
                "posterTrendingSource": "week",
                "posterWeights": "balanced",
                "posterRatingBadges": "0",
                "posterMetaOrder": "year_genre_score",
                "posterScoreScale": "ten",
                "posterTopGradient": "off",
                "posterMuteUnreleased": false,
                "posterLifecycleSash": false,
                "posterForeignSash": false,
                "posterMustSeeSash": false,
                "posterQualityBadges": true,
                "posterAgeRating": true,
                "posterAwardBadge": true,
                "posterRatingSource": "imdb",
                "posterTopNumbers": false,
                "posterTopNumbersRows": "trending",
                "posterTopNumbersCount": "10",
                "posterTopNumbersStyle": "outline",
                "posterTopNumbersColor": "white",
                "posterTopNumbersSize": "default",
                "posterTopNumbersPosition": "beside",
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
        let textless = template.is_none() && self.bool_value("posterTextless");
        if !enabled && !textless && template.is_none() {
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
                sashes::label_keys()
                    .map(|(slug, key)| (slug, localized(key, language)))
                    .to_vec()
            },
            language: self.str_value("language").unwrap_or("en").to_owned(),
            sources: SashSources {
                awards: enabled && self.bool_value("posterAwardBadge"),
                trending: enabled && self.bool_value("posterTrendingBadge"),
                lifecycle: enabled && self.bool_value("posterLifecycleSash"),
                foreign: enabled && self.bool_value("posterForeignSash"),
                must_see: enabled && self.bool_value("posterMustSeeSash"),
            },
            priority: SashPriority::parse(self.str_value("posterSashPriority")),
            sash_style: SashStyle::parse(self.str_value("posterSashStyle")),
            notch_look: NotchLook::parse(self.str_value("posterNotchStyle")),
            meta_order: MetaOrder::parse(self.str_value("posterMetaOrder")),
            score_scale: ScoreScale::parse(self.str_value("posterScoreScale")),
            top_fade: match self.str_value("posterTopGradient") {
                Some("low") => 0.4,
                Some("medium") => 0.7,
                Some("high") => 1.0,
                _ => 0.0,
            },
            mute_unreleased: enabled && self.bool_value("posterMuteUnreleased"),
            watched: enabled && self.bool_value("posterWatchedBadge"),
            progress: enabled && self.bool_value("posterProgressBar"),
            saved: enabled && self.bool_value("posterSavedBadge"),
            textless,
            age_country: self
                .str_value("posterAgeCountry")
                .filter(|country| *country != "auto")
                .map(str::to_owned),
            quality: enabled && self.bool_value("posterQualityBadges"),
            age: enabled && self.bool_value("posterAgeRating"),
            rating_source: RatingSource::parse(self.str_value("posterRatingSource")),
            weights: Weights::parse(self.str_value("posterWeights")),
            badge_count: self
                .str_value("posterRatingBadges")
                .and_then(|count| count.parse().ok())
                .unwrap_or(0),
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
    let (rating, rating_logo_name) = meta::resolve_rating(&overlays, facts, graded);
    let shade = match tones.filter(|_| overlays.fade.tint) {
        Some([_, [r, g, b]]) => {
            let dim = |c: u8| (c as f32 * 0.3) as u8;
            [dim(r), dim(g), dim(b)]
        }
        None => [0, 0, 0],
    };
    let vignette = |painter: &Painter| bottom_vignette(painter, rect, radius, overlays.fade, shade);
    let band_tint = tones.filter(|_| overlays.fade.tint).map(|[_, tint]| tint);
    if overlays.mute_unreleased && !facts.series && facts.status == Some(PosterStatus::Upcoming) {
        painter.rect_filled(
            rect,
            radius,
            Color32::from_rgba_unmultiplied(112, 112, 118, 178),
        );
    }
    if overlays.top_fade > 0.0 {
        meta::top_vignette(&painter, rect, overlays.top_fade);
    }
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
        painter.rect_filled(track, 0.0, crate::shade(crate::Shade::Scrim));
        painter.rect_filled(
            Rect::from_min_size(
                track.min,
                Vec2::new(track.width() * personal.progress, height),
            ),
            0.0,
            crate::accent(),
        );
    }
    let trending = enrichment
        .as_deref()
        .and_then(|enrichment| enrichment.trending(facts));
    let mut drop = 0.0;
    if let Some(kind) = sashes::pick(&overlays, facts, graded, trending) {
        let tab = sashes::draw(&painter, rect, &overlays, kind, tones.map(|[_, tint]| tint));
        match (overlays.sash_style, tab) {
            (SashStyle::Diagonal, _) => stacks[Placement::TopRight as usize] = rect.width() * 0.34,
            (SashStyle::Notch | SashStyle::Chip, Some(tab))
                if tab.left() - rect.left() < rect.width() * 0.42 =>
            {
                drop = tab.bottom() - rect.top();
            }
            _ => {}
        }
    }
    let mark = (rect.width() * 0.13).clamp(14.0, 28.0) * overlays.scale;
    let inset = (rect.width() * 0.04).max(4.0);
    let mut corner = Pos2::new(rect.left() + inset, rect.top() + inset + drop);
    if overlays.watched && personal.watched {
        watched_mark(&painter, Rect::from_min_size(corner, Vec2::splat(mark)));
        corner.x += mark + 3.0;
    }
    if overlays.saved && personal.saved {
        saved_mark(&painter, Rect::from_min_size(corner, Vec2::splat(mark)));
        corner.x += mark + 3.0;
    }
    let certification = facts
        .art_key()
        .filter(|_| overlays.age_country.is_some())
        .and_then(|key| enrichment.as_deref()?.certifications.get(&key).cloned())
        .or_else(|| {
            graded
                .and_then(|graded| graded.certification.clone())
                .filter(|_| overlays.age_country.is_none())
        });
    if let Some(certification) = certification.filter(|_| overlays.age) {
        let size = mark * 0.5;
        let galley = painter.layout_no_wrap(
            certification.clone(),
            FontId::proportional(size),
            text_primary(),
        );
        let badge = Rect::from_min_size(
            Pos2::new(corner.x, corner.y + (mark - size * 1.6) * 0.5),
            Vec2::new(galley.size().x + size * 0.7, size * 1.6),
        );
        components::overlay_chip(&painter, badge, Some(ink(tone::subtle())));
        painter.galley(badge.center() - galley.size() * 0.5, galley, text_primary());
        corner.x += badge.width() + 3.0;
    }
    if corner.x > rect.left() + inset {
        stacks[Placement::TopLeft as usize] = mark + 3.0;
    }
    stacks[Placement::TopLeft as usize] += drop;

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
                let meta = meta::meta(facts, rating, &overlays, graded);
                score_minimal(&painter, rect, meta, overlays.scale)
            }
            Some(Placement::Frosted) => score_frosted(
                &painter,
                rect,
                rating,
                &facts.caption,
                overlays.scale,
                radius,
            ),
            Some(Placement::Band) => {
                let meta = meta::meta(facts, rating, &overlays, graded);
                meta::score_band(&painter, rect, meta, rating, &overlays, band_tint, radius)
            }
            Some(Placement::Clean) => {
                vignette(&painter);
                let meta = meta::meta(facts, rating, &overlays, graded);
                meta::score_clean(&painter, rect, meta, &overlays)
            }
            _ => 0.0,
        };
        stacks[Placement::BottomLeft as usize] = rect.height() * reserved;
        stacks[Placement::BottomRight as usize] = rect.height() * reserved;
    }
    if overlays.quality && personal.quality.contains(&true) {
        let size = (rect.width() * 0.06).clamp(8.0, 13.0) * overlays.scale;
        let top = rect.top() + inset + stacks[Placement::TopRight as usize];
        let bottom = quality_badges(&painter, rect.right() - inset, top, size, personal.quality);
        stacks[Placement::TopRight as usize] = bottom - rect.top() - inset;
    }

    let mut badges = Vec::with_capacity(2);
    if let (Some(placement), Some(value)) = (overlays.rating, rating) {
        let logo = rating_logo(painter.ctx(), rating_logo_name);
        let text = match (logo, rating_logo_name) {
            (None, "imdb") => format!("IMDb {value:.1}"),
            _ => format!("{value:.1}"),
        };
        badges.push((placement, text, logo));
    }
    if let (Some(placement), Some(status)) = (overlays.status, facts.status) {
        badges.push((placement, overlays.label(status.slug()), None));
    }
    badges.retain(|(placement, ..)| {
        !matches!(
            placement,
            Placement::Bar
                | Placement::Number
                | Placement::Minimal
                | Placement::Frosted
                | Placement::Band
                | Placement::Clean
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
                crate::shade(crate::Shade::Scrim),
            );
            painter.galley(
                banner.center() - galley.size() * 0.5,
                galley,
                text_primary(),
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
        components::overlay_chip(&painter, badge, Some(ink(tone::raised())));
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
            text_primary(),
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
        components::overlay_chip(&painter, badge, Some(ink(tone::strong())));
        painter.galley(badge.center() - galley.size() * 0.5, galley, text_primary());
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

    #[test]
    fn awards_first_priority_beats_a_new_episode_status() {
        let mut enrichment = Enrichment::default();
        enrichment.add_mdblist(&[serde_json::json!({
            "ids": {"imdb": "tt9"},
            "keywords": [{"name": "emmy-winner"}, {"name": "golden-globe-nominee"}],
        })]);
        let facts = PosterFacts {
            imdb: Some("tt9".into()),
            status: Some(PosterStatus::NewEpisode),
            ..Default::default()
        };
        let graded = enrichment.graded(&facts);
        let mut overlays = test_overlays();
        assert_eq!(
            sashes::pick(&overlays, &facts, graded, None),
            Some(sashes::SashKind::Status(PosterStatus::NewEpisode))
        );
        overlays.priority = SashPriority::AwardsFirst;
        assert_eq!(
            sashes::pick(&overlays, &facts, graded, None),
            Some(sashes::SashKind::Award(Award::Emmy, true))
        );
    }

    #[test]
    fn ended_status_turns_the_last_new_episode_into_a_finale() {
        let mut enrichment = Enrichment::default();
        enrichment.add_mdblist(&[serde_json::json!({"ids": {"imdb": "tt8"}, "status": "Ended"})]);
        let facts = PosterFacts {
            imdb: Some("tt8".into()),
            status: Some(PosterStatus::NewEpisode),
            ..Default::default()
        };
        let overlays = test_overlays();
        assert_eq!(
            sashes::pick(&overlays, &facts, enrichment.graded(&facts), None),
            Some(sashes::SashKind::Status(PosterStatus::Finale))
        );
    }

    #[test]
    fn first_season_started_within_two_weeks_is_a_premiere() {
        let today = day_number("2026-09-26").unwrap();
        let item = json!({ "videos": [
            {"season": 1, "released": "2026-09-20"},
            {"season": 1, "released": "2026-09-27"},
        ]});
        assert_eq!(status(&item, today), Some(PosterStatus::Premiere));
    }

    fn test_overlays() -> PosterOverlays {
        PosterOverlays {
            rating: None,
            status: Some(Placement::Sash),
            scale: 1.0,
            labels: Vec::new(),
            language: "en".into(),
            sources: SashSources {
                awards: true,
                trending: true,
                ..Default::default()
            },
            priority: SashPriority::StatusFirst,
            sash_style: SashStyle::Diagonal,
            notch_look: NotchLook::Frosted,
            meta_order: MetaOrder::YearGenreScore,
            score_scale: ScoreScale::Ten,
            top_fade: 0.0,
            mute_unreleased: false,
            watched: false,
            progress: false,
            saved: false,
            fade: Fade {
                tint: false,
                reach: 0.5,
                alpha: 225.0,
            },
            textless: false,
            age_country: None,
            quality: false,
            age: false,
            rating_source: RatingSource::Imdb,
            weights: Weights::Balanced,
            badge_count: 0,
            template: None,
        }
    }

    #[test]
    fn meta_follows_order_and_scale() {
        let facts = poster_facts(&serde_json::json!({
            "id": "tt1", "genres": ["Sci-Fi"], "releaseInfo": "2025", "imdbRating": "6.6",
        }));
        let mut overlays = test_overlays();
        let line = meta::meta(&facts, facts.rating, &overlays, None);
        assert_eq!(
            (line.prefix.as_str(), line.score.as_deref()),
            ("2025 · Sci-Fi", Some("6.6"))
        );
        overlays.meta_order = MetaOrder::GenreYearScore;
        overlays.score_scale = ScoreScale::Hundred;
        let line = meta::meta(&facts, facts.rating, &overlays, None);
        assert_eq!(
            (line.prefix.as_str(), line.score.as_deref()),
            ("Sci-Fi · 2025", Some("66"))
        );
    }

    #[test]
    fn weighted_score_renormalizes_over_present_sources() {
        let mut enrichment = Enrichment::default();
        enrichment.add_mdblist(&[serde_json::json!({
            "ids": {"imdb": "tt7"},
            "ratings": [
                {"source": "imdb", "value": 8.0, "score": 80},
                {"source": "letterboxd", "value": 4.0, "score": 90},
                {"source": "tmdb", "value": 70, "score": 70},
            ],
        })]);
        let facts = PosterFacts {
            imdb: Some("tt7".into()),
            ..Default::default()
        };
        let graded = enrichment.graded(&facts).unwrap();
        let weighted = graded.weighted(Weights::Balanced).unwrap();
        assert!((weighted - 8.4545).abs() < 0.001);
        assert_eq!(graded.weighted(Weights::Critics), Some(9.0));
    }

    #[test]
    fn rating_badges_follow_logo_order_and_native_scale() {
        let mut enrichment = Enrichment::default();
        enrichment.add_mdblist(&[serde_json::json!({
            "ids": {"imdb": "tt7"},
            "ratings": [
                {"source": "trakt", "score": 87},
                {"source": "letterboxd", "score": 78},
                {"source": "tomatoes", "score": 55},
                {"source": "imdb", "score": 74},
            ],
        })]);
        let facts = PosterFacts {
            imdb: Some("tt7".into()),
            ..Default::default()
        };
        let mut overlays = test_overlays();
        overlays.badge_count = 3;
        let line = meta::meta(&facts, None, &overlays, enrichment.graded(&facts));
        let shown: Vec<_> = line
            .badges
            .iter()
            .map(|badge| (badge.logo, badge.text.as_str()))
            .collect();
        assert_eq!(
            shown,
            [
                ("imdb", "7.4"),
                ("rt-tomato-rotten", "55%"),
                ("letterboxd", "3.9")
            ]
        );
    }
}
