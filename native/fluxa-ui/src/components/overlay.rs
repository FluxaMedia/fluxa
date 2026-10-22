use super::*;
use crate::flex::{Align, Insets, Justify, leaf, row, text};
use crate::motion::{Effect, Transition, mount};
use crate::{Cast, Edge, Radius, Space, Viewport, surface_frame};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrawerSide {
    Left,
    Right,
}

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
                    let (bar, _) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), Space::S20.px(&metrics)),
                        Sense::hover(),
                    );
                    let handle = row([leaf(metrics.sheet_handle).id(0)])
                        .align(Align::Center)
                        .justify(Justify::Center)
                        .solve(bar);
                    if let Some(rect) = handle.find(0) {
                        ui.painter()
                            .rect_filled(rect, rect.height() * 0.5, ink(tone::strong()));
                    }
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
    side: DrawerSide,
    viewport: Viewport,
    metrics: UiMetrics,
    add: impl FnOnce(&mut Ui),
) -> bool {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let width = (viewport.width * 0.86).min(metrics.drawer_max_width);
    let effect = match side {
        DrawerSide::Right => Effect::SlideLeft,
        DrawerSide::Left => Effect::SlideRight,
    };
    let Some(frame) = mount(
        context,
        Id::new((id, "mount")),
        open,
        Transition::same(effect),
        Vec2::new(width, viewport.height),
    ) else {
        return false;
    };
    let mut dismissed = scrim(context, id, screen, metrics.scrim_alpha, &frame);
    let (anchor, edge, cast) = match side {
        DrawerSide::Right => (Align2::RIGHT_TOP, Edge::Left, Cast::Left),
        DrawerSide::Left => (Align2::LEFT_TOP, Edge::Right, Cast::Right),
    };
    let pad = Space::S20.px(&metrics);
    egui::Area::new(Id::new(id))
        .order(egui::Order::Foreground)
        .anchor(anchor, frame.offset)
        .interactable(!frame.closing)
        .show(context, |ui| {
            ui.set_opacity(frame.alpha);
            surface_frame(&metrics, Radius::Sheet, edge, Some(cast))
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
    let tree = row([
        leaf(Vec2::splat(icon_size)).id(0),
        text(ui.painter(), label, &font).id(1),
    ])
    .gap(Space::S8.px(&metrics))
    .align(Align::Center)
    .justify(Justify::SpaceAround)
    .padding(Insets::xy(Space::S6.px(&metrics), Space::S8.px(&metrics)))
    .min_height(metrics.icon_button_size);
    let size = tree.outer_size();
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let placed = tree.solve(rect);
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
        if let (Some(texture), Some(slot)) = (icon, placed.find(0)) {
            painter.image(texture, slot, full_uv(), color);
        }
        if let Some(slot) = placed.find(1) {
            crate::flex::paint_text(painter, slot, label, &font, color, Align2::LEFT_CENTER);
        }
    });
    response
}

pub(crate) fn toast_message(
    context: &egui::Context,
    id: &str,
    message: Option<&str>,
    viewport: Viewport,
    top_center: Pos2,
) -> bool {
    let text_id = Id::new((id, "text"));
    if let Some(message) = message {
        context.data_mut(|data| data.insert_temp(text_id, message.to_owned()));
    }
    let Some(frame) = mount(
        context,
        Id::new((id, "mount")),
        message.is_some(),
        Transition::new(Effect::SlideDown, Effect::Fade),
        Vec2::new(0.0, 24.0),
    ) else {
        return false;
    };
    let Some(text) = context.data(|data| data.get_temp::<String>(text_id)) else {
        return false;
    };
    let alpha = if frame.closing {
        frame.alpha
    } else {
        frame.shown
    };
    super::toast(
        context,
        viewport,
        top_center + frame.offset,
        &text,
        None,
        alpha,
    );
    true
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
        for tick in 0..90 {
            let open = tick < 30;
            let input = egui::RawInput {
                time: Some(tick as f64 / 60.0),
                predicted_dt: 1.0 / 60.0,
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(390.0, 844.0))),
                ..Default::default()
            };
            let mut body = false;
            context.run_pass(input, |context| {
                bottom_sheet(context, "test-sheet", open, viewport, metrics, |ui| {
                    body = true;
                    ui.label("content");
                });
            }).drop_without_applying_deltas();
            drawn.push(body);
        }
        assert!(drawn[0]);
        assert!(drawn[35]);
        assert!(!drawn[89]);
    }

    #[test]
    fn toast_keeps_its_text_while_fading_out() {
        let context = egui::Context::default();
        let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
        let mut drawn = Vec::new();
        for tick in 0..90 {
            let message = (tick < 30).then_some("Saved");
            let input = egui::RawInput {
                time: Some(tick as f64 / 60.0),
                predicted_dt: 1.0 / 60.0,
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(390.0, 844.0))),
                ..Default::default()
            };
            let mut shown = false;
            context.run_pass(input, |context| {
                shown = toast_message(
                    context,
                    "test-toast",
                    message,
                    viewport,
                    Pos2::new(195.0, 40.0),
                );
            }).drop_without_applying_deltas();
            drawn.push(shown);
        }
        assert!(drawn[10]);
        assert!(drawn[35]);
        assert!(!drawn[89]);
    }
}
