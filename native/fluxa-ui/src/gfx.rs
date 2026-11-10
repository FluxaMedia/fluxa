mod containers;
mod context;
mod painter;
mod response;
mod text_edit;
mod ui;

pub use containers::{
    Area, CentralPanel, Frame, Popup, ScrollArea, ScrollBarVisibility, ScrollOutput, ScrollSource,
};
pub use context::{Context, Data, Fonts, InputState, Pointer, RunOutput};
pub use painter::Painter;
pub use response::{InnerResponse, Response};
pub use text_edit::{CursorRange, TextEdit, TextEditOutput, set_cursor};
pub use ui::Ui;

pub use egui::{
    Align, Align2, Button, Color32, CornerRadius, CursorIcon, Event, FontData, FontDefinitions,
    FontFamily, FontId, Galley, Id, Image, Key, Label, LayerId, Layout, Margin, Mesh, Modifiers,
    Order, PaintCallback, PointerButton, PopupCloseBehavior, Pos2, RawInput, Rect, RichText, Sense,
    Shadow, Shape, Stroke, StrokeKind, SurrenderFocusOn, TextBuffer, TextFormat, TextureId,
    TouchDeviceId, TouchId, TouchPhase, UiBuilder, Vec2, emath, epaint, layers, scroll_area, text,
    vec2,
};

macro_rules! transparent {
    ($outer:ident, $inner:ty) => {
        impl $outer {
            #[allow(dead_code)]
            pub(in crate::gfx) fn wrap(inner: &$inner) -> &Self {
                unsafe { &*(inner as *const $inner as *const Self) }
            }

            #[allow(dead_code)]
            pub(in crate::gfx) fn wrap_mut(inner: &mut $inner) -> &mut Self {
                unsafe { &mut *(inner as *mut $inner as *mut Self) }
            }

            #[allow(dead_code)]
            pub(in crate::gfx) fn raw(&self) -> &$inner {
                &self.0
            }

            #[allow(dead_code)]
            pub(in crate::gfx) fn raw_mut(&mut self) -> &mut $inner {
                &mut self.0
            }
        }
    };
}
pub(in crate::gfx) use transparent;
