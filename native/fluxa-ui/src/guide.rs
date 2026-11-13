use super::*;
use crate::components;
use crate::draw_text;
use serde_json::Value;

const HOUR_MS: i64 = 3_600_000;

#[derive(Clone, Debug, Default)]
pub struct GuideProgramme {
    pub title: String,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Clone, Debug, Default)]
pub struct GuideChannel {
    pub name: String,
    pub logo: Option<String>,
    pub raw: Value,
    pub programmes: Vec<GuideProgramme>,
}

#[derive(Clone, Debug, Default)]
pub struct GuideModel {
    pub channels: Vec<GuideChannel>,
    pub from_ms: i64,
    pub to_ms: i64,
    pub now_ms: i64,
    pub utc_offset_ms: i64,
    pub loading: bool,
    pub failed: bool,
    pub language: String,
}

impl GuideModel {
    pub fn apply(&mut self, guide: &Value) {
        self.failed = guide["failed"].as_bool() == Some(true);
        self.channels = guide["channels"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|channel| GuideChannel {
                name: channel["name"].as_str().unwrap_or_default().to_owned(),
                logo: channel["logo"]
                    .as_str()
                    .or_else(|| channel["poster"].as_str())
                    .map(ToOwned::to_owned),
                raw: channel.clone(),
                programmes: channel["programmes"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|programme| GuideProgramme {
                        title: programme["title"]
                            .as_str()
                            .or_else(|| programme["name"].as_str())
                            .unwrap_or_default()
                            .to_owned(),
                        start_ms: programme["startMs"].as_i64().unwrap_or_default(),
                        end_ms: programme["endMs"].as_i64().unwrap_or_default(),
                    })
                    .collect(),
            })
            .collect();
    }

    pub fn channel_for_node(&self, node: u64) -> Option<&GuideChannel> {
        let index = if (NODE_GUIDE_CELL_BASE..NODE_GUIDE_CELL_BASE + NODE_GUIDE_CELL_SPAN)
            .contains(&node)
        {
            (node - NODE_GUIDE_CELL_BASE) / NODE_GUIDE_ROW_STRIDE
        } else if (NODE_GUIDE_CHANNEL_BASE..NODE_GUIDE_CELL_BASE).contains(&node) {
            node - NODE_GUIDE_CHANNEL_BASE
        } else {
            return None;
        };
        self.channels.get(index as usize)
    }

    pub fn node_label(&self, node: u64) -> Option<String> {
        let channel = self.channel_for_node(node)?;
        if node >= NODE_GUIDE_CELL_BASE {
            let index = ((node - NODE_GUIDE_CELL_BASE) % NODE_GUIDE_ROW_STRIDE) as usize;
            if let Some(programme) = channel.programmes.get(index) {
                return Some(format!("{}, {}", channel.name, programme.title));
            }
        }
        Some(channel.name.clone())
    }

    pub fn last_end_ms(&self) -> Option<i64> {
        self.channels
            .iter()
            .filter_map(|channel| channel.programmes.last())
            .map(|programme| programme.end_ms)
            .max()
    }
}

pub const NODE_GUIDE_ROW_STRIDE: u64 = 512;
pub const NODE_GUIDE_CELL_SPAN: u64 = 400 * NODE_GUIDE_ROW_STRIDE;

