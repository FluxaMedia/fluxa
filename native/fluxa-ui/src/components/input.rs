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
        let reach = Vec2::new(0.0, inset + Space::S16.px(&crate::metrics::active()));
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
        },
    );
}

fn input_look(ui: &Ui, metrics: &UiMetrics, focused: bool) -> crate::stylesheet::Style {
    crate::stylesheet::resolve(
        ui.ctx(),
        "input",
        Id::new("fluxa-input-look"),
        crate::stylesheet::State {
            focused,
            enabled: true,
            ..Default::default()
        },
        Some(metrics),
    )
}

fn input_stroke(ui: &Ui, metrics: &UiMetrics, focused: bool) -> crate::gfx::Stroke {
    let look = input_look(ui, metrics, focused);
    crate::gfx::Stroke::new(look.stroke, look.border)
}

fn input_fill(ui: &Ui, metrics: &UiMetrics) -> Color32 {
    input_look(ui, metrics, false).fill
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
    let corner = height * metrics.pill_ratio;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    ui.painter()
        .rect_filled(rect, corner, metrics.ink(tone::faint()));
    let icon_color = metrics.ink(tone::muted());
    let placed = flex::row([
        flex::slot(Dp::D16.px()).id(0),
        flex::fill().id(1).height(height),
    ])
    .align(Align::Center)
    .padding(Insets {
        left: radius,
        right: radius * metrics.control_search_trail_ratio,
        ..Insets::default()
    })
    .solve(rect);
    let icon = placed.find(0).unwrap_or(rect).left_center();
    ui.painter().circle_stroke(
        icon - Vec2::splat(metrics.control_search_ring_offset),
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
            crate::fonts::regular(metrics.screen_body_size + 1.0),
        ),
        id,
        crate::gfx::TextEdit::singleline(query)
            .frame(crate::gfx::Frame::NONE)
            .font(crate::fonts::regular(metrics.screen_body_size + 1.0))
            .vertical_align(crate::gfx::Align::Center)
            .hint_text(RichText::new(hint).color(metrics.text_muted))
            .text_color(metrics.text_primary)
            .margin(Vec2::ZERO),
    );
    focus_on_hit(ui, &hit, id, len);
    record_focus(
        ui,
        &output,
        text_rect,
        metrics.screen_body_size + 1.0,
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
            crate::fonts::regular(metrics.screen_body_size + 1.0),
        );
    }
    let stroke = input_stroke(ui, &metrics, response.has_focus());
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
    let layout = *ui.layout();
    let mut child = ui.new_child(
        crate::gfx::UiBuilder::new()
            .id_salt(id)
            .max_rect(rect)
            .layout(layout),
    );
    edit.id(id).desired_width(rect.width()).show(&mut child)
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
    text_edit(ui, value, hint, size, font_size, kind, 0.0, metrics)
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
        0.0,
        metrics,
    )
}

pub(crate) fn text_input_trailing(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    width: f32,
    kind: InputKind,
    trailing: f32,
    metrics: UiMetrics,
) -> Response {
    text_edit(
        ui,
        value,
        hint,
        Vec2::new(width, button_height()),
        metrics.text.body,
        kind,
        trailing,
        metrics,
    )
}

