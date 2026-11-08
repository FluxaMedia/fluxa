use super::*;

const SEGMENT_GAP: f32 = 3.0;

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
    painter: &egui::Painter,
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
    for (index, span) in spans.iter().enumerate() {
        let left = track.left() + track.width() * span.start_fraction;
        let right = track.left() + track.width() * span.end_fraction;
        let trailing = if index + 1 < spans.len() {
            SEGMENT_GAP
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
            painter_bar_at(painter, track, left, filled, thickness + 0.5, accent());
        }
    }
}

pub(super) fn segment_tints(
    painter: &egui::Painter,
    track: Rect,
    thickness: f32,
    segments: &[SegmentSpan],
) {
    for segment in segments {
        let left = track.left() + track.width() * segment.start_fraction;
        let right = track.left() + track.width() * segment.end_fraction;
        if right > left {
            painter_bar_at(
                painter,
                track,
                left,
                right - left,
                thickness + 0.5,
                segment.kind.tint().gamma_multiply(0.85),
            );
        }
    }
}

fn painter_bar_at(
    painter: &egui::Painter,
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
    let tokens = super::tokens::PlayerTokens::for_form_factor(chrome.viewport.form_factor);
    let lift = if chrome.player.controls_visible {
        tokens.card_lift_visible
    } else {
        tokens.card_lift_hidden
    };
    chrome.rect.bottom() - lift - chrome.viewport.safe_bottom
}

fn card_right(chrome: &Chrome) -> f32 {
    chrome.rect.right() - chrome.margin()
}

pub(super) fn pill(
    ui: &mut egui::Ui,
    layout: &mut HomeLayout,
    rect: Rect,
    label: &str,
    size: f32,
    primary: bool,
    node: u64,
) {
    let response = ui
        .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
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

pub(super) fn draw_skip_card(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let Some(card) = chrome.player.skip.as_ref() else {
        return;
    };
    let metrics = super::tokens::PlayerTokens::for_form_factor(chrome.viewport.form_factor);
    let label = localized(card.kind.label_key(), &chrome.player.language);
    let width = (label.chars().count() as f32 * metrics.text * 0.62 + 84.0).max(150.0);
    let mut row = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_size(
                Pos2::new(
                    card_right(chrome) - chrome.rect.width(),
                    card_bottom(chrome) - metrics.pill,
                ),
                Vec2::new(chrome.rect.width(), metrics.pill),
            ))
            .layout(egui::Layout::right_to_left(egui::Align::Min)),
    );
    let rect = row.allocate_space(Vec2::new(width, metrics.pill)).1;
    let response = ui
        .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            crate::components::progress_pill(ui, rect, &label, metrics.text, card.progress)
        })
        .inner;
    components::focus_action(layout, NODE_PLAYER_SKIP, &response);
}

pub(super) fn draw_next_episode_card(
    chrome: &Chrome,
    ui: &mut egui::Ui,
    layout: &mut HomeLayout,
    thumbnail: Option<TextureId>,
) {
    let Some(card) = chrome.player.next_episode.as_ref() else {
        return;
    };
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let metrics = super::tokens::PlayerTokens::for_form_factor(chrome.viewport.form_factor);
    let language = &chrome.player.language;
    let width = metrics
        .card_width
        .min(chrome.rect.width() - chrome.margin() * 2.0);
    let image_height = thumbnail.map_or(0.0, |_| (width - metrics.pad * 2.0) * 9.0 / 16.0);
    let text_height = metrics.title * 2.9;
    let height = metrics.pad * 3.0 + image_height + text_height + metrics.pill;
    let panel = Rect::from_min_size(
        Pos2::new(card_right(chrome) - width, card_bottom(chrome) - height),
        Vec2::new(width, height),
    );
    chrome
        .painter
        .rect_filled(panel, tokens.card_radius, tokens.surface);
    chrome.painter.rect_stroke(
        panel,
        tokens.card_radius,
        egui::Stroke::new(stroke::hairline(), tokens.border),
        egui::StrokeKind::Inside,
    );
    let inner = panel.shrink(metrics.pad);
    let mut column = ui.new_child(egui::UiBuilder::new().max_rect(inner));
    column.spacing_mut().item_spacing = Vec2::ZERO;
    if let Some(texture) = thumbnail {
        let frame = column
            .allocate_space(Vec2::new(inner.width(), image_height))
            .1;
        chrome
            .painter
            .image(texture, frame, full_uv(), Color32::WHITE);
        column.add_space(metrics.pad * 0.6);
    }
    let label_slot = column
        .allocate_space(Vec2::new(inner.width(), metrics.text * 1.5))
        .1;
    chrome.text(
        label_slot.min,
        Align2::LEFT_TOP,
        &localized("player.next_label", language).replace("%s", &card.label),
        metrics.text,
        inner.width(),
        170,
    );
    let title_slot = column
        .allocate_space(Vec2::new(inner.width(), metrics.title * 1.4))
        .1;
    chrome.text(
        title_slot.min,
        Align2::LEFT_TOP,
        &card.title,
        metrics.title,
        inner.width(),
        255,
    );
    column.add_space((inner.bottom() - metrics.pill - column.cursor().min.y).max(0.0));
    let play_label = match card.countdown {
        Some(seconds) => {
            localized("player.playing_in_seconds", language).replace("%s", &seconds.to_string())
        }
        None => localized("player.play", language),
    };
    let play_width = (inner.width() * 0.62).min(inner.width() - 96.0);
    let mut buttons = column.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_size(
                column.cursor().min,
                Vec2::new(inner.width(), metrics.pill),
            ))
            .layout(egui::Layout::left_to_right(egui::Align::Min)),
    );
    buttons.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);
    let play = buttons
        .allocate_space(Vec2::new(play_width, metrics.pill))
        .1;
    pill(
        ui,
        layout,
        play,
        &play_label,
        metrics.text,
        true,
        NODE_PLAYER_NEXT_PLAY,
    );
    let dismiss = buttons
        .allocate_space(Vec2::new(buttons.available_width(), metrics.pill))
        .1;
    pill(
        ui,
        layout,
        dismiss,
        &localized("common.cancel", language),
        metrics.text,
        false,
        NODE_PLAYER_NEXT_DISMISS,
    );
}

pub(super) fn draw_playing_badges(
    chrome: &Chrome,
    ui: &mut egui::Ui,
    assets: &mut impl HomeAssets,
) {
    let Some(track) = chrome.seek_track.get() else {
        return;
    };
    if chrome.player.playing_badges.is_empty() {
        return;
    }
    let font_size = match chrome.viewport.form_factor {
        UiFormFactor::Tv => 18.0,
        UiFormFactor::Mobile => 11.5,
        UiFormFactor::Desktop => 12.5,
    };
    let chips = chrome
        .player
        .playing_badges
        .iter()
        .map(|badge| {
            (
                badge,
                components::badge_chip_layout(chrome.painter, badge, font_size),
            )
        })
        .collect::<Vec<_>>();
    let height = chips
        .iter()
        .map(|(_, (_, size))| size.y)
        .fold(0.0, f32::max);
    let mut strip = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_size(
                Pos2::new(track.left(), track.top() - 18.0 - height),
                Vec2::new(track.width(), height),
            ))
            .layout(egui::Layout::left_to_right(egui::Align::Max)),
    );
    strip.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);
    for (badge, (galley, size)) in chips {
        if size.x > strip.available_width() {
            break;
        }
        let chip = strip.allocate_space(size).1;
        components::paint_badge_chip_rounded(chrome.painter, chip, galley, badge, assets, 4.0);
    }
}
