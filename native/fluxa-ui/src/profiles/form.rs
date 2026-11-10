use super::*;
use crate::flex::Align;
use crate::kit::{self, Role, Type, UiKit, layout_items};
use std::cell::{Cell, RefCell};

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
        kit::lay(ui, vec![layout_items::space(ui, Space::S48)]);
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
        let name = if model.draft.name.trim().is_empty() {
            t("auto.profile")
        } else {
            model.draft.name.trim().to_owned()
        };
        let primary = model.draft_is_primary();
        let duplicate = model.duplicate_name();
        let pin_invalid = !model.draft.pin.is_empty() && model.draft.pin.len() != 4;
        let pin_set = model.draft.has_pin && !model.draft.remove_pin;
        let pin_removed = model.draft.remove_pin;
        let hint = if pin_set {
            t("profiles.pin_set_placeholder")
        } else {
            t("profiles.pin_placeholder")
        };
        let save_label = if model.busy {
            t("common.saving")
        } else if model.draft.editing.is_some() {
            t("profiles.save")
        } else {
            t("profiles.create")
        };
        let change_image = t("profiles.change_image");
        let choose_icon = t("profiles.choose_icon");
        let badge = t("profiles.primary_badge").to_uppercase();
        let name_label = t("profiles.name");
        let name_hint = t("profiles.name_placeholder");
        let duplicate_note = t("profiles.duplicate_name");
        let url_label = t("profiles.image_url");
        let url_hint = t("profiles.image_url_placeholder");
        let pin_label = t("profiles.pin_lock");
        let pin_note = t("profiles.pin_invalid");
        let remove_pin = t("profiles.remove_pin");
        let removed_note = t("profiles.pin_will_be_removed");
        let sharing = t("profiles.sharing");
        let use_addons = t("profiles.use_primary_addons");
        let use_plugins = t("profiles.use_primary_plugins");
        let cancel = t("common.cancel");
        let full = ui.available_width();
        let gap = ui.tokens().space_px(Space::S8);
        let half = (full - gap) * 0.5;
        let save_width = if compact { full } else { half };
        let model = RefCell::new(model);
        let assets = RefCell::new(assets);
        let saved = Cell::new(false);
        let mut items = vec![
            kit::widget(ui, |ui| {
                let mut assets = assets.borrow_mut();
                let avatar = kit::avatar_button(
                    ui,
                    &mut **assets,
                    64.0,
                    &name,
                    model.borrow().draft.avatar_url.as_deref(),
                    &change_image,
                );
                if avatar.on_hover_text(choose_icon.clone()).clicked() {
                    model.borrow_mut().mode = ProfilesMode::Avatars;
                }
            })
            .align_self(Align::Center),
            layout_items::space(ui, Space::S16),
            kit::widget(ui, |ui| {
                kit::text(name.clone(), Type::Subtitle).bold().show(ui);
            })
            .align_self(Align::Center),
        ];
        if primary {
            items.push(
                kit::widget(ui, |ui| {
                    kit::text(badge.clone(), Type::Label)
                        .role(Role::Secondary)
                        .badge()
                        .show(ui);
                })
                .align_self(Align::Center),
            );
        }
        items.push(layout_items::space(ui, Space::S12));
        items.push(
            kit::widget(ui, |ui| {
                if kit::pill(ui, &choose_icon).clicked() {
                    model.borrow_mut().mode = ProfilesMode::Avatars;
                }
            })
            .align_self(Align::Center),
        );
        items.push(layout_items::space(ui, Space::S20));
        items.push(kit::widget(ui, |ui| kit::field(ui, &name_label)));
        items.push(kit::widget(ui, |ui| {
            let mut model = model.borrow_mut();
            kit::input(&mut model.draft.name, &name_hint).show(ui);
        }));
        if duplicate {
            items.push(kit::widget(ui, |ui| kit::note(ui, &duplicate_note)));
        }
        items.push(layout_items::space(ui, Space::S12));
        items.push(kit::widget(ui, |ui| kit::field(ui, &url_label)));
        items.push(kit::widget(ui, |ui| {
            let mut model = model.borrow_mut();
            let url_field = kit::input(&mut model.avatar_input, &url_hint)
                .kind(components::InputKind::Url)
                .show(ui);
            if url_field.changed() {
                let url = model.avatar_input.trim().to_owned();
                model.draft.avatar_url = (!url.is_empty()).then_some(url);
            } else if !url_field.has_focus() {
                model.avatar_input = model
                    .draft
                    .avatar_url
                    .clone()
                    .filter(|url| url.starts_with("http"))
                    .unwrap_or_default();
            }
        }));
        items.push(layout_items::space(ui, Space::S12));
        items.push(kit::widget(ui, |ui| kit::field(ui, &pin_label)));
        items.push(kit::widget(ui, |ui| {
            let mut model = model.borrow_mut();
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
        }));
        if pin_invalid {
            items.push(kit::widget(ui, |ui| kit::note(ui, &pin_note)));
        }
        if pin_set {
            items.push(kit::widget(ui, |ui| {
                if kit::link(ui, &remove_pin).clicked() {
                    let mut model = model.borrow_mut();
                    model.draft.remove_pin = true;
                    model.draft.pin.clear();
                }
            }));
        }
        if pin_removed {
            items.push(kit::widget(ui, |ui| kit::note(ui, &removed_note)));
        }
        if !primary {
            items.push(layout_items::space(ui, Space::S12));
            items.push(kit::widget(ui, |ui| kit::field(ui, &sharing)));
            items.push(kit::widget(ui, |ui| {
                let mut model = model.borrow_mut();
                kit::checkbox(ui, &mut model.draft.uses_primary_addons, &use_addons);
            }));
            items.push(kit::widget(ui, |ui| {
                let mut model = model.borrow_mut();
                kit::checkbox(ui, &mut model.draft.uses_primary_plugins, &use_plugins);
            }));
        }
        items.push(layout_items::space(ui, Space::S20));
        let mut buttons = Vec::new();
        if !compact {
            buttons.push(kit::widget(ui, |ui| {
                if kit::button(&cancel).width(half).show(ui).clicked() {
                    model.borrow_mut().mode = ProfilesMode::Select;
                }
            }));
        }
        buttons.push(kit::widget(ui, |ui| {
            let save = kit::button(&save_label)
                .primary()
                .enabled(model.borrow().can_save())
                .width(save_width)
                .show(ui);
            if save.clicked() {
                saved.set(true);
            }
        }));
        items.push(layout_items::row(buttons).gap(gap));
        kit::lay(ui, items);
        if saved.get() {
            *request = Some(ProfilesRequest::Save);
        }
    });
}