pub(crate) struct MessageInput {
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
    let height = Dp::D48.px();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let id = ui.id().with(("fluxa-message-input", hint));
    take_pending_text(ui, id, value);
    let button_size = height - Space::S6.px(&metrics) * 2.0;
    let placed = flex::row([
        flex::fill().id(0).height(height),
        flex::leaf(Vec2::splat(button_size)).id(1),
    ])
    .align(Align::Center)
    .gap(Space::S10.px(&metrics))
    .padding(Insets {
        left: Dp::D16.px(),
        right: Space::S6.px(&metrics),
        ..Insets::default()
    })
    .solve(rect);
    let button = placed.find(1).unwrap_or(rect);
    let font = crate::fonts::regular(metrics.text.body);
    let text_rect = placed.find(0).unwrap_or(rect);
    let painter = ui.painter();
    painter.rect_filled(rect, height * metrics.pill_ratio, input_fill(ui, &metrics));
    let hit = field_hit(ui, rect, id);
    let len = value.chars().count();
    let color = metrics.text_primary;
    let mut layouter = |ui: &Ui, text: &dyn crate::gfx::TextBuffer, _wrap: f32| {
        let job = crate::emoji::job(text.as_str(), font.clone(), color, f32::INFINITY);
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    };
    let output = put_edit(
        ui,
        row_rect(ui, text_rect, font.clone()),
        id,
        crate::gfx::TextEdit::singleline(value)
            .frame(crate::gfx::Frame::NONE)
            .font(font.clone())
            .vertical_align(crate::gfx::Align::Center)
            .hint_text(RichText::new(hint).color(metrics.text_muted))
            .text_color(color)
            .layouter(&mut layouter)
            .margin(Vec2::ZERO),
    );
    focus_on_hit(ui, &hit, id, len);
    record_focus(
        ui,
        &output,
        text_rect,
        metrics.text.body,
        InputKind::Text,
        value,
        hint,
    );
    let response = output.response.clone();
    crate::emoji::overlay(
        &ui.painter().with_clip_rect(text_rect),
        output.galley_pos,
        &output.galley,
        assets,
    );
    if !response.has_focus() {
        paint_caret(ui, rect, text_rect, value, font);
    }
    let border = input_stroke(ui, &metrics, response.has_focus());
    let painter = ui.painter();
    painter.rect_stroke(
        rect,
        height * metrics.pill_ratio,
        border,
        crate::gfx::StrokeKind::Inside,
    );

    let ready = !value.trim().is_empty();
    let send = ui.interact(button, id.with("send"), Sense::click());
    let (fill, ink) = if ready {
        (Color32::WHITE, Color32::BLACK)
    } else {
        (metrics.border, metrics.text_muted)
    };
    let painter = ui.painter();
    painter.circle_filled(button.center(), button.width() * 0.5, fill);
    let arm = button.width() * metrics.control_step_arm;
    let center = button.center();
    let arrow = crate::gfx::Stroke::new(metrics.control_send_stroke, ink);
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
    let entered =
        response.lost_focus() && ui.input(|input| input.key_pressed(crate::gfx::Key::Enter));
    if entered || send.clicked() {
        response.request_focus();
    }
    ui.advance_cursor_after_rect(rect);
    MessageInput {
        submitted: ready && (entered || send.clicked()),
    }
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
        row_rect(ui, field, crate::fonts::regular(metrics.text.body)),
        id,
        crate::gfx::TextEdit::singleline(value)
            .frame(crate::gfx::Frame::NONE)
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

    let corner = height * metrics.pill_ratio;
    let painter = ui.painter();
    if !boxes {
        painter.rect_filled(rect, corner, input_fill(ui, &metrics));
        let border = input_stroke(ui, &metrics, focused);
        painter.rect_stroke(rect, corner, border, crate::gfx::StrokeKind::Inside);
    }

