use super::panel::{SyncRole, dock, inset_rect};
use super::*;
use crate::dp::Dp;
use crate::flex::{self, Insets, Node};
use std::sync::Arc;

const BACK_TITLE: usize = usize::MAX;
const NUMBER: usize = usize::MAX - 1;
const SUFFIX: usize = usize::MAX - 2;
const PILL: usize = usize::MAX - 3;
const MINUS: usize = usize::MAX - 4;
const PLUS: usize = usize::MAX - 5;
const STATUS: usize = usize::MAX - 6;
const HINT: usize = usize::MAX - 7;
const RULE: usize = usize::MAX - 8;
const AUTO_ICON: usize = usize::MAX - 9;
const AUTO_LABEL: usize = usize::MAX - 10;

struct Fonts {
    back: FontId,
    reset: FontId,
    big: FontId,
    unit: FontId,
    step: FontId,
    auto: FontId,
    label: FontId,
    detail: FontId,
    choice: FontId,
}

impl Fonts {
    fn new(m: &UiMetrics) -> Self {
        Self {
            back: crate::fonts::bold(m.panel_sync_back_font),
            reset: FontId::proportional(m.panel_sync_reset_font),
            big: crate::fonts::bold(m.panel_sync_big_font),
            unit: crate::fonts::bold(m.panel_sync_unit_font),
            step: crate::fonts::bold(m.panel_sync_step_font),
            auto: crate::fonts::bold(m.panel_sync_auto_font),
            label: crate::fonts::bold(m.panel_sync_label_font),
            detail: FontId::proportional(m.panel_sync_detail_font),
            choice: FontId::proportional(m.panel_sync_choice_font),
        }
    }
}

struct SyncSheet<'a> {
    m: &'a UiMetrics,
    fonts: Fonts,
    painter: &'a crate::gfx::Painter,
    panel: &'a PlayerPanel,
    sheet: bool,
    row_w: f32,
    status: Option<Arc<crate::gfx::Galley>>,
    hint: Option<Arc<crate::gfx::Galley>>,
}

fn find_row(panel: &PlayerPanel, role: SyncRole) -> Option<(usize, &PanelRow)> {
    panel
        .rows
        .iter()
        .enumerate()
        .find(|(_, row)| row.sync == Some(role))
}

fn padded(node: Node, bleed: f32) -> Node {
    flex::column([node])
        .align(flex::Align::Stretch)
        .padding(Insets::xy(bleed, 0.0))
}

