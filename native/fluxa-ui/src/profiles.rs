use super::*;

mod form;
mod model;
mod picker_settings;
mod pin;
mod select;

use form::*;
pub use model::*;
use picker_settings::*;
use pin::*;
use select::*;

pub fn draw_profiles(
    context: &egui::Context,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    paint_backdrop(context, &painter, screen, assets);
    let metrics = metrics_for_assets(viewport, assets);
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut request = None;
    let blocked = model.pin_prompt.is_some() || model.confirm_delete.is_some();

    egui::Area::new(Id::new("fluxa-profiles"))
        .fixed_pos(Pos2::ZERO)
        .interactable(!blocked)
        .show(context, |ui| {
            ui.set_min_size(screen.size());
            ui.set_max_size(screen.size());
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match model.mode {
                    ProfilesMode::Select => {
                        draw_select(ui, viewport, model, assets, metrics, &t, &mut request)
                    }
                    ProfilesMode::Form => {
                        draw_form(ui, viewport, model, assets, metrics, &t, &mut request)
                    }
                    ProfilesMode::Settings => {
                        draw_picker_settings(ui, viewport, model, assets, metrics, &t, &mut request)
                    }
                });
        });

    if model.pin_prompt.is_some() {
        draw_pin_prompt(context, screen, model, assets, metrics, &t, &mut request);
    } else if let Some(id) = model.confirm_delete.clone() {
        let name = model
            .profile(&id)
            .map(|profile| profile.name.clone())
            .unwrap_or_default();
        components::modal(context, screen, "fluxa-profiles-confirm", |ui| {
            ui.label(
                RichText::new(t("profiles.delete_confirm_title"))
                    .size(22.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(t("profiles.delete_confirm_body").replacen("%s", &name, 1))
                    .size(14.0)
                    .color(Color32::from_white_alpha(170)),
            );
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                if components::secondary_button(ui, &t("common.cancel"), 140.0).clicked() {
                    model.confirm_delete = None;
                }
                if components::danger_button(ui, &t("profiles.delete"), 140.0).clicked() {
                    model.confirm_delete = None;
                    request = Some(ProfilesRequest::Delete(id.clone()));
                }
            });
        });
    }

    if let Some(notice) = model.notice.clone() {
        egui::Area::new(Id::new("fluxa-profiles-notice"))
            .fixed_pos(Pos2::new(screen.right() - 376.0, 24.0))
            .order(egui::Order::Tooltip)
            .show(context, |ui| {
                egui::Frame::NONE
                    .fill(crate::theme::SURFACE_RAISED)
                    .stroke(crate::theme::border())
                    .corner_radius(10.0)
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.set_width(320.0);
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(notice)
                                        .size(14.0)
                                        .color(Color32::from_white_alpha(220)),
                                )
                                .wrap(),
                            );
                            if components::icon_button(
                                ui,
                                assets.icon("Close"),
                                22.0,
                                Color32::from_white_alpha(170),
                                false,
                                false,
                                true,
                            )
                            .clicked()
                            {
                                model.notice = None;
                            }
                        });
                    });
            });
    }

    let _ = metrics;
    HomeLayout {
        profiles: request,
        ..HomeLayout::default()
    }
}

fn paint_backdrop(
    context: &egui::Context,
    painter: &egui::Painter,
    screen: Rect,
    assets: &mut impl HomeAssets,
) {
    paint_brand_ambient(painter, screen, assets);
    let url = assets.custom_background_url().map(ToOwned::to_owned);
    let size = artwork_target_size(screen.size(), context.pixels_per_point());
    if components::artwork_image(
        painter,
        screen,
        url.as_deref(),
        size,
        ArtworkPriority::Hero,
        Color32::WHITE,
        assets,
    ) {
        painter.rect_filled(
            screen,
            0.0,
            Color32::from_rgba_unmultiplied(12, 12, 12, 210),
        );
    }
}

pub(crate) fn paint_custom_background(
    context: &egui::Context,
    painter: &egui::Painter,
    screen: Rect,
    assets: &mut impl HomeAssets,
) {
    paint_backdrop(context, painter, screen, assets);
}

fn centered_column(ui: &mut egui::Ui, width: f32, add: impl FnOnce(&mut egui::Ui)) {
    let available = ui.available_width();
    let inset = ((available - width) * 0.5).max(16.0);
    ui.horizontal(|ui| {
        ui.add_space(inset);
        ui.vertical(|ui| {
            ui.set_width(width.min(available - 32.0));
            add(ui);
        });
    });
}

fn close_button(
    ui: &mut egui::Ui,
    model: &mut ProfilesModel,
    assets: &impl HomeAssets,
    t: &dyn Fn(&str) -> String,
) {
    ui.add_space(20.0);
    ui.horizontal(|ui| {
        ui.add_space(28.0);
        let response = components::icon_button(
            ui,
            assets.icon("Close"),
            36.0,
            Color32::from_white_alpha(190),
            true,
            false,
            true,
        )
        .on_hover_text(t("common.close"));
        if response.clicked() {
            model.mode = ProfilesMode::Select;
        }
    });
    ui.add_space(24.0);
}
