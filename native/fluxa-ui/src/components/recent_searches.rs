use super::*;
use crate::flex;

pub const NODE_DISCOVER_RECENT_BASE: u64 = 21_000;
const RECENT_NODE_SPAN: u64 = 32;

pub fn discover_recent(node: u64) -> Option<usize> {
    let slot = node.checked_sub(NODE_DISCOVER_RECENT_BASE)?;
    (slot < RECENT_NODE_SPAN).then_some(slot as usize)
}

pub(crate) fn recent_searches_open(focused: Option<u64>, entries: &[String]) -> bool {
    !entries.is_empty()
        && focused.is_some_and(|node| {
            node == crate::NODE_DISCOVER_SEARCH || discover_recent(node).is_some()
        })
}

fn paint_clock(painter: &Painter, center: Pos2, color: Color32) {
    let stroke = crate::gfx::Stroke::new(stroke::icon(), color);
    painter.circle_stroke(center, 7.0, stroke);
    painter.line_segment([center, center + Vec2::new(0.0, -4.0)], stroke);
    painter.line_segment([center, center + Vec2::new(3.0, 1.5)], stroke);
}

pub(crate) fn recent_searches(
    context: &crate::gfx::Context,
    anchor: Rect,
    heading: &str,
    entries: &[String],
    layout: &mut HomeLayout,
    metrics: UiMetrics,
) {
    let row = metrics.screen_control_height.max(38.0);
    let head = row * 0.8;
    let pad = 6.0;
    let shown = entries.len().min(RECENT_NODE_SPAN as usize);
    let panel = Rect::from_min_size(
        Pos2::new(anchor.left(), anchor.bottom() + 6.0),
        Vec2::new(anchor.width(), pad * 2.0 + head + row * shown as f32),
    );
    placed_front(
        context,
        Id::new("fluxa-shared-recent-searches"),
        panel.min,
        |ui| {
            ui.allocate_exact_size(panel.size(), Sense::hover());
            let painter = ui.painter();
            glass(
                painter,
                panel,
                metrics.dialog_radius,
                metrics.surface_raised,
            );
            painter.rect_stroke(
                panel,
                metrics.dialog_radius,
                crate::gfx::Stroke::new(stroke::hairline(), metrics.border),
                crate::gfx::StrokeKind::Inside,
            );
            let font = FontId::proportional(metrics.nav_label_size + 1.0);
            draw_text(
                painter,
                Pos2::new(panel.left() + 16.0, panel.top() + pad + head * 0.5),
                Align2::LEFT_CENTER,
                heading.to_owned(),
                FontId::proportional(metrics.nav_label_size),
                metrics.ink(tone::muted()),
                None,
            );
            let list = flex::column(
                std::iter::once(flex::leaf(Vec2::new(0.0, head)).id(0))
                    .chain((0..shown).map(|index| flex::leaf(Vec2::new(0.0, row)).id(index + 1)))
                    .collect::<Vec<_>>(),
            )
            .padding(flex::Insets::all(pad))
            .shrink(0.0)
            .solve(panel);
            for (index, entry) in entries.iter().take(shown).enumerate() {
                let rect = list.find(index + 1).unwrap_or(panel);
                let node = NODE_DISCOVER_RECENT_BASE + index as u64;
                let response = tap(
                    ui,
                    Id::new(("fluxa-recent-search", index)),
                    rect,
                    radius::control(),
                    node,
                    Some(layout),
                );
                if response.clicked() {
                    layout.activated = Some(node);
                }
                paint_clock(
                    ui.painter(),
                    rect.left_center() + Vec2::new(18.0, 0.0),
                    metrics.ink(tone::muted()),
                );
                draw_text(
                    ui.painter(),
                    rect.left_center() + Vec2::new(40.0, 0.0),
                    Align2::LEFT_CENTER,
                    truncate_to_width(ui.painter(), entry, &font, (rect.width() - 56.0).max(1.0)),
                    font.clone(),
                    metrics.text_primary,
                    None,
                );
            }
        },
    );
}
