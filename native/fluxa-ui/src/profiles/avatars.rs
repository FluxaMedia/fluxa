use super::*;
use crate::kit::{self, Role, Type};

pub(super) fn draw_avatars(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
) {
    let compact = viewport.is_compact();
    profile_header(
        ui,
        model,
        assets,
        metrics,
        compact,
        &t("profiles.choose_icon"),
        t,
    );
    let width = metrics.profile_page_width(viewport.width, metrics.profile_avatars_max_width);
    let layout = page(ui, width);
    kit::column(ui, layout, |ui| {
        if !compact {
            components::heading(
                ui,
                &t("profiles.settings"),
                &t("profiles.choose_icon"),
                &t("profiles.choose_icon_desc"),
                metrics,
            );
        }
        let packs = model.all_packs();
        if packs.is_empty() {
            kit::text(t("profiles.no_avatar_packs"), Type::Meta)
                .role(Role::Muted)
                .show(ui);
        }
        for pack in packs {
            kit::space(ui, Space::S12);
            kit::row(ui, inline(ui), |ui| {
                kit::text(pack.title.as_str(), Type::Title).show(ui);
                kit::text(pack.avatars.len().to_string(), Type::Meta)
                    .role(Role::Muted)
                    .show(ui);
            });
            kit::space(ui, Space::S10);
            let selected = model.draft.avatar_url.clone();
            if let Some(index) =
                kit::avatar_grid(ui, assets, &pack.id, &pack.avatars, selected.as_deref())
            {
                model.draft.avatar_url = Some(pack.avatars[index].url.clone());
                model.mode = ProfilesMode::Form;
            }
        }
        kit::space(ui, Space::S48);
    });
}
