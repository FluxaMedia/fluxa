use super::*;

pub(crate) fn screen_margin(metrics: UiMetrics) -> f32 {
    metrics.gutter
}

pub(crate) fn discover_grid_geometry(
    available_width: f32,
    preferred_card_width: f32,
    gap: f32,
) -> (usize, f32) {
    let available_width = available_width.max(1.0);
    let gap = gap.max(0.0);
    let preferred_card_width = preferred_card_width.min(available_width).max(1.0);
    let columns = ((available_width + gap) / (preferred_card_width + gap))
        .floor()
        .max(1.0) as usize;
    let card_width =
        (available_width - gap * columns.saturating_sub(1) as f32 - 1.0) / columns as f32;
    (columns, card_width.max(1.0))
}

pub(crate) fn mobile_scroll_reserve(viewport: Viewport) -> f32 {
    if viewport.is_compact() {
        mobile_nav_reserve(viewport)
    } else {
        0.0
    }
}

pub(crate) struct PageLayout {
    pub margin: f32,
    pub width: f32,
    pub top: f32,
    pub title_height: f32,
    pub search: Rect,
    pub filters_top: f32,
    pub second_row_top: Option<f32>,
    pub content_top: f32,
    pub sections_top: f32,
}

impl PageLayout {
    pub fn new(viewport: Viewport, metrics: UiMetrics, second_row: bool) -> Self {
        let compact = viewport.is_compact();
        let margin = screen_margin(metrics);
        let width = (viewport.width - margin * 2.0).max(1.0);
        let top = metrics.content_header_top;
        let title_size = metrics.screen_title_size;
        let control = metrics.screen_control_height;
        let search_height = control + 4.0;
        let (title_height, search, filters_top) = if compact {
            let title_height = title_size * 1.25;
            let search_top = top + title_height + metrics.control_gap;
            (
                title_height,
                Rect::from_min_size(
                    Pos2::new(margin, search_top),
                    Vec2::new(width, search_height),
                ),
                search_top + search_height + metrics.control_gap,
            )
        } else {
            let title_height = (title_size * 1.25).max(search_height);
            let search_width = metrics.search_width.of(width);
            (
                title_height,
                Rect::from_min_size(
                    Pos2::new(
                        margin + width - search_width,
                        top + (title_height - search_height) * 0.5,
                    ),
                    Vec2::new(search_width, search_height),
                ),
                top + title_height + metrics.section_gap,
            )
        };
        let second_row_top = second_row.then_some(filters_top + control + metrics.control_gap);
        let content_top =
            second_row_top.unwrap_or(filters_top) + control + metrics.section_gap * 1.5;
        Self {
            margin,
            width,
            top,
            title_height,
            search,
            filters_top,
            second_row_top,
            content_top,
            sections_top: 0.0,
        }
    }

    pub fn with_sections(mut self, viewport: Viewport, metrics: UiMetrics) -> Self {
        let shift = metrics.screen_control_height + metrics.control_gap;
        if viewport.is_compact() {
            self.sections_top = self.search.min.y;
            self.search = self.search.translate(Vec2::new(0.0, shift));
        } else {
            self.sections_top = self.filters_top;
        }
        self.filters_top += shift;
        self.second_row_top = self.second_row_top.map(|top| top + shift);
        self.content_top += shift;
        self
    }
}

pub(crate) struct PosterGrid {
    pub columns: usize,
    pub card_width: f32,
    pub poster_height: f32,
    pub card_height: f32,
    pub gap: f32,
    pub row_gap: f32,
}

impl PosterGrid {
    pub fn new(width: f32, metrics: UiMetrics) -> Self {
        let gap = if metrics.grid_max_gap > 0.0 {
            metrics.horizontal_spacing.min(metrics.grid_max_gap)
        } else {
            metrics.horizontal_spacing
        };
        let across = metrics.grid_min_columns.max(1.0);
        let preferred = metrics
            .poster_card_width
            .min((width - gap * (across - 1.0)) / across);
        let (columns, card_width) = discover_grid_geometry(width, preferred, gap);
        let poster_height =
            metrics.poster_card_height * (card_width / metrics.poster_card_width.max(1.0));
        let card_height = poster_height
            + metrics.control_gap * 2.0
            + metrics.screen_card_title_size
            + metrics.screen_card_subtitle_size;
        Self {
            columns,
            card_width,
            poster_height,
            card_height,
            gap,
            row_gap: metrics.vertical_spacing * 1.4,
        }
    }

    pub fn height(&self, count: usize) -> f32 {
        let rows = count.div_ceil(self.columns);
        rows as f32 * self.card_height + rows.saturating_sub(1) as f32 * self.row_gap
    }

