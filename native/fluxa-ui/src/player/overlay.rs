use super::*;
use crate::dp::Dp;
use crate::flex;

const THUMB: usize = 1;
const LABEL: usize = 2;
const TITLE: usize = 3;
const PLAY: usize = 4;
const DISMISS: usize = 5;
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
    fn tint(self) -> Color32 {
        match self {
            SkipKind::Intro => crate::stylesheet::color("text.body"),
            SkipKind::Recap => crate::stylesheet::color("text.subtle"),
            SkipKind::Outro => crate::stylesheet::color("dim"),
        }
    }

    fn label_key(self) -> &'static str {
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

pub(super) fn segmented_track(
    painter: &crate::gfx::Painter,
    track: Rect,
    thickness: f32,
    spans: &[ChapterSpan],
    played: f32,
    buffered: f32,
) {
    let whole = [ChapterSpan {
        start_fraction: 0.0,
        end_fraction: 1.0,
        title: String::new(),
    }];
    let spans = if spans.len() > 1 { spans } else { &whole };
    let metrics = crate::metrics::active();
    for (index, span) in spans.iter().enumerate() {
        let left = track.left() + track.width() * span.start_fraction;
        let right = track.left() + track.width() * span.end_fraction;
        let trailing = if index + 1 < spans.len() {
            Dp::D3.px()
        } else {
            0.0
        };
        let width = (right - left - trailing).max(1.0);
        painter_bar_at(
            painter,
            track,
            left,
            width,
            thickness,
            crate::stylesheet::color("dim"),
        );
        let ahead = (buffered - left + track.left()).clamp(0.0, width);
        if ahead > 0.0 {
            painter_bar_at(
                painter,
                track,
                left,
                ahead,
                thickness,
                accent().gamma_multiply(0.4),
            );
        }
        let filled = (played - left + track.left()).clamp(0.0, width);
        if filled > 0.0 {
            painter_bar_at(
                painter,
                track,
                left,
                filled,
                thickness + metrics.playerctl_track_boost,
                accent(),
            );
        }
    }
}

pub(super) fn segment_tints(
    painter: &crate::gfx::Painter,
    track: Rect,
    thickness: f32,
    segments: &[SegmentSpan],
) {
    let boost = crate::metrics::active().playerctl_track_boost;
    for segment in segments {
        let left = track.left() + track.width() * segment.start_fraction;
        let right = track.left() + track.width() * segment.end_fraction;
        if right > left {
            painter_bar_at(
                painter,
                track,
                left,
                right - left,
                thickness + boost,
                segment.kind.tint().gamma_multiply(0.85),
            );
        }
    }
}

fn painter_bar_at(
    painter: &crate::gfx::Painter,
    track: Rect,
    left: f32,
    width: f32,
    thickness: f32,
    color: Color32,
) {
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(left, track.center().y - thickness * 0.5),
            Vec2::new(width, thickness),
        ),
        thickness * 0.5,
        color,
    );
}

fn card_bottom(chrome: &Chrome) -> f32 {
    let metrics = UiMetrics::for_viewport(chrome.viewport);
    let controls_top = (chrome.player.controls_visible && !chrome.player.locked)
        .then(|| {
            chrome
                .context
                .data(|data| data.get_temp::<f32>(super::tokens::controls_top_id()))
        })
        .flatten();
    match controls_top {
        Some(top) => top - Space::S8.px(&metrics),
        None => chrome.rect.bottom() - Space::S32.px(&metrics) - chrome.viewport.safe_bottom,
    }
}

fn card_right(chrome: &Chrome) -> f32 {
    chrome.rect.right() - chrome.margin()
}

pub(super) fn pill(
    ui: &mut crate::gfx::Ui,
    layout: &mut HomeLayout,
    rect: Rect,
    label: &str,
    size: f32,
    primary: bool,
    node: u64,
) {
    let response = ui
        .scope_builder(crate::gfx::UiBuilder::new().max_rect(rect), |ui| {
            crate::components::pill_button(
                ui,
                None,
                label,
                Some(rect.width()),
                rect.height(),
                size,
                primary,
                None,
            )
        })
        .inner;
    crate::components::focus_action(layout, node, &response);
}

pub(super) fn draw_skip_card(chrome: &Chrome, ui: &mut crate::gfx::Ui, layout: &mut HomeLayout) {
    let Some(card) = chrome.player.skip.as_ref() else {
        return;
    };
    let metrics = super::tokens::PlayerTokens::for_form_factor(chrome.viewport.form_factor);
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let text = tokens.text;
    let label = localized(card.kind.label_key(), &chrome.player.language);
    let label_width = flex::text_size(chrome.painter, &label, &FontId::proportional(text.body)).x;
    let width = (label_width + Dp::D96.px()).max(tokens.playerctl_skip_min_width);
    let area = Rect::from_min_max(
        chrome.rect.min,
        Pos2::new(card_right(chrome), card_bottom(chrome)),
    );
    let rect = flex::anchored(
        area,
        Align2::RIGHT_BOTTOM,
        Vec2::new(width, metrics.pill),
        0.0,
    );
    let response = ui
        .scope_builder(crate::gfx::UiBuilder::new().max_rect(rect), |ui| {
            crate::components::progress_pill(ui, rect, &label, text.body, card.progress)
        })
        .inner;
    components::focus_action(layout, NODE_PLAYER_SKIP, &response);
}

