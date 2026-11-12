use super::*;
use crate::Space;
use crate::flex;

const QR_PAD: f32 = 32.0;
const QR_RADIUS: f32 = 28.0;

pub(crate) fn wordmark(
    ui: &mut Ui,
    texture: Option<(crate::gfx::TextureId, Vec2)>,
    width: f32,
    height: f32,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let Some((texture, aspect)) = texture else {
        return;
    };
    let placed = flex::row([flex::leaf(Vec2::new(height * aspect.x, height))
        .id(0)
        .shrink(0.0)])
    .justify(flex::Justify::Center)
    .align(flex::Align::Center)
    .solve(rect);
    ui.painter().image(
        texture,
        placed.find(0).unwrap_or(rect),
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
}

pub(crate) fn qr_card(ui: &mut Ui, side: f32, modules: &[bool], columns: usize) {
    let (card, _) = ui.allocate_exact_size(Vec2::splat(side + QR_PAD * 2.0), Sense::hover());
    ui.painter().rect_filled(card, QR_RADIUS, Color32::WHITE);
    if columns == 0 {
        return;
    }
    let area = card.shrink(QR_PAD);
    let module = area.width() / columns as f32;
    for (index, dark) in modules.iter().enumerate() {
        if !dark {
            continue;
        }
        let (x, y) = ((index % columns) as f32, (index / columns) as f32);
        ui.painter().rect_filled(
            crate::gfx::rect_at(
                area.min + Vec2::new(x * module, y * module),
                Vec2::splat(module.ceil()),
            ),
            0.0,
            Color32::BLACK,
        );
    }
}

pub(crate) fn numbered_step(ui: &mut Ui, number: usize, text: &str, metrics: UiMetrics) {
    let badge = metrics.text.subtitle * 1.6;
    crate::kit::view()
        .row()
        .gap(Space::S16)
        .class("brand-step")
        .show(ui, |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(badge), Sense::hover());
            ui.painter().circle_filled(
                rect.center(),
                badge * 0.5,
                crate::stylesheet::color("track"),
            );
            draw_text(
                ui.painter(),
                rect.center(),
                Align2::CENTER_CENTER,
                number.to_string(),
                FontId::proportional(metrics.text.body),
                text_primary(),
                None,
            );
            crate::kit::text(text, crate::kit::Type::Subtitle).show(ui);
        });
}