    pub fn cell(&self, origin: Pos2, index: usize) -> Rect {
        Rect::from_min_size(
            origin
                + Vec2::new(
                    (index % self.columns) as f32 * (self.card_width + self.gap),
                    (index / self.columns) as f32 * (self.card_height + self.row_gap),
                ),
            Vec2::new(self.card_width, self.card_height),
        )
    }
}

pub(crate) fn library_row_height(viewport: Viewport, metrics: UiMetrics) -> f32 {
    if viewport.is_compact() {
        metrics.screen_control_height * 2.4
    } else {
        metrics.screen_control_height * 2.8
    }
}

fn library_list_height(viewport: Viewport, metrics: UiMetrics, count: usize) -> f32 {
    let row = library_row_height(viewport, metrics);
    count as f32 * row + count.saturating_sub(1) as f32 * metrics.control_gap
}

pub fn library_scroll_max(viewport: Viewport, library: &LibraryModel, tab: LibraryTab) -> f32 {
    let metrics = UiMetrics::for_viewport(viewport);
    let page = PageLayout::new(viewport, metrics, false).with_sections(viewport, metrics);
    if library.downloads_open {
        let content = match library.open_download_group() {
            Some((_, group)) => {
                metrics.icon_button_size
                    + metrics.section_gap
                    + components::download_group_height(
                        group,
                        page.width,
                        metrics,
                        !library.list_view,
                    )
            }
            None => {
                components::download_status_height(metrics)
                    + components::download_sections_height(
                        &library.downloads,
                        components::download_sections_width(page.width, metrics),
                        metrics,
                    )
            }
        };
        return (page.filters_top + content - (viewport.height - mobile_scroll_reserve(viewport)))
            .max(0.0);
    }
    let count = if library.downloads_open {
        0
    } else {
        library.cards(tab).len()
    };
    let content = if library.list_view {
        library_list_height(viewport, metrics, count)
    } else {
        PosterGrid::new(page.width, metrics).height(count)
    };
    (page.content_top + content + metrics.section_gap
        - (viewport.height - mobile_scroll_reserve(viewport)))
    .max(0.0)
}

pub fn discover_scroll_max(viewport: Viewport, discover: &DiscoverModel) -> f32 {
    if discover.sections.is_empty() {
        return discover_scroll_max_for_result_count(viewport, discover.results.len());
    }
    let metrics = UiMetrics::for_viewport(viewport);
    let page = PageLayout::new(viewport, metrics, false);
    let grid = PosterGrid::new(page.width, metrics);
    let height = discover_blocks(&grid, metrics, discover)
        .last()
        .map_or(0.0, |block| block.top + block.height);
    (page.content_top + height + metrics.section_gap
        - (viewport.height - mobile_scroll_reserve(viewport)))
    .max(0.0)
}

pub(crate) struct DiscoverBlock<'a> {
    pub title: Option<&'a str>,
    pub start: usize,
    pub len: usize,
    pub top: f32,
    pub header: f32,
    pub height: f32,
    pub pending: bool,
}

pub(crate) fn discover_blocks<'a>(
    grid: &PosterGrid,
    metrics: UiMetrics,
    discover: &'a DiscoverModel,
) -> Vec<DiscoverBlock<'a>> {
    if discover.sections.is_empty() {
        return vec![DiscoverBlock {
            title: None,
            start: 0,
            len: discover.results.len(),
            top: 0.0,
            header: 0.0,
            height: grid.height(discover.results.len()),
            pending: false,
        }];
    }
    let header = metrics.screen_section_title_size + metrics.control_gap * 2.0;
    let mut top = 0.0;
    let mut blocks: Vec<_> = discover
        .sections
        .iter()
        .map(|section| {
            let height = header + grid.card_height;
            let block = DiscoverBlock {
                title: Some(section.title.as_str()),
                start: section.start,
                len: section.len,
                top,
                header,
                height,
                pending: false,
            };
            top += height + metrics.section_gap;
            block
        })
        .collect();
    if discover.is_loading {
        blocks.push(DiscoverBlock {
            title: None,
            start: discover.results.len(),
            len: 0,
            top,
            header: 0.0,
            height: header + grid.card_height,
            pending: true,
        });
    }
    blocks
}

pub fn discover_scroll_max_for_result_count(viewport: Viewport, result_count: usize) -> f32 {
    let metrics = UiMetrics::for_viewport(viewport);
    let page = PageLayout::new(viewport, metrics, false);
    let grid = PosterGrid::new(page.width, metrics);
    (page.content_top + grid.height(result_count) + metrics.section_gap
        - (viewport.height - mobile_scroll_reserve(viewport)))
    .max(0.0)
}

