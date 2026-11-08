use super::*;
use crate::motion::{Effect, Transition, mount};
use crate::{Cast, Edge, Radius, Space, Viewport, surface_frame};

fn scrim(
    context: &egui::Context,
    id: &str,
    screen: Rect,
    scrim_alpha: u8,
    frame: &crate::motion::Frame,
) -> bool {
    let mut tapped = false;
    egui::Area::new(Id::new((id, "scrim")))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .interactable(!frame.closing)
        .show(context, |ui| {
            let response = ui.allocate_rect(screen, Sense::click());
            let alpha = (scrim_alpha as f32 * frame.shown) as u8;
            ui.painter()
                .rect_filled(screen, 0.0, Color32::from_black_alpha(alpha));
            tapped = response.clicked();
        });
    tapped
}

fn remembered_extent(context: &egui::Context, id: Id, fallback: Vec2) -> Vec2 {
    context
        .data(|data| data.get_temp::<Vec2>(id))
        .unwrap_or(fallback)
}

pub(crate) fn bottom_sheet(
    context: &egui::Context,
    id: &str,
    open: bool,
    viewport: Viewport,
    metrics: UiMetrics,
    add: impl FnOnce(&mut Ui),
) -> bool {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let size_id = Id::new((id, "size"));
    let extent = remembered_extent(
        context,
        size_id,
        Vec2::new(viewport.width, viewport.height * 0.5),
    );
    let Some(frame) = mount(
        context,
        Id::new((id, "mount")),
        open,
        Transition::same(Effect::SlideUp),
        extent,
    ) else {
        return false;
    };
    let mut dismissed = scrim(context, id, screen, metrics.scrim_alpha, &frame);
    let width = if viewport.is_compact() {
        viewport.width
    } else {
        viewport.width.min(metrics.sheet_max_width)
    };
    let pad = Space::S16.px(&metrics);
    egui::Area::new(Id::new(id))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, frame.offset)
        .interactable(!frame.closing)
        .show(context, |ui| {
            ui.set_opacity(frame.alpha);
            let shown = surface_frame(&metrics, Radius::Sheet, Edge::Top, Some(Cast::Up))
                .inner_margin(egui::Margin {
                    left: pad as i8,
                    right: pad as i8,
                    top: Space::S8.px(&metrics) as i8,
                    bottom: (pad + viewport.safe_bottom) as i8,
                })
                .show(ui, |ui| {
                    ui.set_width(width - pad * 2.0);
                    ui.set_max_height(viewport.height * 0.85);
                    let handle = crate::kit::view()
                        .row()
                        .height(Space::S20.px(&metrics))
                        .max_width(metrics.sheet_handle.x)
                        .center_x()
                        .class("overlay-handle")
                        .show(ui, |ui| {
                            ui.allocate_exact_size(metrics.sheet_handle, Sense::hover())
                                .0
                        })
                        .inner;
                    ui.painter().rect_filled(
                        handle,
                        handle.height() * 0.5,
                        crate::stylesheet::color("strong"),
                    );
                    add(ui);
                });
            context.data_mut(|data| data.insert_temp(size_id, shown.response.rect.size()));
            if open && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                dismissed = true;
            }
        });
    dismissed
}

pub(crate) fn side_drawer(
    context: &egui::Context,
    id: &str,
    open: bool,
    viewport: Viewport,
    metrics: UiMetrics,
    add: impl FnOnce(&mut Ui),
) -> bool {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let width = (viewport.width * 0.86).min(metrics.drawer_max_width);
    let Some(frame) = mount(
        context,
        Id::new((id, "mount")),
        open,
        Transition::same(Effect::SlideLeft),
        Vec2::new(width, viewport.height),
    ) else {
        return false;
    };
    let mut dismissed = scrim(context, id, screen, metrics.scrim_alpha, &frame);
    let pad = Space::S20.px(&metrics);
    egui::Area::new(Id::new(id))
        .order(egui::Order::Foreground)
        .anchor(Align2::RIGHT_TOP, frame.offset)
        .interactable(!frame.closing)
        .show(context, |ui| {
            ui.set_opacity(frame.alpha);
            surface_frame(&metrics, Radius::Sheet, Edge::Left, Some(Cast::Left))
                .inner_margin(egui::Margin {
                    left: pad as i8,
                    right: pad as i8,
                    top: pad as i8,
                    bottom: (pad + viewport.safe_bottom) as i8,
                })
                .show(ui, |ui| {
                    ui.set_width(width - pad * 2.0);
                    ui.set_min_height(viewport.height - pad * 2.0 - viewport.safe_bottom);
                    add(ui);
                });
            if open && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                dismissed = true;
            }
        });
    dismissed
}

pub(crate) fn labeled_button(
    ui: &mut Ui,
    icon: Option<egui::TextureId>,
    label: &str,
    selected: bool,
    metrics: UiMetrics,
) -> Response {
    let font = FontId::proportional(metrics.labeled_button_text);
    let icon_size = metrics.labeled_button_icon;
    let text_size = crate::flex::text_size(ui.painter(), label, &font);
    let shown = crate::kit::view()
        .row()
        .gap(Space::S8)
        .height(metrics.icon_button_size)
        .max_width(icon_size + Space::S8.px(&metrics) + text_size.x + Space::S6.px(&metrics) * 2.0)
        .clickable()
        .class("overlay-labeled")
        .show(ui, |ui| {
            crate::kit::space(ui, Space::S6);
            let icon = ui
                .allocate_exact_size(Vec2::splat(icon_size), Sense::hover())
                .0;
            let text = ui.allocate_exact_size(text_size, Sense::hover()).0;
            (icon, text)
        });
    let (icon_slot, text_slot) = shown.inner;
    let response = shown.response;
    let rect = response.rect;
    let hover = crate::motion::hover(ui.ctx(), &response);
    crate::motion::press_scale(ui.painter(), rect, || {
        let painter = ui.painter();
        let tint = if selected {
            tone::strong()
        } else {
            (tone::faint() as f32 + hover * 20.0) as u8
        };
        painter.rect_filled(rect, rect.height() * 0.5, ink(tint));
        let color = ink(if selected || response.hovered() {
            tone::full()
        } else {
            tone::body()
        });
        if let Some(texture) = icon {
            painter.image(texture, icon_slot, full_uv(), color);
        }
        crate::flex::paint_text(painter, text_slot, label, &font, color, Align2::LEFT_CENTER);
    });
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiFormFactor, Viewport};

    #[test]
    fn sheet_is_drawn_while_closing_then_dropped() {
        let context = egui::Context::default();
        let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
        let metrics = UiMetrics::for_viewport(viewport);
        let mut drawn = Vec::new();
        crate::kit::provide(&context, metrics);
        for tick in 0..90 {
            let open = tick < 30;
            let input = egui::RawInput {
                time: Some(tick as f64 / 60.0),
                predicted_dt: 1.0 / 60.0,
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(390.0, 844.0))),
                ..Default::default()
            };
            let mut body = false;
            context
                .run_pass(input, |context| {
                    bottom_sheet(context, "test-sheet", open, viewport, metrics, |ui| {
                        body = true;
                        ui.label("content");
                    });
                })
                .drop_without_applying_deltas();
            drawn.push(body);
        }
        assert!(drawn[0]);
        assert!(drawn[35]);
        assert!(!drawn[89]);
    }
}
