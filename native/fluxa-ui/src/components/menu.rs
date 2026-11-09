use super::*;
use crate::Space;
use crate::dp::Dp;

#[derive(Clone)]
pub struct ActionMenuItem {
    pub icon: &'static str,
    pub label: String,
    pub app_icon: Option<&'static str>,
}

pub enum ActionMenuOutcome {
    Pick(usize),
    Dismiss,
}

pub struct ActionMenuLayout {
    pub panel: Rect,
    pub body: Rect,
    pub rows: Vec<Rect>,
    pub max_scroll: f32,
    header: Option<Rect>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActionMenuStyle {
    Sheet,
    Panel,
    Popover,
}

impl ActionMenuStyle {
    fn for_viewport(viewport: Viewport) -> Self {
        if viewport.is_compact() {
            Self::Sheet
        } else if viewport.is_tv() {
            Self::Panel
        } else {
            Self::Popover
        }
    }

    fn has_scrim(self) -> bool {
        self != Self::Popover
    }

    fn large_rows(self) -> bool {
        self != Self::Popover
    }

    fn icon_size(self, metrics: &UiMetrics) -> f32 {
        metrics.surface_menu_icon
    }

    fn row_space(self) -> Space {
        if self == Self::Sheet {
            Space::S16
        } else {
            Space::S12
        }
    }
}

fn scroll_fit(
    content: f32,
    fixed: f32,
    cap: f32,
    min_visible: f32,
    scroll: f32,
) -> (f32, f32, f32) {
    let visible = content.min((cap - fixed).max(min_visible));
    let max_scroll = (content - visible).max(0.0);
    (visible, max_scroll, scroll.clamp(0.0, max_scroll))
}

fn row_rects(area: Rect, count: usize, columns: usize, row: f32, inset: f32) -> Vec<Rect> {
    let lines = count.div_ceil(columns);
    let cell = (area.width() - inset * 2.0) / columns as f32;
    let tree = crate::flex::column((0..lines).map(|line| {
        let first = line * columns;
        crate::flex::row(
            (first..(first + columns).min(count))
                .map(|index| crate::flex::leaf(Vec2::new(cell, row)).id(index)),
        )
    }))
    .padding(crate::flex::Insets::xy(inset, 0.0));
    let placed = tree.solve(area);
    (0..count)
        .map(|index| placed.find(index).unwrap_or(area))
        .collect()
}

fn scrolled(body: Rect, scroll: f32, count: usize, columns: usize, row: f32) -> Rect {
    let lines = count.div_ceil(columns);
    Rect::from_min_size(
        body.min - Vec2::new(0.0, scroll),
        Vec2::new(body.width(), row * lines as f32),
    )
}

pub fn action_menu_width(viewport: Viewport, metrics: UiMetrics, items: &[ActionMenuItem]) -> f32 {
    let style = ActionMenuStyle::for_viewport(viewport);
    if style != ActionMenuStyle::Popover {
        return 0.0;
    }
    let widest = items
        .iter()
        .map(|item| {
            crate::navigation::estimated_navigation_text_width(&item.label, metrics.text.body)
        })
        .fold(0.0, f32::max);
    let chrome = metrics.menu_popover_pad * 2.0
        + metrics.surface_menu_row_inset * 3.0
        + style.icon_size(&metrics);
    (widest + chrome)
        .max(metrics.menu_popover_min_width)
        .min(viewport.width - Space::S16.px(&metrics))
}

pub fn action_menu_layout(
    viewport: Viewport,
    count: usize,
    anchor: Pos2,
    scroll: f32,
    width: f32,
) -> ActionMenuLayout {
    let metrics = UiMetrics::for_viewport(viewport);
    let style = ActionMenuStyle::for_viewport(viewport);
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let (header_id, body_id) = (0, 1);
    let row = metrics.surface_menu_row;
    let header = metrics.surface_menu_header;
    let min_visible = row * Dp::D3.px();
    let edge = Space::S8.px(&metrics);
    match style {
        ActionMenuStyle::Sheet => {
            let columns = if count > metrics.surface_menu_split as usize {
                2
            } else {
                1
            };
            let lines = count.div_ceil(columns) as f32;
            let fixed = metrics.surface_menu_pad_top
                + header
                + viewport.safe_bottom
                + metrics.surface_menu_pad_bottom;
            let (visible, max_scroll, scroll) = scroll_fit(
                row * lines,
                fixed,
                viewport.height * metrics.surface_menu_height_share,
                min_visible,
                scroll,
            );
            let panel = crate::flex::anchored(
                screen,
                Align2::CENTER_BOTTOM,
                Vec2::new(viewport.width, fixed + visible),
                0.0,
            );
            let frame = crate::flex::column([
                crate::flex::leaf(Vec2::new(panel.width(), header)).id(header_id),
                crate::flex::leaf(Vec2::new(panel.width(), visible)).id(body_id),
            ])
            .padding(crate::flex::Insets {
                left: 0.0,
                top: metrics.surface_menu_pad_top,
                right: 0.0,
                bottom: 0.0,
            })
            .solve(panel);
            let body = frame.find(body_id).unwrap_or(panel);
            ActionMenuLayout {
                panel,
                body,
                max_scroll,
                header: frame.find(header_id),
                rows: row_rects(
                    scrolled(body, scroll, count, columns, row),
                    count,
                    columns,
                    row,
                    metrics.surface_menu_pad_side,
                ),
            }
        }
        ActionMenuStyle::Panel => {
            let pad = metrics.surface_menu_pad_top;
            let fixed = header + pad * 2.0;
            let (visible, max_scroll, scroll) = scroll_fit(
                row * count as f32,
                fixed,
                viewport.height * metrics.surface_menu_height_share,
                min_visible,
                scroll,
            );
            let panel = crate::flex::anchored(
                screen,
                Align2::CENTER_CENTER,
                Vec2::new(metrics.surface_menu_panel_width, fixed + visible),
                0.0,
            );
            let inner = metrics.surface_menu_panel_width - pad * 2.0;
            let frame = crate::flex::column([
                crate::flex::leaf(Vec2::new(inner, header)).id(header_id),
                crate::flex::leaf(Vec2::new(inner, visible)).id(body_id),
            ])
            .padding(crate::flex::Insets::all(pad))
            .solve(panel);
            let body = frame.find(body_id).unwrap_or(panel);
            ActionMenuLayout {
                panel,
                body,
                max_scroll,
                header: frame.find(header_id),
                rows: row_rects(scrolled(body, scroll, count, 1, row), count, 1, row, 0.0),
            }
        }
        ActionMenuStyle::Popover => {
            let pad = metrics.menu_popover_pad;
            let width = width.max(metrics.menu_popover_min_width);
            let (visible, max_scroll, scroll) = scroll_fit(
                row * count as f32,
                pad * 2.0,
                viewport.height - edge * 2.0,
                min_visible,
                scroll,
            );
            let height = visible + pad * 2.0;
            let x = if anchor.x + width > viewport.width - edge {
                anchor.x - width
            } else {
                anchor.x
            };
            let y = if anchor.y + height > viewport.height - edge {
                anchor.y - height
            } else {
                anchor.y
            };
            let panel = Rect::from_min_size(
                Pos2::new(x.max(edge), y.max(edge)),
                Vec2::new(width, height),
            );
            let frame =
                crate::flex::column([
                    crate::flex::leaf(Vec2::new(width - pad * 2.0, visible)).id(body_id)
                ])
                .padding(crate::flex::Insets::all(pad))
                .solve(panel);
            let body = frame.find(body_id).unwrap_or(panel);
            ActionMenuLayout {
                panel,
                body,
                max_scroll,
                header: None,
                rows: row_rects(scrolled(body, scroll, count, 1, row), count, 1, row, 0.0),
            }
        }
    }
}

pub fn action_menu_scroll_to(
    viewport: Viewport,
    count: usize,
    anchor: Pos2,
    index: usize,
    scroll: f32,
    width: f32,
) -> f32 {
    let layout = action_menu_layout(viewport, count, anchor, 0.0, width);
    let Some(row) = layout.rows.get(index) else {
        return scroll;
    };
    let from_top = row.top() - layout.body.top();
    let (lo, hi) = (from_top + row.height() - layout.body.height(), from_top);
    scroll.clamp(lo.min(hi), hi).clamp(0.0, layout.max_scroll)
}

pub fn action_menu_hit(layout: &ActionMenuLayout, position: Pos2) -> Option<usize> {
    if !layout.body.contains(position) {
        return None;
    }
    layout.rows.iter().position(|row| row.contains(position))
}

#[derive(Clone)]
struct MenuSnapshot {
    title: String,
    items: Vec<ActionMenuItem>,
    selected: Option<usize>,
    anchor: Pos2,
    scroll: f32,
}

fn snapshot_id() -> Id {
    Id::new("fluxa-action-menu-snapshot")
}

#[allow(clippy::too_many_arguments)]
pub fn draw_action_menu(
    context: &crate::gfx::Context,
    viewport: Viewport,
    metrics: UiMetrics,
    assets: &impl HomeAssets,
    title: &str,
    items: &[ActionMenuItem],
    selected: Option<usize>,
    anchor: Pos2,
    scroll: f32,
    serial: u64,
) -> Option<ActionMenuOutcome> {
    crate::kit::provide(context, metrics);
    if context.data(|data| data.get_temp::<u64>(Id::new("fluxa-action-menu-serial")))
        != Some(serial)
    {
        context.data_mut(|data| data.insert_temp(Id::new("fluxa-action-menu-serial"), serial));
        crate::motion::reset(context, Id::new(("fluxa-action-menu", serial)));
    }
    crate::motion::reset(context, Id::new("fluxa-action-menu-exit"));
    context.data_mut(|data| {
        data.insert_temp(
            snapshot_id(),
            MenuSnapshot {
                title: title.to_owned(),
                items: items.to_vec(),
                selected,
                anchor,
                scroll,
            },
        )
    });
    let eased = crate::motion::appear(
        context,
        Id::new(("fluxa-action-menu", serial)),
        crate::motion::normal(),
        crate::motion::Ease::Out,
    );
    draw_menu(
        context, viewport, metrics, assets, title, items, selected, anchor, scroll, eased,
    )
}

pub fn draw_action_menu_closing(
    context: &crate::gfx::Context,
    viewport: Viewport,
    metrics: UiMetrics,
    assets: &impl HomeAssets,
) {
    crate::kit::provide(context, metrics);
    let Some(menu) = context.data(|data| data.get_temp::<MenuSnapshot>(snapshot_id())) else {
        return;
    };
    let eased = crate::motion::disappear(
        context,
        Id::new("fluxa-action-menu-exit"),
        crate::motion::normal(),
        crate::motion::Ease::Token,
    );
    if eased <= 0.001 {
        context.data_mut(|data| data.remove::<MenuSnapshot>(snapshot_id()));
        crate::motion::reset(context, Id::new("fluxa-action-menu-exit"));
        return;
    }
    draw_menu(
        context,
        viewport,
        metrics,
        assets,
        &menu.title,
        &menu.items,
        menu.selected,
        menu.anchor,
        menu.scroll,
        eased,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_menu(
    context: &crate::gfx::Context,
    viewport: Viewport,
    metrics: UiMetrics,
    assets: &impl HomeAssets,
    title: &str,
    items: &[ActionMenuItem],
    selected: Option<usize>,
    anchor: Pos2,
    scroll: f32,
    eased: f32,
) -> Option<ActionMenuOutcome> {
    let layout = action_menu_layout(
        viewport,
        items.len(),
        anchor,
        scroll,
        action_menu_width(viewport, metrics, items),
    );
    let style = ActionMenuStyle::for_viewport(viewport);
    let sheet = style == ActionMenuStyle::Sheet;
    let tv = style == ActionMenuStyle::Panel;
    let offset = if sheet {
        Vec2::new(0.0, (1.0 - eased) * layout.panel.height())
    } else {
        Vec2::ZERO
    };
    let mut outcome = None;
    crate::gfx::Area::new(Id::new("fluxa-action-menu-area"))
        .fixed_pos(Pos2::ZERO)
        .order(crate::gfx::Order::Debug)
        .show(context, |ui| {
            let screen =
                Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
            let mut painter = ui.painter().clone().with_clip_rect(Rect::EVERYTHING);
            if !sheet {
                painter.set_opacity(eased);
            }
            let backdrop = ui.interact(
                screen,
                Id::new("fluxa-action-menu-backdrop"),
                Sense::click(),
            );
            if style.has_scrim() {
                painter.rect_filled(
                    screen,
                    0.0,
                    Color32::from_black_alpha((150.0 * eased) as u8),
                );
            }
            let panel = layout.panel.translate(offset);
            let fill = metrics.surface;
            let border = crate::gfx::Stroke::new(stroke::hairline(), metrics.border);
            let sheet_radius = metrics.dialog_radius + Space::S4.px(&metrics);
            if sheet && liquid_glass() {
                glass(
                    &painter,
                    panel.with_max_y(panel.max.y + Dp::D40.px()),
                    sheet_radius,
                    fill,
                );
                painter.rect_filled(
                    crate::flex::anchored(
                        panel,
                        Align2::CENTER_TOP,
                        Vec2::new(Dp::D36.px(), Dp::D4.px()),
                        Dp::D10.px(),
                    ),
                    2.0,
                    metrics.ink(tone::strong()),
                );
            } else if liquid_glass() && tv {
                glass(&painter, panel, metrics.card_radius, fill);
            } else if sheet {
                painter.rect(
                    panel.with_max_y(panel.max.y + Dp::D40.px()),
                    crate::gfx::CornerRadius {
                        nw: sheet_radius as u8,
                        ne: sheet_radius as u8,
                        sw: 0,
                        se: 0,
                    },
                    fill,
                    border,
                    crate::gfx::StrokeKind::Inside,
                );
                painter.rect_filled(
                    crate::flex::anchored(
                        panel,
                        Align2::CENTER_TOP,
                        Vec2::new(Dp::D36.px(), Dp::D4.px()),
                        Dp::D10.px(),
                    ),
                    2.0,
                    metrics.ink(tone::strong()),
                );
            } else {
                painter.rect_filled(
                    panel
                        .expand(stroke::hairline())
                        .translate(Vec2::new(0.0, Space::S6.px(&metrics))),
                    metrics.card_radius + Space::S2.px(&metrics),
                    shade(Shade::Faint),
                );
                painter.rect(
                    panel,
                    metrics.card_radius,
                    metrics.surface,
                    border,
                    crate::gfx::StrokeKind::Inside,
                );
            }
            if let Some(header) = layout.header.map(|header| header.translate(offset)) {
                let title_slot = crate::flex::row([crate::flex::fill()])
                    .padding(crate::flex::Insets::xy(
                        metrics.surface_menu_header_inset,
                        0.0,
                    ))
                    .align(crate::flex::Align::Center)
                    .solve(header);
                crate::flex::paint_text(
                    &painter,
                    title_slot.children.first().map_or(header, |slot| slot.rect),
                    title,
                    &FontId::proportional(metrics.text.title),
                    metrics.text_primary,
                    Align2::LEFT_CENTER,
                );
                painter.hline(
                    panel.x_range().shrink(metrics.surface_menu_rule_shrink),
                    header.bottom() - 0.5,
                    crate::gfx::Stroke::new(stroke::hairline(), metrics.border),
                );
            }
            let label_size = if style.large_rows() {
                metrics.text.subtitle
            } else {
                metrics.text.body
            };
            let icon_size = style.icon_size(&metrics);
            let body = layout.body.translate(offset);
            let outer = painter.clone();
            let painter = painter.with_clip_rect(body);
            for (index, (item, row)) in items.iter().zip(&layout.rows).enumerate() {
                let row = row.translate(offset);
                if !row.intersects(body) {
                    continue;
                }
                let response = ui.interact(
                    row.intersect(body),
                    Id::new(("fluxa-action-menu-row", index)),
                    Sense::click(),
                );
                let focused = selected == Some(index);
                let (bg, ink) = if tv && focused {
                    (metrics.text_primary, metrics.background)
                } else if focused || response.hovered() {
                    (metrics.ink(tone::soft()), metrics.text_primary)
                } else {
                    (Color32::TRANSPARENT, metrics.text_primary)
                };
                painter.rect_filled(row, metrics.surface_menu_row_radius, bg);
                let inset = metrics.surface_menu_row_inset;
                let app = item.app_icon.and_then(|id| assets.app_icon(id));
                let check = if app.is_some() {
                    assets.icon(item.icon)
                } else {
                    None
                };
                let leading = match app {
                    Some(texture) => Some((texture, metrics.text_primary)),
                    None => assets.icon(item.icon).map(|texture| (texture, ink)),
                };
                let shown = ui.scope_builder(crate::gfx::UiBuilder::new().max_rect(row), |ui| {
                    crate::kit::view()
                        .row()
                        .gap(style.row_space())
                        .height(row.height())
                        .class("menu-row")
                        .show(ui, |ui| {
                            crate::kit::space(ui, style.row_space());
                            let lead = ui
                                .allocate_exact_size(Vec2::splat(icon_size), Sense::hover())
                                .0;
                            let reserved = if check.is_some() {
                                icon_size + inset * 2.0
                            } else {
                                inset
                            };
                            let label = ui
                                .allocate_exact_size(
                                    Vec2::new(
                                        (ui.available_width() - reserved).max(0.0),
                                        row.height(),
                                    ),
                                    Sense::hover(),
                                )
                                .0;
                            let trailing = check.map(|_| {
                                ui.allocate_exact_size(Vec2::splat(icon_size), Sense::hover())
                                    .0
                            });
                            (lead, label, trailing)
                        })
                        .inner
                });
                let (lead, label, trailing) = shown.inner;
                if let Some((texture, tint)) = leading {
                    let size = if app.is_some() {
                        icon_size + Space::S12.px(&metrics)
                    } else {
                        icon_size
                    };
                    painter.image(
                        texture,
                        Rect::from_center_size(lead.center(), Vec2::splat(size)),
                        full_uv(),
                        tint,
                    );
                }
                if let (Some(check), Some(slot)) = (check, trailing) {
                    painter.image(check, slot, full_uv(), ink);
                }
                draw_text(
                    &painter,
                    Pos2::new(label.left(), row.center().y),
                    Align2::LEFT_CENTER,
                    &item.label,
                    FontId::proportional(label_size),
                    ink,
                    None,
                );
                if response.clicked() {
                    outcome = Some(ActionMenuOutcome::Pick(index));
                }
            }
            if layout.max_scroll > 0.0 {
                let track = body.height();
                let thumb = (track * track / (track + layout.max_scroll)).max(Dp::D24.px());
                let top =
                    body.top() + (track - thumb) * (scroll / layout.max_scroll).clamp(0.0, 1.0);
                let bar = Vec2::new(Dp::D3.px(), thumb);
                outer.rect_filled(
                    Rect::from_min_size(Pos2::new(panel.right() - Dp::D6.px(), top), bar),
                    bar.x * 0.5,
                    metrics.ink(tone::soft()),
                );
            }
            if outcome.is_none()
                && (backdrop.clicked() || backdrop.secondary_clicked())
                && !panel.contains(backdrop.interact_pointer_pos().unwrap_or(panel.center()))
            {
                outcome = Some(ActionMenuOutcome::Dismiss);
            }
        });
    if eased < 1.0 {
        context.request_repaint();
    }
    outcome
}
