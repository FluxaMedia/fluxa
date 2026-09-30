use super::*;

pub(super) fn draw_form(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    _metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
) {
    close_button(ui, model, assets, t);
    let width = (viewport.width - 56.0).min(1100.0);
    let stacked = width < 760.0;
    centered_column(ui, width, |ui| {
        components::heading(
            ui,
            &t("profiles.settings"),
            &if model.draft.editing.is_some() {
                t("profiles.edit")
            } else {
                t("profiles.create_new")
            },
            &t("profiles.form_subtitle"),
        );
        let left_width = if stacked {
            width
        } else {
            (width * 0.38).max(280.0)
        };
        let right_width = if stacked {
            width
        } else {
            width - left_width - 14.0
        };
        if stacked {
            form_details(ui, model, assets, t, request, left_width);
            ui.add_space(14.0);
            form_images(ui, model, assets, t, request, right_width);
        } else {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 14.0;
                ui.vertical(|ui| form_details(ui, model, assets, t, request, left_width));
                ui.vertical(|ui| form_images(ui, model, assets, t, request, right_width));
            });
        }
        ui.add_space(48.0);
    });
}

pub(super) fn form_details(
    ui: &mut egui::Ui,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
    width: f32,
) {
    components::panel(ui, |ui| {
        ui.set_width(width - 44.0);
        ui.vertical_centered(|ui| {
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(128.0), Sense::click());
            let name = if model.draft.name.trim().is_empty() {
                t("auto.profile")
            } else {
                model.draft.name.trim().to_owned()
            };
            components::avatar(
                ui.painter(),
                assets,
                rect.center(),
                64.0,
                &name,
                model.draft.avatar_url.as_deref(),
            );
            if response.hovered() {
                ui.painter()
                    .circle_filled(rect.center(), 64.0, Color32::from_black_alpha(90));
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    t("profiles.change_image"),
                    FontId::proportional(12.0),
                    Color32::WHITE,
                );
            }
            if response.on_hover_text(t("profiles.choose_image")).clicked() {
                *request = Some(ProfilesRequest::PickAvatarImage);
            }
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&name)
                        .size(18.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                if model.draft_is_primary() {
                    ui.label(
                        RichText::new(t("profiles.primary_badge").to_uppercase())
                            .size(10.0)
                            .color(Color32::from_white_alpha(160))
                            .background_color(Color32::from_white_alpha(24)),
                    );
                }
            });
            if model.draft.avatar_url.is_some()
                && components::text_button(ui, &t("profiles.use_initials"), 12.0, 110).clicked()
            {
                model.draft.avatar_url = None;
            }
        });
        ui.add_space(20.0);
        let field_width = ui.available_width();
        components::field_label(ui, &t("profiles.name"));
        components::text_input(
            ui,
            &mut model.draft.name,
            &t("profiles.name_placeholder"),
            field_width,
            false,
        );
        if model.duplicate_name() {
            components::note(ui, &t("profiles.duplicate_name"));
        }
        ui.add_space(14.0);
        components::field_label(ui, &t("profiles.pin_lock"));
        let hint = if model.draft.has_pin && !model.draft.remove_pin {
            t("profiles.pin_set_placeholder")
        } else {
            t("profiles.pin_placeholder")
        };
        if components::text_input(ui, &mut model.draft.pin, &hint, field_width, true).changed() {
            model
                .draft
                .pin
                .retain(|character| character.is_ascii_digit());
            model.draft.pin.truncate(4);
            model.draft.remove_pin = false;
        }
        if !model.draft.pin.is_empty() && model.draft.pin.len() != 4 {
            components::note(ui, &t("profiles.pin_invalid"));
        }
        if model.draft.has_pin && !model.draft.remove_pin {
            if components::text_button(ui, &t("profiles.remove_pin"), 12.0, 120).clicked() {
                model.draft.remove_pin = true;
                model.draft.pin.clear();
            }
        }
        if model.draft.remove_pin {
            components::note(ui, &t("profiles.pin_will_be_removed"));
        }
        if !model.draft_is_primary() {
            ui.add_space(14.0);
            components::field_label(ui, &t("profiles.sharing"));
            ui.checkbox(
                &mut model.draft.uses_primary_addons,
                RichText::new(t("profiles.use_primary_addons")).size(13.0),
            );
            ui.checkbox(
                &mut model.draft.uses_primary_plugins,
                RichText::new(t("profiles.use_primary_plugins")).size(13.0),
            );
        }
        ui.add_space(20.0);
        let half = (field_width - 8.0) * 0.5;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if components::secondary_button(ui, &t("common.cancel"), half).clicked() {
                model.mode = ProfilesMode::Select;
            }
            let label = if model.busy {
                t("common.saving")
            } else if model.draft.editing.is_some() {
                t("profiles.save")
            } else {
                t("profiles.create")
            };
            if components::primary_button(ui, &label, half, model.can_save()).clicked() {
                *request = Some(ProfilesRequest::Save);
            }
        });
    });
}

