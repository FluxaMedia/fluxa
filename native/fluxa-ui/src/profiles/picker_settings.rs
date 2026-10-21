use super::*;

fn field_row(ui: &mut egui::Ui, compact: bool, add: impl FnOnce(&mut egui::Ui)) {
    if compact {
        ui.horizontal_wrapped(add);
    } else {
        ui.horizontal(add);
    }
}

pub(super) fn draw_picker_settings(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
) {
    let compact = viewport.is_compact();
    profile_header(
        ui,
        model,
        assets,
        metrics,
        compact,
        &t("profiles.picker_settings"),
        t,
    );
    let width = if compact {
        viewport.width - 32.0
    } else {
        (viewport.width - 56.0).min(1024.0)
    };
    centered_column(ui, width, |ui| {
        if !compact {
            components::heading(
                ui,
                &t("profiles.settings"),
                &t("profiles.picker_settings"),
                &t("profiles.picker_background_desc"),
                metrics,
            );
        }
        components::section(ui, compact, metrics, |ui| {
            ui.set_width(if compact { width } else { width - 44.0 });
            ui.label(
                RichText::new(t("profiles.picker_background"))
                    .size(metrics.text.title)
                    .strong()
                    .color(text_primary()),
            );
            ui.add_space(16.0);
            let has_background = model.picker.background_url.is_some();
            let buttons = if has_background { 2.0 } else { 1.0 };
            if compact {
                ui.label(
                    RichText::new(t("profiles.picker_background_desc"))
                        .size(metrics.text.body)
                        .color(metrics.text_muted),
                );
                ui.add_space(12.0);
            }
            field_row(ui, compact, |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(10.0);
                let field_width = if compact {
                    ui.available_width()
                } else {
                    ui.available_width() - buttons * 54.0
                };
                let field = components::text_input(
                    ui,
                    &mut model.background_input,
                    &t("profiles.picker_background_placeholder"),
                    field_width,
                    components::InputKind::Url,
                    metrics,
                );
                if field.lost_focus() {
                    *request = Some(ProfilesRequest::SaveBackground);
                }
                if components::icon_button(
                    ui,
                    assets.icon("ImagePlus"),
                    metrics.icon_button_size,
                    ink(tone::body()),
                    true,
                    false,
                    true,
                )
                .on_hover_text(t("profiles.choose_image"))
                .clicked()
                {
                    *request = Some(ProfilesRequest::PickBackgroundImage);
                }
                if has_background
                    && components::icon_button(
                        ui,
                        assets.icon("Delete"),
                        metrics.icon_button_size,
                        ink(tone::body()),
                        true,
                        false,
                        true,
                    )
                    .on_hover_text(t("profiles.clear_background"))
                    .clicked()
                {
                    model.background_input.clear();
                    *request = Some(ProfilesRequest::SaveBackground);
                }
            });
        });
        ui.add_space(20.0);
        components::section(ui, compact, metrics, |ui| {
            ui.set_width(if compact { width } else { width - 44.0 });
            ui.label(
                RichText::new(t("profiles.avatar_packs"))
                    .size(metrics.text.title)
                    .strong()
                    .color(text_primary()),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(t("profiles.avatar_packs_desc"))
                    .size(metrics.text.body)
                    .color(metrics.text_muted),
            );
            ui.add_space(16.0);
            field_row(ui, compact, |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(10.0);
                let field_width = if compact {
                    ui.available_width()
                } else {
                    ui.available_width() - 150.0
                };
                let field = components::text_input(
                    ui,
                    &mut model.repository_input,
                    &t("profiles.avatar_pack_repository_placeholder"),
                    field_width,
                    components::InputKind::Url,
                    metrics,
                );
                let submit =
                    field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                let label = if model.busy {
                    t("common.loading")
                } else {
                    t("profiles.add_pack")
                };
                let can_add = !model.repository_input.trim().is_empty() && !model.busy;
                if (components::primary_button(
                    ui,
                    &label,
                    if compact { width } else { 140.0 },
                    can_add,
                )
                .clicked()
                    || submit)
                    && can_add
                {
                    *request = Some(ProfilesRequest::AddRepository);
                }
            });
            ui.add_space(18.0);
            if model.picker.avatar_packs.is_empty() {
                ui.label(
                    RichText::new(t("profiles.no_avatar_packs"))
                        .size(metrics.text.body)
                        .color(metrics.text_muted),
                );
            }
            for pack in model.picker.avatar_packs.clone() {
                components::surface(
                    ui,
                    if compact {
                        Color32::TRANSPARENT
                    } else {
                        ink(tone::faint())
                    },
                    10.0,
                    egui::Margin::symmetric(14, 12),
                    |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let previews = if compact {
                                0
                            } else {
                                pack.avatars.len().min(6)
                            };
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(
                                    if previews > 0 {
                                        24.0 + previews as f32 * 32.0
                                    } else {
                                        0.0
                                    },
                                    48.0,
                                ),
                                Sense::hover(),
                            );
                            for (index, avatar) in pack.avatars.iter().take(previews).enumerate() {
                                let center = Pos2::new(
                                    rect.left() + 24.0 + index as f32 * 32.0,
                                    rect.center().y,
                                );
                                ui.painter().circle_filled(center, 24.5, surface_color());
                                components::avatar(
                                    ui.painter(),
                                    assets,
                                    center,
                                    23.0,
                                    &avatar.name,
                                    Some(&avatar.url),
                                    metrics,
                                );
                            }
                            ui.add_space(8.0);
                            let label_width = if compact {
                                ui.available_width() - 84.0
                            } else {
                                ui.available_width()
                            };
                            ui.allocate_ui_with_layout(
                                Vec2::new(label_width, 32.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(format!(
                                                "{} · {}",
                                                pack.title,
                                                pack.avatars.len()
                                            ))
                                            .size(metrics.text.body)
                                            .color(metrics.text_primary),
                                        )
                                        .truncate(),
                                    );
                                },
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if components::icon_button(
                                        ui,
                                        assets.icon("Delete"),
                                        32.0,
                                        danger_color(),
                                        false,
                                        false,
                                        true,
                                    )
                                    .on_hover_text(t("profiles.remove_pack"))
                                    .clicked()
                                    {
                                        *request =
                                            Some(ProfilesRequest::RemovePack(pack.id.clone()));
                                    }
                                    if components::icon_button(
                                        ui,
                                        assets.icon("Refresh"),
                                        32.0,
                                        ink(tone::subtle()),
                                        false,
                                        false,
                                        !model.busy,
                                    )
                                    .on_hover_text(t("profiles.refresh_pack"))
                                    .clicked()
                                    {
                                        *request = Some(ProfilesRequest::RefreshRepository(
                                            pack.repository_url.clone(),
                                        ));
                                    }
                                },
                            );
                        });
                    },
                    metrics,
                );
                ui.add_space(10.0);
            }
        });
        ui.add_space(16.0);
        if !compact
            && components::text_button(ui, &format!("‹  {}", t("common.back")), 14.0, 255).clicked()
        {
            model.mode = ProfilesMode::Select;
        }
        ui.add_space(48.0);
    });
}
