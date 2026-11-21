use super::*;
use crate::Space;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets};

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
    if let Some(kind) = crate::skin::field_kind(node) {
        return kind;
    }
    match node {
        crate::NODE_LIBRARY_SEARCH | crate::NODE_DISCOVER_SEARCH | crate::NODE_SETTINGS_SEARCH => {
            InputKind::Search
        }
        crate::NODE_SETTINGS_SERVER_PASSWORD => InputKind::Password,
        crate::NODE_SETTINGS_ADDON_URL
        | crate::NODE_SETTINGS_PLUGIN_URL
        | crate::NODE_SETTINGS_STREAMKIT_ADDON_URL
        | crate::NODE_SETTINGS_SERVER_ADDRESS
        | crate::NODE_SETTINGS_POSTER_URL => InputKind::Url,
        crate::NODE_REPORT_BODY => InputKind::Multiline,
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
    pub composing: Option<[usize; 2]>,
    pub toolbar: Option<Rect>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingField {
    pub id: u64,
    pub text: String,
    pub selection: [usize; 2],
    pub composing: Option<[usize; 2]>,
}

#[derive(Clone)]
struct Composing {
    range: [usize; 2],
    text: String,
}

const CARET_FOCUS: &str = "fluxa-input-caret";
const CARET_RECT: &str = "fluxa-caret-rect";
const FOCUSED_FIELD: &str = "fluxa-focused-field";
const PENDING_TEXT: &str = "fluxa-pending-field-text";
const EGUI_FIELD_TAG: u64 = 1 << 62;

fn field_id(id: Id) -> u64 {
    (id.value() & (EGUI_FIELD_TAG - 1)) | EGUI_FIELD_TAG
}

pub fn set_pending_field(context: &crate::gfx::Context, pending: Option<PendingField>) {
    context.data_mut(|data| {
        data.remove::<PendingField>(Id::new(PENDING_TEXT));
        if let Some(pending) = pending {
            data.insert_temp(Id::new(PENDING_TEXT), pending);
        }
    });
}

fn take_pending_text(ui: &Ui, id: Id, value: &mut String) -> bool {
    let pending = ui.ctx().data_mut(|data| {
        let key = Id::new(PENDING_TEXT);
        match data.get_temp::<PendingField>(key) {
            Some(pending) if pending.id == field_id(id) => {
                data.remove::<PendingField>(key);
                Some(pending)
            }
            _ => None,
        }
    });
    let Some(pending) = pending else {
        return false;
    };
    let [start, end] = pending.selection;
    *value = pending.text.clone();
    crate::gfx::set_selection(ui.ctx(), id, start, end);
    ui.ctx().data_mut(|data| match pending.composing {
        Some(range) => {
            data.insert_temp(
                composing_key(id),
                Composing {
                    range,
                    text: pending.text,
                },
            );
        }
        None => {
            data.remove::<Composing>(composing_key(id));
        }
    });
    true
}

fn composing_key(id: Id) -> Id {
    id.with("fluxa-composing")
}

fn composing_range(ui: &Ui, id: Id, value: &str) -> Option<[usize; 2]> {
    ui.ctx().data_mut(|data| {
        let composing = data.get_temp::<Composing>(composing_key(id))?;
        if composing.text == value {
            Some(composing.range)
        } else {
            data.remove::<Composing>(composing_key(id));
            None
        }
    })
}

fn paint_composing(ui: &Ui, output: &crate::gfx::TextEditOutput, range: [usize; 2]) {
    let from = output.rect_at(range[0]);
    let to = output.rect_at(range[1]);
    let stroke = crate::gfx::Stroke::new(stroke::icon(), text_primary());
    let painter = ui.painter().with_clip_rect(output.response.rect);
    if (from.max.y - to.max.y).abs() < 1.0 {
        painter.line_segment([from.left_bottom(), to.left_bottom()], stroke);
        return;
    }
    let left = output.response.rect.left();
    let right = output.response.rect.right();
    painter.line_segment(
        [from.left_bottom(), crate::gfx::Pos2::new(right, from.max.y)],
        stroke,
    );
    painter.line_segment(
        [crate::gfx::Pos2::new(left, to.max.y), to.left_bottom()],
        stroke,
    );
}

pub fn set_input_caret(context: &crate::gfx::Context, rect: Option<Rect>) {
    context.data_mut(|data| {
        data.insert_temp(Id::new(CARET_FOCUS), rect);
        data.remove::<Rect>(Id::new(CARET_RECT));
        data.remove::<FocusedField>(Id::new(FOCUSED_FIELD));
    });
    super::input_touch::clear_grips(context);
}

pub fn caret_rect(context: &crate::gfx::Context) -> Option<Rect> {
    context.data(|data| data.get_temp::<Rect>(Id::new(CARET_RECT)))
}

pub fn focused_field(context: &crate::gfx::Context) -> Option<FocusedField> {
    context.data(|data| data.get_temp::<FocusedField>(Id::new(FOCUSED_FIELD)))
}

fn record_caret(ui: &Ui, caret: Rect) {
    ui.ctx()
        .data_mut(|data| data.insert_temp(Id::new(CARET_RECT), caret));
}

const KEYBOARD_INSET: &str = "fluxa-keyboard-inset";

pub fn set_keyboard_inset(context: &crate::gfx::Context, inset: f32) {
    context.data_mut(|data| data.insert_temp(Id::new(KEYBOARD_INSET), inset));
}

pub fn keyboard_inset(context: &crate::gfx::Context) -> f32 {
    context.data(|data| data.get_temp::<f32>(Id::new(KEYBOARD_INSET)).unwrap_or(0.0))
}

fn lift_above_keyboard(ui: &Ui, response: &Response, rect: Rect) {
    let inset = keyboard_inset(ui.ctx());
    if inset <= 0.0 {
        return;
    }
    let key = Id::new("fluxa-keyboard-lifted");
    let changed = ui.data_mut(|data| {
        let before = data.get_temp::<f32>(key);
        data.insert_temp(key, inset);
        before != Some(inset)
    });
    if changed || response.gained_focus() {
        let reach = Vec2::new(0.0, inset + Space::S16.px());
        ui.scroll_to_rect(Rect::from_min_max(rect.min, rect.max + reach), None);
    }
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
    let millis = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis());
    if millis / 530 % 2 == 1 {
        return;
    }
    ui.painter().line_segment(
        [
            crate::gfx::Pos2::new(x, text_rect.center().y - half),
            crate::gfx::Pos2::new(x, text_rect.center().y + half),
        ],
        crate::gfx::Stroke::new(stroke::icon(), text_primary()),
    );
}