fn clock(ms: i64, offset_ms: i64) -> String {
    let minutes = (ms + offset_ms).div_euclid(60_000).rem_euclid(24 * 60);
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

fn sized(min: Pos2, size: Vec2) -> Rect {
    Rect::from_min_max(min, Pos2::new(min.x + size.x, min.y + size.y))
}

fn guide_x(id: Id) -> Id {
    id.with("guide-scroll-x")
}

pub fn draw_guide(
    context: &crate::gfx::Context,
    viewport: Viewport,
    guide: &GuideModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let screen = crate::gfx::rect_at(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let background = context.layer_painter(crate::gfx::LayerId::background());
    paint_ambient(&background, screen, assets);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, crate::sidebar::ROUTE_LIVE, assets);
    let focus_cell = focused.and_then(|node| {
        (NODE_GUIDE_CELL_BASE..NODE_GUIDE_CELL_BASE + NODE_GUIDE_CELL_SPAN)
            .contains(&node)
            .then(|| node - NODE_GUIDE_CELL_BASE)
    });

    kit::page(context, viewport, "guide", |ui| {
        let width = ui.available_width();
        kit::scroll_body(ui, viewport, "guide", |ui| {
            let (bar, _) = ui.allocate_exact_size(
                Vec2::new(width, metrics.screen_title_size * 1.25),
                Sense::hover(),
            );
            draw_text(
                ui.painter(),
                bar.left_center(),
                Align2::LEFT_CENTER,
                localized("nav.live_tv", &guide.language),
                crate::fonts::bold(metrics.screen_title_size),
                text_primary(),
                None,
            );
            kit::lay(ui, vec![kit::layout_items::gap(metrics.control_gap)]);
            if guide.channels.is_empty() {
                let key = if guide.loading {
                    "guide.loading"
                } else if guide.failed {
                    "guide.failed"
                } else {
                    "guide.empty"
                };
                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new(width, metrics.calendar_empty_offset),
                    Sense::hover(),
                );
                draw_text(
                    ui.painter(),
                    rect.center(),
                    Align2::CENTER_CENTER,
                    localized(key, &guide.language),
                    crate::fonts::regular(metrics.nav_label_size + 2.0),
                    metrics.text_secondary,
                    Some(width),
                );
                return;
            }
            draw_grid(ui, width, guide, metrics, assets, focus_cell, &mut layout);
        });
    });
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    layout
}

