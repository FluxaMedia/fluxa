use super::{
    Align2, Color32, Context, CornerRadius, Id, InnerResponse, Margin, Order, Pos2, Rect, Response,
    Shadow, Stroke, Ui, Vec2,
};

#[derive(Clone, Copy)]
pub struct Frame(egui::Frame);

impl Frame {
    pub const NONE: Self = Self(egui::Frame::NONE);

    pub fn new() -> Self {
        Self(egui::Frame::new())
    }

    pub fn fill(self, fill: Color32) -> Self {
        Self(self.0.fill(fill))
    }

    pub fn stroke(self, stroke: Stroke) -> Self {
        Self(self.0.stroke(stroke))
    }

    pub fn corner_radius(self, radius: impl Into<CornerRadius>) -> Self {
        Self(self.0.corner_radius(radius))
    }

    pub fn inner_margin(self, margin: impl Into<Margin>) -> Self {
        Self(self.0.inner_margin(margin))
    }

    pub fn outer_margin(self, margin: impl Into<Margin>) -> Self {
        Self(self.0.outer_margin(margin))
    }

    pub fn shadow(self, shadow: Shadow) -> Self {
        Self(self.0.shadow(shadow))
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        InnerResponse::from_egui(self.0.show(ui.raw_mut(), |ui| body(Ui::wrap_mut(ui))))
    }

    pub(in crate::gfx) fn raw(self) -> egui::Frame {
        self.0
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Area(egui::Area);

impl Area {
    pub fn new(id: Id) -> Self {
        Self(egui::Area::new(id))
    }

    pub fn fixed_pos(self, pos: Pos2) -> Self {
        Self(self.0.fixed_pos(pos))
    }

    pub fn order(self, order: Order) -> Self {
        Self(self.0.order(order))
    }

    pub fn constrain(self, constrain: bool) -> Self {
        Self(self.0.constrain(constrain))
    }

    pub fn interactable(self, interactable: bool) -> Self {
        Self(self.0.interactable(interactable))
    }

    pub fn anchor(self, align: Align2, offset: Vec2) -> Self {
        Self(self.0.anchor(align, offset))
    }

    pub fn pivot(self, pivot: Align2) -> Self {
        Self(self.0.pivot(pivot))
    }

    pub fn show<R>(self, context: &Context, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        InnerResponse::from_egui(self.0.show(context.raw(), |ui| body(Ui::wrap_mut(ui))))
    }
}

pub struct CentralPanel(egui::CentralPanel);

impl CentralPanel {
    pub fn default() -> Self {
        Self(egui::CentralPanel::default())
    }

    pub fn frame(self, frame: Frame) -> Self {
        Self(self.0.frame(frame.raw()))
    }

    pub fn show<R>(self, root: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        InnerResponse::from_egui(self.0.show(root.raw_mut(), |ui| body(Ui::wrap_mut(ui))))
    }
}

pub use egui::scroll_area::{ScrollBarVisibility, ScrollSource};

pub struct ScrollOutput<R> {
    pub inner: R,
    pub content_size: Vec2,
    pub inner_rect: Rect,
}

pub struct ScrollArea(egui::ScrollArea);

impl ScrollArea {
    pub fn both() -> Self {
        Self(egui::ScrollArea::both())
    }

    pub fn id_salt(self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self(self.0.id_salt(salt))
    }

    pub fn scroll_bar_visibility(self, visibility: ScrollBarVisibility) -> Self {
        Self(self.0.scroll_bar_visibility(visibility))
    }

    pub fn scroll_source(self, source: ScrollSource) -> Self {
        Self(self.0.scroll_source(source))
    }

    pub fn horizontal_scroll_offset(self, offset: f32) -> Self {
        Self(self.0.horizontal_scroll_offset(offset))
    }

    pub fn vertical_scroll_offset(self, offset: f32) -> Self {
        Self(self.0.vertical_scroll_offset(offset))
    }

    pub fn auto_shrink(self, shrink: [bool; 2]) -> Self {
        Self(self.0.auto_shrink(shrink))
    }

    pub fn min_scrolled_height(self, height: f32) -> Self {
        Self(self.0.min_scrolled_height(height))
    }

    pub fn stick_to_bottom(self, stick: bool) -> Self {
        Self(self.0.stick_to_bottom(stick))
    }

    pub fn max_height(self, height: f32) -> Self {
        Self(self.0.max_height(height))
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> ScrollOutput<R> {
        let out = self.0.show(ui.raw_mut(), |ui| body(Ui::wrap_mut(ui)));
        ScrollOutput {
            inner: out.inner,
            content_size: out.content_size,
            inner_rect: out.inner_rect,
        }
    }
}

pub struct Popup<'a> {
    inner: egui::Popup<'a>,
}

impl<'a> Popup<'a> {
    pub fn from_toggle_button_response(response: &'a Response) -> Self {
        Self {
            inner: egui::Popup::from_toggle_button_response(response.raw()),
        }
    }

    pub fn is_any_open(context: &Context) -> bool {
        egui::Popup::is_any_open(context.raw())
    }

    pub fn is_id_open(context: &Context, id: Id) -> bool {
        egui::Popup::is_id_open(context.raw(), id)
    }

    pub fn close_id(context: &Context, id: Id) {
        egui::Popup::close_id(context.raw(), id);
    }

    pub fn id(self, id: Id) -> Self {
        Self {
            inner: self.inner.id(id),
        }
    }

    pub fn gap(self, gap: f32) -> Self {
        Self {
            inner: self.inner.gap(gap),
        }
    }

    pub fn width(self, width: f32) -> Self {
        Self {
            inner: self.inner.width(width),
        }
    }

    pub fn close_behavior(self, behavior: super::PopupCloseBehavior) -> Self {
        Self {
            inner: self.inner.close_behavior(behavior),
        }
    }

    pub fn frame(self, frame: Frame) -> Self {
        Self {
            inner: self.inner.frame(frame.raw()),
        }
    }

    pub fn show<R>(self, body: impl FnOnce(&mut Ui) -> R) -> Option<InnerResponse<R>> {
        self.inner
            .show(|ui| body(Ui::wrap_mut(ui)))
            .map(InnerResponse::from_egui)
    }
}
