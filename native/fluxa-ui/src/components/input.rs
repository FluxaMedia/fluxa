use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputKind {
    #[default]
    Text,
    Search,
    Password,
    Url,
    Number {
        secret: bool,
    },
    Multiline,
}

impl InputKind {
    pub fn secret(self) -> bool {
        matches!(self, Self::Password | Self::Number { secret: true })
    }
}

pub fn input_kind(node: u64) -> InputKind {
    match node {
        crate::NODE_LIBRARY_SEARCH | crate::NODE_DISCOVER_SEARCH | crate::NODE_SETTINGS_SEARCH => {
            InputKind::Search
        }
        crate::NODE_SETTINGS_SERVER_PASSWORD => InputKind::Password,
        crate::NODE_SETTINGS_ADDON_URL
        | crate::NODE_SETTINGS_PLUGIN_URL
        | crate::NODE_SETTINGS_SERVER_ADDRESS
        | crate::NODE_SETTINGS_POSTER_URL => InputKind::Url,
        _ if crate::collection_input(node) == Some(0) || crate::badge_input(node) == Some(0) => {
            InputKind::Multiline
        }
        _ => InputKind::Text,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FocusedField {
    pub id: u64,
    pub kind: InputKind,
    pub rect: Rect,
    pub text: String,
    pub hint: String,
    pub selection: [usize; 2],
}

const CARET_FOCUS: &str = "fluxa-input-caret";
const CARET_RECT: &str = "fluxa-caret-rect";
const FOCUSED_FIELD: &str = "fluxa-focused-field";
const PENDING_TEXT: &str = "fluxa-pending-field-text";
const EGUI_FIELD_TAG: u64 = 1 << 62;

fn field_id(id: Id) -> u64 {
    id.value() | EGUI_FIELD_TAG
}

pub fn set_pending_field_text(context: &egui::Context, pending: Option<(u64, String)>) {
    context.data_mut(|data| {
        data.remove::<(u64, String)>(Id::new(PENDING_TEXT));
        if let Some(pending) = pending {
            data.insert_temp(Id::new(PENDING_TEXT), pending);
        }
    });
}

fn take_pending_text(ui: &Ui, id: Id, value: &mut String) {
    let pending = ui.ctx().data_mut(|data| {
        let key = Id::new(PENDING_TEXT);
        match data.get_temp::<(u64, String)>(key) {
            Some((target, text)) if target == field_id(id) => {
                data.remove::<(u64, String)>(key);
                Some(text)
            }
            _ => None,
        }
    });
    let Some(text) = pending else {
        return;
    };
    let end = text.chars().count();
    *value = text;
    let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::one(
            egui::text::CCursor::new(end),
        )));
    state.store(ui.ctx(), id);
}

pub fn set_input_caret(context: &egui::Context, rect: Option<Rect>) {
    context.data_mut(|data| {
        data.insert_temp(Id::new(CARET_FOCUS), rect);
        data.remove::<Rect>(Id::new(CARET_RECT));
        data.remove::<FocusedField>(Id::new(FOCUSED_FIELD));
    });
}

pub fn caret_rect(context: &egui::Context) -> Option<Rect> {
    context.data(|data| data.get_temp::<Rect>(Id::new(CARET_RECT)))
}

pub fn focused_field(context: &egui::Context) -> Option<FocusedField> {
    context.data(|data| data.get_temp::<FocusedField>(Id::new(FOCUSED_FIELD)))
}

fn record_caret(ui: &Ui, caret: Rect) {
    ui.ctx()
        .data_mut(|data| data.insert_temp(Id::new(CARET_RECT), caret));
}

fn record_field(ui: &Ui, field: FocusedField) {
    ui.ctx()
        .data_mut(|data| data.insert_temp(Id::new(FOCUSED_FIELD), field));
}

fn paint_caret(ui: &Ui, field: Rect, text_rect: Rect, text: &str, font: FontId) {
    let focused = ui
        .ctx()
        .data(|data| data.get_temp::<Option<Rect>>(Id::new(CARET_FOCUS)))
        .flatten()
        .is_some_and(|rect| field.contains(rect.center()));
    if !focused {
        return;
    }
    let width = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
        .size()
        .x;
    let x = (text_rect.left() + width + 1.0).min(text_rect.right());
    let half = font.size * 0.62;
    record_caret(
        ui,
        Rect::from_min_max(
            Pos2::new(x, text_rect.center().y - half),
            Pos2::new(x + 1.6, text_rect.center().y + half),
        ),
    );
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis());
    if millis / 530 % 2 == 1 {
        return;
    }
    ui.painter().line_segment(
        [
            egui::Pos2::new(x, text_rect.center().y - half),
            egui::Pos2::new(x, text_rect.center().y + half),
        ],
        egui::Stroke::new(stroke::ICON, text_primary()),
    );
}

fn record_native_focus(
    ui: &Ui,
    output: &egui::text_edit::TextEditOutput,
    kind: InputKind,
    value: &str,
    hint: &str,
) {
    if !output.response.has_focus() {
        return;
    }
    let range = output.cursor_range.unwrap_or_default();
    let [start, end] = [range.primary.index, range.secondary.index];
    let caret = output.galley.pos_from_cursor(range.primary);
    record_caret(ui, caret.translate(output.galley_pos.to_vec2()));
    record_field(
        ui,
        FocusedField {
            id: field_id(output.response.id),
            kind,
            rect: output.response.response.rect,
            text: value.to_owned(),
            hint: hint.to_owned(),
            selection: [start.min(end), start.max(end)],
        },
    );
}