fn record_focus(
    ui: &Ui,
    output: &crate::gfx::TextEditOutput,
    rect: Rect,
    size: f32,
    kind: InputKind,
    value: &str,
    hint: &str,
) {
    if !output.response.has_focus() {
        return;
    }
    lift_above_keyboard(ui, &output.response, rect);
    let range = output.cursor_range.unwrap_or_default();
    let [start, end] = [range.primary, range.secondary];
    let composing = composing_range(ui, output.response.id, value);
    if let Some(composing) = composing {
        paint_composing(ui, output, composing);
    }
    if let Some(caret) = output.caret {
        record_caret(ui, caret);
    }
    let toolbar = super::input_touch::drive(ui, output, rect, value);
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
            composing,
            toolbar,
        },
    );
}

fn input_look(ui: &Ui, focused: bool) -> crate::stylesheet::Style {
    crate::stylesheet::resolve(
        ui.ctx(),
        "input",
        Id::new("fluxa-input-look"),
        crate::stylesheet::State {
            focused,
            enabled: true,
            ..Default::default()
        },
    )
}

fn input_stroke(ui: &Ui, focused: bool) -> crate::gfx::Stroke {
    let look = input_look(ui, focused);
    crate::gfx::Stroke::new(look.stroke, look.border)
}

fn input_fill(ui: &Ui) -> Color32 {
    input_look(ui, false).fill
}

