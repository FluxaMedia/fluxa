use super::*;

const SEGMENT_GAP: f32 = 3.0;
const CARD_FILL: Color32 = crate::theme::SURFACE;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChapterSpan {
    pub start_fraction: f32,
    pub end_fraction: f32,
    pub title: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkipKind {
    Intro,
    Recap,
    Outro,
}

impl SkipKind {
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
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NextEpisodeCard {
    pub label: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub countdown: Option<u32>,
}

struct CardMetrics {
    width: f32,
    pill: f32,
    text: f32,
    title: f32,
    pad: f32,
}

impl CardMetrics {
    fn for_form_factor(form_factor: UiFormFactor) -> Self {
        match form_factor {
            UiFormFactor::Tv => Self {
                width: 500.0,
                pill: 60.0,
                text: 22.0,
                title: 24.0,
                pad: 22.0,
            },
            UiFormFactor::Mobile => Self {
                width: 300.0,
                pill: 40.0,
                text: 14.0,
                title: 15.0,
                pad: 14.0,
            },
            UiFormFactor::Desktop => Self {
                width: 380.0,
                pill: 44.0,
                text: 15.0,
                title: 17.0,
                pad: 16.0,
            },
        }
    }
}

pub(super) fn segmented_track(
    painter: &egui::Painter,
    track: Rect,
    thickness: f32,
    spans: &[ChapterSpan],
    played: f32,
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
            Color32::from_white_alpha(95),
        );
        let filled = (played - left + track.left()).clamp(0.0, width);
        if filled > 0.0 {
            painter_bar_at(
                painter,
                track,
                left,
                filled,
                thickness + 0.5,
                Color32::WHITE,
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
    let (rect, safe) = (chrome.rect, chrome.viewport.safe_bottom);
    let visible = chrome.player.controls_visible;
    match chrome.viewport.form_factor {
        UiFormFactor::Desktop if visible => rect.bottom() - 93.0 - safe - 26.0,
        UiFormFactor::Mobile if visible => rect.bottom() - 44.0 - safe - 40.0,
        UiFormFactor::Tv if visible => rect.bottom() - 64.0 - safe - 72.0 - 70.0,
        UiFormFactor::Tv => rect.bottom() - 64.0 - safe,
        _ => rect.bottom() - 32.0 - safe,
    }
}

fn card_right(chrome: &Chrome) -> f32 {
    let margin = match chrome.viewport.form_factor {
        UiFormFactor::Tv => (chrome.rect.width() * 0.05).max(48.0),
        _ => chrome.margin(),
    };
    chrome.rect.right() - margin
}

fn pill(
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
    layout.focusable.push((node, response.rect));
    if response.clicked() {
        layout.activated = Some(node);
    }
}

pub(super) fn draw_skip_card(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let Some(card) = chrome.player.skip.as_ref() else {
        return;
    };
    let metrics = CardMetrics::for_form_factor(chrome.viewport.form_factor);
    let label = localized(card.kind.label_key(), &chrome.player.language);
    let width = (label.chars().count() as f32 * metrics.text * 0.62 + 84.0).max(150.0);
    let right = card_right(chrome);
    let rect = Rect::from_min_size(
        Pos2::new(right - width, card_bottom(chrome) - metrics.pill),
        Vec2::new(width, metrics.pill),
    );
    pill(
        ui,
        layout,
        rect,
        &label,
        metrics.text,
        true,
        NODE_PLAYER_SKIP,
    );
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
    let metrics = CardMetrics::for_form_factor(chrome.viewport.form_factor);
    let language = &chrome.player.language;
    let width = metrics
        .width
        .min(chrome.rect.width() - chrome.margin() * 2.0);
    let image_height = thumbnail.map_or(0.0, |_| (width - metrics.pad * 2.0) * 9.0 / 16.0);
    let text_height = metrics.title * 2.9;
    let height = metrics.pad * 3.0 + image_height + text_height + metrics.pill;
    let panel = Rect::from_min_size(
        Pos2::new(card_right(chrome) - width, card_bottom(chrome) - height),
        Vec2::new(width, height),
    );
    chrome.painter.rect_filled(panel, 14.0, CARD_FILL);
    chrome.painter.rect_stroke(
        panel,
        14.0,
        crate::theme::border(),
        egui::StrokeKind::Inside,
    );
    let inner = panel.shrink(metrics.pad);
    let mut y = inner.top();
    if let Some(texture) = thumbnail {
        let frame = Rect::from_min_size(inner.min, Vec2::new(inner.width(), image_height));
        chrome
            .painter
            .image(texture, frame, full_uv(), Color32::WHITE);
        y = frame.bottom() + metrics.pad * 0.6;
    }
    chrome.text(
        Pos2::new(inner.left(), y),
        Align2::LEFT_TOP,
        &localized("player.next_label", language).replace("%s", &card.label),
        metrics.text,
        inner.width(),
        170,
    );
    chrome.text(
        Pos2::new(inner.left(), y + metrics.text * 1.5),
        Align2::LEFT_TOP,
        &card.title,
        metrics.title,
        inner.width(),
        255,
    );
    let buttons_top = inner.bottom() - metrics.pill;
    let play_label = match card.countdown {
        Some(seconds) => {
            localized("player.playing_in_seconds", language).replace("%s", &seconds.to_string())
        }
        None => localized("player.play", language),
    };
    let play_width = (inner.width() * 0.62).min(inner.width() - 96.0);
    let play = Rect::from_min_size(
        Pos2::new(inner.left(), buttons_top),
        Vec2::new(play_width, metrics.pill),
    );
    pill(
        ui,
        layout,
        play,
        &play_label,
        metrics.text,
        true,
        NODE_PLAYER_NEXT_PLAY,
    );
    let dismiss = Rect::from_min_max(
        Pos2::new(play.right() + 8.0, buttons_top),
        Pos2::new(inner.right(), inner.bottom()),
    );
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
