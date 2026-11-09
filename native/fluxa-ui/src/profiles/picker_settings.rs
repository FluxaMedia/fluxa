use super::*;
use crate::components::{InputKind, PackAction};
use crate::kit::{self, Modifier, Role, Type};

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
            kit::text(t("profiles.picker_background"), Type::Title).show(ui);
            kit::space(ui, Space::S16);
            let has_background = model.picker.background_url.is_some();
            let buttons = if has_background { 2.0 } else { 1.0 };
            if compact {
                kit::text(t("profiles.picker_background_desc"), Type::Body)
                    .role(Role::Muted)
                    .show(ui);
                kit::space(ui, Space::S12);
            }
            kit::row(ui, fields(compact), |ui| {
                let field_width = if compact {
                    ui.available_width()
                } else {
                    ui.available_width() - buttons * 54.0
                };
                let field = kit::input(
                    &mut model.background_input,
                    &t("profiles.picker_background_placeholder"),
                )
                .kind(InputKind::Url)
                .width(field_width)
                .show(ui);
                if field.lost_focus() {
                    *request = Some(ProfilesRequest::SaveBackground);
                }
                let pick = kit::icon_button("ImagePlus").framed().show(ui, assets);
                if pick.on_hover_text(t("profiles.choose_image")).clicked() {
                    *request = Some(ProfilesRequest::PickBackgroundImage);
                }
                if has_background {
                    let clear = kit::icon_button("Delete").framed().show(ui, assets);
                    if clear
                        .on_hover_text(t("profiles.clear_background"))
                        .clicked()
                    {
                        model.background_input.clear();
                        *request = Some(ProfilesRequest::SaveBackground);
                    }
                }
            });
        });
        kit::space(ui, Space::S20);
        kit::section(ui, compact, |ui| {
            kit::text(t("profiles.avatar_packs"), Type::Title).show(ui);
            kit::space(ui, Space::S6);
            kit::text(t("profiles.avatar_packs_desc"), Type::Body)
                .role(Role::Muted)
                .show(ui);
            kit::space(ui, Space::S16);
            kit::row(ui, fields(compact), |ui| {
                let field_width = if compact {
                    ui.available_width()
                } else {
                    ui.available_width() - 150.0
                };
                let field = kit::input(
                    &mut model.repository_input,
                    &t("profiles.avatar_pack_repository_placeholder"),
                )
                .kind(InputKind::Url)
                .width(field_width)
                .show(ui);
                let submit =
                    field.lost_focus() && ui.input(|input| input.key_pressed(crate::gfx::Key::Enter));
                let label = if model.busy {
                    t("common.loading")
                } else {
                    t("profiles.add_pack")
                };
                let can_add = !model.repository_input.trim().is_empty() && !model.busy;
                let button_width = if compact { ui.available_width() } else { 140.0 };
                let add = kit::button(&label)
                    .primary()
                    .enabled(can_add)
                    .width(button_width)
                    .show(ui);
                if (add.clicked() || submit) && can_add {
                    *request = Some(ProfilesRequest::AddRepository);
                }
            });
            kit::space(ui, Space::S20);
            if model.picker.avatar_packs.is_empty() {
                kit::text(t("profiles.no_avatar_packs"), Type::Body)
                    .role(Role::Muted)
                    .show(ui);
            }
            let tips = (t("profiles.remove_pack"), t("profiles.refresh_pack"));
            for pack in model.picker.avatar_packs.clone() {
                match kit::pack_row(ui, assets, compact, &pack, model.busy, (&tips.0, &tips.1)) {
                    Some(PackAction::Remove) => {
                        *request = Some(ProfilesRequest::RemovePack(pack.id.clone()));
                    }
                    Some(PackAction::Refresh) => {
                        *request = Some(ProfilesRequest::RefreshRepository(
                            pack.repository_url.clone(),
                        ));
                    }
                    None => {}
                }
                kit::space(ui, Space::S10);
            }
        });
        kit::space(ui, Space::S16);
        if !compact && kit::back_link(ui, &t("common.back")).clicked() {
            model.mode = ProfilesMode::Select;
        }
        kit::space(ui, Space::S48);
    });
}