impl SyncSheet<'_> {
    fn text_w(&self) -> f32 {
        self.row_w - Dp::D6.px() * 2.0
    }

    fn part(&self, index: usize, k: usize) -> usize {
        index + self.panel.rows.len() * (k + 1)
    }

    fn clipped(&self, text: &str, font: &FontId, width: f32) -> String {
        truncate_to_width(self.painter, text, font, width.max(0.0))
    }

    fn head(&self) -> Node {
        let m = self.m;
        let mut kids = Vec::new();
        if let Some((index, row)) = find_row(self.panel, SyncRole::Back) {
            kids.push(flex::leaf(Vec2::splat(m.panel_sync_back)).id(index));
            kids.push(flex::text(self.painter, &row.label, &self.fonts.back).id(BACK_TITLE));
        }
        kids.push(flex::fill());
        if let Some((index, row)) = find_row(self.panel, SyncRole::Reset) {
            let label = flex::text_size(self.painter, &row.label, &self.fonts.reset);
            kids.push(
                flex::leaf(Vec2::new(
                    label.x + m.panel_sync_reset_pad,
                    m.panel_sync_reset_height,
                ))
                .id(index),
            );
        }
        flex::row(kids)
            .gap(m.panel_sync_back_gap)
            .align(flex::Align::Center)
            .height(m.panel_sync_head_height)
    }

    fn reading(&self) -> Node {
        let m = self.m;
        let height = m.panel_sync_reading_height;
        let Some((_, row)) = find_row(self.panel, SyncRole::Reading) else {
            return flex::row([])
                .height(height)
                .align(crate::flex::Align::Start);
        };
        let (number, suffix) = split_reading(&row.label);
        let stepper = flex::row([flex::fill().id(MINUS), flex::fill().id(PLUS)])
            .align(flex::Align::Stretch)
            .width(m.panel_sync_stepper_width)
            .height(m.panel_sync_stepper_height)
            .id(PILL);
        let unit = flex::column([flex::text(self.painter, suffix, &self.fonts.unit).id(SUFFIX)])
            .padding(Insets {
                left: Dp::D3.px(),
                top: m.panel_sync_unit_lift,
                right: 0.0,
                bottom: 0.0,
            })
            .align(crate::flex::Align::Start);
        flex::row([
            flex::text(self.painter, number, &self.fonts.big).id(NUMBER),
            unit,
            flex::fill(),
            stepper,
        ])
        .align(flex::Align::Center)
        .height(height)
    }

    fn auto(&self) -> Node {
        let m = self.m;
        let Some((index, row)) = find_row(self.panel, SyncRole::Auto) else {
            return flex::row([])
                .height(m.panel_sync_auto_height)
                .align(crate::flex::Align::Start);
        };
        flex::row([
            flex::leaf(Vec2::splat(m.panel_sync_auto_icon)).id(AUTO_ICON),
            flex::text(self.painter, &row.label, &self.fonts.auto).id(AUTO_LABEL),
        ])
        .gap(m.panel_sync_auto_gap)
        .justify(flex::Justify::Center)
        .align(flex::Align::Center)
        .height(m.panel_sync_auto_height)
        .id(index)
    }

    fn detail_room(&self) -> f32 {
        let m = self.m;
        self.row_w
            - m.panel_sync_row_lead
            - m.panel_sync_row_trail
            - m.panel_sync_method_icon * 2.0
            - m.panel_sync_method_icon_gap
            - m.panel_sync_method_chevron_gap
    }

    fn choice_room(&self, row: &PanelRow) -> f32 {
        let m = self.m;
        let room = self.row_w - m.panel_sync_choice_inset - m.panel_sync_row_trail;
        if row.selected {
            room - m.panel_sync_check_room
        } else {
            room
        }
    }

    fn method(&self, index: usize, row: &PanelRow) -> Node {
        let m = self.m;
        let mut lines =
            vec![flex::text(self.painter, &row.label, &self.fonts.label).id(self.part(index, 1))];
        if let Some(detail) = row.detail.as_deref() {
            let clipped = self.clipped(detail, &self.fonts.detail, self.detail_room());
            lines.push(
                flex::text(self.painter, &clipped, &self.fonts.detail).id(self.part(index, 2)),
            );
        }
        flex::row([
            flex::leaf(Vec2::splat(m.panel_sync_method_icon)).id(self.part(index, 0)),
            flex::slot(m.panel_sync_method_icon_gap),
            flex::column(lines)
                .gap(Space::S2.px(m))
                .grow(1.0)
                .align(crate::flex::Align::Start),
            flex::slot(m.panel_sync_method_chevron_gap),
            flex::leaf(Vec2::splat(m.panel_sync_method_icon)).id(self.part(index, 3)),
        ])
        .align(flex::Align::Center)
        .padding(Insets {
            left: m.panel_sync_row_lead,
            top: 0.0,
            right: m.panel_sync_row_trail,
            bottom: 0.0,
        })
        .height(m.panel_sync_method_height)
        .id(index)
    }

    fn choice(&self, index: usize, row: &PanelRow) -> Node {
        let m = self.m;
        let mut kids = Vec::new();
        let label = self.clipped(&row.label, &self.fonts.choice, self.choice_room(row));
        kids.push(flex::text(self.painter, &label, &self.fonts.choice).id(self.part(index, 1)));
        kids.push(flex::fill());
        if row.selected {
            kids.push(flex::leaf(Vec2::splat(m.panel_sync_check)).id(self.part(index, 3)));
        }
        flex::row(kids)
            .align(flex::Align::Center)
            .padding(Insets {
                left: m.panel_sync_choice_inset,
                top: 0.0,
                right: m.panel_sync_row_trail,
                bottom: 0.0,
            })
            .height(m.panel_sync_choice_height)
            .id(index)
    }

    fn tree(&self, choices: usize) -> Node {
        let m = self.m;
        let bleed = Dp::D6.px();
        let mut body = Vec::new();
        if self.sheet {
            body.push(flex::leaf(Vec2::new(0.0, Space::S12.px(m))));
        }
        body.push(
            flex::column([self.head()])
                .align(flex::Align::Stretch)
                .padding(Insets {
                    left: bleed - Dp::D4.px(),
                    top: 0.0,
                    right: bleed,
                    bottom: 0.0,
                }),
        );
        body.push(padded(self.reading(), bleed));
        if let Some(status) = &self.status {
            body.push(
                flex::column([flex::leaf(status.size()).id(STATUS)])
                    .height(m.panel_sync_status_height)
                    .padding(Insets {
                        left: bleed,
                        top: m.panel_sync_status_drop,
                        right: bleed,
                        bottom: 0.0,
                    })
                    .align(crate::flex::Align::Start),
            );
        }
        body.push(flex::leaf(Vec2::new(0.0, m.panel_sync_gap)));
        body.push(padded(self.auto(), bleed));
        if let Some(hint) = &self.hint {
            body.push(
                flex::column([flex::leaf(Vec2::new(self.text_w(), hint.size().y)).id(HINT)])
                    .height(m.panel_sync_hint_height)
                    .padding(Insets {
                        left: bleed,
                        top: m.panel_sync_hint_drop,
                        right: bleed,
                        bottom: 0.0,
                    })
                    .align(crate::flex::Align::Start),
            );
        }
        body.push(
            flex::column([flex::leaf(Vec2::new(self.text_w(), 0.0)).id(RULE)])
                .padding(Insets {
                    left: bleed,
                    top: m.panel_sync_rule_gap,
                    right: bleed,
                    bottom: m.panel_sync_rule_gap,
                })
                .align(crate::flex::Align::Start),
        );
        let mut shown = 0;
        for (index, row) in self.panel.rows.iter().enumerate() {
            match row.sync {
                Some(SyncRole::Method) => body.push(self.method(index, row)),
                Some(SyncRole::Choice) if shown < choices => {
                    shown += 1;
                    body.push(self.choice(index, row));
                }
                _ => {}
            }
        }
        flex::column(body)
            .align(flex::Align::Stretch)
            .padding(Insets {
                left: m.panel_sync_edge,
                top: m.panel_sync_top,
                right: m.panel_sync_edge,
                bottom: m.panel_sync_bottom,
            })
    }
}

