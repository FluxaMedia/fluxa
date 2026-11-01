use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

enum Notice {
    Key(&'static str),
    Verbatim(&'static str),
}

pub(super) struct Credit {
    logo: Option<&'static str>,
    name: &'static str,
    notice: Notice,
}

const CREDITS: [Credit; 8] = [
    Credit {
        logo: None,
        name: "Fluxa",
        notice: Notice::Key("settings.about.credit.fluxa"),
    },
    Credit {
        logo: Some("tmdb"),
        name: "TMDB",
        notice: Notice::Verbatim(
            "This product uses the TMDB API but is not endorsed or certified by TMDB.",
        ),
    },
    Credit {
        logo: None,
        name: "mpv / FFmpeg / libass",
        notice: Notice::Key("settings.about.credit.playback"),
    },
    Credit {
        logo: None,
        name: "Phosphor Icons",
        notice: Notice::Key("settings.about.credit.phosphor"),
    },
    Credit {
        logo: None,
        name: "Archivo / Noto Sans",
        notice: Notice::Key("settings.about.credit.fonts"),
    },
    Credit {
        logo: None,
        name: "Anime4K",
        notice: Notice::Key("settings.about.credit.anime4k"),
    },
    Credit {
        logo: None,
        name: "epaint",
        notice: Notice::Key("settings.about.credit.epaint"),
    },
    Credit {
        logo: None,
        name: "librqbit-dualstack-sockets",
        notice: Notice::Key("settings.about.credit.librqbit"),
    },
];

fn notice(credit: &Credit, language: &str) -> String {
    match credit.notice {
        Notice::Key(key) => localized(key, language),
        Notice::Verbatim(text) => text.to_string(),
    }
}

fn name_size(compact: bool, metrics: UiMetrics) -> f32 {
    if compact {
        metrics.screen_body_size
    } else {
        metrics.settings_row_label_size_desktop
    }
}

static MEASURED: AtomicU32 = AtomicU32::new(0);

pub(super) fn credits_card_height(
    width: f32,
    compact: bool,
    language: &str,
    metrics: UiMetrics,
) -> f32 {
    let measured = f32::from_bits(MEASURED.load(Ordering::Relaxed));
    if measured > 0.0 {
        return measured;
    }
    let size = metrics.text.meta;
    let per_line = ((width - metrics.settings_row_inset * 2.0) / (size * 0.5))
        .floor()
        .max(8.0);
    CREDITS
        .iter()
        .map(|credit| {
            let lines = (notice(credit, language).chars().count() as f32 / per_line).ceil();
            metrics.settings_row_padding * 2.0
                + name_size(compact, metrics) * 1.3
                + metrics.settings_name_gap
                + lines * size * 1.35
        })
        .sum()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_credits(
    painter: &egui::Painter,
    assets: &impl HomeAssets,
    language: &str,
    rect: Rect,
    top: f32,
    compact: bool,
    metrics: UiMetrics,
) {
    let width = rect.width() - metrics.settings_row_inset * 2.0 - 8.0;
    let name_font = crate::fonts::medium(name_size(compact, metrics));
    let rows: Vec<_> = CREDITS
        .iter()
        .map(|credit| {
            let name = painter.layout_no_wrap(
                credit.name.to_owned(),
                name_font.clone(),
                metrics.text_primary,
            );
            let text = painter.layout_job(egui::text::LayoutJob::simple(
                notice(credit, language),
                crate::fonts::regular(metrics.text.meta),
                metrics.text_secondary,
                width,
            ));
            (credit, name, text)
        })
        .collect();
    let height = rows
        .iter()
        .map(|(_, name, text)| {
            metrics.settings_row_padding * 2.0
                + name.size().y
                + metrics.settings_name_gap
                + text.size().y
        })
        .sum::<f32>();
    MEASURED.store(height.to_bits(), Ordering::Relaxed);
    let card = account_group(
        painter,
        rect,
        top,
        height,
        &localized("settings.about.credits", language),
        metrics,
    );
    let left = card.left() + metrics.settings_row_inset + 4.0;
    let right = card.right() - metrics.settings_row_inset - 4.0;
    let mut y = card.top();
    for (index, (credit, name, text)) in rows.into_iter().enumerate() {
        if index > 0 {
            painter.line_segment(
                [Pos2::new(left, y), Pos2::new(right, y)],
                egui::Stroke::new(stroke::hairline(), ink(tone::faint())),
            );
        }
        let name_rect = Rect::from_min_size(
            Pos2::new(left, y + metrics.settings_row_padding),
            name.size(),
        );
        painter.galley(name_rect.min, name, metrics.text_primary);
        if let Some((texture, size)) = credit.logo.and_then(|logo| assets.logo(logo)) {
            let logo = Rect::from_min_size(
                Pos2::new(
                    right - metrics.settings_logo_height * size.x,
                    name_rect.center().y - metrics.settings_logo_height * 0.5,
                ),
                Vec2::new(
                    metrics.settings_logo_height * size.x,
                    metrics.settings_logo_height,
                ),
            );
            painter.image(
                texture,
                logo,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        let text_top = name_rect.bottom() + metrics.settings_name_gap;
        y = text_top + text.size().y + metrics.settings_row_padding;
        painter.galley(Pos2::new(left, text_top), text, metrics.text_secondary);
    }
}
