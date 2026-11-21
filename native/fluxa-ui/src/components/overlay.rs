use super::*;
use crate::Space;
use crate::flex::{Align, Insets, Justify, leaf, row, text};

pub(crate) fn labeled_button(
    ui: &mut Ui,
    icon: Option<crate::gfx::TextureId>,
    label: &str,
    selected: bool,
) -> Response {
    let font = FontId::proportional(crate::vars::px("labeled-button-text"));
    let icon_size = crate::vars::px("labeled-button-icon");
    let tree = row([
        leaf(Vec2::splat(icon_size)).id(0),
        text(ui.painter(), label, &font).id(1),
    ])
    .gap(Space::S8.px())
    .align(Align::Center)
    .justify(Justify::SpaceAround)
    .padding(Insets::xy(Space::S6.px(), Space::S8.px()))
    .min_height(crate::vars::px("icon-button-size"));
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
