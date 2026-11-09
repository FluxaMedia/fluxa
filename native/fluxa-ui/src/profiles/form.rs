use super::*;
use crate::kit::{self, Modifier, Role, Type, UiKit};

pub(super) fn draw_form(
    ui: &mut crate::gfx::Ui,
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
    let width = metrics.profile_page_width(viewport.width, metrics.profile_form_max_width);
    let layout = page(ui, width);
    kit::column(ui, layout, |ui| {
        if !compact {
            components::heading(
                ui,
                &t("profiles.settings"),
                &title,
                &t("profiles.form_subtitle"),
                metrics,
            );
        }
        form_details(ui, model, assets, t, request, compact);
        kit::space(ui, Space::S48);
    });
}

pub(super) fn form_details(
    ui: &mut crate::gfx::Ui,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
    compact: bool,
) {
    kit::section(ui, compact, |ui| {
        kit::center(ui, |ui| {
            let name = if model.draft.name.trim().is_empty() {
                t("auto.profile")
            } else {
                model.draft.name.trim().to_owned()
            };
            let avatar = kit::avatar_button(
                ui,
                assets,
                64.0,
                &name,
                model.draft.avatar_url.as_deref(),
                &t("profiles.change_image"),
            );
            if avatar.on_hover_text(t("profiles.choose_icon")).clicked() {
                model.mode = ProfilesMode::Avatars;
            }
            kit::space(ui, Space::S16);
            kit::text(name, Type::Subtitle).bold().show(ui);
            if model.draft_is_primary() {
                kit::text(t("profiles.primary_badge").to_uppercase(), Type::Label)
                    .role(Role::Secondary)
                    .badge()
                    .show(ui);
            }
            kit::space(ui, Space::S12);
            if kit::pill(ui, &t("profiles.choose_icon")).clicked() {
                model.mode = ProfilesMode::Avatars;
            }
        });
        kit::space(ui, Space::S20);
        kit::field(ui, &t("profiles.name"));
        kit::input(&mut model.draft.name, &t("profiles.name_placeholder")).show(ui);
        if model.duplicate_name() {
            kit::note(ui, &t("profiles.duplicate_name"));
        }
        kit::space(ui, Space::S12);
        kit::field(ui, &t("profiles.image_url"));
        let url_field = kit::input(
            &mut model.avatar_input,
            &t("profiles.image_url_placeholder"),
        )
        .kind(components::InputKind::Url)
        .show(ui);
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
        kit::space(ui, Space::S12);
        kit::field(ui, &t("profiles.pin_lock"));
        let hint = if model.draft.has_pin && !model.draft.remove_pin {
            t("profiles.pin_set_placeholder")
        } else {
            t("profiles.pin_placeholder")
        };
        let pin = kit::input(&mut model.draft.pin, &hint)
            .kind(components::InputKind::Number { secret: true })
            .show(ui);
        if pin.changed() {
            model
                .draft
                .pin
                .retain(|character| character.is_ascii_digit());
            model.draft.pin.truncate(4);
            model.draft.remove_pin = false;
        }
        if !model.draft.pin.is_empty() && model.draft.pin.len() != 4 {
            kit::note(ui, &t("profiles.pin_invalid"));
        }
        if model.draft.has_pin && !model.draft.remove_pin {
            if kit::link(ui, &t("profiles.remove_pin")).clicked() {
                model.draft.remove_pin = true;
                model.draft.pin.clear();
            }
        }
        if model.draft.remove_pin {
            kit::note(ui, &t("profiles.pin_will_be_removed"));
        }
        if !model.draft_is_primary() {
            kit::space(ui, Space::S12);
            kit::field(ui, &t("profiles.sharing"));
            kit::checkbox(
                ui,
                &mut model.draft.uses_primary_addons,
                &t("profiles.use_primary_addons"),
            );
            kit::checkbox(
                ui,
                &mut model.draft.uses_primary_plugins,
                &t("profiles.use_primary_plugins"),
            );
        }
        kit::space(ui, Space::S20);
        let full = ui.available_width();
        let half = (full - ui.tokens().space_px(Space::S8)) * 0.5;
        kit::row(ui, Modifier::new().gap(Space::S8), |ui| {
            if !compact
                && kit::button(&t("common.cancel"))
                    .width(half)
                    .show(ui)
                    .clicked()
            {
                model.mode = ProfilesMode::Select;
            }
            let label = if model.busy {
                t("common.saving")
            } else if model.draft.editing.is_some() {
                t("profiles.save")
            } else {
                t("profiles.create")
            };
            let save = kit::button(&label)
                .primary()
                .enabled(model.can_save())
                .width(if compact { full } else { half })
                .show(ui);
            if save.clicked() {
                *request = Some(ProfilesRequest::Save);
            }
        });
    });
}
