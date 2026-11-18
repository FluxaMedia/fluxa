use super::*;
use crate::dp::Dp;
use crate::flex;

const CHIP_BASE: usize = 0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChapterSpan {
    pub start_fraction: f32,
    pub end_fraction: f32,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SegmentSpan {
    pub start_fraction: f32,
    pub end_fraction: f32,
    pub kind: SkipKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkipKind {
    Intro,
    Recap,
    Outro,
}

impl SkipKind {
    pub(crate) fn band(self) -> &'static str {
        match self {
            SkipKind::Intro => "#e6bf40",
            SkipKind::Recap => "#6fa8dc",
            SkipKind::Outro => "#c27ba0",
        }
    }

    pub(crate) fn label_key(self) -> &'static str {
        match self {
            SkipKind::Intro => "player.skip_intro",
            SkipKind::Recap => "player.skip_recap",
            SkipKind::Outro => "player.skip_outro",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkipCard {
    pub kind: SkipKind,
    pub seek_to: f64,
    pub progress: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NextEpisodeCard {
    pub label: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub countdown: Option<u32>,
}

pub(super) fn draw_playing_badges(
    chrome: &Chrome,
    _ui: &mut crate::gfx::Ui,
    assets: &mut impl HomeAssets,
) {
    let Some(track) = chrome.seek_track.get() else {
        return;
    };
    if chrome.player.playing_badges.is_empty() {
        return;
    }
    let metrics = UiMetrics::for_viewport(chrome.viewport);
    let gap = Space::S6.px(&metrics);
    let chips = chrome
        .player
        .playing_badges
        .iter()
        .map(|badge| {
            (
                badge,
                components::badge_chip_layout(chrome.painter, badge, metrics.playerctl_badge_font),
            )
        })
        .collect::<Vec<_>>();
    let height = chips
        .iter()
        .map(|(_, (_, size))| size.y)
        .fold(0.0, f32::max);
    let mut used = 0.0;
    let mut shown = Vec::new();
    for (badge, (galley, size)) in chips {
        let next = if shown.is_empty() {
            size.x
        } else {
            used + gap + size.x
        };
        if next > track.width() {
            break;
        }
        used = next;
        shown.push((badge, galley, size));
    }
    let cells = shown.iter().enumerate().map(|(index, (_, _, size))| {
        flex::column([flex::fill(), flex::leaf(*size).id(CHIP_BASE + index)])
            .height(height)
            .align(crate::flex::Align::Start)
    });
    let strip = flex::row(cells).gap(gap).align(crate::flex::Align::Start);
    let bounds = crate::gfx::rect_at(
        Pos2::new(track.left(), track.top() - Dp::D16.px() - height),
        Vec2::new(track.width(), height),
    );
    let placed = strip.solve(bounds);
    for (index, (badge, galley, _)) in shown.into_iter().enumerate() {
        if let Some(chip) = placed.find(CHIP_BASE + index) {
            components::paint_badge_chip_rounded(
                chrome.painter,
                chip,
                galley,
                badge,
                assets,
                Space::S4.px(&metrics),
            );
        }
    }
}
