use super::*;
use crate::kit::{self, Role, Type};

pub(super) fn draw_avatars(
    ui: &mut crate::gfx::Ui,
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
    let heading = (
        t("profiles.settings"),
        t("profiles.choose_icon"),
        t("profiles.choose_icon_desc"),
    );
    let empty_label = t("profiles.no_avatar_packs");
    let packs = model.all_packs();
    let selected = model.draft.avatar_url.clone();
    let assets = std::cell::RefCell::new(assets);
    let chosen = std::cell::RefCell::new(None);
    kit::column(ui, layout, |ui| {
        let mut items = Vec::new();
        if !compact {
            items.push(kit::widget(ui, |ui| {
                components::heading(ui, &heading.0, &heading.1, &heading.2, metrics);
            }));
        }
        if packs.is_empty() {
            items.push(kit::widget(ui, |ui| {
                kit::text(empty_label.clone(), Type::Meta)
                    .role(Role::Muted)
                    .show(ui);
            }));
        }
        for pack in &packs {
            items.push(kit::layout_items::space(ui, Space::S12));
            items.push(kit::widget(ui, |ui| {
                kit::row(ui, inline(ui), |ui| {
                    kit::text(pack.title.as_str(), Type::Title).show(ui);
                    kit::text(pack.avatars.len().to_string(), Type::Meta)
                        .role(Role::Muted)
                        .show(ui);
                });
            }));
            items.push(kit::layout_items::space(ui, Space::S10));
            items.push(kit::widget(ui, |ui| {
                let mut assets = assets.borrow_mut();
                if let Some(index) = kit::avatar_grid(
                    ui,
                    &mut **assets,
                    &pack.id,
                    &pack.avatars,
                    selected.as_deref(),
                ) {
                    *chosen.borrow_mut() = Some(pack.avatars[index].url.clone());
                }
            }));
        }
        items.push(kit::layout_items::space(ui, Space::S48));
        kit::lay(ui, items);
    });
    if let Some(url) = chosen.into_inner() {
        model.draft.avatar_url = Some(url);
        model.mode = ProfilesMode::Form;
    }
}
