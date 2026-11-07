use super::*;
use crate::kit;

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
    let brand = metrics.profile_brand_height;
    let header_gap = metrics.profile_header_gap;
    let footer = Space::S24.px(&metrics) + metrics.button_height;
    let content_height = brand + Space::S24.px(&metrics) + 46.0 + header_gap + grid_height + footer;
    let top = if viewport.is_compact() {
        56.0
    } else {
        ((viewport.height - content_height) * 0.5).max(48.0)
    };
    ui.add_space(top);

    let width = ui.available_width();
    let (brand_rect, _) = ui.allocate_exact_size(Vec2::new(width, brand), Sense::hover());
    components::brand_lockup(ui.painter(), brand_rect.center(), brand, 255, assets);
    ui.add_space(Space::S24.px(&metrics));

    let blocked = model.pin_prompt.is_some() || model.confirm_delete.is_some();
    let gear = components::picker_title(
        ui,
        assets,
        width,
        t("profiles.who_watching"),
        &t("profiles.picker_settings"),
        metrics,
    );
    ui.add_space(header_gap);

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
    let tips = (t("profiles.edit"), t("profiles.delete"));
    for index in 0..count {
        let origin = cells[index].min;
        let Some(profile) = profiles.get(index) else {
            components::add_tile(
                ui,
                assets,
                origin,
                cell,
                radius,
                add_active,
                reveal,
                t("profiles.add_profile"),
                |ui, response, rect| {
                    if !blocked && add_active {
                        response
                            .clone()
                            .focus_circle(ui, node(index), rect)
                            .focus_neighbors(ui, node(index), neighbors(index));
                    }
                },
                metrics,
            );
            continue;
        };
        let primary = model.primary_id.as_ref() == Some(&profile.id);
        let spec = components::ProfileTile {
            id: &profile.id,
            name: &profile.name,
            avatar_url: profile.avatar_url.as_deref(),
            locked: profile.locked,
            primary_label: primary.then(|| t("profiles.primary_badge").to_uppercase()),
            origin,
            cell,
            radius,
            manage,
            reveal,
            tips: (&tips.0, &tips.1),
        };
        let chosen = components::profile_tile(
            ui,
            assets,
            &spec,
            |ui, response, rect| {
                if !blocked {
                    response
                        .clone()
                        .focus_circle(ui, node(index), rect)
                        .focus_neighbors(ui, node(index), neighbors(index));
                }
            },
            metrics,
        );
        if let Some(action) = chosen {
            let purpose = match action {
                components::TileAction::Delete => PinPurpose::Delete,
                components::TileAction::Edit => PinPurpose::Edit,
            };
            model.begin(profile, purpose);
        }
    }
    if !blocked {
        gear.focus_node(ui, NODE_PROFILES_GEAR).focus_neighbors(
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
        let button = kit::center(ui, |ui| {
            kit::pill_wide(ui, &label, metrics.profile_select_width, manage)
        });
        if !blocked {
            button.focus_node(ui, NODE_PROFILES_MANAGE).focus_neighbors(
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
