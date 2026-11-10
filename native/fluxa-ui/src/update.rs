use super::*;
use crate::kit::{self, Role, Type};
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
    painter: &crate::gfx::Painter,
    center: Pos2,
    radius: f32,
    fraction: f32,
    color: Color32,
) {
    let width = 2.0;
    painter.circle_stroke(
        center,
        radius,
        crate::gfx::Stroke::new(width, crate::stylesheet::color("track")),
    );
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
    painter.add(crate::gfx::Shape::line(
        points,
        crate::gfx::Stroke::new(width, color),
    ));
}

pub(crate) fn badge_rect(avatar: Pos2, radius: f32) -> Rect {
    Rect::from_center_size(
        avatar + Vec2::new(radius * 0.85, -radius * 0.85),
        Vec2::splat(30.0),
    )
}

pub(crate) fn draw_badge(
    painter: &crate::gfx::Painter,
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
    context: &crate::gfx::Context,
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
    crate::kit::provide(context, metrics);
    crate::kit::dialog(context, "fluxa-update-popup", |ui| {
        let layout = std::cell::RefCell::new(&mut *layout);
        let back = kit::widget(ui, |ui| {
            let back = components::back_button(ui, assets.icon("ArrowLeft"), metrics);
            if ui.is_sizing_pass() {
                return;
            }
            let mut layout = layout.borrow_mut();
            layout.focusable.push((NODE_UPDATE_LATER, back.rect));
            if back.clicked() || ui.input(|input| input.key_pressed(crate::gfx::Key::Escape)) {
                layout.activated = Some(NODE_UPDATE_LATER);
            }
        });
        let title = kit::widget(ui, |ui| {
            kit::text(
                format!("{} v{version}", localized("update.popup_title", language)),
                Type::Subtitle,
            )
            .bold()
            .show(ui);
        });
        let body = kit::widget(ui, |ui| {
            kit::text(localized("update.popup_body", language), Type::Meta)
                .role(Role::Secondary)
                .show(ui);
        });
        let gap = Space::S8.px(&metrics);
        let actions = kit::widget(ui, |ui| {
            let side = ((ui.available_width() - gap) / 2.0).max(1.0);
            kit::grid(2)
                .cell(Vec2::new(side, metrics.button_height))
                .per_row(2)
                .gap(Space::S8)
                .show(ui, |ui, index| {
                    let (label, node, primary) = if index == 0 {
                        ("update.download_only", NODE_UPDATE_DOWNLOAD, false)
                    } else {
                        ("update.download_install", NODE_UPDATE_INSTALL, true)
                    };
                    let text = localized(label, language);
                    let button = kit::button(&text);
                    let response = if primary { button.primary() } else { button }.show(ui);
                    if ui.is_sizing_pass() {
                        return;
                    }
                    let mut layout = layout.borrow_mut();
                    layout.focusable.push((node, response.rect));
                    if response.clicked() {
                        layout.activated = Some(node);
                    }
                });
        });
        kit::lay(
            ui,
            vec![
                back,
                kit::layout_items::space(ui, Space::S14),
                title,
                kit::layout_items::space(ui, Space::S6),
                body,
                kit::layout_items::space(ui, Space::S20),
                actions,
            ],
        );
    });
    context.move_to_top(crate::gfx::LayerId::new(
        crate::gfx::Order::Foreground,
        Id::new("fluxa-update-popup"),
    ));
}