pub(crate) fn search_field(
    ui: &mut Ui,
    query: &mut String,
    hint: &str,
    width: f32,
    height: f32,
) -> Response {
    let radius = height * 0.5;
    let corner = height * crate::vars::px("pill-ratio");
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    ui.painter()
        .rect_filled(rect, corner, crate::metrics::ink(tone::faint()));
    let icon_color = crate::metrics::ink(tone::muted());
    let placed = flex::row([
        flex::slot(Dp::D16.px()).id(0),
        flex::fill().id(1).height(height),
    ])
    .align(Align::Center)
    .padding(Insets {
        left: radius,
        right: radius * crate::vars::px("control-search-trail-ratio"),
        ..Insets::default()
    })
    .solve(rect);
    let icon = placed.find(0).unwrap_or(rect).left_center();
    ui.painter().circle_stroke(
        icon - Vec2::splat(crate::vars::px("control-search-ring-offset")),
        Dp::D6.px(),
        crate::gfx::Stroke::new(stroke::icon(), icon_color),
    );
    ui.painter().line_segment(
        [
            icon + Vec2::splat(Dp::D3.px()),
            icon + Vec2::splat(Dp::D6.px()),
        ],
        crate::gfx::Stroke::new(stroke::icon(), icon_color),
    );
    let text_rect = placed.find(1).unwrap_or(rect);
    let id = ui.id().with(("fluxa-input", hint));
    let pending = take_pending_text(ui, id, query);
    let hit = field_hit(ui, rect, id);
    let len = query.chars().count();
    let output = put_edit(
        ui,
        row_rect(
            ui,
            text_rect,
            crate::fonts::regular(crate::vars::px("screen-body-size") + 1.0),
        ),
        id,
        crate::gfx::TextEdit::singleline(query)
            .frame(crate::gfx::Frame::NONE)
            .font(crate::fonts::regular(
                crate::vars::px("screen-body-size") + 1.0,
            ))
            .vertical_align(crate::gfx::Align::Center)
            .hint_text(RichText::new(hint).color(crate::metrics::theme().text_muted))
            .text_color(crate::metrics::theme().text_primary)
            .margin(Vec2::ZERO),
    );
    focus_on_hit(ui, &hit, id, len);
    record_focus(
        ui,
        &output,
        text_rect,
        crate::vars::px("screen-body-size") + 1.0,
        InputKind::Search,
        query,
        hint,
    );
    let mut response = output.response.clone();
    if pending {
        response.mark_changed();
    }
    if !response.has_focus() {
        paint_caret(
            ui,
            rect,
            text_rect,
            query,
            crate::fonts::regular(crate::vars::px("screen-body-size") + 1.0),
        );
    }
    let stroke = input_stroke(ui, response.has_focus());
    ui.painter()
        .rect_stroke(rect, corner, stroke, crate::gfx::StrokeKind::Inside);
    response
}

fn field_hit(ui: &Ui, rect: Rect, id: Id) -> Response {
    ui.interact(rect, id.with("hit"), Sense::CLICK)
}

fn focus_on_hit(ui: &Ui, hit: &Response, id: Id, len: usize) {
    if !hit.clicked() || ui.has_focus(id) {
        return;
    }
    ui.request_focus(id);
    crate::gfx::set_cursor(ui.ctx(), id, len);
}

fn row_rect(ui: &Ui, rect: Rect, font: FontId) -> Rect {
    let row = ui.fonts_mut(|fonts| fonts.row_height(&font));
    crate::flex::centered(rect, Vec2::new(rect.width(), row))
}

fn put_edit(
    ui: &mut Ui,
    rect: Rect,
    id: Id,
    edit: crate::gfx::TextEdit<'_>,
) -> crate::gfx::TextEditOutput {
    ui.ctx()
        .keep_focus_on_click(super::input_touch::keeps_focus(ui.ctx()));
    let layout = *ui.layout();
    let mut child = ui.new_child(
        crate::gfx::UiBuilder::new()
            .id_salt(id)
            .max_rect(rect)
            .layout(layout),
    );
    edit.id(id).desired_width(rect.width()).show(&mut child)
}

fn put_scrolling_edit(
    ui: &mut Ui,
    rect: Rect,
    id: Id,
    edit: crate::gfx::TextEdit<'_>,
) -> crate::gfx::TextEditOutput {
    ui.ctx()
        .keep_focus_on_click(super::input_touch::keeps_focus(ui.ctx()));
    let layout = *ui.layout();
    let mut child = ui.new_child(
        crate::gfx::UiBuilder::new()
            .id_salt(id)
            .max_rect(rect)
            .layout(layout),
    );
    let width = rect.width();
    super::pan_area(
        &mut child,
        id,
        |area| {
            area.scroll([false, true])
                .scroll_source(crate::gfx::ScrollSource::MOUSE_WHEEL)
                .auto_shrink([false, false])
        },
        |ui| edit.id(id).desired_width(width).show(ui),
    )
    .inner
}

pub(crate) fn text_input(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    width: f32,
    kind: InputKind,
) -> Response {
    text_edit(
        ui,
        value,
        hint,
        Vec2::new(width, button_height()),
        crate::vars::px("text-body"),
        kind,
        0.0,
    )
}