pub fn calendar_scroll_max(viewport: Viewport, calendar: &CalendarModel) -> f32 {
    if calendar.year <= 0 || !(1..=12).contains(&calendar.month) {
        return 0.0;
    }
    let metrics = UiMetrics::for_viewport(viewport);
    let grid = calendar::CalendarGrid::new(viewport, metrics, calendar);
    let panel_height = calendar
        .selected_day
        .filter(|_| viewport.is_compact())
        .map(|day| {
            150.0
                + calendar.entries_for_day(day).count().min(10) as f32
                    * (metrics.horizontal_card_height * 0.34 + metrics.control_gap)
        })
        .unwrap_or(0.0);
    (grid.bottom() + panel_height - (viewport.height - mobile_scroll_reserve(viewport))).max(0.0)
}

const SCREEN_SCROLL_PENDING: &str = "fluxa-screen-scroll-pending";
const SCROLL_DIRECT: &str = "fluxa-scroll-direct";

pub fn set_scroll_direct(context: &crate::gfx::Context, direct: bool) {
    context.data_mut(|data| data.insert_temp(Id::new(SCROLL_DIRECT), direct));
}

pub fn scroll_blocked(context: &crate::gfx::Context) -> bool {
    crate::gfx::Popup::is_any_open(context) || crate::components::screen_covered(context)
}

pub fn pointer_over_mini_player(context: &crate::gfx::Context, pos: Pos2) -> bool {
    crate::components::over_mini_player(context, pos)
}

pub fn screen_scroll_pending(context: &crate::gfx::Context) -> f32 {
    context
        .data(|data| data.get_temp::<f32>(Id::new(SCREEN_SCROLL_PENDING)))
        .unwrap_or(0.0)
}

fn row_pending_id(row: usize) -> Id {
    Id::new(("fluxa-home-row-scroll", row)).with("pending")
}

pub fn row_scroll_pending(context: &crate::gfx::Context, row: usize) -> f32 {
    context
        .data(|data| data.get_temp::<f32>(row_pending_id(row)))
        .unwrap_or(0.0)
}

pub(crate) fn scroll_display(context: &crate::gfx::Context, id: Id, target: f32) -> f32 {
    let direct = context
        .data(|data| data.get_temp::<bool>(Id::new(SCROLL_DIRECT)))
        .unwrap_or(false);
    if direct {
        return target;
    }
    ease_scroll(context, id, target)
}

pub(crate) fn resolve_row_scroll(
    context: &crate::gfx::Context,
    row: usize,
    target: f32,
    eased: bool,
) -> f32 {
    if !eased {
        return target;
    }
    let shown = scroll_display(context, Id::new(("fluxa-home-row-scroll", row)), target);
    context.data_mut(|data| data.insert_temp(row_pending_id(row), target - shown));
    shown
}

pub(crate) fn ease_scroll(context: &crate::gfx::Context, id: Id, target: f32) -> f32 {
    let frame = context.cumulative_pass_nr();
    let (shown, seen) = context
        .data(|data| data.get_temp::<(f32, u64)>(id))
        .unwrap_or((target, frame));
    let shown = if frame.saturating_sub(seen) > 3 {
        target
    } else {
        let dt = context.input(|input| input.stable_dt()).clamp(0.001, 0.05);
        let step = (target - shown) * (1.0 - (-dt * 14.0).exp());
        if (target - shown - step).abs() < 0.5 {
            target
        } else {
            shown + step
        }
    };
    context.data_mut(|data| data.insert_temp(id, (shown, frame)));
    if shown != target {
        context.request_repaint();
    }
    shown
}

pub(crate) fn resolve_screen_scroll(
    context: &crate::gfx::Context,
    viewport: Viewport,
    id: Id,
    max_offset: f32,
) -> f32 {
    crate::report_scroll_region(
        context,
        crate::ScrollRegion {
            id: crate::ScrollId::Screen,
            axis: crate::ScrollAxis::Vertical,
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height)),
            max: max_offset,
        },
    );
    let target = viewport.scroll_y.clamp(0.0, max_offset);
    if viewport.form_factor == UiFormFactor::Mobile {
        return target;
    }
    let shown = scroll_display(context, id, target);
    context.data_mut(|data| data.insert_temp(Id::new(SCREEN_SCROLL_PENDING), target - shown));
    shown
}

pub fn backdrop_target_size(width: f32, pixels_per_point: f32) -> [u32; 2] {
    artwork_target_size(Vec2::new(width, width * 9.0 / 16.0), pixels_per_point)
}

pub(crate) fn artwork_target_size(size: Vec2, pixels_per_point: f32) -> [u32; 2] {
    let scale = if pixels_per_point.is_finite() {
        pixels_per_point.max(0.25)
    } else {
        1.0
    };
    fluxa_artwork::bounded_target([
        (size.x.max(1.0) * scale).ceil() as u32,
        (size.y.max(1.0) * scale).ceil() as u32,
    ])
}
