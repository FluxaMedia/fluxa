use super::*;

pub fn compact_settings_content_top(metrics: UiMetrics, settings: &SettingsModel) -> f32 {
    let top = metrics.detail_header_top_mobile;
    if settings.section_open && settings.search.trim().is_empty() {
        top + 40.0 + metrics.section_gap
    } else {
        top + metrics.screen_title_size
            + metrics.section_gap
            + metrics.screen_control_height
            + metrics.section_gap
    }
}

fn compact_groups(viewport: Viewport) -> Vec<(&'static str, Vec<usize>)> {
    let mut groups: Vec<(&'static str, Vec<usize>)> = Vec::new();
    for index in categories(viewport) {
        let group = category_group(SETTINGS_SECTIONS[index].category);
        match groups.last_mut() {
            Some((name, items)) if *name == group => items.push(index),
            _ => groups.push((group, vec![index])),
        }
    }
    groups
}

pub fn compact_settings_list_height(viewport: Viewport, metrics: UiMetrics) -> f32 {
    let groups = compact_groups(viewport);
    let rows = groups.iter().map(|(_, items)| items.len()).sum::<usize>() + 1;
    rows as f32 * metrics.settings_compact_row
        + (groups.len() as f32 * metrics.settings_nav_group_height)
        + (groups.len() as f32 * metrics.settings_nav_group_gap)
        + metrics.section_gap
}

pub(super) fn draw_compact_settings_list(
    context: &egui::Context,
    viewport: Viewport,
    painter: &egui::Painter,
    assets: &impl HomeAssets,
    language: &str,
    metrics: UiMetrics,
    anchor: Rect,
    layout: &mut HomeLayout,
) {
    let mut groups: Vec<(Option<&'static str>, Vec<(u64, &str, String)>)> =
        compact_groups(viewport)
            .into_iter()
            .map(|(group, items)| {
                let rows = items
                    .into_iter()
                    .map(|index| {
                        let section = &SETTINGS_SECTIONS[index];
                        (
                            NODE_SETTINGS_SECTION_BASE + index as u64,
                            section.category,
                            section_label(section.category, language),
                        )
                    })
                    .collect();
                (Some(group), rows)
            })
            .collect();
    groups.push((
        None,
        vec![(
            NODE_SETTINGS_SWITCH_PROFILE,
            "Profile",
            localized("settings.switch_profiles", language),
        )],
    ));

    let row_height = metrics.settings_compact_row;
    let mut top = anchor.top();
    for (group, rows) in groups {
        if let Some(group) = group {
            nav_heading(
                painter,
                Pos2::new(
                    anchor.left() + 16.0,
                    top + metrics.settings_nav_group_height * 0.6,
                ),
                group,
                language,
                metrics,
            );
            top += metrics.settings_nav_group_height;
        } else {
            top += metrics.section_gap;
        }
        let card = Rect::from_min_size(
            Pos2::new(anchor.left(), top),
            Vec2::new(anchor.width(), rows.len() as f32 * row_height),
        );
        painter.rect_filled(card, metrics.card_radius, metrics.surface_raised);
        for (index, (node, icon, label)) in rows.into_iter().enumerate() {
            let rect = Rect::from_min_size(
                Pos2::new(anchor.left(), top + index as f32 * row_height),
                Vec2::new(anchor.width(), row_height),
            );
            if index > 0 {
                painter.line_segment(
                    [
                        Pos2::new(rect.left() + metrics.settings_compact_text_left, rect.top()),
                        rect.right_top(),
                    ],
                    egui::Stroke::new(stroke::hairline(), ink(tone::faint())),
                );
            }
            let mut clicked = false;
            components::placed(
                context,
                Id::new(("fluxa-settings-compact-row", node)),
                rect.min,
                |ui| {
                    let (row, response) = ui.allocate_exact_size(rect.size(), Sense::click());
                    if response.is_pointer_button_down_on() {
                        ui.painter()
                            .rect_filled(row, metrics.card_radius, ink(tone::faint()));
                    }
                    settings_category_icon(
                        assets,
                        ui.painter(),
                        row.left_center() + Vec2::new(16.0 + metrics.settings_nav_icon * 0.5, 0.0),
                        icon,
                        ink(tone::subtle()),
                        metrics.settings_nav_icon,
                    );
                    draw_text(
                        ui.painter(),
                        row.left_center() + Vec2::new(metrics.settings_compact_text_left, 0.0),
                        Align2::LEFT_CENTER,
                        label,
                        crate::fonts::medium(metrics.nav_label_size + 3.0),
                        text_primary(),
                        Some(row.width() - metrics.settings_compact_text_left - 44.0),
                    );
                    let tip = row.right_center() - Vec2::new(18.0, 0.0);
                    let stroke = egui::Stroke::new(stroke::icon(), ink(tone::muted()));
                    ui.painter()
                        .line_segment([tip + Vec2::new(-4.5, -5.5), tip], stroke);
                    ui.painter()
                        .line_segment([tip + Vec2::new(-4.5, 5.5), tip], stroke);
                    clicked = response.clicked();
                },
            );
            layout.focusable.push((node, rect));
            if clicked {
                layout.activated = Some(node);
            }
        }
        top += card.height() + metrics.settings_nav_group_gap;
    }
}