pub(crate) fn text_area(ui: &mut Ui, value: &mut String, hint: &str, size: Vec2) -> Response {
    text_edit(
        ui,
        value,
        hint,
        size,
        crate::vars::px("text-body"),
        InputKind::Multiline,
        0.0,
    )
}

const DASH_SLOT: usize = usize::MAX;

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

pub(crate) fn code_input_styled(
    ui: &mut Ui,
    value: &mut String,
    length: usize,
    width: f32,
    look: CodeLook,
) -> CodeInput {
    let height = Dp::D56.px();
    let boxes = look == CodeLook::Boxes;
    let pad = if boxes { 0.0 } else { Dp::D24.px() };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let id = ui.id().with("fluxa-code-input");
    take_pending_text(ui, id, value);
    let frame = flex::row([
        flex::fill().id(0).height(height),
        flex::leaf(Vec2::splat(height)).id(1),
    ])
    .align(Align::Center)
    .padding(Insets {
        left: pad,
        ..Insets::default()
    })
    .solve(rect);
    let field = frame.find(0).unwrap_or(rect);
    let hit = field_hit(ui, rect, id);
    let output = put_edit(
        ui,
        row_rect(
            ui,
            field,
            crate::fonts::regular(crate::vars::px("text-body")),
        ),
        id,
        crate::gfx::TextEdit::singleline(value)
            .frame(crate::gfx::Frame::NONE)
            .char_limit(length)
            .font(crate::fonts::regular(crate::vars::px("text-body")))
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
    record_focus(
        ui,
        &output,
        field,
        if boxes { Dp::D20.px() } else { Dp::D24.px() },
        InputKind::Text,
        value,
        "code",
    );
    let response = output.response.clone();
    let focused = response.has_focus();
    let complete = count == length;

    let corner = height * crate::vars::px("pill-ratio");
    let painter = ui.painter();
    if !boxes {
        painter.rect_filled(rect, corner, input_fill(ui));
        let border = input_stroke(ui, focused);
        painter.rect_stroke(rect, corner, border, crate::gfx::StrokeKind::Inside);
    }

    let button = flex::anchored(
        frame.find(1).unwrap_or(rect),
        Align2::CENTER_CENTER,
        Vec2::splat(height - Space::S6.px() * 2.0),
        0.0,
    );
    let go = ui.interact(button, id.with("go"), Sense::click());
    let (fill, ink) = if complete {
        (Color32::WHITE, Color32::BLACK)
    } else {
        (
            crate::metrics::theme().border,
            crate::metrics::theme().text_muted,
        )
    };
    let painter = ui.painter();
    painter.circle_filled(button.center(), button.width() * 0.5, fill);
    let arm = button.width() * crate::vars::px("control-step-arm");
    let center = button.center();
    let arrow = crate::gfx::Stroke::new(crate::vars::px("control-send-stroke"), ink);
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

    let font = crate::fonts::bold(if boxes { Dp::D20.px() } else { Dp::D24.px() });
    let split = if boxes { Dp::D20.px() } else { Dp::D20.px() };
    let slot = if boxes {
        ((width - height - split - Dp::D4.px()) / length as f32).clamp(Dp::D28.px(), Dp::D48.px())
    } else {
        Dp::D28.px()
    };
    let half = length / 2;
    let mut cells = Vec::new();
    for index in 0..length {
        if index == half {
            cells.push(flex::slot(split).id(DASH_SLOT));
        }
        cells.push(flex::leaf(Vec2::new(slot, height)).width(slot).id(index));
    }
    if half >= length {
        cells.push(flex::slot(split).id(DASH_SLOT));
    }
    let digits = flex::row(cells)
        .align(Align::Center)
        .padding(Insets {
            left: pad,
            ..Insets::default()
        })
        .solve(rect);
    for index in 0..length {
        let cell_slot = digits.find(index).unwrap_or(rect);
        let center = cell_slot.center();
        if boxes {
            let cell_width = slot - Dp::D6.px();
            let cell = flex::anchored(
                cell_slot,
                Align2::CENTER_CENTER,
                Vec2::new(
                    cell_width,
                    crate::vars::scaled("input-cell-height", cell_width),
                ),
                0.0,
            );
            let current = focused && index == count;
            painter.rect_filled(cell, Dp::D12.px(), Color32::from_white_alpha(13));
            let edge = if current {
                crate::gfx::Stroke::new(
                    crate::vars::px("control-code-edge"),
                    Color32::from_white_alpha(170),
                )
            } else {
                crate::gfx::Stroke::new(stroke::hairline(), Color32::from_white_alpha(26))
            };
            painter.rect_stroke(cell, Dp::D12.px(), edge, crate::gfx::StrokeKind::Inside);
            if current {
                painter.rect_stroke(
                    cell.expand(crate::vars::px("control-code-halo")),
                    Dp::D14.px(),
                    crate::gfx::Stroke::new(Dp::D2.px(), Color32::from_white_alpha(22)),
                    crate::gfx::StrokeKind::Outside,
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
                    crate::metrics::theme().text_primary,
                );
            }
            None if focused && index == count => {
                let blink = (ui.input(|input| input.time()) * 2.0) as i64 % 2 == 0;
                if blink {
                    let caret = flex::anchored(
                        cell_slot,
                        Align2::CENTER_CENTER,
                        Vec2::new(Dp::D2.px(), Dp::D24.px()),
                        0.0,
                    );
                    painter.rect_filled(caret, 1.0, crate::accent());
                }
            }
            None => {
                painter.circle_filled(
                    center,
                    Dp::D3.px(),
                    crate::metrics::theme().text_muted.gamma_multiply(0.6),
                );
            }
        }
    }
    let gap = digits.find(DASH_SLOT).unwrap_or(rect);
    let shift = if boxes { Dp::D4.px() } else { 0.0 };
    let dash = flex::anchored(
        gap.translate(Vec2::new(shift, 0.0)),
        Align2::CENTER_CENTER,
        Vec2::new(Dp::D4.px() * 2.0, 0.0),
        0.0,
    );
    painter.line_segment(
        [dash.left_center(), dash.right_center()],
        crate::gfx::Stroke::new(2.0, crate::metrics::theme().text_muted.gamma_multiply(0.7)),
    );
    if focused {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(500));
    }
    let entered =
        response.lost_focus() && ui.input(|input| input.key_pressed(crate::gfx::Key::Enter));
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
    trailing: f32,
) -> Response {
    let radius = radius::control();
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, radius, input_fill(ui));
    let side = Space::S14.px();
    let placed = flex::row([
        flex::fill().id(0).height(rect.height()),
        flex::slot(trailing).id(1),
    ])
    .align(Align::Center)
    .padding(Insets::xy(
        side,
        if kind == InputKind::Multiline {
            side
        } else {
            0.0
        },
    ))
    .solve(rect);
    let text_rect = placed.find(0).unwrap_or(rect);
    let id = ui.id().with(("fluxa-input", hint));
    let pending = take_pending_text(ui, id, value);
    let hit_rect = flex::row([flex::fill().id(0), flex::slot(trailing)])
        .solve(rect)
        .find(0)
        .unwrap_or(rect);
    let hit = field_hit(ui, hit_rect, id);
    let len = value.chars().count();
    let edit = if kind == InputKind::Multiline {
        crate::gfx::TextEdit::multiline(value).desired_rows(1)
    } else {
        crate::gfx::TextEdit::singleline(value)
    };
    let edit_rect = if kind == InputKind::Multiline {
        text_rect
    } else {
        row_rect(ui, text_rect, crate::fonts::regular(font_size))
    };
    let put = if kind == InputKind::Multiline {
        put_scrolling_edit
    } else {
        put_edit
    };
    let output = put(
        ui,
        edit_rect,
        id,
        edit.frame(crate::gfx::Frame::NONE)
            .password(kind.secret())
            .font(crate::fonts::regular(font_size))
            .vertical_align(if kind == InputKind::Multiline {
                crate::gfx::Align::Min
            } else {
                crate::gfx::Align::Center
            })
            .hint_text(RichText::new(hint).color(crate::metrics::theme().text_muted))
            .text_color(crate::metrics::theme().text_primary)
            .margin(Vec2::ZERO),
    );
    focus_on_hit(ui, &hit, id, len);
    record_focus(ui, &output, text_rect, font_size, kind, value, hint);
    let mut response = output.response.clone();
    if pending {
        response.mark_changed();
    }
    let masked = if kind.secret() {
        "\u{2022}".repeat(value.chars().count())
    } else {
        value.clone()
    };
    if !response.has_focus() {
        paint_caret(
            ui,
            rect,
            text_rect,
            &masked,
            crate::fonts::regular(font_size),
        );
    }
    let stroke = input_stroke(ui, response.has_focus());
    ui.painter()
        .rect_stroke(rect, radius, stroke, crate::gfx::StrokeKind::Inside);
    response
}
