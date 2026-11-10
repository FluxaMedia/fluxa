use super::{Context, Id, LayerId, Pos2, Rect, Sense, Vec2};

#[derive(Clone)]
pub struct Response {
    pub rect: Rect,
    pub id: Id,
    pub layer_id: LayerId,
    pub sense: Sense,
    raw: egui::Response,
}

impl Response {
    pub(in crate::gfx) fn from_egui(raw: egui::Response) -> Self {
        Self {
            rect: raw.rect,
            id: raw.id,
            layer_id: raw.layer_id,
            sense: raw.sense,
            raw,
        }
    }

    pub(in crate::gfx) fn raw(&self) -> &egui::Response {
        &self.raw
    }

    pub fn ctx(&self) -> &Context {
        Context::wrap(&self.raw.ctx)
    }

    pub fn clicked(&self) -> bool {
        self.raw.clicked()
    }

    pub fn double_clicked(&self) -> bool {
        self.raw.double_clicked()
    }

    pub fn secondary_clicked(&self) -> bool {
        self.raw.secondary_clicked()
    }

    pub fn hover_pos(&self) -> Option<Pos2> {
        self.raw.hover_pos()
    }

    pub fn on_hover_cursor(self, icon: super::CursorIcon) -> Self {
        Self::from_egui(self.raw.on_hover_cursor(icon))
    }

    pub fn hovered(&self) -> bool {
        self.raw.hovered()
    }

    pub fn has_focus(&self) -> bool {
        self.raw.has_focus()
    }

    pub fn gained_focus(&self) -> bool {
        self.raw.gained_focus()
    }

    pub fn lost_focus(&self) -> bool {
        self.raw.lost_focus()
    }

    pub fn request_focus(&self) {
        self.raw.request_focus();
    }

    pub fn is_pointer_button_down_on(&self) -> bool {
        self.raw.is_pointer_button_down_on()
    }

    pub fn dragged(&self) -> bool {
        self.raw.dragged()
    }

    pub fn drag_started(&self) -> bool {
        self.raw.drag_started()
    }

    pub fn drag_stopped(&self) -> bool {
        self.raw.drag_stopped()
    }

    pub fn drag_delta(&self) -> Vec2 {
        self.raw.drag_delta()
    }

    pub fn changed(&self) -> bool {
        self.raw.changed()
    }

    pub fn mark_changed(&mut self) {
        self.raw.mark_changed();
    }

    pub fn interact_pointer_pos(&self) -> Option<Pos2> {
        self.raw.interact_pointer_pos()
    }

    pub fn on_hover_text(self, text: impl Into<egui::WidgetText>) -> Self {
        Self::from_egui(self.raw.on_hover_text(text))
    }

    pub fn scroll_to_me(&self, align: Option<super::Align>) {
        self.raw.scroll_to_me(align);
    }
}

pub struct InnerResponse<R> {
    pub inner: R,
    pub response: Response,
}

impl<R> InnerResponse<R> {
    pub(in crate::gfx) fn from_egui(raw: egui::InnerResponse<R>) -> Self {
        Self {
            inner: raw.inner,
            response: Response::from_egui(raw.response),
        }
    }
}