pub(crate) fn search_field(
    ui: &mut Ui,
    query: &mut String,
    hint: &str,
    width: f32,
    height: f32,
    metrics: UiMetrics,
) -> Response {
    let radius = height * 0.5;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    ui.painter()
        .rect_filled(rect, radius, metrics.ink(tone::FAINT));
    let icon_color = metrics.ink(tone::MUTED);
    let icon = egui::Pos2::new(rect.left() + radius, rect.center().y);
    ui.painter().circle_stroke(
        icon - Vec2::splat(1.5),
        6.0,
        egui::Stroke::new(stroke::ICON, icon_color),
    );
    ui.painter().line_segment(
        [icon + Vec2::splat(3.0), icon + Vec2::splat(7.0)],
        egui::Stroke::new(stroke::ICON, icon_color),
    );
    let row = ui.fonts_mut(|fonts| {
        fonts.row_height(&crate::fonts::regular(metrics.screen_body_size + 1.0))
    });
    let text_rect = Rect::from_center_size(
        egui::Pos2::new(
            (rect.left() + radius + 16.0 + rect.right() - radius * 0.6) * 0.5,
            rect.center().y,
        ),
        Vec2::new(rect.width() - radius * 1.6 - 16.0, row),
    );
    let id = ui.id().with(("fluxa-input", hint));
    take_pending_text(ui, id, query);
    let hit = field_hit(ui, rect, id);
    let len = query.chars().count();
    let output = put_edit(
        ui,
        text_rect,
        id,
        egui::TextEdit::singleline(query)
            .frame(egui::Frame::NONE)
            .font(crate::fonts::regular(metrics.screen_body_size + 1.0))
            .vertical_align(egui::Align::Center)
            .hint_text(RichText::new(hint).color(metrics.text_muted))
            .text_color(metrics.text_primary)
            .margin(Vec2::ZERO),
    );
    focus_on_hit(ui, &hit, id, len);
    record_native_focus(ui, &output, InputKind::Search, query, hint);
    let response = output.response.response.clone();
    paint_caret(
        ui,
        rect,
        text_rect,
        query,
        crate::fonts::regular(metrics.screen_body_size + 1.0),
    );
    let stroke = if response.has_focus() {
        egui::Stroke::new(stroke::HAIRLINE, crate::accent())
    } else {
        egui::Stroke::new(stroke::HAIRLINE, metrics.border)
    };
    ui.painter()
        .rect_stroke(rect, radius, stroke, egui::StrokeKind::Inside);
    response
}

fn field_hit(ui: &Ui, rect: Rect, id: Id) -> Response {
    ui.interact(rect, id.with("hit"), Sense::click())
}

fn focus_on_hit(ui: &Ui, hit: &Response, id: Id, len: usize) {
    if !hit.clicked() || ui.memory(|memory| memory.has_focus(id)) {
        return;
    }
    ui.memory_mut(|memory| memory.request_focus(id));
    let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::one(
            egui::text::CCursor::new(len),
        )));
    state.store(ui.ctx(), id);
}

fn put_edit(
    ui: &mut Ui,
    rect: Rect,
    id: Id,
    edit: egui::TextEdit<'_>,
) -> egui::text_edit::TextEditOutput {
    let layout = *ui.layout();
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt(id)
            .max_rect(rect)
            .layout(layout),
        |ui| edit.id(id).show(ui),
    )
    .inner
}

pub(crate) fn text_field(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    size: Vec2,
    font_size: f32,
    kind: InputKind,
    metrics: UiMetrics,
) -> Response {
    text_edit(ui, value, hint, size, font_size, kind, metrics)
}

pub(crate) fn text_input(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    width: f32,
    kind: InputKind,
    metrics: UiMetrics,
) -> Response {
    text_edit(
        ui,
        value,
        hint,
        Vec2::new(width, button_height()),
        metrics.text.body,
        kind,
        metrics,
    )
}

fn text_edit(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    size: Vec2,
    font_size: f32,
    kind: InputKind,
    metrics: UiMetrics,
) -> Response {
    let radius = radius::control();
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_filled(rect, radius, metrics.surface_raised);
    let mut text_rect = rect.shrink2(Vec2::new(14.0, 0.0));
    if kind != InputKind::Multiline {
        let row = ui.fonts_mut(|fonts| fonts.row_height(&crate::fonts::regular(font_size)));
        text_rect = Rect::from_center_size(rect.center(), Vec2::new(text_rect.width(), row));
    }
    let id = ui.id().with(("fluxa-input", hint));
    take_pending_text(ui, id, value);
    let hit = field_hit(ui, rect, id);
    let len = value.chars().count();
    let edit = if kind == InputKind::Multiline {
        egui::TextEdit::multiline(value).desired_rows(1)
    } else {
        egui::TextEdit::singleline(value)
    };
    let output = put_edit(
        ui,
        text_rect,
        id,
        edit.frame(egui::Frame::NONE)
            .password(kind.secret())
            .font(crate::fonts::regular(font_size))
            .vertical_align(egui::Align::Center)
            .hint_text(RichText::new(hint).color(metrics.text_muted))
            .text_color(metrics.text_primary)
            .margin(Vec2::ZERO),
    );
    focus_on_hit(ui, &hit, id, len);
    record_native_focus(ui, &output, kind, value, hint);
    let response = output.response.response.clone();
    let shown = if kind.secret() {
        "\u{2022}".repeat(value.chars().count())
    } else {
        value.clone()
    };
    paint_caret(
        ui,
        rect,
        text_rect,
        &shown,
        crate::fonts::regular(font_size),
    );
    let stroke = if response.has_focus() {
        egui::Stroke::new(stroke::HAIRLINE, crate::accent())
    } else {
        egui::Stroke::new(stroke::HAIRLINE, metrics.border)
    };
    ui.painter()
        .rect_stroke(rect, radius, stroke, egui::StrokeKind::Inside);
    response
}
