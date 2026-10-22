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
    pub size: f32,
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
    (id.value() & (EGUI_FIELD_TAG - 1)) | EGUI_FIELD_TAG
}

pub fn set_pending_field_text(context: &egui::Context, pending: Option<(u64, String)>) {
    context.data_mut(|data| {
        data.remove::<(u64, String)>(Id::new(PENDING_TEXT));
        if let Some(pending) = pending {
            data.insert_temp(Id::new(PENDING_TEXT), pending);
        }
    });
}

fn take_pending_text(ui: &Ui, id: Id, value: &mut String) -> bool {
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
        return false;
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
    true
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
        egui::Stroke::new(stroke::icon(), text_primary()),
    );
}

static NATIVE_FIELD: std::sync::Mutex<Option<Rect>> = std::sync::Mutex::new(None);

pub fn set_native_field(rect: Option<Rect>) {
    if let Ok(mut field) = NATIVE_FIELD.lock() {
        *field = rect;
    }
}

fn native_covers(rect: Rect) -> bool {
    NATIVE_FIELD
        .lock()
        .ok()
        .and_then(|field| *field)
        .is_some_and(|field| rect.contains(field.center()) || field.contains(rect.center()))
}

fn record_native_focus(
    ui: &Ui,
    output: &egui::text_edit::TextEditOutput,
    rect: Rect,
    size: f32,
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
            size,
            rect,
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
        .rect_filled(rect, radius, metrics.ink(tone::faint()));
    let icon_color = metrics.ink(tone::muted());
    let icon = egui::Pos2::new(rect.left() + radius, rect.center().y);
    ui.painter().circle_stroke(
        icon - Vec2::splat(1.5),
        6.0,
        egui::Stroke::new(stroke::icon(), icon_color),
    );
    ui.painter().line_segment(
        [icon + Vec2::splat(3.0), icon + Vec2::splat(7.0)],
        egui::Stroke::new(stroke::icon(), icon_color),
    );
    let text_rect = Rect::from_center_size(
        egui::Pos2::new(
            (rect.left() + radius + 16.0 + rect.right() - radius * 0.6) * 0.5,
            rect.center().y,
        ),
        Vec2::new(rect.width() - radius * 1.6 - 16.0, rect.height()),
    );
    let id = ui.id().with(("fluxa-input", hint));
    let pending = take_pending_text(ui, id, query);
    let hit = field_hit(ui, rect, id);
    let len = query.chars().count();
    let native = native_covers(text_rect);
    let output = put_edit(
        ui,
        row_rect(
            ui,
            text_rect,
            crate::fonts::regular(metrics.screen_body_size + 1.0),
        ),
        id,
        egui::TextEdit::singleline(query)
            .frame(egui::Frame::NONE)
            .font(crate::fonts::regular(metrics.screen_body_size + 1.0))
            .vertical_align(egui::Align::Center)
            .hint_text(RichText::new(hint).color(metrics.text_muted))
            .text_color(shown(metrics.text_primary, native))
            .margin(Vec2::ZERO),
    );
    focus_on_hit(ui, &hit, id, len);
    record_native_focus(
        ui,
        &output,
        text_rect,
        metrics.screen_body_size + 1.0,
        InputKind::Search,
        query,
        hint,
    );
    let mut response = output.response.response.clone();
    if pending {
        response.mark_changed();
    }
    if !native {
        paint_caret(
            ui,
            rect,
            text_rect,
            query,
            crate::fonts::regular(metrics.screen_body_size + 1.0),
        );
    }
    let stroke = if response.has_focus() {
        egui::Stroke::new(stroke::hairline(), crate::accent())
    } else {
        egui::Stroke::new(stroke::hairline(), metrics.border)
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

fn row_rect(ui: &Ui, rect: Rect, font: FontId) -> Rect {
    let row = ui.fonts_mut(|fonts| fonts.row_height(&font));
    Rect::from_center_size(rect.center(), Vec2::new(rect.width(), row))
}

fn shown(color: Color32, native: bool) -> Color32 {
    if native { Color32::TRANSPARENT } else { color }
}

fn put_edit(
    ui: &mut Ui,
    rect: Rect,
    id: Id,
    edit: egui::TextEdit<'_>,
) -> egui::text_edit::TextEditOutput {
    let layout = *ui.layout();
    let native = native_covers(rect);
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt(id)
            .max_rect(rect)
            .layout(layout),
        |ui| {
            if native {
                ui.visuals_mut().text_cursor.stroke = egui::Stroke::NONE;
                ui.visuals_mut().selection.bg_fill = Color32::TRANSPARENT;
                ui.visuals_mut().selection.stroke = egui::Stroke::NONE;
            }
            edit.id(id).show(ui)
        },
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

pub(crate) struct MessageInput {
    pub response: Response,
    pub submitted: bool,
}

pub(crate) fn message_input(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    width: f32,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
) -> MessageInput {
    let height = 48.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let id = ui.id().with(("fluxa-message-input", hint));
    take_pending_text(ui, id, value);
    let button = Rect::from_center_size(
        Pos2::new(rect.right() - height * 0.5, rect.center().y),
        Vec2::splat(height - 12.0),
    );
    let font = crate::fonts::regular(metrics.text.body);
    let text_rect = Rect::from_min_max(
        Pos2::new(rect.left() + 18.0, rect.top()),
        Pos2::new(button.left() - 10.0, rect.bottom()),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, height * 0.5, metrics.surface_raised);
    let hit = field_hit(ui, rect, id);
    let len = value.chars().count();
    let native = native_covers(text_rect);
    let color = shown(metrics.text_primary, native);
    let mut layouter = |ui: &Ui, text: &dyn egui::TextBuffer, _wrap: f32| {
        let job = crate::emoji::job(text.as_str(), font.clone(), color, f32::INFINITY);
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    };
    let output = put_edit(
        ui,
        row_rect(ui, text_rect, font.clone()),
        id,
        egui::TextEdit::singleline(value)
            .frame(egui::Frame::NONE)
            .font(font.clone())
            .vertical_align(egui::Align::Center)
            .hint_text(RichText::new(hint).color(metrics.text_muted))
            .text_color(color)
            .layouter(&mut layouter)
            .margin(Vec2::ZERO),
    );
    focus_on_hit(ui, &hit, id, len);
    record_native_focus(
        ui,
        &output,
        text_rect,
        metrics.text.body,
        InputKind::Text,
        value,
        hint,
    );
    let response = output.response.response.clone();
    if !native {
        crate::emoji::overlay(
            &ui.painter().with_clip_rect(text_rect),
            output.galley_pos,
            &output.galley,
            assets,
        );
        paint_caret(ui, rect, text_rect, value, font);
    }
    let border = if response.has_focus() {
        egui::Stroke::new(1.5, crate::accent())
    } else {
        egui::Stroke::new(stroke::hairline(), metrics.border)
    };
    let painter = ui.painter();
    painter.rect_stroke(rect, height * 0.5, border, egui::StrokeKind::Inside);

    let ready = !value.trim().is_empty();
    let send = ui.interact(button, id.with("send"), Sense::click());
    let (fill, ink) = if ready {
        (Color32::WHITE, Color32::BLACK)
    } else {
        (metrics.border, metrics.text_muted)
    };
    let painter = ui.painter();
    painter.circle_filled(button.center(), button.width() * 0.5, fill);
    let arm = button.width() * 0.2;
    let center = button.center();
    let arrow = egui::Stroke::new(2.2, ink);
    painter.line_segment(
        [center + Vec2::new(0.0, arm), center - Vec2::new(0.0, arm)],
        arrow,
    );
    painter.line_segment(
        [center - Vec2::new(0.0, arm), center + Vec2::new(-arm, 0.0)],
        arrow,
    );
    painter.line_segment(
        [center - Vec2::new(0.0, arm), center + Vec2::new(arm, 0.0)],
        arrow,
    );
    let entered = response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
    if entered || send.clicked() {
        response.request_focus();
    }
    ui.advance_cursor_after_rect(rect);
    MessageInput {
        submitted: ready && (entered || send.clicked()),
        response,
    }
}

pub(crate) struct CodeInput {
    pub response: Response,
    pub rect: Rect,
    pub button: Rect,
    pub submitted: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CodeLook {
    Pill,
    Boxes,
}

pub(crate) fn code_input(
    ui: &mut Ui,
    value: &mut String,
    length: usize,
    width: f32,
    metrics: UiMetrics,
) -> CodeInput {
    code_input_styled(ui, value, length, width, metrics, CodeLook::Pill)
}

pub(crate) fn code_input_styled(
    ui: &mut Ui,
    value: &mut String,
    length: usize,
    width: f32,
    metrics: UiMetrics,
    look: CodeLook,
) -> CodeInput {
    let height = 60.0;
    let boxes = look == CodeLook::Boxes;
    let pad = if boxes { 0.0 } else { 26.0 };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let id = ui.id().with("fluxa-code-input");
    take_pending_text(ui, id, value);
    let field = Rect::from_min_max(
        Pos2::new(rect.left() + pad, rect.top()),
        Pos2::new(rect.right() - height, rect.bottom()),
    );
    let hit = field_hit(ui, rect, id);
    let output = put_edit(
        ui,
        row_rect(ui, field, crate::fonts::regular(metrics.text.body)),
        id,
        egui::TextEdit::singleline(value)
            .frame(egui::Frame::NONE)
            .char_limit(length)
            .font(crate::fonts::regular(metrics.text.body))
            .text_color(Color32::TRANSPARENT)
            .margin(Vec2::ZERO),
    );
    *value = value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(length)
        .collect::<String>()
        .to_uppercase();
    let count = value.chars().count();
    focus_on_hit(ui, &hit, id, count);
    record_native_focus(ui, &output, field, if boxes { 22.0 } else { 26.0 }, InputKind::Text, value, "code");
    let response = output.response.response.clone();
    let focused = response.has_focus();
    let complete = count == length;

    let radius = height * 0.5;
    let painter = ui.painter();
    if !boxes {
        painter.rect_filled(rect, radius, metrics.surface_raised);
        let border = if focused {
            egui::Stroke::new(1.5, crate::accent())
        } else {
            egui::Stroke::new(stroke::hairline(), metrics.border)
        };
        painter.rect_stroke(rect, radius, border, egui::StrokeKind::Inside);
    }

    let button = Rect::from_center_size(
        Pos2::new(rect.right() - height * 0.5, rect.center().y),
        Vec2::splat(height - 12.0),
    );
    let go = ui.interact(button, id.with("go"), Sense::click());
    let (fill, ink) = if complete {
        (Color32::WHITE, Color32::BLACK)
    } else {
        (metrics.border, metrics.text_muted)
    };
    let painter = ui.painter();
    painter.circle_filled(button.center(), button.width() * 0.5, fill);
    let arm = button.width() * 0.2;
    let center = button.center();
    let arrow = egui::Stroke::new(2.2, ink);
    painter.line_segment(
        [center - Vec2::new(arm, 0.0), center + Vec2::new(arm, 0.0)],
        arrow,
    );
    painter.line_segment(
        [center + Vec2::new(arm, 0.0), center + Vec2::new(0.0, -arm)],
        arrow,
    );
    painter.line_segment(
        [center + Vec2::new(arm, 0.0), center + Vec2::new(0.0, arm)],
        arrow,
    );

    let font = crate::fonts::bold(if boxes { 22.0 } else { 26.0 });
    let split = if boxes { 20.0 } else { 22.0 };
    let slot = if boxes {
        ((width - height - split - 4.0) / length as f32).clamp(30.0, 48.0)
    } else {
        28.0
    };
    let left = rect.left() + pad;
    let half = length / 2;
    let x_of = |index: usize| left + index as f32 * slot + if index >= half { split } else { 0.0 };
    for index in 0..length {
        let center = Pos2::new(x_of(index) + slot * 0.5, rect.center().y);
        if boxes {
            let cell_width = slot - 6.0;
            let cell = Rect::from_center_size(center, Vec2::new(cell_width, (cell_width * 1.2).clamp(42.0, 52.0)));
            let current = focused && index == count;
            painter.rect_filled(cell, 12.0, Color32::from_white_alpha(13));
            let edge = if current {
                egui::Stroke::new(1.5, Color32::from_white_alpha(170))
            } else {
                egui::Stroke::new(stroke::hairline(), Color32::from_white_alpha(26))
            };
            painter.rect_stroke(cell, 12.0, edge, egui::StrokeKind::Inside);
            if current {
                painter.rect_stroke(
                    cell.expand(2.5),
                    14.0,
                    egui::Stroke::new(2.0, Color32::from_white_alpha(22)),
                    egui::StrokeKind::Outside,
                );
            }
        }
        match value.chars().nth(index) {
            Some(character) => {
                painter.text(
                    center,
                    Align2::CENTER_CENTER,
                    character,
                    font.clone(),
                    metrics.text_primary,
                );
            }
            None if focused && index == count => {
                let blink = (ui.input(|input| input.time) * 2.0) as i64 % 2 == 0;
                if blink {
                    let caret = Rect::from_center_size(center, Vec2::new(2.0, 26.0));
                    painter.rect_filled(caret, 1.0, crate::accent());
                }
            }
            None => {
                painter.circle_filled(center, 3.0, metrics.text_muted.gamma_multiply(0.6));
            }
        }
    }
    let dash = Pos2::new(x_of(half) - split * 0.5 + if boxes { 4.0 } else { 0.0 }, rect.center().y);
    painter.line_segment(
        [dash - Vec2::new(5.0, 0.0), dash + Vec2::new(5.0, 0.0)],
        egui::Stroke::new(2.0, metrics.text_muted.gamma_multiply(0.7)),
    );
    if focused {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(500));
    }
    let entered = response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
    let was_complete =
        ui.data_mut(|data| data.get_temp::<bool>(id.with("complete")).unwrap_or(false));
    ui.data_mut(|data| data.insert_temp(id.with("complete"), complete));
    CodeInput {
        rect: field,
        button,
        submitted: complete && (go.clicked() || entered || !was_complete),
        response,
    }
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
    let text_rect = rect.shrink2(Vec2::new(14.0, 0.0));
    let id = ui.id().with(("fluxa-input", hint));
    let pending = take_pending_text(ui, id, value);
    let hit = field_hit(ui, rect, id);
    let len = value.chars().count();
    let edit = if kind == InputKind::Multiline {
        egui::TextEdit::multiline(value).desired_rows(1)
    } else {
        egui::TextEdit::singleline(value)
    };
    let native = native_covers(text_rect);
    let edit_rect = if kind == InputKind::Multiline {
        text_rect
    } else {
        row_rect(ui, text_rect, crate::fonts::regular(font_size))
    };
    let output = put_edit(
        ui,
        edit_rect,
        id,
        edit.frame(egui::Frame::NONE)
            .password(kind.secret())
            .font(crate::fonts::regular(font_size))
            .vertical_align(egui::Align::Center)
            .hint_text(RichText::new(hint).color(metrics.text_muted))
            .text_color(shown(metrics.text_primary, native))
            .margin(Vec2::ZERO),
    );
    focus_on_hit(ui, &hit, id, len);
    record_native_focus(ui, &output, text_rect, font_size, kind, value, hint);
    let mut response = output.response.response.clone();
    if pending {
        response.mark_changed();
    }
    let masked = if kind.secret() {
        "\u{2022}".repeat(value.chars().count())
    } else {
        value.clone()
    };
    if !native {
        paint_caret(
            ui,
            rect,
            text_rect,
            &masked,
            crate::fonts::regular(font_size),
        );
    }
    let stroke = if response.has_focus() {
        egui::Stroke::new(stroke::hairline(), crate::accent())
    } else {
        egui::Stroke::new(stroke::hairline(), metrics.border)
    };
    ui.painter()
        .rect_stroke(rect, radius, stroke, egui::StrokeKind::Inside);
    response
}
