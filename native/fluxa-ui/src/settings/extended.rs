use super::*;

fn below_groups(cards: &[Rect], rect: Rect, metrics: UiMetrics) -> f32 {
    cards.last().map_or(
        rect.top() + metrics.settings_appearance_header_height,
        |card| card.bottom() + metrics.settings_group_gap,
    )
}

pub(super) fn draw_settings_extended_section(
    context: &egui::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    section: &str,
    language: &str,
    rect: Rect,
    cards: &[Rect],
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    match section {
        "Account" => draw_account(
            context, viewport, settings, assets, language, rect, metrics, layout,
        ),
        "TMDB" => {
            let ready = tmdb_ready(settings);
            let card = draw_field_group(
                context,
                settings,
                rect,
                if ready {
                    rect.top() + metrics.settings_appearance_header_height
                } else {
                    page_top(rect, metrics)
                },
                &localized("settings.group.api_key", language),
                &[1],
                metrics,
                viewport.is_compact(),
                layout,
            );
            if !ready {
                painter.galley(
                    Pos2::new(rect.left() + 16.0, card.bottom() + 12.0),
                    wrapped_regular(
                        &painter,
                        &localized("settings.tmdb_metadata_source_inactive", language),
                        metrics.screen_card_subtitle_size + 1.0,
                        ink(120),
                        rect.width() - 32.0,
                        2,
                    ),
                    ink(120),
                );
            }
        }
        "Metadatakit" | "Metadatakit Catalogs" | "Streamkit" => {
            let section = SETTINGS_SECTIONS
                .iter()
                .find(|candidate| candidate.title == section)
                .unwrap_or(&SETTINGS_SECTIONS[0]);
            for (slot, top) in slot_tops(settings, section.title, rect, cards, metrics) {
                draw_field_group(
                    context,
                    settings,
                    rect,
                    top,
                    &localized(slot.title, language),
                    slot.fields,
                    metrics,
                    viewport.is_compact(),
                    layout,
                );
            }
        }
        "MDBList" => {
            let card = draw_field_group(
                context,
                settings,
                rect,
                page_top(rect, metrics),
                &localized("settings.group.api_key", language),
                &[2],
                metrics,
                viewport.is_compact(),
                layout,
            );
            account::draw_source_group(
                context,
                viewport,
                settings,
                rect,
                card.bottom() + metrics.settings_group_gap,
                (account::MDBLIST_LIST_SOURCE, "settings.group.account_sync"),
                metrics,
                layout,
            );
        }
        "Skip Segments" => {
            draw_field_group(
                context,
                settings,
                rect,
                page_top(rect, metrics),
                &localized("settings.group.api_keys", language),
                &[3, 4, 5],
                metrics,
                viewport.is_compact(),
                layout,
            );
        }
        "Summary" => {
            draw_summary(&painter, settings, language, rect, metrics);
        }
        "Watch Party" => {
            draw_watch_party(context, settings, language, rect, metrics, layout, assets)
        }
        "Licenses" => about::draw_credits(
            &painter,
            assets,
            language,
            rect,
            page_top(rect, metrics),
            metrics,
        ),
        "About" => draw_about(
            context,
            &painter,
            &*assets,
            settings,
            language,
            rect,
            viewport.is_compact(),
            metrics,
            layout,
        ),
        "Shortcuts" => {
            let mut top = rect.top() + metrics.settings_appearance_header_height;
            if settings.gamepad_active {
                let card = draw_binding_group(
                    &painter,
                    rect,
                    top,
                    &localized("settings.group.controller", language),
                    &CONTROLLER_BINDINGS,
                    language,
                    metrics,
                );
                top = card.bottom() + metrics.settings_group_gap;
            }
            for (title, player) in [
                ("settings.shortcuts_group_general", false),
                ("settings.shortcuts_group_player", true),
            ] {
                let rows = settings
                    .shortcuts
                    .iter()
                    .enumerate()
                    .filter(|(_, row)| row.player == player)
                    .collect::<Vec<_>>();
                let card = draw_shortcut_group(
                    context,
                    &painter,
                    rect,
                    top,
                    &localized(title, language),
                    &rows,
                    settings,
                    language,
                    metrics,
                    layout,
                );
                top = card.bottom() + metrics.settings_group_gap;
            }
            let card = account_group(
                &painter,
                rect,
                top,
                settings_group_card_height(1, metrics),
                &localized("settings.shortcuts_reset_all", language),
                metrics,
            );
            let row = account_row(card, 0, metrics);
            let label = localized("settings.shortcuts_reset_all", language);
            let size = metrics.settings_row_label_size_desktop;
            let width = painter
                .layout_no_wrap(label.clone(), FontId::proportional(size), Color32::WHITE)
                .size()
                .x
                + 32.0;
            let button = Rect::from_center_size(
                Pos2::new(row.right() - 4.0 - width * 0.5, row.center().y),
                Vec2::new(width, 32.0_f32.min(row.height() - 4.0)),
            );
            pill_button(
                context,
                layout,
                NODE_SETTINGS_SHORTCUT_RESET_ALL,
                button,
                &label,
                size,
            );
            if !settings.gamepad_active {
                draw_binding_group(
                    &painter,
                    rect,
                    card.bottom() + metrics.settings_group_gap,
                    &localized("settings.group.controller", language),
                    &CONTROLLER_BINDINGS,
                    language,
                    metrics,
                );
            }
        }
        "Posters" => {
            let top = below_groups(cards, rect, metrics);
            draw_field_group(
                context,
                settings,
                rect,
                top,
                &localized("settings.group.custom_poster", language),
                &[0],
                metrics,
                viewport.is_compact(),
                layout,
            );
        }
        "Subtitles" => {
            let top = below_groups(cards, rect, metrics);
            let top = account::draw_source_group(
                context,
                viewport,
                settings,
                rect,
                top,
                (
                    account::EXTERNAL_AUDIO_SOURCE_INDEX,
                    "settings.group.external_audio_source",
                ),
                metrics,
                layout,
            ) + metrics.settings_group_gap;
            draw_subtitle_preview(context, settings, rect, top, language, metrics);
        }
        "Previews" => {
            let top = below_groups(cards, rect, metrics);
            let top = account::draw_source_group(
                context,
                viewport,
                settings,
                rect,
                top,
                (3, "settings.group.seek_preview_source"),
                metrics,
                layout,
            ) + metrics.settings_group_gap;
            draw_field_group(
                context,
                settings,
                rect,
                top,
                &localized("settings.group.seek_preview_filter", language),
                &[6, 7],
                metrics,
                viewport.is_compact(),
                layout,
            );
        }
        "Badges" => {
            let top = below_groups(cards, rect, metrics);
            let top = account::draw_source_group(
                context,
                viewport,
                settings,
                rect,
                top,
                (
                    account::HERO_BADGE_SOURCE_INDEX,
                    "settings.group.hero_badge_source",
                ),
                metrics,
                layout,
            ) + metrics.settings_group_gap;
            badges::draw_badges(
                context, settings, assets, language, rect, top, metrics, layout,
            );
        }
        "Collections" => collections::draw_collections(
            context,
            settings,
            language,
            rect,
            rect.top() + metrics.settings_subpage_header_offset,
            metrics,
            layout,
        ),
        "Home" => {
            let top = below_groups(cards, rect, metrics);
            hero::draw_hero_settings(context, settings, language, rect, top, metrics, layout);
        }
        "Ambient Mode" => {
            let top = below_groups(cards, rect, metrics);
            ambient::draw_ambient_settings(context, settings, language, rect, top, metrics, layout);
        }
        "Details" => {
            let top = below_groups(cards, rect, metrics);
            detail_sections::draw_detail_sections(
                context, settings, language, rect, top, metrics, layout,
            );
        }
        "Add-ons" => addons::draw_addons(
            context,
            settings,
            assets,
            language,
            rect,
            rect.top() + metrics.settings_subpage_header_offset,
            metrics,
            layout,
        ),
        "Plugins" => addons::draw_plugins(
            context,
            settings,
            assets,
            language,
            rect,
            rect.top() + metrics.settings_subpage_header_offset,
            metrics,
            layout,
        ),
        "Streamkit Addons" => addons::draw_streamkit_addons(
            context,
            settings,
            assets,
            language,
            rect,
            rect.top() + metrics.settings_subpage_header_offset,
            metrics,
            layout,
        ),
        "Servers" => addons::draw_servers(
            context,
            settings,
            assets,
            language,
            rect,
            rect.top() + metrics.settings_subpage_header_offset,
            metrics,
            layout,
        ),
        _ => {}
    }
}
