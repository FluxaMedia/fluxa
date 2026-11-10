use super::*;
use crate::components::{glass_shape, liquid_glass};
use crate::dp::Dp;
use crate::navigation::{nav_bar_padding, nav_bar_top, nav_item_height, nav_pill_gap};

fn bar_visible(viewport: Viewport, home: &HomeModel) -> bool {
    home.category_bar && home.categories.len() > 1 && viewport.side_inset > 0.0
}

pub(crate) fn category_bar_space(viewport: Viewport, home: &HomeModel) -> f32 {
    if bar_visible(viewport, home) && !home_shows_hero(home) {
        nav_bar_top()
            + nav_item_height()
            + nav_bar_padding() * 2.0
            + Space::S8.px(&crate::metrics::active())
    } else {
        0.0
    }
}

pub(crate) fn draw_category_bar(
    context: &crate::gfx::Context,
    viewport: Viewport,
    home: &HomeModel,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    if !bar_visible(viewport, home) {
        return;
    }
    let font = FontId::proportional(metrics.navigation_label_size());
    let labels: Vec<String> = home
        .categories
        .iter()
        .map(|category| localized(category.label_key(), &home.language))
        .collect();
    let chip_pad = Dp::D40.px();
    let widths: Vec<f32> = labels
        .iter()
        .map(|label| estimated_navigation_text_width(label, font.size) + chip_pad)
        .collect();
    let cells: Vec<crate::flex::Node> = widths
        .iter()
        .enumerate()
        .map(|(index, width)| crate::flex::leaf(Vec2::new(*width, nav_item_height())).id(index))
        .collect();
    let bar = crate::flex::row(cells)
        .gap(nav_pill_gap())
        .padding(crate::flex::Insets::all(nav_bar_padding()))
        .align(crate::flex::Align::Start);
    let size = bar.outer_size();
    let top = nav_bar_top() + home_nav_viewport(viewport, home).nav_lead;
    let lane = Rect::from_min_size(
        Pos2::new(0.0, top),
        Vec2::new(viewport.width.max(size.x + Space::S12.px(&metrics)), size.y),
    );
    let rect = crate::flex::anchored(lane, Align2::CENTER_TOP, size, 0.0);
    let placed = bar.solve(rect);
    let mut clicked = None;
    crate::gfx::Area::new(Id::new("fluxa-home-category-bar"))
        .fixed_pos(rect.min)
        .order(crate::gfx::Order::Foreground)
        .show(context, |ui| {
            ui.allocate_rect(rect, Sense::hover());
            let fill = overlay_panel();
            let radius = rect.height() * 0.5;
            if liquid_glass() {
                ui.painter().add(glass_shape(rect, radius, fill));
            } else {
                ui.painter().rect_filled(rect, radius, fill);
            }
            ui.painter().rect_stroke(
                rect,
                radius,
                crate::gfx::Stroke::new(stroke::hairline(), metrics.border),
                crate::gfx::StrokeKind::Inside,
            );
            for (index, (category, label)) in home.categories.iter().zip(&labels).enumerate() {
                let Some(cell) = placed.find(index) else {
                    continue;
                };
                let response = ui.interact(
                    cell,
                    Id::new(("fluxa-home-category-chip", index)),
                    Sense::click(),
                );
                let active = *category == home.active_category;
                if active {
                    ui.painter().add(glass_shape(
                        cell,
                        nav_item_height() * 0.5,
                        crate::stylesheet::color("strong"),
                    ));
                } else if response.hovered() {
                    ui.painter().rect_filled(
                        cell,
                        nav_item_height() * 0.5,
                        crate::stylesheet::color("well"),
                    );
                }
                draw_text(
                    ui.painter(),
                    cell.center(),
                    Align2::CENTER_CENTER,
                    label.clone(),
                    font.clone(),
                    if active || response.hovered() {
                        text_primary()
                    } else {
                        crate::stylesheet::color("text.subtle")
                    },
                    None,
                );
                layout.focusable.push((category.node(), cell));
                if response.clicked() {
                    clicked = Some(category.node());
                }
            }
        });
    if clicked.is_some() {
        layout.activated = clicked;
    }
}
