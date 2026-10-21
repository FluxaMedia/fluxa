use super::*;

pub(super) fn draw_form(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
) {
    let compact = viewport.is_compact();
    let title = if model.draft.editing.is_some() {
        t("profiles.edit")
    } else {
        t("profiles.create_new")
    };
    profile_header(ui, model, assets, metrics, compact, &title, t);
    let width = if compact {
        viewport.width - 32.0
    } else {
        (viewport.width - 56.0).min(520.0)
    };
    centered_column(ui, width, |ui| {
        if !compact {
            components::heading(
                ui,
                &t("profiles.settings"),
                &title,
                &t("profiles.form_subtitle"),
                metrics,
            );
        }
        form_details(ui, model, assets, t, request, width, metrics, compact);
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
    metrics: UiMetrics,
    compact: bool,
) {
    components::section(ui, compact, metrics, |ui| {
        ui.set_width(if compact { width } else { width - 44.0 });
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
                metrics,
            );
            if response.hovered() {
                ui.painter()
                    .circle_filled(rect.center(), 64.0, shade(Shade::Faint));
                draw_text(
                    ui.painter(),
                    rect.center(),
                    Align2::CENTER_CENTER,
                    t("profiles.change_image"),
                    FontId::proportional(metrics.text.label),
                    text_primary(),
                    None,
                );
            }
            if response.on_hover_text(t("profiles.choose_icon")).clicked() {
                model.mode = ProfilesMode::Avatars;
            }
            ui.add_space(16.0);
            ui.label(
                RichText::new(&name)
                    .size(metrics.text.subtitle)
                    .strong()
                    .color(text_primary()),
            );
            if model.draft_is_primary() {
                ui.label(
                    RichText::new(t("profiles.primary_badge").to_uppercase())
                        .size(metrics.text.label)
                        .color(metrics.text_secondary)
                        .background_color(ink(tone::soft())),
                );
            }
            if model.draft.avatar_url.is_some()
                && components::text_button(ui, &t("profiles.use_initials"), 12.0, 110).clicked()
            {
                model.draft.avatar_url = None;
            }
        });
        ui.add_space(20.0);
        let field_width = ui.available_width();
        components::field_label(ui, &t("profiles.name"), metrics);
        components::text_input(
            ui,
            &mut model.draft.name,
            &t("profiles.name_placeholder"),
            field_width,
            components::InputKind::Text,
            metrics,
        );
        if model.duplicate_name() {
            components::note(ui, &t("profiles.duplicate_name"), metrics);
        }
        ui.add_space(14.0);
        components::field_label(ui, &t("profiles.image_url"), metrics);
        let url_field = components::text_input(
            ui,
            &mut model.avatar_input,
            &t("profiles.image_url_placeholder"),
            field_width,
            components::InputKind::Url,
            metrics,
        );
        if url_field.changed() {
            let url = model.avatar_input.trim();
            model.draft.avatar_url = (!url.is_empty()).then(|| url.to_owned());
        } else if !url_field.has_focus() {
            model.avatar_input = model
                .draft
                .avatar_url
                .clone()
                .filter(|url| url.starts_with("http"))
                .unwrap_or_default();
        }
        ui.add_space(14.0);
        components::field_label(ui, &t("profiles.pin_lock"), metrics);
        let hint = if model.draft.has_pin && !model.draft.remove_pin {
            t("profiles.pin_set_placeholder")
        } else {
            t("profiles.pin_placeholder")
        };
        if components::text_input(
            ui,
            &mut model.draft.pin,
            &hint,
            field_width,
            components::InputKind::Number { secret: true },
            metrics,
        )
        .changed()
        {
            model
                .draft
                .pin
                .retain(|character| character.is_ascii_digit());
            model.draft.pin.truncate(4);
            model.draft.remove_pin = false;
        }
        if !model.draft.pin.is_empty() && model.draft.pin.len() != 4 {
            components::note(ui, &t("profiles.pin_invalid"), metrics);
        }
        if model.draft.has_pin && !model.draft.remove_pin {
            if components::text_button(ui, &t("profiles.remove_pin"), 12.0, 120).clicked() {
                model.draft.remove_pin = true;
                model.draft.pin.clear();
            }
        }
        if model.draft.remove_pin {
            components::note(ui, &t("profiles.pin_will_be_removed"), metrics);
        }
        if !model.draft_is_primary() {
            ui.add_space(14.0);
            components::field_label(ui, &t("profiles.sharing"), metrics);
            ui.checkbox(
                &mut model.draft.uses_primary_addons,
                RichText::new(t("profiles.use_primary_addons")).size(metrics.text.meta),
            );
            ui.checkbox(
                &mut model.draft.uses_primary_plugins,
                RichText::new(t("profiles.use_primary_plugins")).size(metrics.text.meta),
            );
        }
        ui.add_space(20.0);
        let half = (field_width - 8.0) * 0.5;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if !compact && components::secondary_button(ui, &t("common.cancel"), half).clicked() {
                model.mode = ProfilesMode::Select;
            }
            let label = if model.busy {
                t("common.saving")
            } else if model.draft.editing.is_some() {
                t("profiles.save")
            } else {
                t("profiles.create")
            };
            if components::primary_button(
                ui,
                &label,
                if compact { field_width } else { half },
                model.can_save(),
            )
            .clicked()
            {
                *request = Some(ProfilesRequest::Save);
            }
        });
    });
}
