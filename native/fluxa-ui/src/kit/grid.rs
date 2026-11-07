use super::*;
use egui::Sense;

pub(crate) struct Grid {
    count: usize,
    cell: Cell,
    gap: Option<Space>,
    gap_px: Option<f32>,
    center_last: bool,
    pad_last: bool,
    per_row: Option<usize>,
    height: Option<f32>,
}

enum Cell {
    Columns { count: usize, aspect: f32 },
    Fixed(Vec2),
}

pub(crate) struct GridLayout {
    pub columns: usize,
    pub cells: Vec<Rect>,
}

pub(crate) fn grid(count: usize) -> Grid {
    Grid {
        count,
        cell: Cell::Columns {
            count: 1,
            aspect: 1.0,
        },
        gap: None,
        gap_px: None,
        center_last: false,
        pad_last: false,
        per_row: None,
        height: None,
    }
}

impl Grid {
    pub fn columns(mut self, count: usize, aspect: f32) -> Self {
        self.cell = Cell::Columns {
            count: count.max(1),
            aspect,
        };
        self
    }

    pub fn cell(mut self, size: Vec2) -> Self {
        self.cell = Cell::Fixed(size);
        self
    }

    pub fn gap(mut self, space: Space) -> Self {
        self.gap = Some(space);
        self
    }

    pub fn gap_px(mut self, gap: f32) -> Self {
        self.gap_px = Some(gap);
        self
    }

    pub fn per_row(mut self, count: usize) -> Self {
        self.per_row = Some(count.max(1));
        self
    }

    pub fn pad_last(mut self) -> Self {
        self.pad_last = true;
        self
    }

    pub fn reserve_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn center_last(mut self) -> Self {
        self.center_last = true;
        self
    }

    pub fn show(self, ui: &mut Ui, mut add: impl FnMut(&mut Ui, usize)) -> GridLayout {
        let tokens = ui.tokens();
        let gap = self
            .gap_px
            .or_else(|| self.gap.map(|space| space.px(&tokens)))
            .unwrap_or(0.0);
        let width = ui.available_width();
        let (columns, cell) = match self.cell {
            Cell::Columns { count, aspect } => {
                let columns = count.min(self.count.max(1));
                let side = ((width - gap * (columns - 1) as f32) / columns as f32).max(1.0);
                (columns, Vec2::new(side, side * aspect))
            }
            Cell::Fixed(size) => {
                let fit = ((width + gap) / (size.x + gap)).floor() as usize;
                let columns = self.per_row.unwrap_or(fit).clamp(1, self.count.max(1));
                (columns, size)
            }
        };
        let rows = self.count.div_ceil(columns);
        let natural = rows as f32 * cell.y + rows.saturating_sub(1) as f32 * gap;
        let height = self.height.unwrap_or(natural);
        let (area, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let mut cells = Vec::with_capacity(self.count);
        for index in 0..self.count {
            let row = index / columns;
            let column = index % columns;
            let in_row = if self.pad_last {
                columns
            } else {
                (self.count - row * columns).min(columns)
            };
            let row_width = in_row as f32 * cell.x + in_row.saturating_sub(1) as f32 * gap;
            let lead = if self.center_last || self.is_fixed() {
                (width - row_width) * 0.5
            } else {
                0.0
            };
            let origin = area.min
                + Vec2::new(
                    lead + column as f32 * (cell.x + gap),
                    row as f32 * (cell.y + gap),
                );
            let rect = Rect::from_min_size(origin, cell);
            let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
            add(&mut child, index);
            cells.push(rect);
        }
        GridLayout { columns, cells }
    }

    fn is_fixed(&self) -> bool {
        matches!(self.cell, Cell::Fixed(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lay(count: usize, build: impl FnOnce(Grid) -> Grid) -> GridLayout {
        let context = egui::Context::default();
        context.set_fonts(crate::fonts::definitions());
        provide(
            &context,
            UiMetrics::for_viewport(Viewport::new(400, 300, UiFormFactor::Desktop)),
        );
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 600.0))),
            ..Default::default()
        };
        let mut layout = None;
        let mut build = Some(build);
        context
            .run_ui(input, |root| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(root, |ui| {
                        if let Some(build) = build.take() {
                            layout = Some(build(grid(count)).show(ui, |_, _| {}));
                        }
                    });
            })
            .drop_without_applying_deltas();
        layout.expect("grid laid out")
    }

    #[test]
    fn fixed_cells_pick_the_column_count_from_the_width() {
        let layout = lay(5, |grid| grid.cell(Vec2::new(100.0, 50.0)).gap_px(10.0));
        assert_eq!(layout.columns, 3);
        assert_eq!(layout.cells[3].top(), layout.cells[0].bottom() + 10.0);
    }

    #[test]
    fn short_last_row_is_centered_for_fixed_cells() {
        let layout = lay(4, |grid| grid.cell(Vec2::new(100.0, 50.0)).gap_px(10.0));
        let first = layout.cells[0].center().x;
        let last = layout.cells[3].center().x;
        assert!((last - 200.0).abs() < 1.0);
        assert!(first < last);
    }

    #[test]
    fn padded_last_row_stays_on_the_grid_columns() {
        let layout = lay(3, |grid| {
            grid.cell(Vec2::new(150.0, 50.0))
                .gap_px(10.0)
                .per_row(2)
                .pad_last()
        });
        assert_eq!(layout.cells[2].left(), layout.cells[0].left());
        assert_eq!(layout.cells[1].left(), layout.cells[0].right() + 10.0);
    }

    #[test]
    fn fixed_columns_split_the_width_evenly() {
        let layout = lay(6, |grid| grid.columns(3, 1.5).gap_px(8.0));
        let cell = layout.cells[0];
        assert!((cell.width() * 3.0 + 16.0 - 400.0).abs() < 1.0);
        assert!((cell.height() - cell.width() * 1.5).abs() < 0.01);
    }
}