fn split_reading(label: &str) -> (&str, &str) {
    label
        .strip_suffix('s')
        .map_or((label, ""), |number| (number, "s"))
}

fn wrapped(
    painter: &crate::gfx::Painter,
    text: &str,
    font: FontId,
    color: Color32,
    width: f32,
    center: bool,
) -> Arc<crate::gfx::Galley> {
    let mut job = crate::gfx::text::LayoutJob::simple(text.to_owned(), font, color, width);
    job.wrap.max_rows = 2;
    if center {
        job.halign = crate::gfx::Align::Center;
    }
    painter.layout_job(job)
}

pub(super) fn draw(
    chrome: &Chrome,
    ui: &mut crate::gfx::Ui,
    layout: &mut HomeLayout,
    panel: &PlayerPanel,
    slide: f32,
) {
    let m = UiMetrics::for_viewport(chrome.viewport);
    let form_factor = chrome.viewport.form_factor;
    let sheet = form_factor == UiFormFactor::Mobile;
    let available = chrome.rect.width() - chrome.margin() * 2.0;
    let width = m.panel_sync_width.min(if sheet {
        available.min(m.sheet_max_width)
    } else {
        available
    });
    let edge = m.panel_sync_edge;
    let row_w = width - edge * 2.0;
    let text_w = row_w - Dp::D6.px() * 2.0;
    let painter = ui.painter().clone();
    let status = find_row(panel, SyncRole::Reading)
        .and_then(|(_, row)| row.value.as_deref())
        .map(|text| {
            wrapped(
                &painter,
                text,
                FontId::proportional(m.panel_sync_status_font),
                crate::stylesheet::color("text.subtle"),
                text_w,
                false,
            )
        });
    let hint = find_row(panel, SyncRole::Auto)
        .and_then(|(_, row)| row.detail.as_deref())
        .map(|text| {
            wrapped(
                &painter,
                text,
                FontId::proportional(m.panel_sync_hint_font),
                crate::stylesheet::color("glyph"),
                text_w,
                true,
            )
        });
    let spec = SyncSheet {
        m: &m,
        fonts: Fonts::new(&m),
        painter: &painter,
        panel,
        sheet,
        row_w,
        status,
        hint,
    };

    let reserved = match form_factor {
        UiFormFactor::Desktop => m.panel_reserved_desktop + chrome.viewport.safe_bottom,
        _ if sheet => chrome.viewport.safe_bottom,
        _ => Space::S32.px(&m),
    };
    let available_height = if sheet {
        chrome.rect.height() * m.panel_sync_sheet_ratio
    } else {
        chrome.rect.height()
    };
    let fixed = spec.tree(0).outer_size().y;
    let room = available_height - reserved - chrome.margin() - fixed;
    let shown_choices = ((room / m.panel_sync_choice_height).floor().max(1.0)) as usize;
    let choices = panel
        .rows
        .iter()
        .filter(|row| row.sync == Some(SyncRole::Choice))
        .count()
        .min(shown_choices);
    let tree = spec.tree(choices);
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
        _ if sheet => dock(
            chrome.rect,
            Align2::CENTER_BOTTOM,
            Vec2::new(width, height + reserved),
            Insets::default(),
        ),
        _ => dock(
            chrome.rect,
            Align2::CENTER_CENTER,
            Vec2::new(width, height),
            Insets::default(),
        ),
    };

    let veil = chrome.modal_veil(ui, card, layout);
    if sheet {
        veil.rect_filled(chrome.rect, 0.0, Color32::from_black_alpha(110));
    } else if form_factor != UiFormFactor::Desktop {
        veil.rect_filled(chrome.rect, 0.0, shade(Shade::Veil));
    }
    let layer = veil.layer_id();
    let first = chrome
        .context
        .graphics_mut(|graphics| graphics.entry(layer).next_idx());
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
    let icon = |name: &str, slot: Rect, color: Color32| {
        if let Some(texture) = chrome.icons.get(name) {
            veil.image(texture, slot, full_uv(), color);
        }
    };
    let node_of = |index: usize| NODE_PLAYER_PANEL_ROW_BASE + index as u64;
    let text = |slot: Rect, label: &str, font: FontId, color: Color32| {
        veil.text(slot.left_center(), Align2::LEFT_CENTER, label, font, color);
    };
    let fonts = &spec.fonts;

    if let Some((index, row)) = find_row(panel, SyncRole::Back)
        && let Some(hit) = placed.find(index)
    {
        let node = node_of(index);
        let response = ui.interact(hit, Id::new(("fluxa-sync-back", node)), Sense::click());
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
        icon(
            "ArrowLeft",
            flex::anchored(
                hit,
                Align2::CENTER_CENTER,
                Vec2::splat(m.panel_sync_back_icon),
                0.0,
            ),
            if on { Color32::BLACK } else { ink(215) },
        );
        if response.clicked() {
            layout.activated = Some(node);
        }
        if let Some(title) = placed.find(BACK_TITLE) {
            text(title, &row.label, fonts.back.clone(), text_primary());
        }
    }
    if let Some((index, row)) = find_row(panel, SyncRole::Reset)
        && let Some(hit) = placed.find(index)
    {
        let node = node_of(index);
        let response = ui.interact(hit, Id::new(("fluxa-sync-reset", node)), Sense::click());
        layout.focusable.push((node, hit));
        let on = chrome.focused == Some(node);
        if on || response.hovered() {
            veil.rect_filled(
                hit,
                hit.height() / 2.0,
                if on {
                    text_primary()
                } else {
                    crate::stylesheet::color("track")
                },
            );
        }
        veil.text(
            hit.center(),
            Align2::CENTER_CENTER,
            &row.label,
            fonts.reset.clone(),
            if on {
                Color32::BLACK
            } else {
                crate::stylesheet::color("glyph")
            },
        );
        if response.clicked() {
            layout.activated = Some(node);
        }
    }

    if let Some((_, row)) = find_row(panel, SyncRole::Reading)
        && let Some(pill) = placed.find(PILL)
    {
        let (number, suffix) = split_reading(&row.label);
        if let Some(slot) = placed.find(NUMBER) {
            text(slot, number, fonts.big.clone(), text_primary());
        }
        if let Some(slot) = placed.find(SUFFIX) {
            text(
                slot,
                suffix,
                fonts.unit.clone(),
                crate::stylesheet::color("glyph"),
            );
        }
        veil.rect_filled(pill, pill.height() / 2.0, crate::stylesheet::color("track"));
        veil.rect_stroke(
            pill,
            pill.height() / 2.0,
            crate::gfx::Stroke::new(stroke::hairline(), m.border),
            crate::gfx::StrokeKind::Inside,
        );
        veil.line_segment(
            [
                pill.center_top() + Vec2::new(0.0, Dp::D6.px()),
                pill.center_bottom() - Vec2::new(0.0, Dp::D6.px()),
            ],
            crate::gfx::Stroke::new(stroke::hairline(), m.border),
        );
        for (role, slot) in [(SyncRole::Minus, MINUS), (SyncRole::Plus, PLUS)] {
            let Some((index, step)) = find_row(panel, role) else {
                continue;
            };
            let Some(half) = placed.find(slot) else {
                continue;
            };
            let node = node_of(index);
            let response = ui.interact(half, Id::new(("fluxa-sync-step", node)), Sense::click());
            layout.focusable.push((node, half));
            let on = chrome.focused == Some(node);
            let lit = half.shrink(stroke::hairline());
            if on {
                veil.rect_filled(lit, pill.height() / 2.0, text_primary());
            } else if response.is_pointer_button_down_on() {
                veil.rect_filled(lit, pill.height() / 2.0, crate::stylesheet::color("strong"));
            } else if response.hovered() {
                veil.rect_filled(lit, pill.height() / 2.0, crate::stylesheet::color("well"));
            }
            veil.text(
                half.center(),
                Align2::CENTER_CENTER,
                &step.label,
                fonts.step.clone(),
                if on { Color32::BLACK } else { text_primary() },
            );
            if response.clicked() {
                layout.activated = Some(node);
            }
        }
    }
    if let (Some(status), Some(slot)) = (&spec.status, placed.find(STATUS)) {
        veil.galley(
            slot.min,
            status.clone(),
            crate::stylesheet::color("text.subtle"),
        );
    }

    if let Some((index, row)) = find_row(panel, SyncRole::Auto)
        && let Some(rect) = placed.find(index)
    {
        let node = node_of(index);
        let response = ui.interact(rect, Id::new(("fluxa-sync-auto", node)), Sense::click());
        layout.focusable.push((node, rect));
        let on = chrome.focused == Some(node);
        veil.rect_filled(
            rect,
            rect.height() / 2.0,
            if response.hovered() {
                crate::stylesheet::color("text.body")
            } else {
                text_primary()
            },
        );
        if on {
            veil.rect_stroke(
                rect.expand(m.focus_ring_expand),
                rect.height() / 2.0 + m.focus_ring_expand,
                crate::gfx::Stroke::new(stroke::emphasis(), text_primary()),
                crate::gfx::StrokeKind::Outside,
            );
        }
        if let Some(slot) = placed.find(AUTO_ICON) {
            icon(
                row.icon.as_deref().unwrap_or("AudioLines"),
                slot,
                Color32::BLACK,
            );
        }
        if let Some(slot) = placed.find(AUTO_LABEL) {
            text(slot, &row.label, fonts.auto.clone(), Color32::BLACK);
        }
        if response.clicked() {
            layout.activated = Some(node);
        }
    }
    if let (Some(hint), Some(slot)) = (&spec.hint, placed.find(HINT)) {
        veil.galley(
            Pos2::new(slot.center().x, slot.top()),
            hint.clone(),
            crate::stylesheet::color("glyph"),
        );
    }
    if let Some(slot) = placed.find(RULE) {
        veil.line_segment(
            [slot.left_top(), slot.right_top()],
            crate::gfx::Stroke::new(stroke::hairline(), m.border),
        );
    }

    for (index, row) in panel.rows.iter().enumerate() {
        let Some(role) = row.sync else { continue };
        let Some(rect) = placed.find(index) else {
            continue;
        };
        if !matches!(role, SyncRole::Method | SyncRole::Choice) {
            continue;
        }
        let node = node_of(index);
        let response = ui.interact(rect, Id::new(("fluxa-sync-row", node)), Sense::click());
        layout.focusable.push((node, rect));
        let on = chrome.focused == Some(node);
        let fill = if on {
            text_primary()
        } else if role == SyncRole::Choice && row.selected {
            crate::stylesheet::color("track")
        } else if response.hovered() {
            crate::stylesheet::color("well")
        } else {
            Color32::TRANSPARENT
        };
        veil.rect_filled(
            inset_rect(rect, Insets::xy(0.0, Dp::D2.px())),
            m.panel_sync_row_radius,
            fill,
        );
        let strong = if on { Color32::BLACK } else { text_primary() };
        let soft = if on {
            Color32::from_black_alpha(150)
        } else {
            crate::stylesheet::color("glyph")
        };
        if role == SyncRole::Method {
            let name = if row.chevron {
                "ChevronRight"
            } else if row.selected {
                "ChevronUp"
            } else {
                "ChevronDown"
            };
            if let Some(slot) = placed.find(spec.part(index, 3)) {
                icon(name, slot, soft);
            }
            if let (Some(name), Some(slot)) =
                (row.icon.as_deref(), placed.find(spec.part(index, 0)))
            {
                icon(
                    name,
                    slot,
                    if on {
                        Color32::BLACK
                    } else {
                        crate::stylesheet::color("text.subtle")
                    },
                );
            }
            if let Some(slot) = placed.find(spec.part(index, 1)) {
                text(slot, &row.label, fonts.label.clone(), strong);
            }
            if let (Some(detail), Some(slot)) =
                (row.detail.as_deref(), placed.find(spec.part(index, 2)))
            {
                text(
                    slot,
                    &spec.clipped(detail, &fonts.detail, spec.detail_room()),
                    fonts.detail.clone(),
                    soft,
                );
            }
        } else {
            if row.selected
                && let Some(slot) = placed.find(spec.part(index, 3))
            {
                icon("Check", slot, strong);
            }
            if let Some(slot) = placed.find(spec.part(index, 1)) {
                text(
                    slot,
                    &spec.clipped(&row.label, &fonts.choice, spec.choice_room(row)),
                    fonts.choice.clone(),
                    if row.selected || on { strong } else { ink(215) },
                );
            }
        }
        if response.clicked() {
            layout.activated = Some(node);
        }
    }

    if slide < 1.0 && sheet {
        let offset = Vec2::new(0.0, (1.0 - slide) * card.height());
        chrome.context.graphics_mut(|graphics| {
            let list = graphics.entry(layer);
            let last = list.next_idx();
            list.transform_range(
                first,
                last,
                crate::gfx::emath::TSTransform::from_translation(offset),
            );
        });
    }
}
