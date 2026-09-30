use super::*;

pub(super) fn draw_picker_settings(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    _metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
) {
    close_button(ui, model, assets, t);
    let width = (viewport.width - 56.0).min(1024.0);
    centered_column(ui, width, |ui| {
        heading(
            ui,
            &t("profiles.settings"),
            &t("profiles.picker_settings"),
            &t("profiles.picker_background_desc"),
        );
        panel(ui, |ui| {
            ui.set_width(width - 44.0);
            ui.label(
                RichText::new(t("profiles.picker_background"))
                    .size(19.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(16.0);
            let has_background = model.picker.background_url.is_some();
            let buttons = if has_background { 2.0 } else { 1.0 };
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let field_width = ui.available_width() - buttons * 54.0;
                let field = components::text_input(
                    ui,
                    &mut model.background_input,
                    &t("profiles.picker_background_placeholder"),
                    field_width,
                    false,
                );
                if field.lost_focus() {
                    *request = Some(ProfilesRequest::SaveBackground);
                }
                if components::icon_button(
                    ui,
                    assets.icon("ImagePlus"),
                    44.0,
                    Color32::from_white_alpha(200),
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
                        44.0,
                        Color32::from_white_alpha(200),
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
        panel(ui, |ui| {
            ui.set_width(width - 44.0);
            ui.label(
                RichText::new(t("profiles.avatar_packs"))
                    .size(19.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(t("profiles.avatar_packs_desc"))
                    .size(14.0)
                    .color(Color32::from_white_alpha(120)),
            );
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let field_width = ui.available_width() - 150.0;
                let field = components::text_input(
                    ui,
                    &mut model.repository_input,
                    &t("profiles.avatar_pack_repository_placeholder"),
                    field_width,
                    false,
                );
                let submit =
                    field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                let label = if model.busy {
                    t("common.loading")
                } else {
                    t("profiles.add_pack")
                };
                let can_add = !model.repository_input.trim().is_empty() && !model.busy;
                if (primary_button(ui, &label, 140.0, can_add).clicked() || submit) && can_add {
                    *request = Some(ProfilesRequest::AddRepository);
                }
            });
            ui.add_space(18.0);
            if model.picker.avatar_packs.is_empty() {
                ui.label(
                    RichText::new(t("profiles.no_avatar_packs"))
                        .size(14.0)
                        .color(Color32::from_white_alpha(110)),
                );
            }
            for pack in model.picker.avatar_packs.clone() {
                egui::Frame::NONE
                    .fill(Color32::from_white_alpha(8))
                    .corner_radius(10.0)
                    .inner_margin(egui::Margin::symmetric(14, 12))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let previews = pack.avatars.len().min(6);
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(24.0 + previews as f32 * 32.0, 48.0),
                                Sense::hover(),
                            );
                            for (index, avatar) in pack.avatars.iter().take(previews).enumerate() {
                                let center = Pos2::new(
                                    rect.left() + 24.0 + index as f32 * 32.0,
                                    rect.center().y,
                                );
                                ui.painter().circle_filled(
                                    center,
                                    24.5,
                                    Color32::from_rgb(20, 20, 20),
                                );
                                components::avatar(
                                    ui.painter(),
                                    assets,
                                    center,
                                    23.0,
                                    &avatar.name,
                                    Some(&avatar.url),
                                );
                            }
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(format!(
                                    "{} · {}",
                                    pack.title,
                                    t("profiles.avatar_pack_count").replacen(
                                        "%s",
                                        &pack.avatars.len().to_string(),
                                        1
                                    )
                                ))
                                .size(14.0)
                                .color(Color32::from_white_alpha(220)),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if components::icon_button(
                                        ui,
                                        assets.icon("Delete"),
                                        32.0,
                                        Color32::from_rgb(211, 74, 74),
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
                                        Color32::from_white_alpha(170),
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
                    });
                ui.add_space(10.0);
            }
        });
        ui.add_space(16.0);
        if ui
            .add(
                egui::Button::new(
                    RichText::new(format!("‹  {}", t("common.back")))
                        .size(14.0)
                        .color(Color32::WHITE),
                )
                .frame(false),
            )
            .clicked()
        {
            model.mode = ProfilesMode::Select;
        }
        ui.add_space(48.0);
    });
}