fn draw_grid(
    ui: &mut crate::gfx::Ui,
    width: f32,
    guide: &GuideModel,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    focus_cell: Option<u64>,
    layout: &mut HomeLayout,
) {
    let channel_width = metrics.guide_channel_width.min(width * 0.4);
    let hour_width = metrics.guide_hour_width;
    let row_height = metrics.guide_row_height;
    let head_height = metrics.screen_card_subtitle_size * 2.4;
    let gap = metrics.calendar_grid_gap;
    let total_height = head_height + guide.channels.len() as f32 * row_height;
    let (area, _) = ui.allocate_exact_size(Vec2::new(width, total_height), Sense::hover());
    let timeline = Rect::from_min_max(
        Pos2::new(area.left() + channel_width + gap, area.top()),
        area.right_bottom(),
    );
    let span = ((guide.to_ms - guide.from_ms) as f32 / HOUR_MS as f32) * hour_width;
    let max_scroll = (span - timeline.width()).max(0.0);
    let now_x = (guide.now_ms - guide.from_ms) as f32 / HOUR_MS as f32 * hour_width;

    let state_id = guide_x(Id::new(("guide", guide.from_ms)));
    let mut scroll_x = ui
        .ctx()
        .data(|data| data.get_temp::<f32>(state_id))
        .unwrap_or_else(|| now_x - hour_width * 0.5);
    let drag = ui.interact(timeline, state_id.with("drag"), Sense::drag());
    scroll_x -= drag.drag_delta().x;
    if ui.rect_contains_pointer(timeline) {
        scroll_x -= ui.input(|input| input.smooth_scroll_delta().x);
    }
    if let Some(cell) = focus_cell {
        let channel = (cell / NODE_GUIDE_ROW_STRIDE) as usize;
        let index = (cell % NODE_GUIDE_ROW_STRIDE) as usize;
        if let Some(programme) = guide
            .channels
            .get(channel)
            .and_then(|channel| channel.programmes.get(index))
        {
            let left = (programme.start_ms - guide.from_ms) as f32 / HOUR_MS as f32 * hour_width;
            let right = (programme.end_ms - guide.from_ms) as f32 / HOUR_MS as f32 * hour_width;
            let last = ui
                .ctx()
                .data(|data| data.get_temp::<u64>(state_id.with("focus")));
            if last != Some(cell) {
                if left < scroll_x {
                    scroll_x = left;
                } else if right > scroll_x + timeline.width() {
                    scroll_x = (right - timeline.width()).max(left.min(scroll_x));
                }
                ui.ctx()
                    .data_mut(|data| data.insert_temp(state_id.with("focus"), cell));
            }
        }
    }
    let scroll_x = scroll_x.clamp(0.0, max_scroll);
    ui.ctx()
        .data_mut(|data| data.insert_temp(state_id, scroll_x));

    let painter = ui.painter_at(area);
    let clip = ui.clip_rect();
    let first_row = ((clip.top() - area.top() - head_height) / row_height)
        .floor()
        .max(0.0) as usize;
    let last_row = (((clip.bottom() - area.top() - head_height) / row_height)
        .ceil()
        .max(0.0) as usize)
        .min(guide.channels.len());
    let line = crate::stylesheet::color("raised");
    let origin = timeline.left() - scroll_x;
    let timeline_painter = ui.painter_at(Rect::from_min_max(
        Pos2::new(timeline.left(), area.top()),
        area.right_bottom(),
    ));

    let first_tick = guide.from_ms.div_euclid(HOUR_MS) * HOUR_MS;
    let mut tick = first_tick;
    while tick <= guide.to_ms {
        let x = origin + (tick - guide.from_ms) as f32 / HOUR_MS as f32 * hour_width;
        if x + hour_width >= timeline.left() && x <= timeline.right() {
            timeline_painter.line_segment(
                [
                    Pos2::new(x, area.top() + head_height * 0.2),
                    Pos2::new(x, area.top() + head_height),
                ],
                crate::gfx::Stroke::new(stroke::hairline(), line),
            );
            draw_text(
                &timeline_painter,
                Pos2::new(x + gap, area.top() + head_height * 0.6),
                Align2::LEFT_CENTER,
                clock(tick, guide.utc_offset_ms),
                crate::fonts::regular(metrics.screen_card_subtitle_size),
                metrics.text_muted,
                None,
            );
        }
        tick += HOUR_MS;
    }

    for row in first_row..last_row {
        let top = area.top() + head_height + row as f32 * row_height;
        let channel = &guide.channels[row];
        let row_rect = sized(
            Pos2::new(area.left(), top),
            Vec2::new(width, row_height - gap),
        );
        draw_channel(
            ui,
            &painter,
            sized(row_rect.min, Vec2::new(channel_width, row_rect.height())),
            channel,
            row,
            metrics,
            assets,
            layout,
        );
        for (index, programme) in channel
            .programmes
            .iter()
            .enumerate()
            .take(NODE_GUIDE_ROW_STRIDE as usize)
        {
            let left =
                origin + (programme.start_ms - guide.from_ms) as f32 / HOUR_MS as f32 * hour_width;
            let right =
                origin + (programme.end_ms - guide.from_ms) as f32 / HOUR_MS as f32 * hour_width;
            if right < timeline.left() || left > timeline.right() {
                continue;
            }
            let rect = Rect::from_min_max(
                Pos2::new(left.max(timeline.left()) + gap * 0.5, row_rect.top()),
                Pos2::new(right.min(timeline.right()) - gap * 0.5, row_rect.bottom()),
            );
            if rect.width() < gap {
                continue;
            }
            let node = NODE_GUIDE_CELL_BASE + row as u64 * NODE_GUIDE_ROW_STRIDE + index as u64;
            let response = ui.interact(rect, state_id.with(("cell", node)), Sense::click());
            layout.focusable.push((node, rect));
            if response.clicked() {
                layout.activated = Some(node);
            }
            let live = programme.start_ms <= guide.now_ms && programme.end_ms > guide.now_ms;
            let past = programme.end_ms <= guide.now_ms;
            let fill = if live || response.hovered() {
                crate::stylesheet::color("raised")
            } else {
                crate::stylesheet::color("well")
            };
            timeline_painter.rect_filled(rect, metrics.calendar_cell_radius, fill);
            if live {
                timeline_painter.rect_filled(
                    sized(
                        rect.left_top(),
                        Vec2::new(rect.width(), stroke::hairline() * 3.0),
                    ),
                    metrics.calendar_cell_radius,
                    accent(),
                );
            }
            let pad = metrics.calendar_day_padding;
            let text_width = (rect.width() - pad * 2.0).max(1.0);
            if text_width < metrics.screen_card_subtitle_size * 3.0 {
                continue;
            }
            let color = if past {
                metrics.text_muted
            } else {
                text_primary()
            };
            let text_painter =
                timeline_painter.with_clip_rect(rect.intersect(timeline_painter.clip_rect()));
            draw_text(
                &text_painter,
                Pos2::new(
                    rect.left() + pad,
                    rect.center().y - metrics.screen_card_subtitle_size * 0.6,
                ),
                Align2::LEFT_CENTER,
                &programme.title,
                crate::fonts::bold(metrics.screen_card_subtitle_size),
                color,
                Some(text_width),
            );
            draw_text(
                &text_painter,
                Pos2::new(
                    rect.left() + pad,
                    rect.center().y + metrics.screen_card_subtitle_size * 0.7,
                ),
                Align2::LEFT_CENTER,
                format!(
                    "{} – {}",
                    clock(programme.start_ms, guide.utc_offset_ms),
                    clock(programme.end_ms, guide.utc_offset_ms)
                ),
                crate::fonts::regular(metrics.screen_card_subtitle_size - 1.0),
                metrics.text_muted,
                Some(text_width),
            );
        }
    }

    if now_x >= scroll_x && now_x <= scroll_x + timeline.width() {
        let x = origin + now_x;
        timeline_painter.line_segment(
            [Pos2::new(x, area.top()), Pos2::new(x, area.bottom())],
            crate::gfx::Stroke::new(stroke::hairline() * 2.0, accent()),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_channel(
    ui: &mut crate::gfx::Ui,
    painter: &crate::gfx::Painter,
    rect: Rect,
    channel: &GuideChannel,
    row: usize,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let node = NODE_GUIDE_CHANNEL_BASE + row as u64;
    let response = ui.interact(rect, ui.id().with(("guide-channel", row)), Sense::click());
    layout.focusable.push((node, rect));
    if response.clicked() {
        layout.activated = Some(node);
    }
    painter.rect_filled(
        rect,
        metrics.calendar_cell_radius,
        if response.hovered() {
            crate::stylesheet::color("raised")
        } else {
            crate::stylesheet::color("well")
        },
    );
    let pad = metrics.calendar_day_padding;
    let logo_size = (rect.height() - pad * 2.0).max(1.0);
    let mut text_left = rect.left() + pad;
    let mut has_logo = false;
    if let Some(url) = channel.logo.as_deref() {
        let logo = sized(
            Pos2::new(rect.left() + pad, rect.top() + pad),
            Vec2::splat(logo_size),
        );
        has_logo = components::rounded_artwork(
            painter,
            logo,
            metrics.calendar_cell_radius * 0.5,
            Some(url),
            [(logo_size * 2.0) as u32, (logo_size * 2.0) as u32],
            ArtworkPriority::Visible,
            Color32::WHITE,
            assets,
        );
        if has_logo {
            text_left = logo.right() + pad;
        }
    }
    let name_width = rect.right() - text_left - pad;
    if name_width > metrics.screen_card_subtitle_size * 3.0 || !has_logo {
        draw_text(
            painter,
            Pos2::new(text_left, rect.center().y),
            Align2::LEFT_CENTER,
            &channel.name,
            crate::fonts::bold(metrics.screen_card_subtitle_size),
            text_primary(),
            Some(name_width.max(1.0)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn model() -> GuideModel {
        let mut model = GuideModel::default();
        model.apply(&json!({
            "failed": false,
            "channels": [
                {"id": "c1", "name": "One", "programmes": [
                    {"title": "A", "startMs": 0, "endMs": 10},
                    {"title": "B", "startMs": 10, "endMs": 20}
                ]},
                {"id": "c2", "name": "Two", "programmes": [
                    {"title": "C", "startMs": 5, "endMs": 30}
                ]}
            ]
        }));
        model
    }

    #[test]
    fn programme_node_maps_back_to_its_channel() {
        let model = model();
        let node = NODE_GUIDE_CELL_BASE + NODE_GUIDE_ROW_STRIDE + 0;
        assert_eq!(model.channel_for_node(node).unwrap().name, "Two");
        assert_eq!(model.node_label(node).unwrap(), "Two, C");
        assert_eq!(model.node_label(NODE_GUIDE_CHANNEL_BASE).unwrap(), "One");
    }

    #[test]
    fn nodes_outside_the_guide_are_ignored() {
        let model = model();
        assert!(
            model
                .channel_for_node(NODE_GUIDE_CHANNEL_BASE + 5)
                .is_none()
        );
        assert!(model.channel_for_node(NODE_HOME).is_none());
    }

    #[test]
    fn last_end_is_the_latest_programme_end() {
        assert_eq!(model().last_end_ms(), Some(30));
    }

    #[test]
    fn clock_applies_the_utc_offset_and_wraps_the_day() {
        assert_eq!(clock(23 * HOUR_MS, 2 * HOUR_MS), "01:00");
        assert_eq!(clock(0, -HOUR_MS), "23:00");
    }
}
