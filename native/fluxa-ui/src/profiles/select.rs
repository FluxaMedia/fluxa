use super::*;

pub(super) fn draw_select(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
) {
    let manage = model.manage;
    let empty = model.profiles.is_empty();
    let reveal = if empty {
        1.0
    } else {
        crate::motion::ease(
            ui.ctx(),
            Id::new("fluxa-profiles-manage-reveal"),
            f32::from(u8::from(manage)),
            crate::motion::normal(),
            crate::motion::Ease::Out,
        )
    };
    let add_active = manage || empty;
    let add_slot = empty || reveal > 0.001;
    let count = model.profiles.len() + usize::from(add_slot);
    let fade = |alpha: u8| (f32::from(alpha) * reveal) as u8;
    let gap = metrics.profile_gap;
    let (radius, cell) = if viewport.is_compact() {
        let cell_width = ((viewport.width - Space::S16.px(&metrics) * 2.0 - gap) / 2.0)
            .min(metrics.profile_cell_width);
        let radius = cell_width * 0.325;
        (radius, Vec2::new(cell_width, radius * 2.0 + 72.0))
    } else {
        (
            metrics.profile_avatar_radius,
            Vec2::new(metrics.profile_cell_width, 222.0),
        )
    };
    let side = metrics.profile_side_margin;
    let per_row =
        (((viewport.width - side + gap) / (cell.x + gap)).floor() as usize).clamp(1, count);
    let rows = count.div_ceil(per_row);
    let base_rows = model.profiles.len().div_ceil(per_row).max(1);
    let shown_rows = base_rows as f32 + (rows - base_rows) as f32 * reveal;
    let grid_height = shown_rows * cell.y + (shown_rows - 1.0) * gap;
    let content_height = 72.0 + 24.0 + 46.0 + 52.0 + grid_height + 90.0;
    let top = if viewport.is_compact() {
        56.0
    } else {
        ((viewport.height - content_height) * 0.5).max(48.0)
    };
    ui.add_space(top);

    let width = ui.available_width();
    let (brand_rect, _) = ui.allocate_exact_size(Vec2::new(width, 72.0), Sense::hover());
    components::brand_lockup(ui.painter(), brand_rect.center(), 72.0, 255, assets);
    ui.add_space(Space::S24.px(&metrics));

    let title = ui.painter().layout_no_wrap(
        t("profiles.who_watching"),
        crate::fonts::bold(metrics.profile_title_size),
        text_primary(),
    );
    let (title_rect, _) = ui.allocate_exact_size(Vec2::new(width, 46.0), Sense::hover());
    let title_pos = Pos2::new(
        title_rect.center().x - title.size().x * 0.5,
        title_rect.center().y - title.size().y * 0.5,
    );
    let title_right = title_pos.x + title.size().x;
    ui.painter().galley(title_pos, title, text_primary());
    let gear_rect = Rect::from_center_size(
        Pos2::new(title_right + 38.0, title_rect.center().y),
        Vec2::splat(metrics.icon_button_size),
    );
    let blocked = model.pin_prompt.is_some() || model.confirm_delete.is_some();
    let gear = ui
        .interact(gear_rect, Id::new("fluxa-profiles-gear"), Sense::click())
        .on_hover_text(t("profiles.picker_settings"));
    if let Some(icon) = assets.icon("Settings") {
        ui.painter().image(
            icon,
            Rect::from_center_size(gear_rect.center(), Vec2::splat(26.0)),
            full_uv(),
            Color32::from_white_alpha(if gear.hovered() { 255 } else { 180 }),
        );
    }
    ui.add_space(52.0);

    let profiles = model.profiles.clone();
    let (grid_rect, _) = ui.allocate_exact_size(Vec2::new(width, grid_height), Sense::hover());
    let cells = grid_cells(grid_rect, cell, gap, count, per_row, viewport.is_compact());
    let node = |index: usize| NODE_PROFILES_CELL_BASE + index as u64;
    let neighbors = |index: usize| {
        let target = |dir| grid_step(index, count, per_row, dir).map(node);
        Neighbors {
            up: target(Dir::Up).or((index < per_row).then_some(NODE_PROFILES_GEAR)),
            down: target(Dir::Down).or((!profiles.is_empty()).then_some(NODE_PROFILES_MANAGE)),
            left: target(Dir::Left),
            right: target(Dir::Right),
        }
    };
    for index in 0..count {
        let origin = cells[index].min;
        let avatar_center = Pos2::new(origin.x + cell.x * 0.5, origin.y + radius);
        let avatar_rect = Rect::from_center_size(avatar_center, Vec2::splat(radius * 2.0));
        let hit = avatar_rect.union(Rect::from_min_size(
            Pos2::new(origin.x, avatar_center.y + radius),
            Vec2::new(cell.x, 40.0),
        ));
        let Some(profile) = profiles.get(index) else {
            let response = ui.interact(
                hit,
                Id::new("fluxa-profiles-add"),
                if add_active {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );
            if !blocked && add_active {
                response
                    .clone()
                    .focus_circle(ui, node(index), avatar_rect, &metrics)
                    .focus_neighbors(ui, node(index), neighbors(index));
            }
            let grow = 0.88 + 0.12 * reveal;
            let r = radius
                * grow
                * if response.hovered() && add_active {
                    1.04
                } else {
                    1.0
                };
            ui.painter().circle(
                avatar_center,
                r,
                ink(fade(if response.hovered() { 26 } else { 10 })),
                egui::Stroke::new(stroke::hairline(), metrics.border.gamma_multiply(reveal)),
            );
            if let Some(icon) = assets.icon("Plus") {
                ui.painter().image(
                    icon,
                    Rect::from_center_size(avatar_center, Vec2::splat(36.0 * grow)),
                    full_uv(),
                    ink(tone::muted()).gamma_multiply(reveal),
                );
            }
            draw_text(
                ui.painter(),
                Pos2::new(avatar_center.x, avatar_center.y + radius + 24.0),
                Align2::CENTER_CENTER,
                t("profiles.add_profile"),
                FontId::proportional(metrics.text.body),
                metrics.text_muted.gamma_multiply(reveal),
                None,
            );
            continue;
        };
        let response = ui.interact(hit, Id::new(("fluxa-profile", &profile.id)), Sense::click());
        if !blocked {
            response
                .clone()
                .focus_circle(ui, node(index), avatar_rect, &metrics)
                .focus_neighbors(ui, node(index), neighbors(index));
        }
        let card_hovered = ui.rect_contains_pointer(Rect::from_min_size(origin, cell));
        let r = if response.hovered() {
            radius * 1.04
        } else {
            radius
        };
        components::avatar_sized(
            ui.painter(),
            assets,
            avatar_center,
            r,
            radius * 1.04,
            &profile.name,
            profile.avatar_url.as_deref(),
            metrics,
        );
        if !response.hovered() {
            ui.painter()
                .circle_filled(avatar_center, r, Color32::from_black_alpha(30 + fade(70)));
        }
        if profile.locked {
            let badge = avatar_center + Vec2::splat(radius * 0.72);
            ui.painter().circle(
                badge,
                17.0,
                background().gamma_multiply(0.92),
                egui::Stroke::new(stroke::hairline(), ink(tone::raised())),
            );
            if let Some(icon) = assets.icon("Lock") {
                ui.painter().image(
                    icon,
                    Rect::from_center_size(badge, Vec2::splat(16.0)),
                    full_uv(),
                    ink(tone::body()),
                );
            }
        }
        let name_font = FontId::proportional(metrics.text.subtitle);
        let name = truncate_to_width(ui.painter(), &profile.name, &name_font, cell.x);
        let name_y = avatar_center.y + radius + 24.0;
        draw_text(
            ui.painter(),
            Pos2::new(avatar_center.x, name_y),
            Align2::CENTER_CENTER,
            name,
            name_font,
            ink(if response.hovered() { 255 } else { 215 }),
            None,
        );
        let mut actions_y = name_y + 24.0;
        if model.primary_id.as_ref() == Some(&profile.id) {
            let label = t("profiles.primary_badge").to_uppercase();
            let galley = ui.painter().layout_no_wrap(
                label,
                FontId::proportional(metrics.text.label),
                metrics.text_secondary,
            );
            let badge = Rect::from_center_size(
                Pos2::new(avatar_center.x, name_y + 22.0),
                galley.size() + Vec2::new(14.0, 6.0),
            );
            ui.painter()
                .rect_filled(badge, radius::chip(), ink(tone::soft()));
            ui.painter()
                .galley(badge.center() - galley.size() * 0.5, galley, text_primary());
            actions_y += 20.0;
        }
        let alpha = if card_hovered { 230 } else { 120 };
        let actions: &[(f32, &str, bool)] = if reveal > 0.001 {
            &[(-16.0, "Edit", false), (16.0, "Delete", true)]
        } else {
            &[]
        };
        for &(offset, icon, is_delete) in actions {
            let label = if is_delete {
                t("profiles.delete")
            } else {
                t("profiles.edit")
            };
            let rect = Rect::from_center_size(
                Pos2::new(
                    avatar_center.x + offset,
                    actions_y + 6.0 + (1.0 - reveal) * 10.0,
                ),
                Vec2::splat(28.0),
            );
            let action = ui
                .interact(
                    rect,
                    Id::new(("fluxa-profile-action", &profile.id, is_delete)),
                    if manage {
                        Sense::click()
                    } else {
                        Sense::hover()
                    },
                )
                .on_hover_text(label);
            if action.hovered() && manage {
                ui.painter()
                    .rect_filled(rect, radius::control(), ink(tone::soft()));
            }
            let color = ink(fade(if action.hovered() { 255 } else { alpha }));
            if let Some(texture) = assets.icon(icon) {
                ui.painter().image(
                    texture,
                    Rect::from_center_size(rect.center(), Vec2::splat(16.0)),
                    full_uv(),
                    color,
                );
            }
            if manage && action.clicked() {
                let purpose = if is_delete {
                    PinPurpose::Delete
                } else {
                    PinPurpose::Edit
                };
                model.begin(profile, purpose);
            }
        }
    }
    if !blocked {
        gear.focus_node(ui, NODE_PROFILES_GEAR, &metrics)
            .focus_neighbors(
                ui,
                NODE_PROFILES_GEAR,
                Neighbors {
                    down: Some(node(0)),
                    ..Neighbors::default()
                },
            );
    }
    if !profiles.is_empty() {
        ui.add_space(Space::S24.px(&metrics));
        let label = if manage {
            t("profiles.done")
        } else {
            t("profiles.manage")
        };
        let button = ui
            .vertical_centered(|ui| {
                components::pill_button(
                    ui,
                    None,
                    &label,
                    Some(if viewport.is_compact() { 220.0 } else { 240.0 }),
                    metrics.button_height,
                    metrics.text.body,
                    manage,
                    None,
                )
            })
            .inner;
        if !blocked {
            button
                .focus_node(ui, NODE_PROFILES_MANAGE, &metrics)
                .focus_neighbors(
                    ui,
                    NODE_PROFILES_MANAGE,
                    Neighbors {
                        up: Some(node(count - 1)),
                        ..Neighbors::default()
                    },
                );
        }
    }
    ui.add_space(Space::S48.px(&metrics));
}

pub(super) fn grid_step(index: usize, count: usize, per_row: usize, dir: Dir) -> Option<usize> {
    let row = index / per_row;
    let column = index % per_row;
    let row_start = row * per_row;
    let row_len = (count - row_start).min(per_row);
    match dir {
        Dir::Left => Some(row_start + (column + row_len - 1) % row_len),
        Dir::Right => Some(row_start + (column + 1) % row_len),
        Dir::Up => row.checked_sub(1).map(|up| up * per_row + column),
        Dir::Down => {
            let next = row_start + per_row;
            (next < count).then(|| (next + column).min(count - 1))
        }
    }
}

fn grid_cells(
    grid: Rect,
    cell: Vec2,
    gap: f32,
    count: usize,
    per_row: usize,
    padded_rows: bool,
) -> Vec<Rect> {
    let rows = (0..count.div_ceil(per_row)).map(|row| {
        let slots = if padded_rows {
            per_row
        } else {
            (count - row * per_row).min(per_row)
        };
        crate::flex::row((0..slots).map(|column| leaf(cell).id(row * per_row + column)))
            .gap(gap)
            .justify(Justify::Center)
    });
    let placed = column(rows).gap(gap).align(Align::Stretch).solve(grid);
    placed.tagged().into_iter().map(|(_, rect)| rect).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> Rect {
        Rect::from_min_size(Pos2::new(0.0, 100.0), Vec2::new(1000.0, 500.0))
    }

    #[test]
    fn short_last_row_is_centered_on_desktop() {
        let cells = grid_cells(grid(), Vec2::new(150.0, 222.0), 40.0, 7, 4, false);
        assert_eq!(cells.len(), 7);
        assert_eq!(cells[0].left(), (1000.0 - (4.0 * 150.0 + 3.0 * 40.0)) * 0.5);
        assert_eq!(cells[4].left(), (1000.0 - (3.0 * 150.0 + 2.0 * 40.0)) * 0.5);
        assert_eq!(cells[4].top(), 100.0 + 222.0 + 40.0);
    }

    #[test]
    fn left_and_right_wrap_within_the_row() {
        assert_eq!(grid_step(0, 7, 4, Dir::Left), Some(3));
        assert_eq!(grid_step(3, 7, 4, Dir::Right), Some(0));
        assert_eq!(grid_step(6, 7, 4, Dir::Right), Some(4));
        assert_eq!(grid_step(4, 7, 4, Dir::Left), Some(6));
    }

    #[test]
    fn vertical_moves_clamp_to_the_short_row() {
        assert_eq!(grid_step(3, 7, 4, Dir::Down), Some(6));
        assert_eq!(grid_step(1, 7, 4, Dir::Down), Some(5));
        assert_eq!(grid_step(5, 7, 4, Dir::Up), Some(1));
        assert_eq!(grid_step(1, 7, 4, Dir::Up), None);
        assert_eq!(grid_step(5, 7, 4, Dir::Down), None);
    }

    #[test]
    fn single_cell_row_stays_put() {
        assert_eq!(grid_step(0, 1, 1, Dir::Left), Some(0));
        assert_eq!(grid_step(0, 1, 1, Dir::Right), Some(0));
    }

    #[test]
    fn short_last_row_stays_on_the_grid_columns_on_mobile() {
        let cells = grid_cells(grid(), Vec2::new(180.0, 202.0), 28.0, 3, 2, true);
        assert_eq!(cells[2].left(), cells[0].left());
        assert_eq!(cells[1].left(), cells[0].right() + 28.0);
        assert_eq!(cells[2].top(), cells[0].bottom() + 28.0);
    }
}
