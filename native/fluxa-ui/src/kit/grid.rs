use super::*;
use egui::Sense;

pub(crate) struct Grid {
    count: usize,
    cell: Cell,
    gap: Option<Space>,
    gap_px: Option<f32>,
    center_last: bool,
    pad_last: bool,
    start: bool,
    row_gap_px: Option<f32>,
    per_row: Option<usize>,
    height: Option<f32>,
    cell_height: Option<f32>,
    every_cell: bool,
}

enum Cell {
    Columns { count: usize, aspect: f32 },
    Fixed(Vec2),
}

pub(crate) struct GridLayout {
    pub columns: usize,
    pub cells: Vec<Rect>,
}

#[derive(Clone, Copy)]
pub(crate) struct Lattice {
    pub origin: Pos2,
    pub cell: Vec2,
    pub gap: f32,
    pub row_gap: f32,
    pub columns: usize,
}

impl Lattice {
    pub fn rect(&self, index: usize) -> Rect {
        let row = index / self.columns;
        let column = index % self.columns;
        Rect::from_min_size(
            self.origin
                + Vec2::new(
                    column as f32 * (self.cell.x + self.gap),
                    row as f32 * (self.cell.y + self.row_gap),
                ),
            self.cell,
        )
    }

    pub fn rows(&self, count: usize) -> usize {
        count.div_ceil(self.columns)
    }

    pub fn visible_rows(&self, count: usize, top: f32, bottom: f32) -> std::ops::Range<usize> {
        let stride = self.cell.y + self.row_gap;
        let first = ((top - self.origin.y) / stride).floor().max(0.0) as usize;
        let end = ((bottom - self.origin.y) / stride).ceil().max(0.0) as usize;
        let rows = self.rows(count);
        first.min(rows)..end.min(rows)
    }

    pub fn width(&self, count: usize) -> f32 {
        let columns = count.min(self.columns);
        (columns as f32 * (self.cell.x + self.gap) - self.gap).max(0.0)
    }
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
        start: false,
        row_gap_px: None,
        per_row: None,
        height: None,
        cell_height: None,
        every_cell: false,
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

    pub fn cell_height(mut self, height: f32) -> Self {
        self.cell_height = Some(height);
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

    pub fn start(mut self) -> Self {
        self.start = true;
        self
    }

    pub fn row_gap_px(mut self, gap: f32) -> Self {
        self.row_gap_px = Some(gap);
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

    pub fn every_cell(mut self) -> Self {
        self.every_cell = true;
        self
    }

    pub fn show(self, ui: &mut Ui, mut add: impl FnMut(&mut Ui, usize)) -> GridLayout {
        let tokens = ui.tokens();
        let gap = self
            .gap_px
            .or_else(|| self.gap.map(|space| space.px(&tokens)))
            .unwrap_or(0.0);
        let row_gap = self.row_gap_px.unwrap_or(gap);
        let width = ui.available_width();
        let (columns, cell) = match self.cell {
            Cell::Columns { count, aspect } => {
                let columns = count.min(self.count.max(1));
                let side = ((width - gap * (columns - 1) as f32) / columns as f32).max(1.0);
                (
                    columns,
                    Vec2::new(side, self.cell_height.unwrap_or(side * aspect)),
                )
            }
            Cell::Fixed(size) => {
                let fit = ((width + gap) / (size.x + gap)).floor() as usize;
                let columns = self.per_row.unwrap_or(fit).clamp(1, self.count.max(1));
                (columns, size)
            }
        };
        let rows = self.count.div_ceil(columns);
        let natural = rows as f32 * cell.y + rows.saturating_sub(1) as f32 * row_gap;
        let height = self.height.unwrap_or(natural);
        let (area, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let clip = ui.clip_rect();
        let reach = clip.height();
        let built = if self.every_cell {
            0..rows
        } else {
            Lattice {
                origin: area.min,
                cell,
                gap,
                row_gap,
                columns,
            }
            .visible_rows(self.count, clip.top() - reach, clip.bottom() + reach)
        };
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
            let lead = if !self.start && (self.center_last || self.is_fixed()) {
                (width - row_width) * 0.5
            } else {
                0.0
            };
            let origin = area.min
                + Vec2::new(
                    lead + column as f32 * (cell.x + gap),
                    row as f32 * (cell.y + row_gap),
                );
            let rect = Rect::from_min_size(origin, cell);
            if built.contains(&row) {
                let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
                add(&mut child, index);
            }
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
        lay_counting(count, build).0
    }

    fn lay_counting(count: usize, build: impl FnOnce(Grid) -> Grid) -> (GridLayout, Vec<usize>) {
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
        let mut built = Vec::new();
        let mut build = Some(build);
        context
            .run_ui(input, |root| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(root, |ui| {
                        if let Some(build) = build.take() {
                            layout =
                                Some(build(grid(count)).show(ui, |_, index| built.push(index)));
                        }
                    });
            })
            .drop_without_applying_deltas();
        (layout.expect("grid laid out"), built)
    }

    #[test]
    fn off_screen_rows_are_positioned_but_not_built() {
        let (layout, built) =
            lay_counting(3000, |grid| grid.cell(Vec2::new(100.0, 50.0)).gap_px(10.0));
        assert_eq!(layout.cells.len(), 3000);
        assert!(built.contains(&0));
        assert!(!built.contains(&2999));
        assert!(built.len() < 300);
    }

    #[test]
    fn every_cell_builds_all_rows() {
        let (layout, built) = lay_counting(3000, |grid| {
            grid.cell(Vec2::new(100.0, 50.0)).gap_px(10.0).every_cell()
        });
        assert_eq!(built.len(), layout.cells.len());
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
    fn start_aligned_rows_use_separate_row_gap() {
        let layout = lay(8, |grid| {
            grid.cell(Vec2::new(60.0, 80.0))
                .gap_px(14.0)
                .row_gap_px(8.0)
                .start()
        });
        assert_eq!(layout.cells[0].left(), 0.0);
        assert_eq!(layout.cells[1].left(), 74.0);
        assert_eq!(layout.cells[layout.columns].top(), 88.0);
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