pub(super) fn draw_next_episode_card(
    chrome: &Chrome,
    ui: &mut crate::gfx::Ui,
    layout: &mut HomeLayout,
    thumbnail: Option<TextureId>,
) {
    let Some(card) = chrome.player.next_episode.as_ref() else {
        return;
    };
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let metrics = super::tokens::PlayerTokens::for_form_factor(chrome.viewport.form_factor);
    let language = &chrome.player.language;
    let pad = Space::S16.px(&tokens);
    let width = tokens
        .player_panel_card
        .of(chrome.rect.width())
        .min(chrome.rect.width() - chrome.margin() * 2.0);
    let label_font = FontId::proportional(tokens.text.body);
    let title_font = FontId::proportional(tokens.text.subtitle);
    let label = localized("player.next_label", language).replace("%s", &card.label);
    let play_label = match card.countdown {
        Some(seconds) => {
            localized("player.playing_in_seconds", language).replace("%s", &seconds.to_string())
        }
        None => localized("player.play", language),
    };
    let image_height =
        thumbnail.map_or(0.0, |_| (width - pad * 2.0) * tokens.playerctl_thumb_aspect);
    let mut rows = Vec::new();
    if thumbnail.is_some() {
        rows.push(flex::leaf(Vec2::new(0.0, image_height)).id(THUMB));
    }
    rows.push(
        flex::leaf(Vec2::new(
            0.0,
            flex::text_size(chrome.painter, &label, &label_font).y,
        ))
        .id(LABEL),
    );
    rows.push(
        flex::leaf(Vec2::new(
            0.0,
            flex::text_size(chrome.painter, &card.title, &title_font).y,
        ))
        .id(TITLE),
    );
    let buttons = flex::row([
        flex::fill()
            .grow(tokens.playerctl_next_play_weight)
            .height(metrics.pill)
            .id(PLAY),
        flex::fill().height(metrics.pill).id(DISMISS),
    ])
    .gap(Space::S8.px(&tokens));
    let spacer = flex::slot(0.0).height(Space::S12.px(&tokens));
    let tree = flex::column(rows.into_iter().chain([spacer, buttons]))
        .align(flex::Align::Stretch)
        .padding(flex::Insets::all(pad))
        .width(width);
    let size = tree.outer_size();
    let panel = flex::anchored(
        Rect::from_min_max(
            chrome.rect.min,
            Pos2::new(card_right(chrome), card_bottom(chrome)),
        ),
        Align2::RIGHT_BOTTOM,
        size,
        0.0,
    );
    let placed = tree.solve(panel);
    chrome
        .painter
        .rect_filled(panel, tokens.card_radius, tokens.surface);
    chrome.painter.rect_stroke(
        panel,
        tokens.card_radius,
        crate::gfx::Stroke::new(stroke::hairline(), tokens.border),
        crate::gfx::StrokeKind::Inside,
    );
    if let (Some(texture), Some(frame)) = (thumbnail, placed.find(THUMB)) {
        chrome
            .painter
            .image(texture, frame, full_uv(), Color32::WHITE);
    }
    if let Some(slot) = placed.find(LABEL) {
        chrome.text(
            slot.left_top(),
            Align2::LEFT_TOP,
            &label,
            tokens.text.body,
            slot.width(),
            170,
        );
    }
    if let Some(slot) = placed.find(TITLE) {
        chrome.text(
            slot.left_top(),
            Align2::LEFT_TOP,
            &card.title,
            tokens.text.subtitle,
            slot.width(),
            255,
        );
    }
    if let Some(play) = placed.find(PLAY) {
        pill(
            ui,
            layout,
            play,
            &play_label,
            tokens.text.body,
            true,
            NODE_PLAYER_NEXT_PLAY,
        );
    }
    if let Some(dismiss) = placed.find(DISMISS) {
        pill(
            ui,
            layout,
            dismiss,
            &localized("common.cancel", language),
            tokens.text.body,
            false,
            NODE_PLAYER_NEXT_DISMISS,
        );
    }
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
        flex::column([flex::fill(), flex::leaf(*size).id(CHIP_BASE + index)]).height(height)
    });
    let strip = flex::row(cells).gap(gap);
    let bounds = Rect::from_min_size(
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
