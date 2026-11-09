use super::*;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets, Justify};

const ICON: usize = 1;
const LABEL: usize = 2;
const TRAIL: usize = 3;

pub fn compact_settings_content_top(metrics: UiMetrics, settings: &SettingsModel) -> f32 {
    let top = metrics.detail_header_top_mobile;
    if settings.section_open && settings.search.trim().is_empty() {
        top + Dp::D40.px() + metrics.section_gap
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
    let mut children = Vec::new();
    let mut ids = Vec::new();
    let mut next = 0;
    for (group, rows) in &groups {
        let heading = next;
        next += 1;
        children.push(match group {
            Some(_) => flex::leaf(Vec2::ZERO)
                .width(anchor.width())
                .height(metrics.settings_nav_group_height)
                .id(heading),
            None => flex::leaf(Vec2::ZERO).height(metrics.section_gap),
        });
        let card = next;
        next += 1;
        let first = next;
        next += rows.len();
        children.push(
            flex::column(
                (0..rows.len())
                    .map(|row| flex::leaf(Vec2::ZERO).height(row_height).id(first + row)),
            )
            .width(anchor.width())
            .id(card),
        );
        children.push(flex::leaf(Vec2::ZERO).height(metrics.settings_nav_group_gap));
        ids.push((heading, card, first));
    }
    let placed = flex::column(children).solve(Rect::from_min_size(
        anchor.min,
        Vec2::new(
            anchor.width(),
            compact_settings_list_height(viewport, metrics) + 1.0,
        ),
    ));
    for ((group, rows), (heading, card, first)) in groups.into_iter().zip(ids) {
        if let Some(group) = group {
            if let Some(band) = placed.find(heading) {
                nav_heading(
                    painter,
                    Rect::from_min_max(
                        Pos2::new(band.left() + Dp::D16.px(), band.top()),
                        band.right_bottom(),
                    ),
                    group,
                    language,
                    metrics,
                );
            }
        }
        let card = placed.find(card).unwrap_or(anchor);
        painter.rect_filled(card, metrics.card_radius, metrics.surface_raised);
        for (index, (node, icon, label)) in rows.into_iter().enumerate() {
            let rect = placed.find(first + index).unwrap_or(card);
            if index > 0 {
                painter.line_segment(
                    [
                        Pos2::new(rect.left() + metrics.settings_compact_text_left, rect.top()),
                        rect.right_top(),
                    ],
                    egui::Stroke::new(stroke::hairline(), crate::stylesheet::color("outline")),
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
                        ui.painter().rect_filled(
                            row,
                            metrics.card_radius,
                            crate::stylesheet::color("well"),
                        );
                    }
                    let line = flex::row([
                        flex::leaf(Vec2::splat(metrics.settings_nav_icon)).id(ICON),
                        flex::leaf(Vec2::new(0.0, row_height)).grow(1.0).id(LABEL),
                        flex::row([flex::leaf(Vec2::new(
                            metrics.settings_compact_chevron_width,
                            metrics.settings_compact_chevron_height * 2.0,
                        ))
                        .id(TRAIL)])
                        .justify(Justify::End)
                        .align(Align::Center)
                        .padding(Insets {
                            right: Dp::D16.px(),
                            ..Insets::default()
                        })
                        .width(Dp::D44.px()),
                    ])
                    .align(Align::Center)
                    .gap(
                        metrics.settings_compact_text_left
                            - Dp::D16.px()
                            - metrics.settings_nav_icon,
                    )
                    .padding(Insets {
                        left: Dp::D16.px(),
                        ..Insets::default()
                    })
                    .solve(row);
                    if let Some(slot) = line.find(ICON) {
                        settings_category_icon(
                            assets,
                            ui.painter(),
                            slot,
                            icon,
                            crate::stylesheet::color("text.subtle"),
                            metrics.settings_nav_icon,
                        );
                    }
                    if let Some(slot) = line.find(LABEL) {
                        flex::paint_text(
                            ui.painter(),
                            slot,
                            &label,
                            &crate::fonts::medium(metrics.nav_label_size + Dp::D3.px()),
                            text_primary(),
                            Align2::LEFT_CENTER,
                        );
                    }
                    if let Some(slot) = line.find(TRAIL) {
                        row::chevron(ui.painter(), slot, crate::stylesheet::color("glyph"));
                    }
                    clicked = response.clicked();
                },
            );
            layout.focusable.push((node, rect));
            if clicked {
                layout.activated = Some(node);
            }
        }
    }
}
