use super::*;
use crate::kit::{self, Role, Type};

mod avatars;
mod form;
mod model;
mod picker_settings;
mod pin;
mod select;

use avatars::*;
use form::*;
pub use model::*;
use picker_settings::*;
use pin::*;
use select::*;

pub fn draw_profiles(
    context: &crate::gfx::Context,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let screen = Rect::from_min_max(Pos2::ZERO, Pos2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    paint_backdrop(context, &painter, screen, assets);
    let metrics = metrics_for_assets(viewport, assets);
    crate::kit::provide(context, metrics);
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut request = None;
    let blocked = model.pin_prompt.is_some() || model.confirm_delete.is_some();

    let area_id = Id::new("fluxa-profiles");
    let tv = viewport.is_tv();
    if tv {
        crate::components::fit_to_safe_area(context, viewport, area_id);
    }
    crate::set_auto_focus(context, !blocked && model.mode != ProfilesMode::Select);
    crate::gfx::Area::new(area_id)
        .fixed_pos(Pos2::ZERO)
        .interactable(!blocked)
        .show(context, |ui| {
            if !tv {
                ui.set_min_size(screen.size());
            }
            ui.set_max_size(screen.size());
            let mut body = |ui: &mut crate::gfx::Ui| {
                match model.mode {
                    ProfilesMode::Select => draw_select(ui, viewport, model, assets, metrics, &t),
                    ProfilesMode::Form => {
                        draw_form(ui, viewport, model, assets, metrics, &t, &mut request)
                    }
                    ProfilesMode::Avatars => draw_avatars(ui, viewport, model, assets, metrics, &t),
                    ProfilesMode::Settings => {
                        draw_picker_settings(ui, viewport, model, assets, metrics, &t, &mut request)
                    }
                }
                let inset = components::keyboard_inset(ui.ctx());
                kit::lay(ui, vec![kit::layout_items::gap(inset)]);
            };
            if tv {
                crate::components::unclip_for_fit(ui, screen.size());
                body(ui);
                crate::components::record_fit_content(context, area_id, ui.min_rect());
                return;
            }
            crate::components::pan_area(
                ui,
                "fluxa-profiles-scroll",
                |area| area.auto_shrink([false, false]),
                body,
            );
        });

    crate::set_auto_focus(context, false);
    if model.pin_prompt.is_some() {
        draw_pin_prompt(context, screen, model, assets, metrics, &t, &mut request);
    } else if let Some(id) = model.confirm_delete.clone() {
        crate::set_auto_focus(context, true);
        let name = model
            .profile(&id)
            .map(|profile| profile.name.clone())
            .unwrap_or_default();
        components::modal(
            context,
            screen,
            "fluxa-profiles-confirm",
            |ui| {
                let title = t("profiles.delete_confirm_title");
                let body = t("profiles.delete_confirm_body").replacen("%s", &name, 1);
                let cancel = t("common.cancel");
                let delete = t("profiles.delete");
                let cancelled = std::cell::Cell::new(false);
                let deleted = std::cell::Cell::new(false);
                let items = vec![
                    kit::widget(ui, |ui| {
                        kit::text(title.clone(), Type::Title).show(ui);
                    }),
                    kit::layout_items::space(ui, Space::S8),
                    kit::widget(ui, |ui| {
                        kit::text(body.clone(), Type::Body)
                            .role(Role::Secondary)
                            .show(ui);
                    }),
                    kit::layout_items::space(ui, Space::S20),
                    kit::widget(ui, |ui| {
                        kit::view()
                            .class("profiles-confirm-actions")
                            .row()
                            .show(ui, |ui| {
                                if components::secondary_button(ui, &cancel, 140.0).clicked() {
                                    cancelled.set(true);
                                }
                                if components::danger_button(ui, &delete, 140.0, metrics).clicked()
                                {
                                    deleted.set(true);
                                }
                            });
                    }),
                ];
                kit::lay(ui, items);
                if cancelled.get() {
                    model.confirm_delete = None;
                }
                if deleted.get() {
                    model.confirm_delete = None;
                    request = Some(ProfilesRequest::Delete(id.clone()));
                }
            },
            metrics,
        );
    }

    if let Some(notice) = model.notice.clone() {
        crate::gfx::Area::new(Id::new("fluxa-profiles-notice"))
            .anchor(
                crate::gfx::Align2::RIGHT_TOP,
                Vec2::new(-Space::S24.px(&metrics), Space::S24.px(&metrics)),
            )
            .order(crate::gfx::Order::Tooltip)
            .show(context, |ui| {
                kit::view().class("profiles-notice").row().show(ui, |ui| {
                    ui.add(
                        crate::gfx::Label::new(
                            RichText::new(notice)
                                .size(metrics.text.body)
                                .color(metrics.text_primary),
                        )
                        .wrap(),
                    );
                    if components::icon_button(
                        ui,
                        assets.icon("Close"),
                        22.0,
                        crate::stylesheet::color("text.subtle"),
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
    }

    let _ = metrics;
    HomeLayout {
        profiles: request,
        ..HomeLayout::default()
    }
}

fn paint_backdrop(
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
    screen: Rect,
    assets: &mut impl HomeAssets,
) {
    paint_brand_ambient(painter, screen, assets);
    let url = assets.custom_background_url().map(ToOwned::to_owned);
    let size = artwork_target_size(screen.size(), context.pixels_per_point());
    let Some(media) = assets.media_texture_for(url.as_deref(), size, ArtworkPriority::Hero) else {
        if assets.dom_images() {
            components::artwork_image(
                painter,
                screen,
                url.as_deref(),
                size,
                ArtworkPriority::Hero,
                Color32::WHITE,
                assets,
            );
        }
        return;
    };
    let crop = cover_uv(media.image_size, screen);
    let tile = media.uv;
    let uv = Rect::from_min_max(
        Pos2::new(
            tile.min.x + crop.min.x * tile.width(),
            tile.min.y + crop.min.y * tile.height(),
        ),
        Pos2::new(
            tile.min.x + crop.max.x * tile.width(),
            tile.min.y + crop.max.y * tile.height(),
        ),
    );
    painter.image(media.texture, screen, uv, Color32::WHITE);
    let scrim = crate::stylesheet::resolve(
        context,
        "scrim.backdrop",
        Id::new("fluxa-profiles-scrim"),
        crate::stylesheet::State {
            enabled: true,
            ..Default::default()
        },
        None,
    );
    painter.rect_filled(screen, 0.0, scrim.fill);
}

pub(crate) fn paint_custom_background(
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
    screen: Rect,
    assets: &mut impl HomeAssets,
) {
    paint_backdrop(context, painter, screen, assets);
}

fn page(ui: &crate::gfx::Ui, width: f32) -> crate::kit::Modifier {
    crate::kit::Modifier::new()
        .max_width(width)
        .margin(Space::S16)
        .center_x()
        .gap_px(ui.item_spacing().y)
}

fn inline(ui: &crate::gfx::Ui) -> crate::kit::Modifier {
    crate::kit::Modifier::new().gap_px(ui.item_spacing().x)
}

fn profile_header(
    ui: &mut crate::gfx::Ui,
    model: &mut ProfilesModel,
    assets: &impl HomeAssets,
    metrics: UiMetrics,
    compact: bool,
    title: &str,
    t: &dyn Fn(&str) -> String,
) {
    let leave = if compact {
        components::screen_bar(ui, assets, title, &t("common.back"), metrics)
    } else {
        components::close_button(ui, assets, &t("common.close"), metrics)
    };
    if leave {
        model.back();
    }
}
