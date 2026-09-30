use super::*;

pub(crate) fn screen_margin(metrics: UiMetrics) -> f32 {
    metrics.gutter
}

pub(crate) const LIBRARY_CARD_LIMIT: usize = 240;

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
            let search_width = (width * 0.36).clamp(220.0, 420.0);
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
    let count = if library.downloads_open {
        0
    } else {
        library.cards(tab).len().min(LIBRARY_CARD_LIMIT)
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
        }];
    }
    let header = metrics.screen_section_title_size + metrics.control_gap * 2.0;
    let mut top = 0.0;
    discover
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
            };
            top += height + metrics.section_gap;
            block
        })
        .collect()
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

pub fn settings_scroll_max(viewport: Viewport, settings: &SettingsModel) -> f32 {
    let metrics = UiMetrics::for_viewport(viewport);
    if viewport.is_compact() {
        let content_top = crate::settings::compact_settings_content_top(metrics, settings);
        let content_height = if settings.section_open || !settings.search.trim().is_empty() {
            let section_index = settings.active_section.min(SETTINGS_SECTIONS.len() - 1);
            settings_card_height(
                viewport,
                metrics,
                &SETTINGS_SECTIONS[section_index],
                settings,
            )
        } else {
            crate::settings::compact_settings_list_height(viewport, metrics)
        };
        return (content_top + content_height + metrics.page_padding
            - (viewport.height - mobile_scroll_reserve(viewport)))
        .max(0.0);
    }
    let desktop = !viewport.is_compact() && !viewport.is_tv();
    let top = if viewport.is_compact() {
        metrics.detail_header_top_mobile
    } else if desktop {
        metrics.settings_screen_padding_desktop
    } else {
        metrics.content_header_top
    };
    let section_index = settings.active_section.min(SETTINGS_SECTIONS.len() - 1);
    let card_height = settings_card_height(
        viewport,
        metrics,
        &SETTINGS_SECTIONS[section_index],
        settings,
    );
    let title_height = if desktop {
        0.0
    } else {
        metrics.screen_title_size
    };
    let header_bottom = if desktop {
        top
    } else {
        top + title_height
            + metrics.control_gap
            + metrics.screen_body_size
            + metrics.section_gap
            + metrics.screen_control_height
    };
    let content_top = if viewport.is_compact() {
        header_bottom
            + metrics.section_gap
            + metrics.screen_control_height * 2.0
            + metrics.control_gap
            + metrics.section_gap
    } else {
        top
    };
    (content_top + card_height + metrics.page_padding
        - (viewport.height - mobile_scroll_reserve(viewport)))
    .max(0.0)
}

