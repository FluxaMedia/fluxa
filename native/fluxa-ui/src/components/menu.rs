use super::*;

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
    pub rows: Vec<Rect>,
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

    fn icon_size(self) -> f32 {
        if self.large_rows() { 24.0 } else { 18.0 }
    }

    fn row_radius(self) -> f32 {
        if self == Self::Sheet { 12.0 } else { 8.0 }
    }

    fn row_inset(self) -> f32 {
        if self == Self::Sheet { 16.0 } else { 12.0 }
    }

    fn header_inset(self) -> f32 {
        if self == Self::Sheet { 24.0 } else { 12.0 }
    }

    fn rule_shrink(self) -> f32 {
        if self == Self::Sheet { 0.0 } else { 16.0 }
    }
}

pub fn action_menu_layout(viewport: Viewport, count: usize, anchor: Pos2) -> ActionMenuLayout {
    let count = count as f32;
    let style = ActionMenuStyle::for_viewport(viewport);
    if style == ActionMenuStyle::Sheet {
        let (header, row) = (60.0, 56.0);
        let columns = if count > 8.0 { 2 } else { 1 };
        let lines = (count / columns as f32).ceil();
        let column_width = (viewport.width - 16.0) / columns as f32;
        let height = 20.0 + header + row * lines + viewport.safe_bottom + 12.0;
        let panel = Rect::from_min_size(
            Pos2::new(0.0, viewport.height - height),
            Vec2::new(viewport.width, height),
        );
        let top = panel.top() + 20.0 + header;
        ActionMenuLayout {
            panel,
            header: Some(Rect::from_min_size(
                Pos2::new(panel.left(), panel.top() + 20.0),
                Vec2::new(panel.width(), header),
            )),
            rows: (0..count as usize)
                .map(|index| {
                    let (line, column) = (index / columns, index % columns);
                    Rect::from_min_size(
                        Pos2::new(8.0 + column_width * column as f32, top + row * line as f32),
                        Vec2::new(column_width, row),
                    )
                })
                .collect(),
        }
    } else if style == ActionMenuStyle::Panel {
        let (header, row, pad, width) = (72.0, 60.0, 16.0, 520.0);
        let height = header + row * count + pad * 2.0;
        let panel = Rect::from_center_size(
            Pos2::new(viewport.width * 0.5, viewport.height * 0.5),
            Vec2::new(width, height),
        );
        ActionMenuLayout {
            panel,
            header: Some(Rect::from_min_size(
                Pos2::new(panel.left() + pad, panel.top() + pad),
                Vec2::new(width - pad * 2.0, header),
            )),
            rows: (0..count as usize)
                .map(|index| {
                    Rect::from_min_size(
                        Pos2::new(
                            panel.left() + pad,
                            panel.top() + pad + header + row * index as f32,
                        ),
                        Vec2::new(width - pad * 2.0, row),
                    )
                })
                .collect(),
        }
    } else {
        let (row, pad, width) = (40.0, 6.0, 240.0);
        let height = row * count + pad * 2.0;
        let x = if anchor.x + width > viewport.width - 8.0 {
            anchor.x - width
        } else {
            anchor.x
        };
        let y = if anchor.y + height > viewport.height - 8.0 {
            anchor.y - height
        } else {
            anchor.y
        };
        let panel =
            Rect::from_min_size(Pos2::new(x.max(8.0), y.max(8.0)), Vec2::new(width, height));
        ActionMenuLayout {
            panel,
            header: None,
            rows: (0..count as usize)
                .map(|index| {
                    Rect::from_min_size(
                        Pos2::new(panel.left() + pad, panel.top() + pad + row * index as f32),
                        Vec2::new(width - pad * 2.0, row),
                    )
                })
                .collect(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_action_menu(
    context: &egui::Context,
    viewport: Viewport,
    metrics: UiMetrics,
    assets: &impl HomeAssets,
    title: &str,
    items: &[ActionMenuItem],
    selected: Option<usize>,
    anchor: Pos2,
    serial: u64,
) -> Option<ActionMenuOutcome> {
    let layout = action_menu_layout(viewport, items.len(), anchor);
    let style = ActionMenuStyle::for_viewport(viewport);
    let sheet = style == ActionMenuStyle::Sheet;
    let tv = style == ActionMenuStyle::Panel;
    if context.data(|data| data.get_temp::<u64>(Id::new("fluxa-action-menu-serial")))
        != Some(serial)
    {
        context.data_mut(|data| data.insert_temp(Id::new("fluxa-action-menu-serial"), serial));
        crate::motion::reset(context, Id::new(("fluxa-action-menu", serial)));
    }
    let eased = crate::motion::appear(
        context,
        Id::new(("fluxa-action-menu", serial)),
        crate::motion::normal(),
        crate::motion::Ease::Out,
    );
    let offset = if sheet {
        Vec2::new(0.0, (1.0 - eased) * layout.panel.height())
    } else {
        Vec2::ZERO
    };
    let mut outcome = None;
    egui::Area::new(Id::new("fluxa-action-menu-area"))
        .fixed_pos(Pos2::ZERO)
        .order(egui::Order::Debug)
        .show(context, |ui| {
            let screen =
                Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
            let painter = ui.painter().clone().with_clip_rect(Rect::EVERYTHING);
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
            let border = egui::Stroke::new(stroke::hairline(), metrics.border);
            let sheet_radius = metrics.dialog_radius + 4.0;
            if sheet && liquid_glass() {
                glass(
                    &painter,
                    panel.with_max_y(panel.max.y + 40.0),
                    sheet_radius,
                    fill,
                );
                painter.rect_filled(
                    Rect::from_center_size(
                        Pos2::new(panel.center().x, panel.top() + 10.0),
                        Vec2::new(36.0, 4.0),
                    ),
                    2.0,
                    metrics.ink(tone::strong()),
                );
            } else if liquid_glass() && tv {
                glass(&painter, panel, metrics.card_radius, fill);
            } else if sheet {
                painter.rect(
                    panel.with_max_y(panel.max.y + 40.0),
                    egui::CornerRadius {
                        nw: sheet_radius as u8,
                        ne: sheet_radius as u8,
                        sw: 0,
                        se: 0,
                    },
                    fill,
                    border,
                    egui::StrokeKind::Inside,
                );
                painter.rect_filled(
                    Rect::from_center_size(
                        Pos2::new(panel.center().x, panel.top() + 10.0),
                        Vec2::new(36.0, 4.0),
                    ),
                    2.0,
                    metrics.ink(tone::strong()),
                );
            } else {
                painter.rect_filled(
                    panel.expand(1.0).translate(Vec2::new(0.0, 6.0)),
                    metrics.card_radius + 2.0,
                    shade(Shade::Faint),
                );
                painter.rect(
                    panel,
                    metrics.card_radius,
                    metrics.surface,
                    border,
                    egui::StrokeKind::Inside,
                );
            }
            if let Some(header) = layout.header.map(|header| header.translate(offset)) {
                let inset = style.header_inset();
                draw_text(
                    &painter,
                    Pos2::new(header.left() + inset, header.center().y),
                    Align2::LEFT_CENTER,
                    truncate_to_width(
                        &painter,
                        title,
                        &FontId::proportional(metrics.text.title),
                        header.width() - inset * 2.0,
                    ),
                    FontId::proportional(metrics.text.title),
                    metrics.text_primary,
                    None,
                );
                painter.hline(
                    panel.x_range().shrink(style.rule_shrink()),
                    header.bottom() - 0.5,
                    egui::Stroke::new(stroke::hairline(), metrics.border),
                );
            }
            let label_size = if style.large_rows() {
                metrics.text.subtitle
            } else {
                metrics.text.body
            };
            let icon_size = style.icon_size();
            for (index, (item, row)) in items.iter().zip(&layout.rows).enumerate() {
                let row = row.translate(offset);
                let response = ui.interact(
                    row,
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
                painter.rect_filled(row, style.row_radius(), bg);
                let inset = style.row_inset();
                if let Some(texture) = item.app_icon.and_then(|id| assets.app_icon(id)) {
                    let size = icon_size + 12.0;
                    painter.image(
                        texture,
                        Rect::from_center_size(
                            Pos2::new(row.left() + inset + icon_size * 0.5, row.center().y),
                            Vec2::splat(size),
                        ),
                        full_uv(),
                        metrics.text_primary,
                    );
                    if let Some(check) = assets.icon(item.icon) {
                        painter.image(
                            check,
                            Rect::from_center_size(
                                Pos2::new(row.right() - inset - icon_size * 0.5, row.center().y),
                                Vec2::splat(icon_size),
                            ),
                            full_uv(),
                            ink,
                        );
                    }
                } else if let Some(icon) = assets.icon(item.icon) {
                    painter.image(
                        icon,
                        Rect::from_center_size(
                            Pos2::new(row.left() + inset + icon_size * 0.5, row.center().y),
                            Vec2::splat(icon_size),
                        ),
                        full_uv(),
                        ink,
                    );
                }
                draw_text(
                    &painter,
                    Pos2::new(row.left() + inset * 2.0 + icon_size, row.center().y),
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