    let button = flex::anchored(
        frame.find(1).unwrap_or(rect),
        Align2::CENTER_CENTER,
        Vec2::splat(height - Space::S6.px(&metrics) * 2.0),
        0.0,
    );
    let go = ui.interact(button, id.with("go"), Sense::click());
    let (fill, ink) = if complete {
        (Color32::WHITE, Color32::BLACK)
    } else {
        (metrics.border, metrics.text_muted)
    };
    let painter = ui.painter();
    painter.circle_filled(button.center(), button.width() * 0.5, fill);
    let arm = button.width() * metrics.control_step_arm;
    let center = button.center();
    let arrow = crate::gfx::Stroke::new(metrics.control_send_stroke, ink);
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
                Vec2::new(cell_width, metrics.input_cell_height.of(cell_width)),
                0.0,
            );
            let current = focused && index == count;
            painter.rect_filled(cell, Dp::D12.px(), Color32::from_white_alpha(13));
            let edge = if current {
                crate::gfx::Stroke::new(metrics.control_code_edge, Color32::from_white_alpha(170))
            } else {
                crate::gfx::Stroke::new(stroke::hairline(), Color32::from_white_alpha(26))
            };
            painter.rect_stroke(cell, Dp::D12.px(), edge, crate::gfx::StrokeKind::Inside);
            if current {
                painter.rect_stroke(
                    cell.expand(metrics.control_code_halo),
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
                    metrics.text_primary,
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
                painter.circle_filled(center, Dp::D3.px(), metrics.text_muted.gamma_multiply(0.6));
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
        crate::gfx::Stroke::new(2.0, metrics.text_muted.gamma_multiply(0.7)),
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

pub(crate) fn text_area(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    width: f32,
    lines: usize,
    limit: usize,
    metrics: UiMetrics,
) -> Response {
    let font = crate::fonts::regular(metrics.text.body);
    let row = ui.fonts_mut(|fonts| fonts.row_height(&font));
    let pad = Insets::xy(Space::S14.px(&metrics), Space::S12.px(&metrics));
    let height = row * lines as f32 + pad.top + pad.bottom;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    ui.painter()
        .rect_filled(rect, radius::control(), input_fill(ui, &metrics));
    let text_rect = flex::stack(Align2::LEFT_TOP, [flex::fill().id(0)])
        .padding(pad)
        .solve(rect)
        .find(0)
        .unwrap_or(rect);
    let id = ui.id().with(("fluxa-text-area", hint));
    let pending = take_pending_text(ui, id, value);
    let hit = field_hit(ui, rect, id);
    let len = value.chars().count();
    let mut child = ui.new_child(
        crate::gfx::UiBuilder::new()
            .id_salt(id)
            .max_rect(text_rect)
            .layout(*ui.layout()),
    );
    let output = super::pan_area(
        &mut child,
        id.with("scroll"),
        |area| {
            area.max_height(text_rect.height())
                .auto_shrink([false, false])
        },
        |ui| {
            crate::gfx::TextEdit::multiline(value)
                .id(id)
                .frame(crate::gfx::Frame::NONE)
                .char_limit(limit)
                .desired_rows(lines)
                .desired_width(text_rect.width())
                .font(font.clone())
                .hint_text(RichText::new(hint).color(metrics.text_muted))
                .text_color(metrics.text_primary)
                .margin(Vec2::ZERO)
                .show(ui)
        },
    )
    .inner;
    focus_on_hit(ui, &hit, id, len);
    record_focus(
        ui,
        &output,
        text_rect,
        metrics.text.body,
        InputKind::Multiline,
        value,
        hint,
    );
    let mut response = output.response.clone();
    if pending {
        response.mark_changed();
    }
    let stroke = input_stroke(ui, &metrics, response.has_focus());
    ui.painter().rect_stroke(
        rect,
        radius::control(),
        stroke,
        crate::gfx::StrokeKind::Inside,
    );
    response
}

fn text_edit(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    size: Vec2,
    font_size: f32,
    kind: InputKind,
    trailing: f32,
    metrics: UiMetrics,
) -> Response {
    let radius = radius::control();
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_filled(rect, radius, input_fill(ui, &metrics));
    let side = Space::S14.px(&metrics);
    let placed = flex::row([
        flex::fill().id(0).height(rect.height()),
        flex::slot(trailing).id(1),
    ])
    .align(Align::Center)
    .padding(Insets::xy(side, 0.0))
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
    let output = put_edit(
        ui,
        edit_rect,
        id,
        edit.frame(crate::gfx::Frame::NONE)
            .password(kind.secret())
            .font(crate::fonts::regular(font_size))
            .vertical_align(crate::gfx::Align::Center)
            .hint_text(RichText::new(hint).color(metrics.text_muted))
            .text_color(metrics.text_primary)
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
    let stroke = input_stroke(ui, &metrics, response.has_focus());
    ui.painter()
        .rect_stroke(rect, radius, stroke, crate::gfx::StrokeKind::Inside);
    response
}
