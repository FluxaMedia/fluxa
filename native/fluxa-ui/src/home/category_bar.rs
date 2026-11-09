use super::*;
use crate::components::{glass_shape, liquid_glass};
use crate::navigation::{nav_bar_padding, nav_bar_top, nav_item_height, nav_pill_gap};

const BAR_GAP: f32 = 8.0;

fn bar_visible(viewport: Viewport, home: &HomeModel) -> bool {
    home.category_bar && home.categories.len() > 1 && viewport.side_inset > 0.0
}

pub(crate) fn category_bar_space(viewport: Viewport, home: &HomeModel) -> f32 {
    if bar_visible(viewport, home) && !home_shows_hero(home) {
        nav_bar_top() + nav_item_height() + nav_bar_padding() * 2.0 + BAR_GAP
    } else {
        0.0
    }
}

pub(crate) fn draw_category_bar(
    context: &egui::Context,
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
    let widths: Vec<f32> = labels
        .iter()
        .map(|label| estimated_navigation_text_width(label, font.size) + 40.0)
        .collect();
    let inner = widths.iter().sum::<f32>() + nav_pill_gap() * (widths.len() - 1) as f32;
    let width = inner + nav_bar_padding() * 2.0;
    let top = nav_bar_top() + home_nav_viewport(viewport, home).nav_lead;
    let mut clicked = None;
    egui::Area::new(Id::new("fluxa-home-category-bar"))
        .fixed_pos(Pos2::new((viewport.width - width).max(12.0) * 0.5, top))
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let fill = overlay_panel();
            let backdrop = ui.painter().add(egui::Shape::Noop);
            let radius = nav_item_height() * 0.5 + nav_bar_padding();
            let frame = egui::Frame::NONE
                .fill(if liquid_glass() {
                    Color32::TRANSPARENT
                } else {
                    fill
                })
                .stroke(egui::Stroke::new(stroke::hairline(), metrics.border))
                .corner_radius(radius)
                .inner_margin(egui::Margin::same(nav_bar_padding() as i8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = nav_pill_gap();
                        for ((category, label), item_width) in
                            home.categories.iter().zip(&labels).zip(&widths)
                        {
                            let (rect, response) = ui.allocate_exact_size(
                                Vec2::new(*item_width, nav_item_height()),
                                Sense::click(),
                            );
                            let active = *category == home.active_category;
                            if active {
                                ui.painter().add(glass_shape(
                                    rect,
                                    nav_item_height() * 0.5,
                                    crate::stylesheet::color("strong"),
                                ));
                            } else if response.hovered() {
                                ui.painter().rect_filled(
                                    rect,
                                    nav_item_height() * 0.5,
                                    crate::stylesheet::color("well"),
                                );
                            }
                            draw_text(
                                ui.painter(),
                                rect.center(),
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
                            layout.focusable.push((category.node(), rect));
                            if response.clicked() {
                                clicked = Some(category.node());
                            }
                        }
                    });
                });
            if liquid_glass() {
                ui.painter()
                    .set(backdrop, glass_shape(frame.response.rect, radius, fill));
            }
        });
    if clicked.is_some() {
        layout.activated = clicked;
    }
}
