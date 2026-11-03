use super::*;
use crate::navigation::nav_text;
use std::sync::Mutex;

static STATE: Mutex<UpdateState> = Mutex::new(UpdateState::Idle);

pub fn set_nav_update(state: &UpdateState) {
    if let Ok(mut current) = STATE.lock()
        && *current != *state
    {
        *current = state.clone();
    }
}

pub(crate) struct Status {
    pub title: String,
    pub detail: String,
    pub progress: Option<f32>,
    pub icon: &'static str,
}

pub(crate) fn nav_status() -> Option<Status> {
    let state = STATE.lock().ok()?.clone();
    Some(match state {
        UpdateState::Available(version) => Status {
            title: nav_text("settings.updates.available"),
            detail: format!("v{version}"),
            progress: None,
            icon: "Downloads",
        },
        UpdateState::Downloading(percent) => Status {
            title: nav_text("update.downloading"),
            detail: format!("{percent}%"),
            progress: Some(f32::from(percent) / 100.0),
            icon: "Downloads",
        },
        UpdateState::Ready(version) => Status {
            title: nav_text("settings.updates.restart"),
            detail: format!("v{version}"),
            progress: None,
            icon: "Check",
        },
        UpdateState::Failed(_) => Status {
            title: nav_text("settings.updates.failed"),
            detail: nav_text("common.retry"),
            progress: None,
            icon: "Refresh",
        },
        _ => return None,
    })
}

pub(crate) fn progress_ring(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    fraction: f32,
    color: Color32,
) {
    let width = 2.0;
    painter.circle_stroke(center, radius, egui::Stroke::new(width, ink(tone::soft())));
    let steps = 48;
    let sweep = std::f32::consts::TAU * fraction.clamp(0.0, 1.0);
    if sweep <= 0.0 {
        return;
    }
    let points = (0..=steps)
        .map(|step| {
            let angle = -std::f32::consts::FRAC_PI_2 + sweep * step as f32 / steps as f32;
            center + Vec2::new(angle.cos(), angle.sin()) * radius
        })
        .collect();
    painter.add(egui::Shape::line(points, egui::Stroke::new(width, color)));
}

pub(crate) fn badge_rect(avatar: Pos2, radius: f32) -> Rect {
    Rect::from_center_size(
        avatar + Vec2::new(radius * 0.85, -radius * 0.85),
        Vec2::splat(30.0),
    )
}

pub(crate) fn draw_badge(
    painter: &egui::Painter,
    assets: &impl HomeAssets,
    status: &Status,
    avatar: Pos2,
    radius: f32,
) {
    let center = badge_rect(avatar, radius).center();
    painter.circle_filled(center, 10.0, overlay_panel());
    painter.circle_filled(center, 8.0, accent());
    if let Some(icon) = assets.icon(status.icon) {
        painter.image(
            icon,
            Rect::from_center_size(center, Vec2::splat(11.0)),
            full_uv(),
            Color32::BLACK,
        );
    }
    if let Some(fraction) = status.progress {
        progress_ring(painter, center, 11.5, fraction, accent());
    }
}

pub fn draw_update_popup(
    context: &egui::Context,
    viewport: Viewport,
    update: &UpdateView,
    language: &str,
    assets: &impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let UpdateState::Available(version) = &update.state else {
        return;
    };
    if !update.popup {
        return;
    }
    let metrics = UiMetrics::for_viewport(viewport);
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let width = (viewport.width - 48.0).clamp(280.0, 380.0);
    components::modal_width(
        context,
        screen,
        "fluxa-update-popup",
        width,
        |ui| {
            let back = components::back_button(ui, assets.icon("ArrowLeft"), metrics);
            layout.focusable.push((NODE_UPDATE_LATER, back.rect));
            if back.clicked() || ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                layout.activated = Some(NODE_UPDATE_LATER);
            }
            ui.add_space(14.0);
            ui.label(
                RichText::new(format!(
                    "{} v{version}",
                    localized("update.popup_title", language)
                ))
                .size(metrics.text.subtitle)
                .strong()
                .color(text_primary()),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(localized("update.popup_body", language))
                    .size(metrics.text.meta)
                    .color(metrics.text_secondary),
            );
            ui.add_space(18.0);
            let full = ui.available_width();
            let install = components::primary_button(
                ui,
                &localized("update.download_install", language),
                full,
                true,
            );
            layout.focusable.push((NODE_UPDATE_INSTALL, install.rect));
            if install.clicked() {
                layout.activated = Some(NODE_UPDATE_INSTALL);
            }
            ui.add_space(8.0);
            let download = components::button_with_text_size(
                ui,
                &localized("update.download_only", language),
                full,
                crate::metrics::button_height(),
                components::ButtonKind::Secondary,
                crate::style::size::BUTTON_TEXT,
                metrics,
            );
            layout.focusable.push((NODE_UPDATE_DOWNLOAD, download.rect));
            if download.clicked() {
                layout.activated = Some(NODE_UPDATE_DOWNLOAD);
            }
        },
        metrics,
    );
    context.move_to_top(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-update-popup"),
    ));
}
