use super::*;

pub(super) fn draw_select(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
) {
    let count = model.profiles.len() + 1;
    let (radius, cell, gap) = if viewport.is_compact() {
        let gap = 18.0;
        let cell_width = ((viewport.width - 32.0 - gap * 2.0) / 3.0).min(150.0);
        let radius = (cell_width * 0.43).min(65.0);
        (radius, Vec2::new(cell_width, radius * 2.0 + 64.0), gap)
    } else {
        (65.0, Vec2::new(150.0, 222.0), 40.0)
    };
    let side = metrics.profile_side_margin;
    let per_row =
        (((viewport.width - side + gap) / (cell.x + gap)).floor() as usize).clamp(1, count);
    let rows = count.div_ceil(per_row);
    let grid_height = rows as f32 * cell.y + (rows - 1) as f32 * gap;
    let content_height = 60.0 + 24.0 + 46.0 + 52.0 + grid_height;
    let top = ((viewport.height - content_height) * 0.5).max(48.0);
    ui.add_space(top);

    let width = ui.available_width();
    let (brand_rect, _) = ui.allocate_exact_size(Vec2::new(width, 60.0), Sense::hover());
    components::brand_lockup(ui.painter(), brand_rect.center(), 40.0, 32.0, 255, assets);
    ui.add_space(24.0);

    let title = ui.painter().layout_no_wrap(
        t("profiles.who_watching"),
        FontId::proportional(metrics.profile_title_size),
        Color32::WHITE,
    );
    let (title_rect, _) = ui.allocate_exact_size(Vec2::new(width, 46.0), Sense::hover());
    let title_pos = Pos2::new(
        title_rect.center().x - title.size().x * 0.5,
        title_rect.center().y - title.size().y * 0.5,
    );
    let title_right = title_pos.x + title.size().x;
    ui.painter().galley(title_pos, title, Color32::WHITE);
    let gear_rect = Rect::from_center_size(
        Pos2::new(title_right + 38.0, title_rect.center().y),
        Vec2::splat(44.0),
    );
    let gear = ui
        .interact(gear_rect, Id::new("fluxa-profiles-gear"), Sense::click())
        .on_hover_text(t("profiles.picker_settings"));
    if let Some(icon) = assets.icon("Settings") {
        ui.painter().image(
            icon,
            Rect::from_center_size(gear_rect.center(), Vec2::splat(26.0)),
            full_uv(),
            Color32::from_white_alpha(if gear.hovered() { 255 } else { 180 }),
        );
    }
    if gear.clicked() {
        model.background_input = model.picker.background_url.clone().unwrap_or_default();
        model.mode = ProfilesMode::Settings;
    }
    ui.add_space(52.0);

    let (grid_rect, _) = ui.allocate_exact_size(Vec2::new(width, grid_height), Sense::hover());
    let profiles = model.profiles.clone();
    for index in 0..count {
        let row = index / per_row;
        let column = index % per_row;
        let in_row = (count - row * per_row).min(per_row);
        let row_width = in_row as f32 * cell.x + (in_row - 1) as f32 * gap;
        let origin = Pos2::new(
            grid_rect.center().x - row_width * 0.5 + column as f32 * (cell.x + gap),
            grid_rect.top() + row as f32 * (cell.y + gap),
        );
        let avatar_center = Pos2::new(origin.x + cell.x * 0.5, origin.y + radius);
        let hit = Rect::from_center_size(avatar_center, Vec2::splat(radius * 2.0)).union(
            Rect::from_min_size(
                Pos2::new(origin.x, avatar_center.y + radius),
                Vec2::new(cell.x, 40.0),
            ),
        );
        let Some(profile) = profiles.get(index) else {
            let response = ui.interact(hit, Id::new("fluxa-profiles-add"), Sense::click());
            let r = if response.hovered() {
                radius * 1.04
            } else {
                radius
            };
            ui.painter().circle(
                avatar_center,
                r,
                Color32::from_white_alpha(if response.hovered() { 26 } else { 10 }),
                egui::Stroke::new(1.0, metrics.border),
            );
            if let Some(icon) = assets.icon("Plus") {
                ui.painter().image(
                    icon,
                    Rect::from_center_size(avatar_center, Vec2::splat(36.0)),
                    full_uv(),
                    Color32::from_white_alpha(140),
                );
            }
            ui.painter().text(
                Pos2::new(avatar_center.x, avatar_center.y + radius + 24.0),
                Align2::CENTER_CENTER,
                t("profiles.add_profile"),
                FontId::proportional(14.0),
                metrics.text_muted,
            );
            if response.clicked() {
                model.open_form(None);
            }
            continue;
        };
        let response = ui.interact(hit, Id::new(("fluxa-profile", &profile.id)), Sense::click());
        let card_hovered = ui.rect_contains_pointer(Rect::from_min_size(origin, cell));
        let r = if response.hovered() {
            radius * 1.04
        } else {
            radius
        };
        components::avatar(
            ui.painter(),
            assets,
            avatar_center,
            r,
            &profile.name,
            profile.avatar_url.as_deref(),
            metrics,
        );
        if !response.hovered() {
            ui.painter()
                .circle_filled(avatar_center, r, Color32::from_black_alpha(30));
        }
        if profile.locked {
            let badge = avatar_center + Vec2::splat(radius * 0.72);
            ui.painter().circle(
                badge,
                17.0,
                Color32::from_rgba_unmultiplied(12, 12, 12, 235),
                egui::Stroke::new(1.0, Color32::from_white_alpha(38)),
            );
            if let Some(icon) = assets.icon("Lock") {
                ui.painter().image(
                    icon,
                    Rect::from_center_size(badge, Vec2::splat(16.0)),
                    full_uv(),
                    Color32::from_white_alpha(220),
                );
            }
        }
        let name_font = FontId::proportional(16.0);
        let name = truncate_to_width(ui.painter(), &profile.name, &name_font, cell.x);
        let name_y = avatar_center.y + radius + 24.0;
        ui.painter().text(
            Pos2::new(avatar_center.x, name_y),
            Align2::CENTER_CENTER,
            name,
            name_font,
            Color32::from_white_alpha(if response.hovered() { 255 } else { 215 }),
        );
        let mut actions_y = name_y + 24.0;
        if model.primary_id.as_ref() == Some(&profile.id) {
            let label = t("profiles.primary_badge").to_uppercase();
            let galley = ui.painter().layout_no_wrap(
                label,
                FontId::proportional(10.0),
                metrics.text_secondary,
            );
            let badge = Rect::from_center_size(
                Pos2::new(avatar_center.x, name_y + 22.0),
                galley.size() + Vec2::new(14.0, 6.0),
            );
            ui.painter()
                .rect_filled(badge, 4.0, Color32::from_white_alpha(24));
            ui.painter()
                .galley(badge.center() - galley.size() * 0.5, galley, Color32::WHITE);
            actions_y += 20.0;
        }
        let alpha = if card_hovered { 230 } else { 120 };
        for (offset, icon, label, is_delete) in [
            (-16.0, "Edit", t("profiles.edit"), false),
            (16.0, "Delete", t("profiles.delete"), true),
        ] {
            let rect = Rect::from_center_size(
                Pos2::new(avatar_center.x + offset, actions_y + 6.0),
                Vec2::splat(28.0),
            );
            let action = ui
                .interact(
                    rect,
                    Id::new(("fluxa-profile-action", &profile.id, is_delete)),
                    Sense::click(),
                )
                .on_hover_text(label);
            if action.hovered() {
                ui.painter()
                    .rect_filled(rect, 6.0, Color32::from_white_alpha(18));
            }
            let color = Color32::from_white_alpha(if action.hovered() { 255 } else { alpha });
            if let Some(texture) = assets.icon(icon) {
                ui.painter().image(
                    texture,
                    Rect::from_center_size(rect.center(), Vec2::splat(16.0)),
                    full_uv(),
                    color,
                );
            }
            if action.clicked() {
                let purpose = if is_delete {
                    PinPurpose::Delete
                } else {
                    PinPurpose::Edit
                };
                if profile.locked {
                    model.pin_prompt = Some(PinPrompt {
                        profile_id: profile.id.clone(),
                        purpose,
                        pin: String::new(),
                        error: false,
                    });
                } else if is_delete {
                    model.confirm_delete = Some(profile.id.clone());
                } else {
                    model.open_form(Some(profile));
                }
            }
        }
        if response.clicked() {
            if profile.locked {
                model.pin_prompt = Some(PinPrompt {
                    profile_id: profile.id.clone(),
                    purpose: PinPurpose::Enter,
                    pin: String::new(),
                    error: false,
                });
            } else {
                *request = Some(ProfilesRequest::Select(profile.id.clone()));
            }
        }
    }
    ui.add_space(48.0);
}
