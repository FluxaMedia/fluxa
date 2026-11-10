use super::*;
use crate::dp::Dp;
use crate::flex::{self, Insets, Node};
use std::cell::RefCell;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PanelRow {
    pub label: String,
    pub value: Option<String>,
    pub primary: bool,
    pub selected: bool,
    pub heading: bool,
    pub group: bool,
    pub switch: Option<bool>,
    pub thumbnail: Option<String>,
    pub detail: Option<String>,
    pub frame: Option<f64>,
    pub badge: Option<String>,
    pub icon: Option<String>,
    pub chevron: bool,
    pub step: Option<PanelStep>,
    pub sync: Option<SyncRole>,
    pub subline: Option<String>,
    pub separated: bool,
    pub back: bool,
    pub chips: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncRole {
    Back,
    Reset,
    Reading,
    Minus,
    Plus,
    Auto,
    Method,
    Choice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelStep {
    Value,
    Minus,
    Plus,
    Swatches(usize),
    Swatch,
    Choices(usize),
    Choice,
}

impl PanelStep {
    fn follows_lead(self) -> bool {
        matches!(self, Self::Minus | Self::Plus | Self::Swatch | Self::Choice)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerPanel {
    pub title: String,
    pub message: Option<String>,
    pub rows: Vec<PanelRow>,
    pub list: bool,
    pub look: Option<crate::SubtitleLook>,
    pub chips: bool,
}

pub(super) fn draw_panel(
    chrome: &Chrome,
    ui: &mut crate::gfx::Ui,
    layout: &mut HomeLayout,
    panel: &PlayerPanel,
    assets: &mut impl HomeAssets,
    slide: f32,
) {
    if panel.rows.iter().any(|row| row.sync.is_some()) {
        super::sync_panel::draw(chrome, ui, layout, panel, slide);
        return;
    }
    if panel.chips && chrome.viewport.form_factor == UiFormFactor::Mobile {
        draw_chips(chrome, ui, layout, panel, slide);
        return;
    }
    if panel.list {
        draw_list(chrome, ui, layout, panel, assets, slide);
        return;
    }
    let rows = panel.rows.len().min(PLAYER_PANEL_ROW_LIMIT);
    let language = chrome.player.language.clone();
    crate::kit::provide(chrome.context, UiMetrics::for_viewport(chrome.viewport));
    crate::kit::dialog(chrome.context, "fluxa-player-panel", |ui| {
        let responses = RefCell::new(vec![None; rows + 1]);
        let mut items = vec![crate::kit::widget(ui, |ui| {
            crate::kit::text(&panel.title, crate::kit::Type::Subtitle)
                .bold()
                .show(ui);
        })];
        if let Some(message) = panel.message.as_deref() {
            items.push(crate::kit::layout_items::space(ui, Space::S6));
            items.push(crate::kit::widget(ui, move |ui| {
                crate::kit::text(message, crate::kit::Type::Meta)
                    .role(crate::kit::Role::Secondary)
                    .show(ui);
            }));
        }
        items.push(crate::kit::layout_items::space(ui, Space::S14));
        for (index, row) in panel.rows.iter().take(rows).enumerate() {
            let label = match row.value.as_deref() {
                Some(value) => format!("{}  {}", row.label, value),
                None => row.label.clone(),
            };
            let responses = &responses;
            items.push(crate::kit::widget(ui, move |ui| {
                let button = crate::kit::button(&label);
                let response = if row.primary {
                    button.primary()
                } else {
                    button
                }
                .show(ui);
                responses.borrow_mut()[index] = Some(response);
            }));
            items.push(crate::kit::layout_items::space(ui, Space::S8));
        }
        let close_label = localized("common.close", &language);
        let responses_ref = &responses;
        items.push(crate::kit::widget(ui, move |ui| {
            let close = crate::kit::button(&close_label).show(ui);
            responses_ref.borrow_mut()[rows] = Some(close);
        }));
        crate::kit::lay(ui, items);
        for (index, response) in responses.into_inner().into_iter().enumerate() {
            let Some(response) = response else {
                continue;
            };
            let node = if index == rows {
                NODE_PLAYER_PANEL_CLOSE
            } else {
                NODE_PLAYER_PANEL_ROW_BASE + index as u64
            };
            crate::components::focus_action(layout, node, &response);
        }
    });
}

const HEAD_BACK: usize = usize::MAX;
const HEAD_TITLE: usize = usize::MAX - 1;
const CURRENT: usize = usize::MAX - 2;
const ART: usize = 0;
const TITLE: usize = 1;
const DETAIL: usize = 2;
const LABEL: usize = 3;
const BADGE: usize = 4;
const ICON: usize = 5;
const CHEVRON: usize = 6;
const SWITCH: usize = 7;
const CHECK: usize = 8;
const VALUE: usize = 9;
const SUBLINE: usize = 10;
const MINUS: usize = 11;
const VALUE_SLOT: usize = 12;
const PLUS: usize = 13;
const SWATCH: usize = 100;
const CHIP: usize = 200;

pub(super) fn dock(bounds: Rect, align: Align2, size: Vec2, margin: Insets) -> Rect {
    flex::stack(align, [flex::leaf(size).id(0)])
        .padding(margin)
        .solve(bounds)
        .find(0)
        .unwrap_or(bounds)
}

pub(super) fn inset_rect(bounds: Rect, margin: Insets) -> Rect {
    flex::stack(Align2::LEFT_TOP, [flex::fill().id(0)])
        .padding(margin)
        .solve(bounds)
        .find(0)
        .unwrap_or(bounds)
}

struct Cx<'a, 'c> {
    chrome: &'a Chrome<'c>,
    veil: &'a crate::gfx::Painter,
    m: &'a UiMetrics,
    panel: &'a PlayerPanel,
    col_w: f32,
    row_h: f32,
    text: f32,
}

struct Entry<'a> {
    index: usize,
    row: &'a PanelRow,
    rect: Rect,
    tint: Color32,
}

impl Cx<'_, '_> {
    fn node(&self, entry: &Entry) -> u64 {
        NODE_PLAYER_PANEL_ROW_BASE + entry.index as u64
    }

    fn clipped(&self, text: &str, font: &FontId, width: f32) -> String {
        truncate_to_width(self.veil, text, font, width.max(0.0))
    }

    fn step(
        &self,
        ui: &mut crate::gfx::Ui,
        layout: &mut HomeLayout,
        index: usize,
        hit: Rect,
        hit_pad: f32,
    ) -> crate::gfx::Response {
        let node = NODE_PLAYER_PANEL_ROW_BASE + index as u64;
        let response = ui.interact(
            hit.expand(hit_pad),
            Id::new(("fluxa-player-panel-step", node)),
            Sense::click(),
        );
        layout.focusable.push((node, hit));
        response
    }
}

fn trail(size: Vec2, gap: f32, id: usize) -> Node {
    flex::column([flex::leaf(size).id(id)])
        .padding(Insets {
            left: gap,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        })
        .align(crate::flex::Align::Start)
}

fn rich_row(
    cx: &Cx,
    layout: &mut HomeLayout,
    assets: &mut impl HomeAssets,
    entry: &Entry,
    response: &crate::gfx::Response,
) {
    let m = cx.m;
    let (row, tint) = (entry.row, entry.tint);
    let s8 = Space::S8.px(m);
    let s12 = Space::S12.px(m);
    let art_h = cx.row_h - s8 * 2.0;
    let art_w = art_h * m.panel_rich_aspect;
    let text_w = cx.col_w - s8 - s12 - art_w - s12;
    let title_font = FontId::proportional(cx.text);
    let title = cx.clipped(&row.label, &title_font, text_w);
    let mut lines = vec![flex::text(cx.veil, &title, &title_font).id(TITLE)];
    let detail = row.detail.as_deref().map(|detail| {
        let mut job = crate::gfx::text::LayoutJob::simple(
            detail.to_owned(),
            FontId::proportional(m.panel_rich_detail_font),
            tint.gamma_multiply(0.6),
            text_w,
        );
        job.wrap.max_rows = m.panel_rich_detail_rows;
        cx.veil.layout_job(job)
    });
    if let Some(galley) = &detail {
        lines.push(flex::leaf(galley.size()).id(DETAIL));
    }
    let placed = flex::row([
        flex::leaf(Vec2::new(art_w, art_h)).id(ART),
        flex::column(lines)
            .gap(Space::S4.px(m))
            .padding(Insets {
                left: 0.0,
                top: Space::S4.px(m),
                right: 0.0,
                bottom: 0.0,
            })
            .grow(1.0),
    ])
    .gap(s12)
    .padding(Insets {
        left: s8,
        top: s8,
        right: s12,
        bottom: s8,
    })
    .align(crate::flex::Align::Start)
    .solve(entry.rect);
    if let Some(art) = placed.find(ART) {
        cx.veil
            .rect_filled(art, radius::control(), crate::stylesheet::color("well"));
        if let Some(time) = row.frame {
            let frame = cx
                .chrome
                .player
                .filmstrip
                .iter()
                .filter(|frame| (frame.0 - time).abs() < 4.0)
                .min_by(|a, b| (a.0 - time).abs().total_cmp(&(b.0 - time).abs()));
            if let Some((_, texture, _)) = frame {
                cx.veil.image(*texture, art, full_uv(), Color32::WHITE);
            }
        }
        components::rounded_artwork(
            cx.veil,
            art,
            Dp::D6.px(),
            row.thumbnail.as_deref(),
            [320, 180],
            ArtworkPriority::Visible,
            Color32::WHITE,
            assets,
        );
    }
    if let Some(slot) = placed.find(TITLE) {
        cx.veil
            .text(slot.min, Align2::LEFT_TOP, title, title_font, tint);
    }
    if let (Some(galley), Some(slot)) = (detail, placed.find(DETAIL)) {
        cx.veil.galley(slot.min, galley, tint);
    }
    if response.clicked() {
        layout.activated = Some(cx.node(entry));
    }
}

fn swatch_row(
    cx: &Cx,
    ui: &mut crate::gfx::Ui,
    layout: &mut HomeLayout,
    entry: &Entry,
    count: usize,
) {
    let m = cx.m;
    let swatches = flex::row(
        (1..=count).map(|offset| flex::leaf(Vec2::splat(Dp::D20.px())).id(SWATCH + offset)),
    )
    .gap(Dp::D4.px())
    .align(crate::flex::Align::Start);
    let font = FontId::proportional(cx.text);
    let placed = flex::row([
        flex::text(cx.veil, &entry.row.label, &font).id(LABEL),
        flex::fill(),
        swatches,
    ])
    .align(flex::Align::Center)
    .padding(Insets::xy(Space::S8.px(m), 0.0))
    .solve(entry.rect);
    for offset in (1..=count).rev() {
        let swatch = &cx.panel.rows[entry.index + offset];
        let Some(hit) = placed.find(SWATCH + offset) else {
            continue;
        };
        let response = cx.step(ui, layout, entry.index + offset, hit, Dp::D3.px());
        let color = swatch
            .value
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(Color32::GRAY);
        let radius = hit.width() / 2.0;
        cx.veil.circle_filled(hit.center(), radius, color);
        cx.veil.circle_stroke(
            hit.center(),
            radius,
            crate::gfx::Stroke::new(stroke::hairline(), Color32::from_white_alpha(60)),
        );
        if swatch.selected {
            cx.veil.circle_stroke(
                hit.center(),
                radius + Dp::D3.px(),
                crate::gfx::Stroke::new(stroke::emphasis(), Color32::WHITE),
            );
        }
        if response.clicked() {
            layout.activated = Some(NODE_PLAYER_PANEL_ROW_BASE + (entry.index + offset) as u64);
        }
    }
    if let Some(slot) = placed.find(LABEL) {
        cx.veil.text(
            slot.left_center(),
            Align2::LEFT_CENTER,
            &entry.row.label,
            font,
            entry.tint,
        );
    }
}

fn choices_row(
    cx: &Cx,
    ui: &mut crate::gfx::Ui,
    layout: &mut HomeLayout,
    entry: &Entry,
    count: usize,
) {
    let m = cx.m;
    let font = FontId::proportional(m.panel_choice_font);
    let chips = flex::row((1..=count).map(|offset| {
        let size = flex::text_size(cx.veil, &cx.panel.rows[entry.index + offset].label, &font);
        flex::leaf(Vec2::new(size.x + Dp::D20.px(), Dp::D28.px())).id(CHIP + offset)
    }))
    .gap(Space::S6.px(m))
    .align(crate::flex::Align::Start);
    let label_font = FontId::proportional(cx.text);
    let mut kids = Vec::new();
    if !entry.row.label.is_empty() {
        kids.push(flex::text(cx.veil, &entry.row.label, &label_font).id(LABEL));
    }
    kids.push(flex::fill());
    kids.push(chips);
    let placed = flex::row(kids)
        .align(flex::Align::Center)
        .padding(Insets::xy(Space::S8.px(m), 0.0))
        .solve(entry.rect);
    for offset in 1..=count {
        let choice = &cx.panel.rows[entry.index + offset];
        let Some(hit) = placed.find(CHIP + offset) else {
            continue;
        };
        let response = cx.step(ui, layout, entry.index + offset, hit, 0.0);
        let (fill, color) = if choice.selected {
            (text_primary(), Color32::BLACK)
        } else {
            (crate::stylesheet::color("well"), entry.tint)
        };
        cx.veil.rect_filled(hit, hit.height() / 2.0, fill);
        cx.veil.text(
            hit.center(),
            Align2::CENTER_CENTER,
            &choice.label,
            font.clone(),
            color,
        );
        if response.clicked() {
            layout.activated = Some(NODE_PLAYER_PANEL_ROW_BASE + (entry.index + offset) as u64);
        }
    }
    if let Some(slot) = placed.find(LABEL) {
        cx.veil.text(
            slot.left_center(),
            Align2::LEFT_CENTER,
            &entry.row.label,
            label_font,
            entry.tint,
        );
    }
}

fn value_row(cx: &Cx, ui: &mut crate::gfx::Ui, layout: &mut HomeLayout, entry: &Entry) {
    let m = cx.m;
    let font = FontId::proportional(m.panel_step_font);
    let button = Dp::D28.px();
    let gap = Space::S12.px(m);
    let room = cx.col_w - Space::S8.px(m) * 2.0 - button * 2.0 - Dp::D64.px() - gap;
    let label = cx.clipped(&entry.row.label, &font, room);
    let placed = flex::row([
        flex::text(cx.veil, &label, &font).id(LABEL).grow(1.0),
        flex::row([
            flex::leaf(Vec2::splat(button)).id(MINUS),
            flex::leaf(Vec2::new(Dp::D64.px(), button)).id(VALUE_SLOT),
            flex::leaf(Vec2::splat(button)).id(PLUS),
        ])
        .width(button * 2.0 + Dp::D64.px())
        .align(crate::flex::Align::Start),
    ])
    .gap(gap)
    .align(flex::Align::Center)
    .padding(Insets::xy(Space::S8.px(m), 0.0))
    .solve(entry.rect);
    for (slot, glyph, offset) in [(MINUS, "−", 1), (PLUS, "+", 2)] {
        let Some(hit) = placed.find(slot) else {
            continue;
        };
        let response = cx.step(ui, layout, entry.index + offset, hit, 0.0);
        cx.veil.circle_filled(
            hit.center(),
            hit.width() / 2.0,
            crate::ink(if response.is_pointer_button_down_on() {
                tone::strong()
            } else {
                tone::soft()
            }),
        );
        cx.veil.text(
            hit.center(),
            Align2::CENTER_CENTER,
            glyph,
            FontId::proportional(m.panel_step_glyph),
            entry.tint,
        );
        if response.clicked() {
            layout.activated = Some(NODE_PLAYER_PANEL_ROW_BASE + (entry.index + offset) as u64);
        }
    }
    if let Some(slot) = placed.find(VALUE_SLOT) {
        cx.veil.text(
            slot.center(),
            Align2::CENTER_CENTER,
            entry.row.value.as_deref().unwrap_or_default(),
            font.clone(),
            entry.tint,
        );
    }
    if let Some(slot) = placed.find(LABEL) {
        cx.veil.text(
            slot.left_center(),
            Align2::LEFT_CENTER,
            &label,
            font,
            entry.tint,
        );
    }
}

fn plain_row(cx: &Cx, layout: &mut HomeLayout, entry: &Entry, response: &crate::gfx::Response) {
    let m = cx.m;
    let (row, tint, node) = (entry.row, entry.tint, cx.node(entry));
    let icons = &cx.chrome.icons;
    let s8 = Space::S8.px(m);
    let s10 = Space::S10.px(m);
    let chevron = icons.get("ChevronRight").filter(|_| row.chevron);
    let check = icons.get("Check").filter(|_| row.selected);
    let value = row.value.as_deref().map(|value| {
        let font = FontId::proportional(m.panel_value_font);
        let clipped = cx.clipped(value, &font, cx.col_w * m.panel_value_share);
        cx.veil.layout_no_wrap(clipped, font, tint)
    });
    let mut trailing = Vec::new();
    let mut trailing_w = 0.0;
    let mut add = |size: Vec2, gap: f32, id: usize| {
        trailing.push(trail(size, gap, id));
        trailing_w += size.x + gap;
    };
    if let Some(galley) = &value {
        add(galley.size(), s10, VALUE);
    }
    if check.is_some() {
        add(Vec2::splat(m.panel_list_check), s8, CHECK);
    }
    if row.switch.is_some() {
        add(
            Vec2::new(Dp::D40.px(), Dp::D20.px()),
            Space::S12.px(m),
            SWITCH,
        );
    }
    if chevron.is_some() {
        add(Vec2::splat(m.panel_list_chevron), s8, CHEVRON);
    }
    let icon = row
        .icon
        .as_deref()
        .and_then(|name| icons.get(name).map(|texture| (name, texture)));
    let mut leading = Vec::new();
    let mut fixed = 0.0;
    if row.badge.is_some() {
        leading.push(flex::leaf(m.panel_badge).id(BADGE));
        fixed += m.panel_badge.x + s10;
    }
    if icon.is_some() {
        leading.push(flex::leaf(Vec2::splat(m.panel_list_icon)).id(ICON));
        fixed += m.panel_list_icon + s10;
    }
    let inner = cx.col_w - s8 - Space::S12.px(m);
    let room = (inner - trailing_w - fixed - s8).max(0.0);
    let font = if row.selected {
        crate::fonts::bold(m.panel_list_label_font)
    } else {
        FontId::proportional(cx.text)
    };
    let label = cx.clipped(&row.label, &font, room);
    let mut lines = vec![flex::text(cx.veil, &label, &font).id(LABEL)];
    let small = FontId::proportional(m.panel_list_subline_font);
    let subline = row
        .subline
        .as_deref()
        .map(|subline| cx.clipped(subline, &small, room));
    if let Some(subline) = &subline {
        lines.push(flex::text(cx.veil, subline, &small).id(SUBLINE));
    }
    let chip_font = FontId::proportional(m.panel_list_chip_font);
    let mut chips = Vec::new();
    let mut remaining = room + s8;
    let chip_gap = Space::S6.px(m);
    for (slot, chip) in row.chips.iter().enumerate() {
        let label = cx.clipped(chip, &chip_font, room.max(Dp::D40.px()));
        let width = flex::text_size(cx.veil, &label, &chip_font).x + Space::S16.px(m);
        if width > remaining {
            break;
        }
        remaining -= width + chip_gap;
        chips.push((slot, label, width));
    }
    if !chips.is_empty() {
        lines.push(
            flex::row(chips.iter().map(|(slot, _, width)| {
                flex::leaf(Vec2::new(*width, m.panel_list_chip_height)).id(CHIP + slot)
            }))
            .gap(chip_gap)
            .align(crate::flex::Align::Start),
        );
    }
    leading.push(
        flex::column(lines)
            .gap(Space::S2.px(m))
            .grow(1.0)
            .align(crate::flex::Align::Start),
    );
    let mut kids = vec![
        flex::row(leading)
            .gap(s10)
            .align(flex::Align::Center)
            .padding(Insets {
                left: 0.0,
                top: 0.0,
                right: s8,
                bottom: 0.0,
            })
            .grow(1.0),
    ];
    kids.extend(trailing);
    let placed = flex::row(kids)
        .align(flex::Align::Center)
        .padding(Insets {
            left: s8,
            top: 0.0,
            right: Space::S12.px(m),
            bottom: 0.0,
        })
        .solve(entry.rect);
    if let (Some(badge), Some(pill)) = (row.badge.as_deref(), placed.find(BADGE)) {
        cx.veil.rect_stroke(
            pill,
            Dp::D4.px(),
            crate::gfx::Stroke::new(stroke::hairline(), tint.gamma_multiply(0.5)),
            crate::gfx::StrokeKind::Inside,
        );
        cx.veil.text(
            pill.center(),
            Align2::CENTER_CENTER,
            badge,
            FontId::proportional(m.panel_badge_font),
            tint.gamma_multiply(0.75),
        );
    }
    if let (Some((_, texture)), Some(slot)) = (icon, placed.find(ICON)) {
        cx.veil.image(texture, slot, full_uv(), tint);
    }
    if let (Some(texture), Some(slot)) = (chevron, placed.find(CHEVRON)) {
        cx.veil.image(texture, slot, full_uv(), tint);
    }
    if let (Some(on_state), Some(track)) = (row.switch, placed.find(SWITCH)) {
        components::toggle(
            cx.chrome.context,
            cx.veil,
            Id::new(("fluxa-player-panel-switch", node)),
            track,
            on_state,
            *m,
        );
    }
    if let (Some(texture), Some(slot)) = (check, placed.find(CHECK)) {
        cx.veil.image(texture, slot, full_uv(), tint);
    }
    if let (Some(galley), Some(slot)) = (value, placed.find(VALUE)) {
        cx.veil.galley(slot.min, galley, tint);
    }
    if let Some(slot) = placed.find(LABEL) {
        cx.veil
            .text(slot.left_center(), Align2::LEFT_CENTER, &label, font, tint);
    }
    if let (Some(subline), Some(slot)) = (subline, placed.find(SUBLINE)) {
        cx.veil.text(
            slot.left_center(),
            Align2::LEFT_CENTER,
            subline,
            small,
            tint.gamma_multiply(0.55),
        );
    }
    for (slot, label, _) in chips {
        if let Some(pill) = placed.find(CHIP + slot) {
            cx.veil
                .rect_filled(pill, pill.height() / 2.0, tint.gamma_multiply(0.14));
            cx.veil.text(
                pill.center(),
                Align2::CENTER_CENTER,
                label,
                chip_font.clone(),
                tint,
            );
        }
    }
    if response.clicked() {
        layout.activated = Some(node);
    }
}

fn draw_list(
    chrome: &Chrome,
    ui: &mut crate::gfx::Ui,
    layout: &mut HomeLayout,
    panel: &PlayerPanel,
    assets: &mut impl HomeAssets,
    slide: f32,
) {
    let m = UiMetrics::for_viewport(chrome.viewport);
    let form_factor = chrome.viewport.form_factor;
    let mut row_h = m.button_height;
    let text = m.text.body;
    let pad = Space::S14.px(&m);
    let mut base = m.player_panel_narrow.of(chrome.rect.width());
    let rich = panel
        .rows
        .iter()
        .any(|row| row.thumbnail.is_some() || row.detail.is_some());
    if rich {
        row_h = m.button_height * 2.0;
        base = m.player_panel_wide.of(chrome.rect.width());
    } else if panel
        .rows
        .iter()
        .any(|row| row.subline.is_some() && !row.chips.is_empty())
    {
        row_h = m.button_height + Space::S20.px(&m);
    }
    let mut columns: Vec<(Option<&str>, Vec<(usize, &PanelRow)>)> = Vec::new();
    let mut back_row: Option<usize> = None;
    for (index, row) in panel.rows.iter().enumerate().take(PLAYER_PANEL_ROW_LIMIT) {
        if row.back {
            back_row = Some(index);
        } else if row.heading {
            columns.push((Some(row.label.as_str()), Vec::new()));
        } else if row.step.is_some_and(PanelStep::follows_lead) {
            continue;
        } else {
            if columns.is_empty() {
                columns.push((None, Vec::new()));
            }
            if let Some(column) = columns.last_mut() {
                column.1.push((index, row));
            }
        }
    }
    let count = columns.len().max(1) as f32;
    let gap = pad;
    let available = chrome.rect.width() - chrome.margin() * 2.0;
    let mobile = form_factor == UiFormFactor::Mobile;
    let drawer = mobile && panel.look.is_some();
    let sheet = mobile && !drawer;
    if drawer {
        base = base.max(m.panel_drawer_width - pad * 2.0);
    }
    let usable = if sheet {
        available.min(m.sheet_max_width)
    } else {
        available
    };
    let fit_width = (usable - pad * 2.0 - gap * (count - 1.0)) / count;
    let col_w = if sheet {
        fit_width
    } else {
        fit_width.min(base)
    };
    let width = if drawer {
        m.panel_drawer_width.min(available)
    } else {
        col_w * count + gap * (count - 1.0) + pad * 2.0
    };
    let col_w = if drawer { width - pad * 2.0 } else { col_w };
    let title_h = if panel.title.is_empty() && back_row.is_none() {
        0.0
    } else if back_row.is_some() {
        row_h * m.panel_list_back_header_scale
    } else {
        row_h
    };
    let head_h = if columns.iter().any(|(heading, _)| heading.is_some()) {
        row_h * m.panel_list_head_scale
    } else {
        0.0
    };
    let reserved = match form_factor {
        UiFormFactor::Desktop => m.panel_reserved_desktop + chrome.viewport.safe_bottom,
        _ if sheet => chrome.viewport.safe_bottom,
        _ => Space::S32.px(&m),
    };
    let available_height = if sheet {
        chrome.rect.height() * m.panel_list_sheet_ratio
    } else if panel.look.is_some() && form_factor != UiFormFactor::Desktop {
        chrome.rect.height() - chrome.margin()
    } else {
        chrome.rect.height()
    };
    let handle_h = if sheet { Space::S12.px(&m) } else { 0.0 };
    let max_body =
        available_height - reserved - chrome.margin() - pad * 2.0 - handle_h - title_h - head_h;
    let longest = columns
        .iter()
        .map(|(_, rows)| rows.len())
        .max()
        .unwrap_or(0);
    let visible = (((max_body / row_h).floor() as usize).max(3)).min(longest.max(1));
    let painter = ui.painter().clone();
    let title_font = FontId::proportional(m.panel_list_title_font);
    let mut header = Vec::new();
    if back_row.is_some() {
        header.push(flex::leaf(Vec2::splat(row_h * m.panel_list_back_scale)).id(HEAD_BACK));
    }
    if title_h > 0.0 && !panel.title.is_empty() {
        header.push(flex::text(&painter, &panel.title, &title_font).id(HEAD_TITLE));
    }
    let slots: Vec<Node> = columns
        .iter()
        .enumerate()
        .map(|(column, _)| {
            flex::column([
                flex::leaf(Vec2::new(col_w, head_h)).id(column * 3),
                flex::leaf(Vec2::new(col_w, visible as f32 * row_h)).id(column * 3 + 1),
            ])
            .width(col_w)
            .id(column * 3 + 2)
            .align(crate::flex::Align::Start)
        })
        .collect();
    let mut body = Vec::new();
    if sheet {
        body.push(flex::leaf(Vec2::new(0.0, handle_h)));
    }
    body.push(
        flex::row(header)
            .gap(Space::S6.px(&m))
            .align(flex::Align::Center)
            .height(title_h),
    );
    body.push(
        flex::row(slots)
            .gap(gap)
            .height(head_h + visible as f32 * row_h)
            .align(crate::flex::Align::Start),
    );
    let tree = flex::column(body)
        .align(flex::Align::Stretch)
        .padding(Insets::all(pad));
    let height = tree.outer_size().y;

    let card = match form_factor {
        UiFormFactor::Desktop => dock(
            chrome.rect,
            Align2::RIGHT_BOTTOM,
            Vec2::new(width, height),
            Insets {
                left: 0.0,
                top: 0.0,
                right: chrome.margin(),
                bottom: reserved,
            },
        ),
        _ if drawer => dock(
            chrome.rect,
            Align2::RIGHT_TOP,
            Vec2::new(width, chrome.rect.height()),
            Insets::default(),
        ),
        _ if sheet => dock(
            chrome.rect,
            Align2::CENTER_BOTTOM,
            Vec2::new(width, height + reserved),
            Insets::default(),
        ),
        _ if panel.look.is_some() => dock(
            chrome.rect,
            Align2::CENTER_TOP,
            Vec2::new(width, height),
            Insets {
                left: 0.0,
                top: chrome.margin(),
                right: 0.0,
                bottom: 0.0,
            },
        ),
        _ => dock(
            chrome.rect,
            Align2::CENTER_CENTER,
            Vec2::new(width, height),
            Insets::default(),
        ),
    };
    let veil = chrome.modal_veil(ui, card, layout);
    if let Some(look) = panel.look.as_ref() {
        let lift = if chrome.player.controls_visible {
            super::tokens::subtitle_lift(form_factor)
        } else {
            0.0
        };
        let area = inset_rect(
            chrome.rect,
            Insets {
                left: 0.0,
                top: 0.0,
                right: if drawer { card.width() } else { 0.0 },
                bottom: chrome.viewport.safe_bottom + Space::S24.px(&m) + lift,
            },
        );
        components::subtitle_sample(
            &veil,
            area,
            look,
            &localized("settings.subtitle_preview_text", &chrome.player.language),
            chrome.rect.height() * m.panel_sample_ratio,
        );
    } else if form_factor != UiFormFactor::Desktop && !sheet {
        veil.rect_filled(chrome.rect, 0.0, shade(Shade::Veil));
    } else if sheet {
        veil.rect_filled(chrome.rect, 0.0, Color32::from_black_alpha(110));
    }
    let layer = veil.layer_id();
    let first = chrome.context.layer_cursor(layer);
    let radius = if sheet {
        let top = Radius::Sheet.corner(&m);
        crate::gfx::CornerRadius {
            nw: top,
            ne: top,
            sw: 0,
            se: 0,
        }
    } else {
        crate::gfx::CornerRadius::same(m.card_radius as u8)
    };
    veil.rect_filled(card, radius, m.surface);
    veil.rect_stroke(
        card,
        radius,
        crate::gfx::Stroke::new(stroke::hairline(), m.border),
        crate::gfx::StrokeKind::Inside,
    );
    if sheet {
        veil.rect_filled(
            flex::anchored(
                card,
                Align2::CENTER_TOP,
                m.sheet_handle,
                Dp::D8.px() - m.sheet_handle.y / 2.0,
            ),
            m.sheet_handle.y / 2.0,
            crate::stylesheet::color("strong"),
        );
    }
    let placed = tree.solve(card);
    if let Some(index) = back_row
        && let Some(hit) = placed.find(HEAD_BACK)
    {
        let node = NODE_PLAYER_PANEL_ROW_BASE + index as u64;
        let response = ui.interact(
            hit,
            Id::new(("fluxa-player-panel-back", node)),
            Sense::click(),
        );
        layout.focusable.push((node, hit));
        let on = chrome.focused == Some(node);
        if on || response.hovered() {
            veil.circle_filled(
                hit.center(),
                hit.width() / 2.0,
                if on {
                    text_primary()
                } else {
                    crate::stylesheet::color("track")
                },
            );
        }
        if let Some(texture) = chrome.icons.get("ArrowLeft") {
            veil.image(
                texture,
                flex::anchored(
                    hit,
                    Align2::CENTER_CENTER,
                    Vec2::splat(m.panel_list_back_icon),
                    0.0,
                ),
                full_uv(),
                if on { Color32::BLACK } else { ink(235) },
            );
        }
        if response.clicked() {
            layout.activated = Some(node);
        }
    }
    if let Some(slot) = placed.find(HEAD_TITLE) {
        veil.text(
            slot.left_center(),
            Align2::LEFT_CENTER,
            &panel.title,
            title_font,
            text_primary(),
        );
    }
    let cx = Cx {
        chrome,
        veil: &veil,
        m: &m,
        panel,
        col_w,
        row_h,
        text,
    };
    let mut previous_right = None;
    for (column, (heading, rows)) in columns.iter().enumerate() {
        let (Some(heading_rect), Some(rows_rect), Some(slot)) = (
            placed.find(column * 3),
            placed.find(column * 3 + 1),
            placed.find(column * 3 + 2),
        ) else {
            continue;
        };
        if let Some(heading) = heading {
            let label = if sheet {
                heading.to_uppercase()
            } else {
                heading.to_string()
            };
            veil.text(
                heading_rect.left_center(),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(if sheet {
                    m.panel_list_heading_font_sheet
                } else {
                    m.panel_list_heading_font
                }),
                crate::stylesheet::color("glyph"),
            );
        }
        if let (true, Some(before)) = (sheet, previous_right) {
            let x = (before + slot.left()) / 2.0;
            veil.line_segment(
                [
                    Pos2::new(x, slot.top() + Dp::D4.px()),
                    Pos2::new(x, rows_rect.bottom()),
                ],
                crate::gfx::Stroke::new(stroke::hairline(), m.border),
            );
        }
        previous_right = Some(slot.right());
        let state_id = Id::new(("fluxa-player-panel-offset", column));
        let mut start: usize = ui.data(|data| data.get_temp(state_id)).unwrap_or(0);
        let focused = rows
            .iter()
            .position(|(index, _)| {
                chrome.focused == Some(NODE_PLAYER_PANEL_ROW_BASE + *index as u64)
            })
            .or_else(|| {
                (chrome.focused.is_none() && start == 0)
                    .then(|| rows.iter().position(|(_, row)| row.selected))
                    .flatten()
            });
        let marked_id = Id::new(("fluxa-player-panel-marked", column));
        let marked = rows.iter().position(|(_, row)| row.selected);
        let previous: Option<Option<usize>> = ui.data(|data| data.get_temp(marked_id));
        ui.data_mut(|data| data.insert_temp(marked_id, marked));
        let followed = marked.filter(|_| previous.is_some_and(|before| before != marked));
        if let Some(position) = focused.or(followed) {
            if position < start {
                start = position;
            } else if position >= start + visible {
                start = position + 1 - visible;
            }
        }
        if ui.rect_contains_pointer(rows_rect) {
            let scroll = ui.input(|input| input.smooth_scroll_delta().y);
            if scroll.abs() > 1.0 {
                let step = if scroll > 0.0 { -1 } else { 1 };
                start = (start as i32 + step).clamp(0, rows.len().saturating_sub(visible) as i32)
                    as usize;
            }
        }
        let drag = ui.interact(
            rows_rect,
            Id::new(("fluxa-player-panel-drag", column)),
            Sense::drag(),
        );
        if drag.dragged() {
            let carry_id = Id::new(("fluxa-player-panel-carry", column));
            let carry: f32 = ui.data(|data| data.get_temp(carry_id)).unwrap_or(0.0)
                - drag.drag_delta().y / row_h;
            let whole = carry.trunc();
            ui.data_mut(|data| data.insert_temp(carry_id, carry - whole));
            start = (start as i32 + whole as i32).max(0) as usize;
        }
        start = start.min(rows.len().saturating_sub(visible));
        ui.data_mut(|data| data.insert_temp(state_id, start));
        let list = flex::column(
            rows.iter()
                .enumerate()
                .map(|(position, _)| flex::leaf(Vec2::new(col_w, row_h)).id(position)),
        )
        .align(crate::flex::Align::Start)
        .solve(Rect::from_min_size(
            rows_rect.min - Vec2::new(0.0, start as f32 * row_h),
            Vec2::new(col_w, row_h * rows.len() as f32),
        ));
        for (position, (index, row)) in rows.iter().enumerate() {
            let node = NODE_PLAYER_PANEL_ROW_BASE + *index as u64;
            let Some(rect) = list.find(position) else {
                continue;
            };
            if row.group {
                if position >= start && position < start + visible {
                    let label = if drawer {
                        row.label.to_uppercase()
                    } else {
                        row.label.clone()
                    };
                    let font = FontId::proportional(m.panel_list_group_font);
                    let group = flex::column([flex::text(&veil, &label, &font).id(LABEL)])
                        .padding(Insets {
                            left: Space::S8.px(&m),
                            top: row_h * m.panel_list_group_lift,
                            right: 0.0,
                            bottom: 0.0,
                        })
                        .align(flex::Align::Start)
                        .justify(flex::Justify::Center)
                        .height(row_h)
                        .solve(rect);
                    if let Some(slot) = group.find(LABEL) {
                        veil.text(
                            slot.left_center(),
                            Align2::LEFT_CENTER,
                            label,
                            font,
                            crate::stylesheet::color("glyph"),
                        );
                    }
                }
                continue;
            }
            layout.focusable.push((node, rect));
            if position < start || position >= start + visible {
                continue;
            }
            let response = ui.interact(
                rect,
                Id::new(("fluxa-player-panel-row", node)),
                Sense::click(),
            );
            let on = chrome.focused == Some(node);
            let fill = if on {
                text_primary()
            } else if row.selected {
                crate::stylesheet::color("track")
            } else if response.hovered() {
                crate::stylesheet::color("well")
            } else {
                Color32::TRANSPARENT
            };
            veil.rect_filled(rect, row_h * m.panel_list_radius_scale, fill);
            if row.separated {
                let y = rect.top() - Dp::D1.px();
                let inset = Space::S8.px(&m);
                veil.line_segment(
                    [
                        Pos2::new(rect.left() + inset, y),
                        Pos2::new(rect.right() - inset, y),
                    ],
                    crate::gfx::Stroke::new(stroke::hairline(), m.border),
                );
            }
            let entry = Entry {
                index: *index,
                row,
                rect,
                tint: if on { Color32::BLACK } else { ink(255) },
            };
            if rich {
                rich_row(&cx, layout, assets, &entry, &response);
            } else if let Some(PanelStep::Swatches(count)) = row.step {
                swatch_row(&cx, ui, layout, &entry, count);
            } else if let Some(PanelStep::Choices(count)) = row.step {
                choices_row(&cx, ui, layout, &entry, count);
            } else if row.step == Some(PanelStep::Value) {
                value_row(&cx, ui, layout, &entry);
            } else {
                plain_row(&cx, layout, &entry, &response);
            }
        }
    }
    if slide < 1.0 && (sheet || drawer) {
        let offset = if drawer {
            Vec2::new((1.0 - slide) * card.width(), 0.0)
        } else {
            Vec2::new(0.0, (1.0 - slide) * card.height())
        };
        chrome.context.transform_layer_since(
            layer,
            first,
            crate::gfx::emath::TSTransform::from_translation(offset),
        );
    }
}

fn parse_hex(value: &str) -> Option<Color32> {
    let hex = value.strip_prefix('#')?;
    let number = u32::from_str_radix(hex, 16).ok()?;
    Some(Color32::from_rgb(
        (number >> 16) as u8,
        (number >> 8) as u8,
        number as u8,
    ))
}

fn draw_chips(
    chrome: &Chrome,
    ui: &mut crate::gfx::Ui,
    layout: &mut HomeLayout,
    panel: &PlayerPanel,
    slide: f32,
) {
    let m = UiMetrics::for_viewport(chrome.viewport);
    let pad = Dp::D16.px();
    let width = (chrome.rect.width() - chrome.margin() * 2.0).min(m.sheet_max_width);
    let inner = width - pad * 2.0;
    let font = FontId::proportional(Dp::D14.px());
    let title_font = FontId::proportional(Dp::D14.px());
    let value_font = FontId::proportional(Dp::D32.px());
    let chip_nodes = panel
        .rows
        .iter()
        .enumerate()
        .take(PLAYER_PANEL_ROW_LIMIT)
        .filter(|(_, item)| !item.heading)
        .map(|(index, item)| {
            let size = flex::text_size(ui.painter(), &item.label, &font);
            flex::leaf(Vec2::new(size.x + Dp::D32.px(), Dp::D36.px())).id(index)
        })
        .collect::<Vec<_>>();
    let chips = flex::row(chip_nodes)
        .wrap()
        .gap(Space::S8.px(&m))
        .line_gap(Space::S8.px(&m))
        .width(inner)
        .align(crate::flex::Align::Start);
    let chips_height = chips
        .solve(Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(inner, f32::INFINITY),
        ))
        .tagged()
        .iter()
        .map(|(_, rect)| rect.bottom())
        .fold(Dp::D36.px(), f32::max);
    let current = panel
        .rows
        .iter()
        .find(|item| item.selected)
        .map(|item| item.value.as_deref().unwrap_or(item.label.as_str()))
        .unwrap_or_default();
    let painter = ui.painter().clone();
    let tree = flex::column([
        flex::leaf(Vec2::new(0.0, Space::S12.px(&m))),
        flex::row([flex::text(&painter, &panel.title, &title_font).id(HEAD_TITLE)])
            .align(flex::Align::Center)
            .height(Dp::D28.px()),
        flex::row([flex::text(&painter, current, &value_font).id(CURRENT)])
            .align(flex::Align::Center)
            .height(Dp::D44.px()),
        chips.height(chips_height),
    ])
    .padding(Insets {
        left: pad,
        top: pad,
        right: pad,
        bottom: pad + chrome.viewport.safe_bottom,
    })
    .align(crate::flex::Align::Start);
    let height = tree.outer_size().y;
    let card = dock(
        chrome.rect,
        Align2::CENTER_BOTTOM,
        Vec2::new(width, height),
        Insets::default(),
    );
    let veil = chrome.modal_veil(ui, card, layout);
    veil.rect_filled(chrome.rect, 0.0, Color32::from_black_alpha(110));
    let layer = veil.layer_id();
    let first = chrome.context.layer_cursor(layer);
    let top = Radius::Sheet.corner(&m);
    let radius = crate::gfx::CornerRadius {
        nw: top,
        ne: top,
        sw: 0,
        se: 0,
    };
    veil.rect_filled(card, radius, m.surface);
    veil.rect_stroke(
        card,
        radius,
        crate::gfx::Stroke::new(stroke::hairline(), m.border),
        crate::gfx::StrokeKind::Inside,
    );
    veil.rect_filled(
        flex::anchored(
            card,
            Align2::CENTER_TOP,
            m.sheet_handle,
            Dp::D8.px() - m.sheet_handle.y / 2.0,
        ),
        m.sheet_handle.y / 2.0,
        crate::stylesheet::color("strong"),
    );
    let placed = tree.solve(card);
    if let Some(slot) = placed.find(HEAD_TITLE) {
        veil.text(
            slot.left_center(),
            Align2::LEFT_CENTER,
            &panel.title,
            title_font,
            text_primary(),
        );
    }
    if let Some(slot) = placed.find(CURRENT) {
        veil.text(
            slot.left_center(),
            Align2::LEFT_CENTER,
            current,
            value_font,
            text_primary(),
        );
    }
    for (index, rect) in placed.tagged() {
        let Some(item) = panel.rows.get(index) else {
            continue;
        };
        let node = NODE_PLAYER_PANEL_ROW_BASE + index as u64;
        layout.focusable.push((node, rect));
        let response = ui.interact(
            rect,
            Id::new(("fluxa-player-panel-row", node)),
            Sense::click(),
        );
        let on = chrome.focused == Some(node);
        let (fill, ink_color) = if item.selected || on {
            (text_primary(), Color32::BLACK)
        } else {
            (crate::stylesheet::color("well"), ink(235))
        };
        veil.rect_filled(rect, rect.height() / 2.0, fill);
        veil.text(
            rect.center(),
            Align2::CENTER_CENTER,
            &item.label,
            font.clone(),
            ink_color,
        );
        if response.clicked() {
            layout.activated = Some(node);
        }
    }
    if slide < 1.0 {
        let offset = Vec2::new(0.0, (1.0 - slide) * card.height());
        chrome.context.transform_layer_since(
            layer,
            first,
            crate::gfx::emath::TSTransform::from_translation(offset),
        );
    }
}
