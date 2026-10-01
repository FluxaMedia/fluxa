use super::*;

const GAP: f32 = 16.0;
const LIST_GAP: f32 = 8.0;
const GRID_MIN: f32 = 220.0;
const CHIP: f32 = 44.0;
const CHIP_GAP: f32 = 8.0;
const NUMBERS_THUMB: f32 = 260.0;
const PANEL_PAD: f32 = 14.0;

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

fn chips_per_row(avail: f32) -> usize {
    (((avail + CHIP_GAP) / (CHIP + CHIP_GAP)) as usize).max(1)
}

fn numbers_chips_height(avail: f32, count: usize) -> f32 {
    let rows = count.div_ceil(chips_per_row(avail));
    rows as f32 * (CHIP + CHIP_GAP)
}

fn numbers_panel_height(avail: f32, compact: bool) -> f32 {
    if compact {
        (avail - PANEL_PAD * 2.0) * 9.0 / 16.0 + PANEL_PAD * 2.0 + 12.0 + text_height(true, 4)
    } else {
        NUMBERS_THUMB * 9.0 / 16.0 + PANEL_PAD * 2.0
    }
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
        EpisodeLayout::Numbers => {
            numbers_chips_height(avail, count) + 8.0 + numbers_panel_height(avail, compact)
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
                        .rect_filled(rect, 12.0, Color32::from_white_alpha(10));
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
        EpisodeLayout::Numbers => draw_numbers(ui, cx, assets, layout, origin, avail),
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
        EpisodeLayout::Numbers if compact => avail - PANEL_PAD * 2.0,
        EpisodeLayout::Numbers => NUMBERS_THUMB,
        EpisodeLayout::Cards => return,
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

fn draw_numbers(
    ui: &mut egui::Ui,
    cx: &EpisodeContext,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
    origin: Pos2,
    avail: f32,
) {
    let compact = cx.viewport.is_compact();
    let selection_id = Id::new(("fluxa-detail-number", &cx.detail.id, cx.season));
    let selected = ui
        .ctx()
        .data(|data| data.get_temp::<usize>(selection_id))
        .filter(|slot| *slot < cx.episodes.len())
        .unwrap_or(0);
    let per_row = chips_per_row(avail);
    for (slot, (_, episode)) in cx.episodes.iter().enumerate() {
        let pos = Pos2::new(
            origin.x + (slot % per_row) as f32 * (CHIP + CHIP_GAP),
            origin.y + (slot / per_row) as f32 * (CHIP + CHIP_GAP),
        );
        let rect = Rect::from_min_size(pos, Vec2::splat(CHIP));
        if !ui.is_rect_visible(rect) {
            continue;
        }
        let mut chip = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        let response = components::button_with_text_size(
            &mut chip,
            &episode.number.to_string(),
            CHIP,
            CHIP,
            if slot == selected {
                components::ButtonKind::Selected
            } else {
                components::ButtonKind::Secondary
            },
            15.0,
            cx.metrics,
        );
        if response.clicked() {
            ui.ctx()
                .data_mut(|data| data.insert_temp(selection_id, slot));
        }
    }
    let Some((index, episode)) = cx.episodes.get(selected) else {
        return;
    };
    let top = origin.y + numbers_chips_height(avail, cx.episodes.len()) + 8.0;
    let panel = Rect::from_min_size(
        Pos2::new(origin.x, top),
        Vec2::new(avail, numbers_panel_height(avail, compact)),
    );
    let response = episode_hit(ui, cx, layout, *index, panel);
    ui.painter().rect(
        panel,
        12.0,
        Color32::from_white_alpha(8),
        egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
        egui::StrokeKind::Inside,
    );
    let thumb_width = if compact {
        avail - PANEL_PAD * 2.0
    } else {
        NUMBERS_THUMB
    };
    let thumb = Rect::from_min_size(
        panel.min + Vec2::splat(PANEL_PAD),
        Vec2::new(thumb_width, thumb_width * 9.0 / 16.0),
    );
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
    let (text_origin, text_width) = if compact {
        (
            Pos2::new(thumb.left(), thumb.bottom() + 12.0),
            thumb.width(),
        )
    } else {
        (
            Pos2::new(thumb.right() + 16.0, panel.top() + PANEL_PAD),
            panel.right() - thumb.right() - 16.0 - PANEL_PAD,
        )
    };
    components::episode_text(
        ui.painter(),
        text_origin,
        text_width,
        episode.number,
        view.title,
        view.overview.map(|text| (text, 4)),
    );
}
