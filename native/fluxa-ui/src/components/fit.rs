use crate::Viewport;
use crate::gfx::emath::TSTransform;
use crate::gfx::{Context, Id, LayerId, Order, Pos2, Rect, Ui, Vec2};

pub fn fit_to_safe_area(context: &Context, viewport: Viewport, area: Id) {
    let measured = context.data(|data| data.get_temp::<Rect>(fit_id(area)));
    if measured.is_none() {
        context.request_repaint();
    }
    context.set_transform_layer(
        LayerId::new(Order::Middle, area),
        fit_transform(viewport, measured),
    );
}

pub fn unclip_for_fit(ui: &mut Ui, screen: Vec2) {
    ui.set_clip_rect(Rect::from_min_size(Pos2::ZERO, screen * 4.0));
}

pub fn record_fit_content(context: &Context, area: Id, content: Rect) {
    context.data_mut(|data| data.insert_temp(fit_id(area), content));
}

fn fit_id(area: Id) -> Id {
    area.with("fit")
}

pub(crate) fn fit_transform(viewport: Viewport, content: Option<Rect>) -> TSTransform {
    let Some(content) = content.filter(|rect| rect.height() > 0.0) else {
        return TSTransform::IDENTITY;
    };
    let top = viewport.safe_top.max(viewport.height * 0.05);
    let bottom = viewport.safe_bottom.max(viewport.height * 0.05);
    let room = (viewport.height - top - bottom).max(1.0);
    let scale = (room / content.height()).min(1.0);
    let offset = Vec2::new(
        content.center().x * (1.0 - scale),
        top + (room - content.height() * scale) * 0.5 - content.min.y * scale,
    );
    TSTransform::new(offset, scale)
}
