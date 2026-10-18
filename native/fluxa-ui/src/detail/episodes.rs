use super::*;

const GAP: f32 = 16.0;
const LIST_GAP: f32 = 8.0;
const GRID_MIN: f32 = 260.0;

fn text_height(show_overview: bool, rows: usize) -> f32 {
    if show_overview {
        22.0 + rows as f32 * 17.0 + 4.0
    } else {
        30.0
    }
}

fn list_thumb_width(compact: bool) -> f32 {
    if compact { 128.0 } else { 200.0 }
}

fn list_row_height(compact: bool) -> f32 {
    (list_thumb_width(compact) * 9.0 / 16.0).max(84.0) + 20.0
}

fn grid_columns(avail: f32, compact: bool) -> usize {
    if compact {
        3
    } else {
        (((avail + GAP) / (GRID_MIN + GAP)) as usize).clamp(2, 6)
    }
}

fn grid_cell(avail: f32, compact: bool, show_overview: bool) -> Vec2 {
    let columns = grid_columns(avail, compact);
    let gap = if compact { 8.0 } else { GAP };
    let width = (avail - gap * (columns as f32 - 1.0)) / columns as f32;
    let text = text_height(show_overview && !compact, 2);
    Vec2::new(width, width * 9.0 / 16.0 + 8.0 + text)
}

pub(super) fn available_width(viewport: Viewport, margin: f32) -> f32 {
    viewport.width - margin * 2.0
}

pub(super) fn episodes_height(viewport: Viewport, detail: &DetailModel, margin: f32) -> f32 {
    let compact = viewport.is_compact();
    let avail = available_width(viewport, margin);
    let season = detail.current_season();
    let count = detail
        .episodes
        .iter()
        .filter(|episode| episode.season == season)
        .count();
    let overview = detail.show_episode_descriptions;
    match detail.episode_layout_for(compact) {
        EpisodeLayout::Cards => EPISODE_WIDTH * 9.0 / 16.0 + 12.0 + text_height(overview, 2),
        EpisodeLayout::List => count as f32 * (list_row_height(compact) + LIST_GAP),
        EpisodeLayout::Grid => {
            let cell = grid_cell(avail, compact, overview);
            let gap = if compact { 8.0 } else { GAP };
            let rows = count.div_ceil(grid_columns(avail, compact));
            rows as f32 * (cell.y + gap)
        }
    }
}

pub(super) struct EpisodeContext<'a> {
    pub viewport: Viewport,
    pub detail: &'a DetailModel,
    pub episodes: &'a [(usize, &'a DetailEpisode)],
    pub season: i64,
    pub metrics: UiMetrics,
    pub margin: f32,
}

pub(super) fn draw_episodes(
    ui: &mut egui::Ui,
    cx: &EpisodeContext,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let compact = cx.viewport.is_compact();
    let avail = available_width(cx.viewport, cx.margin);
    let overview = cx.detail.show_episode_descriptions;
    let origin = ui.cursor().min;
    let (_, _) = ui.allocate_exact_size(
        Vec2::new(avail, episodes_height(cx.viewport, cx.detail, cx.margin)),
        Sense::hover(),
    );
    match cx.detail.episode_layout_for(compact) {
        EpisodeLayout::List => {
            let row = list_row_height(compact);
            let thumb_width = list_thumb_width(compact);
            for (slot, (index, episode)) in cx.episodes.iter().enumerate() {
                let rect = Rect::from_min_size(
                    Pos2::new(origin.x, origin.y + slot as f32 * (row + LIST_GAP)),
                    Vec2::new(avail, row),
                );
                if !ui.is_rect_visible(rect) {
                    continue;
                }
                let response = episode_hit(ui, cx, layout, *index, rect);
                let thumb = Rect::from_min_size(
                    rect.min + Vec2::new(10.0, 10.0),
                    Vec2::new(thumb_width, thumb_width * 9.0 / 16.0),
                );
                if response.hovered() {
                    ui.painter()
                        .rect_filled(rect, radius::control(), ink(tone::FAINT));
                }
                let view = cx.detail.episode_view(episode);
                components::episode_artwork(
                    ui.painter(),
                    assets,
                    thumb,
                    episode,
                    view.hide_still,
                    response.hovered(),
                    false,
                    cx.metrics,
                );
                let x = thumb.right() + 14.0;
                components::episode_text(
                    ui.painter(),
                    Pos2::new(x, rect.top() + 12.0),
                    rect.right() - x - 10.0,
                    episode.number,
                    view.title,
                    view.overview.map(|text| (text, 3)),
                );
            }
        }
        EpisodeLayout::Grid => {
            let cell = grid_cell(avail, compact, overview);
            let gap = if compact { 8.0 } else { GAP };
            let columns = grid_columns(avail, compact);
            for (slot, (index, episode)) in cx.episodes.iter().enumerate() {
                let rect = Rect::from_min_size(
                    Pos2::new(
                        origin.x + (slot % columns) as f32 * (cell.x + gap),
                        origin.y + (slot / columns) as f32 * (cell.y + gap),
                    ),
                    cell,
                );
                if !ui.is_rect_visible(rect) {
                    continue;
                }
                let response = episode_hit(ui, cx, layout, *index, rect);
                let thumb = Rect::from_min_size(rect.min, Vec2::new(cell.x, cell.x * 9.0 / 16.0));
                let view = cx.detail.episode_view(episode);
                components::episode_artwork(
                    ui.painter(),
                    assets,
                    thumb,
                    episode,
                    view.hide_still,
                    response.hovered(),
                    false,
                    cx.metrics,
                );
                components::episode_text(
                    ui.painter(),
                    Pos2::new(rect.left(), thumb.bottom() + 8.0),
                    cell.x,
                    episode.number,
                    view.title,
                    view.overview.filter(|_| !compact).map(|text| (text, 2)),
                );
            }
        }
        EpisodeLayout::Cards => {}
    }
}

pub(super) fn prefetch(
    detail: &DetailModel,
    episodes: &[(usize, &DetailEpisode)],
    viewport: Viewport,
    margin: f32,
    pixels_per_point: f32,
    assets: &mut impl HomeAssets,
) {
    let compact = viewport.is_compact();
    let avail = available_width(viewport, margin);
    let width = match detail.episode_layout_for(compact) {
        EpisodeLayout::List => list_thumb_width(compact),
        EpisodeLayout::Grid => grid_cell(avail, compact, detail.show_episode_descriptions).x,
        EpisodeLayout::Cards => EPISODE_WIDTH,
    };
    let target = artwork_target_size(Vec2::new(width, width * 9.0 / 16.0), pixels_per_point);
    for (_, episode) in episodes {
        if detail.episode_view(episode).hide_still {
            continue;
        }
        assets.prefetch_for(
            episode.thumbnail.as_deref(),
            target,
            ArtworkPriority::Prefetch,
        );
    }
}

fn episode_hit(
    ui: &mut egui::Ui,
    cx: &EpisodeContext,
    layout: &mut HomeLayout,
    index: usize,
    rect: Rect,
) -> egui::Response {
    let response = ui.interact(
        rect,
        Id::new(("fluxa-detail-episode-hit", cx.season, index)),
        Sense::click(),
    );
    let node = NODE_DETAIL_EPISODE_BASE + index as u64;
    layout.focusable.push((node, rect));
    if response.clicked() {
        layout.activated = Some(node);
    }
    response
}