pub(crate) fn resolve_screen_scroll(
    context: &egui::Context,
    viewport: Viewport,
    id: Id,
    max_offset: f32,
) -> f32 {
    if viewport.form_factor == UiFormFactor::Mobile {
        return viewport.scroll_y.clamp(0.0, max_offset);
    }
    // A dropdown menu owns wheel input while open. Without this guard both
    // its ScrollArea and the screen's custom document scroll consume the same
    // wheel delta, moving posters behind the popup while the options scroll.
    if egui::Popup::is_any_open(context) {
        return context.data_mut(|data| {
            data.get_temp::<f32>(id)
                .unwrap_or(viewport.scroll_y)
                .clamp(0.0, max_offset)
        });
    }
    let wheel_delta = context.input(|input| {
        // Mouse wheels report line-sized deltas while the Web renderer moves
        // by CSS pixels. Scale the native event once at the shared boundary.
        input.smooth_scroll_delta.y * 1.65
    });
    let offset = context.data_mut(|data| {
        let previous = data.get_temp::<f32>(id).unwrap_or(viewport.scroll_y);
        let offset = (previous - wheel_delta).clamp(0.0, max_offset);
        data.insert_temp(id, offset);
        offset
    });
    let offset = apply_desktop_drag_scroll(
        context,
        id.with("desktop-drag"),
        Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height)),
        DesktopScrollAxis::Vertical,
        offset,
        max_offset,
    );
    context.data_mut(|data| data.insert_temp(id, offset));
    offset
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DesktopDragState {
    active: bool,
    axis: DesktopScrollAxis,
    last_pos: Pos2,
    velocity: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DesktopScrollAxis {
    #[default]
    Undecided,
    Vertical,
    Horizontal,
}

/// Applies the same direct-manipulation gesture users expect from a touch
/// surface to desktop-rendered content. The gesture deliberately lives in
/// the shared UI crate so every host can use the same scroll physics.
pub(crate) fn apply_desktop_drag_scroll(
    context: &egui::Context,
    id: Id,
    hit_rect: Rect,
    axis: DesktopScrollAxis,
    current: f32,
    max_offset: f32,
) -> f32 {
    if max_offset <= 0.0 {
        return 0.0;
    }

    let (pressed, released, down, interact_pos) = context.input(|input| {
        (
            input.pointer.primary_pressed(),
            input.pointer.primary_released(),
            input.pointer.primary_down(),
            input.pointer.interact_pos(),
        )
    });

    let mut state = context
        .data_mut(|data| data.get_temp::<DesktopDragState>(id))
        .unwrap_or_default();
    let mut offset = current.clamp(0.0, max_offset);

    if pressed {
        if let Some(pos) = interact_pos.filter(|pos| hit_rect.contains(*pos)) {
            state.active = true;
            state.axis = DesktopScrollAxis::Undecided;
            state.last_pos = pos;
            state.velocity = 0.0;
        }
    }

    if state.active && down {
        if let Some(pos) = interact_pos {
            let movement = pos - state.last_pos;
            state.last_pos = pos;
            if movement.length_sq() > 0.01 {
                if state.axis == DesktopScrollAxis::Undecided {
                    state.axis = if movement.y.abs() >= movement.x.abs() {
                        DesktopScrollAxis::Vertical
                    } else {
                        DesktopScrollAxis::Horizontal
                    };
                }
                if state.axis == axis {
                    let axis_delta = if axis == DesktopScrollAxis::Vertical {
                        movement.y
                    } else {
                        movement.x
                    };
                    offset = (offset - axis_delta).clamp(0.0, max_offset);
                    state.velocity = -axis_delta;
                    if (offset <= 0.0 && state.velocity < 0.0)
                        || (offset >= max_offset && state.velocity > 0.0)
                    {
                        state.velocity = 0.0;
                    }
                    context.request_repaint_after(Duration::from_millis(16));
                }
            }
        }
    } else if state.active && released {
        state.active = false;
        if state.velocity.abs() > 0.25 {
            context.request_repaint_after(Duration::from_millis(16));
        }
    } else if !state.active && state.axis == axis && state.velocity.abs() > 0.25 {
        offset = (offset + state.velocity).clamp(0.0, max_offset);
        if offset <= 0.0 || offset >= max_offset {
            state.velocity = 0.0;
        } else {
            state.velocity *= 0.90;
            context.request_repaint_after(Duration::from_millis(16));
        }
    } else if !down && !pressed {
        state.velocity = 0.0;
    }

    context.data_mut(|data| data.insert_temp(id, state));
    offset
}

pub(crate) fn resolve_desktop_horizontal_scroll(
    context: &egui::Context,
    id: Id,
    row_clip: Rect,
    current: f32,
    max_offset: f32,
) -> f32 {
    if max_offset <= 0.0 {
        return 0.0;
    }
    let wheel_delta = context.input(|input| {
        if input
            .pointer
            .hover_pos()
            .is_some_and(|pos| row_clip.contains(pos))
        {
            input.smooth_scroll_delta.x * 1.65
        } else {
            0.0
        }
    });
    let offset = context.data_mut(|data| {
        let previous = data.get_temp::<f32>(id).unwrap_or(current);
        let offset = (previous - wheel_delta).clamp(0.0, max_offset);
        data.insert_temp(id, offset);
        offset
    });
    let offset = apply_desktop_drag_scroll(
        context,
        id.with("desktop-drag"),
        row_clip,
        DesktopScrollAxis::Horizontal,
        offset,
        max_offset,
    );
    context.data_mut(|data| data.insert_temp(id, offset));
    offset
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
