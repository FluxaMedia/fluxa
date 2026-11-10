use super::{
    Align, Color32, Context, FontId, Frame, Galley, Id, Pos2, Response, RichText, TextBuffer, Ui,
    Vec2,
};
use std::sync::Arc;

type Layouter<'a> = &'a mut dyn FnMut(&Ui, &dyn TextBuffer, f32) -> Arc<Galley>;

pub struct TextEdit<'a> {
    text: &'a mut dyn TextBuffer,
    multiline: bool,
    id: Option<Id>,
    frame: Option<Frame>,
    font: Option<FontId>,
    vertical_align: Option<Align>,
    hint: Option<RichText>,
    text_color: Option<Color32>,
    margin: Option<Vec2>,
    desired_width: Option<f32>,
    desired_rows: Option<usize>,
    char_limit: Option<usize>,
    password: bool,
    layouter: Option<Layouter<'a>>,
}

pub struct TextEditOutput {
    pub response: Response,
    pub galley: Arc<Galley>,
    pub galley_pos: Pos2,
    pub cursor_range: Option<CursorRange>,
    pub caret: Option<super::Rect>,
}

#[derive(Clone, Copy, Default)]
pub struct CursorRange {
    pub primary: usize,
    pub secondary: usize,
}

impl<'a> TextEdit<'a> {
    fn blank(text: &'a mut dyn TextBuffer, multiline: bool) -> Self {
        Self {
            text,
            multiline,
            id: None,
            frame: None,
            font: None,
            vertical_align: None,
            hint: None,
            text_color: None,
            margin: None,
            desired_width: None,
            desired_rows: None,
            char_limit: None,
            password: false,
            layouter: None,
        }
    }

    pub fn singleline(text: &'a mut dyn TextBuffer) -> Self {
        Self::blank(text, false)
    }

    pub fn multiline(text: &'a mut dyn TextBuffer) -> Self {
        Self::blank(text, true)
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn frame(mut self, frame: Frame) -> Self {
        self.frame = Some(frame);
        self
    }

    pub fn font(mut self, font: FontId) -> Self {
        self.font = Some(font);
        self
    }

    pub fn vertical_align(mut self, align: Align) -> Self {
        self.vertical_align = Some(align);
        self
    }

    pub fn hint_text(mut self, hint: RichText) -> Self {
        self.hint = Some(hint);
        self
    }

    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn margin(mut self, margin: Vec2) -> Self {
        self.margin = Some(margin);
        self
    }

    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    pub fn desired_rows(mut self, rows: usize) -> Self {
        self.desired_rows = Some(rows);
        self
    }

    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = Some(limit);
        self
    }

    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    pub fn layouter(mut self, layouter: Layouter<'a>) -> Self {
        self.layouter = Some(layouter);
        self
    }

    pub fn show(self, ui: &mut Ui) -> TextEditOutput {
        let Self {
            text,
            multiline,
            id,
            frame,
            font,
            vertical_align,
            hint,
            text_color,
            margin,
            desired_width,
            desired_rows,
            char_limit,
            password,
            layouter,
        } = self;
        let mut edit = if multiline {
            egui::TextEdit::multiline(text)
        } else {
            egui::TextEdit::singleline(text)
        };
        if let Some(id) = id {
            edit = edit.id(id);
        }
        if let Some(frame) = frame {
            edit = edit.frame(frame.raw());
        }
        if let Some(font) = font {
            edit = edit.font(font);
        }
        if let Some(align) = vertical_align {
            edit = edit.vertical_align(align);
        }
        if let Some(hint) = hint {
            edit = edit.hint_text(hint);
        }
        if let Some(color) = text_color {
            edit = edit.text_color(color);
        }
        if let Some(margin) = margin {
            edit = edit.margin(margin);
        }
        if let Some(width) = desired_width {
            edit = edit.desired_width(width);
        }
        if let Some(rows) = desired_rows {
            edit = edit.desired_rows(rows);
        }
        if let Some(limit) = char_limit {
            edit = edit.char_limit(limit);
        }
        edit = edit.password(password);
        let has_layouter = layouter.is_some();
        let mut layouter = layouter;
        let mut adapter = |ui: &egui::Ui, text: &dyn TextBuffer, wrap: f32| {
            let inner = layouter.as_mut().expect("layouter set");
            inner(Ui::wrap(ui), text, wrap)
        };
        if has_layouter {
            edit = edit.layouter(&mut adapter);
        }
        let output = edit.show(ui.raw_mut());
        let caret = output.cursor_range.map(|range| {
            output
                .galley
                .pos_from_cursor(range.primary)
                .translate(output.galley_pos.to_vec2())
        });
        TextEditOutput {
            response: Response::from_egui(output.response.response),
            galley: output.galley,
            galley_pos: output.galley_pos,
            caret,
            cursor_range: output.cursor_range.map(|range| CursorRange {
                primary: range.primary.index.0,
                secondary: range.secondary.index.0,
            }),
        }
    }
}

pub fn set_cursor(context: &Context, id: Id, index: usize) {
    let mut state = egui::TextEdit::load_state(context.raw(), id).unwrap_or_default();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::one(
            egui::text::CCursor::new(index),
        )));
    state.store(context.raw(), id);
}
