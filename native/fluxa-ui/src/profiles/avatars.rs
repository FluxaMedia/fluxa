use super::*;

pub(super) fn draw_avatars(
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
        &t("profiles.choose_icon"),
        t,
    );
    let width = if compact {
        viewport.width - 32.0
    } else {
        (viewport.width - 56.0).min(1100.0)
    };
    centered_column(ui, width, |ui| {
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
            ui.label(
                RichText::new(t("profiles.no_avatar_packs"))
                    .size(metrics.text.meta)
                    .color(metrics.text_muted),
            );
        }
        let tile = if compact { 64.0 } else { 84.0 };
        let gap = 14.0;
        let step = tile + gap;
        let label_height = 22.0;
        for pack in packs {
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&pack.title)
                        .size(metrics.text.title)
                        .strong()
                        .color(metrics.text_primary),
                );
                ui.label(
                    RichText::new(pack.avatars.len().to_string())
                        .size(metrics.text.meta)
                        .color(metrics.text_muted),
                );
            });
            ui.add_space(10.0);
            let columns = (((width + gap) / step).floor() as usize).max(1);
            let rows = pack.avatars.len().div_ceil(columns);
            let (grid, _) = ui.allocate_exact_size(
                Vec2::new(width, rows as f32 * (tile + label_height + 8.0)),
                Sense::hover(),
            );
            for (index, avatar) in pack.avatars.iter().enumerate() {
                let rect = Rect::from_min_size(
                    Pos2::new(
                        grid.left() + (index % columns) as f32 * step,
                        grid.top() + (index / columns) as f32 * (tile + label_height + 8.0),
                    ),
                    Vec2::new(tile, tile + label_height),
                );
                let response = ui.interact(
                    rect,
                    Id::new(("fluxa-avatar", &pack.id, index)),
                    Sense::click(),
                );
                let center = Pos2::new(rect.center().x, rect.top() + tile * 0.5);
                components::avatar(
                    ui.painter(),
                    assets,
                    center,
                    tile * 0.5,
                    &avatar.name,
                    Some(&avatar.url),
                    metrics,
                );
                let selected = model.draft.avatar_url.as_deref() == Some(avatar.url.as_str());
                if selected || response.hovered() {
                    ui.painter().circle_stroke(
                        center,
                        tile * 0.5 + 2.0,
                        egui::Stroke::new(
                            stroke::emphasis(),
                            ink(if selected { 255 } else { 90 }),
                        ),
                    );
                }
                draw_text(
                    ui.painter(),
                    Pos2::new(center.x, rect.bottom() - 8.0),
                    Align2::CENTER_CENTER,
                    &avatar.name,
                    FontId::proportional(metrics.text.label),
                    metrics.text_secondary,
                    Some(step - 4.0),
                );
                if response.clicked() {
                    model.draft.avatar_url = Some(avatar.url.clone());
                    model.mode = ProfilesMode::Form;
                }
            }
        }
        ui.add_space(48.0);
    });
}