pub(super) fn form_images(
    ui: &mut egui::Ui,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
    width: f32,
) {
    components::panel(ui, |ui| {
        ui.set_width(width - 44.0);
        let inner = ui.available_width();
        if components::secondary_button(ui, &t("profiles.choose_image"), inner).clicked() {
            *request = Some(ProfilesRequest::PickAvatarImage);
        }
        if model.picker.avatar_packs.is_empty() {
            ui.add_space(14.0);
            ui.label(
                RichText::new(t("profiles.no_avatar_packs"))
                    .size(13.0)
                    .color(Color32::from_white_alpha(110)),
            );
            return;
        }
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            components::field_label(ui, &t("profiles.avatar_packs"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                let refresh = components::icon_button(
                    ui,
                    assets.icon("Refresh"),
                    24.0,
                    Color32::from_white_alpha(150),
                    false,
                    false,
                    !model.busy,
                )
                .on_hover_text(t("profiles.refresh_pack"));
                if refresh.clicked() {
                    *request = Some(ProfilesRequest::RefreshAllPacks);
                }
            });
        });
        let tile = 64.0;
        let gap = 10.0;
        let per_row = (((inner + gap) / (tile + gap)).floor() as usize).max(1);
        for pack in model.picker.avatar_packs.clone() {
            ui.add_space(12.0);
            ui.label(
                RichText::new(&pack.title)
                    .size(13.0)
                    .strong()
                    .color(Color32::from_white_alpha(210)),
            );
            ui.add_space(8.0);
            for chunk in pack.avatars.chunks(per_row) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for avatar in chunk {
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::new(tile, tile + 20.0), Sense::click());
                        let center = Pos2::new(rect.center().x, rect.top() + tile * 0.5);
                        components::avatar(
                            ui.painter(),
                            assets,
                            center,
                            tile * 0.5,
                            &avatar.name,
                            Some(&avatar.url),
                        );
                        let selected =
                            model.draft.avatar_url.as_deref() == Some(avatar.url.as_str());
                        if selected || response.hovered() {
                            ui.painter().circle_stroke(
                                center,
                                tile * 0.5 + 2.0,
                                egui::Stroke::new(
                                    2.0,
                                    Color32::from_white_alpha(if selected { 255 } else { 90 }),
                                ),
                            );
                        }
                        let font = FontId::proportional(11.0);
                        ui.painter().text(
                            Pos2::new(center.x, rect.bottom() - 6.0),
                            Align2::CENTER_CENTER,
                            truncate_to_width(ui.painter(), &avatar.name, &font, tile),
                            font,
                            Color32::from_white_alpha(170),
                        );
                        if response.on_hover_text(&avatar.name).clicked() {
                            model.draft.avatar_url = Some(avatar.url.clone());
                        }
                    }
                });
                ui.add_space(gap);
            }
        }
    });
}
