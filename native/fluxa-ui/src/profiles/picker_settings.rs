use super::*;
use crate::components::{InputKind, PackAction};
use crate::kit::{self, Modifier, Role, Type, layout_items};
use std::cell::{Cell, RefCell};

fn fields(compact: bool) -> Modifier {
    let row = Modifier::new().gap(Space::S10);
    if compact { row.wrap() } else { row }
}

pub(super) fn draw_picker_settings(
    ui: &mut crate::gfx::Ui,
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
    let width = metrics.profile_page_width(viewport.width, metrics.profile_picker_max_width);
    let layout = page(ui, width);
    kit::column(ui, layout, |ui| {
        if !compact {
            components::heading(
                ui,
                &t("profiles.settings"),
                &t("profiles.picker_settings"),
                &t("profiles.picker_background_desc"),
                metrics,
            );
        }
        kit::section(ui, compact, |ui| {
            let has_background = model.picker.background_url.is_some();
            let buttons = if has_background { 2.0 } else { 1.0 };
            let title = t("profiles.picker_background");
            let desc = t("profiles.picker_background_desc");
            let placeholder = t("profiles.picker_background_placeholder");
            let choose = t("profiles.choose_image");
            let clear_tip = t("profiles.clear_background");
            let mut items = vec![
                kit::widget(ui, |ui| {
                    kit::text(title.clone(), Type::Title).show(ui);
                }),
                layout_items::space(ui, Space::S16),
            ];
            if compact {
                items.push(kit::widget(ui, |ui| {
                    kit::text(desc.clone(), Type::Body)
                        .role(Role::Muted)
                        .show(ui);
                }));
                items.push(layout_items::space(ui, Space::S12));
            }
            items.push(kit::widget(ui, |ui| {
                kit::row(ui, fields(compact), |ui| {
                    let field_width = if compact {
                        ui.available_width()
                    } else {
                        ui.available_width() - buttons * 54.0
                    };
                    let field = kit::input(&mut model.background_input, &placeholder)
                        .kind(InputKind::Url)
                        .width(field_width)
                        .show(ui);
                    if field.lost_focus() {
                        *request = Some(ProfilesRequest::SaveBackground);
                    }
                    let pick = kit::icon_button("ImagePlus").framed().show(ui, assets);
                    if pick.on_hover_text(choose.clone()).clicked() {
                        *request = Some(ProfilesRequest::PickBackgroundImage);
                    }
                    if has_background {
                        let clear = kit::icon_button("Delete").framed().show(ui, assets);
                        if clear.on_hover_text(clear_tip.clone()).clicked() {
                            model.background_input.clear();
                            *request = Some(ProfilesRequest::SaveBackground);
                        }
                    }
                });
            }));
            kit::lay(ui, items);
        });
        kit::lay(ui, vec![layout_items::space(ui, Space::S20)]);
        kit::section(ui, compact, |ui| {
            let busy = model.busy;
            let packs = model.picker.avatar_packs.clone();
            let title = t("profiles.avatar_packs");
            let desc = t("profiles.avatar_packs_desc");
            let placeholder = t("profiles.avatar_pack_repository_placeholder");
            let none = t("profiles.no_avatar_packs");
            let tips = (t("profiles.remove_pack"), t("profiles.refresh_pack"));
            let label = if busy {
                t("common.loading")
            } else {
                t("profiles.add_pack")
            };
            let assets = RefCell::new(assets);
            let request = RefCell::new(&mut *request);
            let mut items = vec![
                kit::widget(ui, |ui| {
                    kit::text(title.clone(), Type::Title).show(ui);
                }),
                layout_items::space(ui, Space::S6),
                kit::widget(ui, |ui| {
                    kit::text(desc.clone(), Type::Body)
                        .role(Role::Muted)
                        .show(ui);
                }),
                layout_items::space(ui, Space::S16),
                kit::widget(ui, |ui| {
                    kit::row(ui, fields(compact), |ui| {
                        let field_width = if compact {
                            ui.available_width()
                        } else {
                            ui.available_width() - 150.0
                        };
                        let field = kit::input(&mut model.repository_input, &placeholder)
                            .kind(InputKind::Url)
                            .width(field_width)
                            .show(ui);
                        let submit = field.lost_focus()
                            && ui.input(|input| input.key_pressed(crate::gfx::Key::Enter));
                        let can_add = !model.repository_input.trim().is_empty() && !busy;
                        let button_width = if compact { ui.available_width() } else { 140.0 };
                        let add = kit::button(&label)
                            .primary()
                            .enabled(can_add)
                            .width(button_width)
                            .show(ui);
                        if (add.clicked() || submit) && can_add {
                            **request.borrow_mut() = Some(ProfilesRequest::AddRepository);
                        }
                    });
                }),
                layout_items::space(ui, Space::S20),
            ];
            if packs.is_empty() {
                items.push(kit::widget(ui, |ui| {
                    kit::text(none.clone(), Type::Body)
                        .role(Role::Muted)
                        .show(ui);
                }));
            }
            for pack in &packs {
                items.push(kit::widget(ui, |ui| {
                    let mut assets = assets.borrow_mut();
                    match kit::pack_row(ui, &mut **assets, compact, pack, busy, (&tips.0, &tips.1))
                    {
                        Some(PackAction::Remove) => {
                            **request.borrow_mut() =
                                Some(ProfilesRequest::RemovePack(pack.id.clone()));
                        }
                        Some(PackAction::Refresh) => {
                            **request.borrow_mut() = Some(ProfilesRequest::RefreshRepository(
                                pack.repository_url.clone(),
                            ));
                        }
                        None => {}
                    }
                }));
                items.push(layout_items::space(ui, Space::S10));
            }
            kit::lay(ui, items);
        });
        let back = t("common.back");
        let went_back = Cell::new(false);
        let mut tail = vec![layout_items::space(ui, Space::S16)];
        if !compact {
            tail.push(kit::widget(ui, |ui| {
                if kit::back_link(ui, &back).clicked() {
                    went_back.set(true);
                }
            }));
        }
        tail.push(layout_items::space(ui, Space::S48));
        kit::lay(ui, tail);
        if went_back.get() {
            model.mode = ProfilesMode::Select;
        }
    });
}
