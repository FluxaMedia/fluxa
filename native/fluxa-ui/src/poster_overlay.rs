use egui::{Align2, Color32, FontId, Id, Painter, Pos2, Rect, Vec2};
use std::sync::OnceLock;

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Banner,
}

impl Placement {
    fn parse(value: Option<&str>) -> Self {
        match value {
            Some("top_right") => Placement::TopRight,
            Some("bottom_left") => Placement::BottomLeft,
            Some("bottom_right") => Placement::BottomRight,
            Some("banner") => Placement::Banner,
            _ => Placement::TopLeft,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PosterOverlays {
    pub rating: Option<Placement>,
    pub status: Option<Placement>,
    pub scale: f32,
    pub language: String,
}

pub(super) fn setting_default(key: &str) -> Option<&'static serde_json::Value> {
    static DEFAULTS: OnceLock<serde_json::Value> = OnceLock::new();
    DEFAULTS
        .get_or_init(|| {
            serde_json::json!({
                "posterOverlaysEnabled": true,
                "posterRatingBadge": true,
                "posterRatingPosition": "top_left",
                "posterStatusBadge": true,
                "posterStatusPosition": "banner",
                "posterBadgeSize": "default",
            })
        })
        .get(key)
}

impl super::SettingsModel {
    pub fn poster_overlays(&self) -> Option<PosterOverlays> {
        if !self.bool_value("posterOverlaysEnabled") {
            return None;
        }
        let placement = |toggle: &str, position: &str| {
            self.bool_value(toggle)
                .then(|| Placement::parse(self.str_value(position)))
        };
        Some(PosterOverlays {
            rating: placement("posterRatingBadge", "posterRatingPosition"),
            status: placement("posterStatusBadge", "posterStatusPosition"),
            scale: match self.str_value("posterBadgeSize") {
                Some("small") => 0.8,
                Some("large") => 1.3,
                _ => 1.0,
            },
            language: self.str_value("language").unwrap_or("en").to_owned(),
        })
    }
}

fn overlays_id() -> Id {
    Id::new("fluxa-poster-overlays")
}

pub fn set_poster_overlays(context: &egui::Context, overlays: Option<PosterOverlays>) {
    context.data_mut(|data| match overlays {
        Some(overlays) => {
            data.insert_temp(overlays_id(), overlays);
        }
        None => data.remove::<PosterOverlays>(overlays_id()),
    });
}

pub(super) fn poster_facts(item: &serde_json::Value) -> PosterFacts {
    PosterFacts {
        rating: rating(item),
        status: status(item, today()),
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
    let Some(overlays) = painter
        .ctx()
        .data(|data| data.get_temp::<PosterOverlays>(overlays_id()))
    else {
        return;
    };
    let font_size = (rect.width() * 0.085).clamp(9.0, 15.0) * overlays.scale;
    let font = FontId::proportional(font_size);
    let pad = Vec2::new(font_size * 0.45, font_size * 0.2);
    let inset = (rect.width() * 0.04).max(4.0);
    let mut stacks = [0.0f32; 4];

    let mut badges = Vec::with_capacity(2);
    if let (Some(placement), Some(value)) = (overlays.rating, card.overlay.rating) {
        badges.push((placement, format!("IMDb {value:.1}")));
    }
    if let (Some(placement), Some(status)) = (overlays.status, card.overlay.status) {
        badges.push((placement, localized(status.key(), &overlays.language)));
    }
    badges.sort_by_key(|(placement, _)| *placement != Placement::Banner);

    for (placement, text) in badges {
        let galley = painter.layout_no_wrap(text, font.clone(), Color32::WHITE);
        if placement == Placement::Banner {
            let height = galley.size().y + pad.y * 4.0;
            let banner =
                Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - height), rect.max);
            painter.rect_filled(
                banner,
                egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: radius as u8,
                    se: radius as u8,
                },
                Color32::from_black_alpha(190),
            );
            painter.galley(
                banner.center() - galley.size() * 0.5,
                galley,
                Color32::WHITE,
            );
            stacks[Placement::BottomLeft as usize] = height;
            stacks[Placement::BottomRight as usize] = height;
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
            egui::Stroke::new(1.0, Color32::from_white_alpha(28)),
            egui::StrokeKind::Inside,
        );
        painter.galley(badge.min + pad, galley, Color32::WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
