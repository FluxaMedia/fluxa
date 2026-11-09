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
            let top = if ready {
                rect.top() + metrics.settings_appearance_header_height
            } else {
                page_top(rect, metrics)
            };
            flow_area(context, section, rect, top, |flow| {
                draw_field_group(
                    context,
                    settings,
                    flow,
                    &localized("settings.group.api_key", language),
                    &[1],
                    metrics,
                    viewport.is_compact(),
                    layout,
                );
                if !ready {
                    flow.add_space(Space::S12.px(&metrics));
                    let note = wrapped_regular(
                        &painter,
                        &localized("settings.tmdb_metadata_source_inactive", language),
                        metrics.screen_card_subtitle_size + metrics.settings_row_value_boost,
                        ink(120),
                        rect.width() - Space::S16.px(&metrics) * 2.0,
                        2,
                    );
                    let slot = flow
                        .allocate_space(Vec2::new(rect.width(), note.size().y))
                        .1;
                    painter.galley(
                        super::page::padded(
                            slot,
                            crate::flex::Insets::xy(Space::S16.px(&metrics), 0.0),
                        )
                        .min,
                        note,
                        ink(120),
                    );
                }
            });
        }
        "Metadatakit" | "Metadatakit Catalogs" | "Streamkit" => {
            let section = SETTINGS_SECTIONS
                .iter()
                .find(|candidate| candidate.title == section)
                .unwrap_or(&SETTINGS_SECTIONS[0]);
            for (slot, top) in slot_tops(settings, section.title, rect, cards, metrics) {
                flow_area(context, slot.title, rect, top, |flow| {
                    draw_field_group(
                        context,
                        settings,
                        flow,
                        &localized(slot.title, language),
                        slot.fields,
                        metrics,
                        viewport.is_compact(),
                        layout,
                    );
                });
            }
        }
        "MDBList" => flow_area(context, section, rect, page_top(rect, metrics), |flow| {
            draw_field_group(
                context,
                settings,
                flow,
                &localized("settings.group.api_key", language),
                &[2],
                metrics,
                viewport.is_compact(),
                layout,
            );
            flow.add_space(metrics.settings_group_gap);
            account::source_group(
                context,
                viewport,
                settings,
                flow,
                (account::MDBLIST_LIST_SOURCE, "settings.group.account_sync"),
                metrics,
                layout,
            );
        }),
        "Skip Segments" => {
            flow_area(context, section, rect, page_top(rect, metrics), |flow| {
                draw_field_group(
                    context,
                    settings,
                    flow,
                    &localized("settings.group.api_keys", language),
                    &[3, 4, 5],
                    metrics,
                    viewport.is_compact(),
                    layout,
                );
            });
        }
        "Summary" => {
            draw_summary(context, settings, language, rect, metrics);
        }
        "Watch Party" => {
            draw_watch_party(context, settings, language, rect, metrics, layout, assets)
        }
        "Licenses" => about::draw_credits(
            context,
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
            let top = rect.top() + metrics.settings_appearance_header_height;
            flow_area(context, section, rect, top, |flow| {
                if settings.gamepad_active {
                    draw_binding_group(
                        flow,
                        &localized("settings.group.controller", language),
                        &CONTROLLER_BINDINGS,
                        language,
                        metrics,
                    );
                    flow.add_space(metrics.settings_group_gap);
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
                    draw_shortcut_group(
                        context,
                        flow,
                        &localized(title, language),
                        &rows,
                        settings,
                        language,
                        metrics,
                        layout,
                    );
                    flow.add_space(metrics.settings_group_gap);
                }
                let label = localized("settings.shortcuts_reset_all", language);
                account_group(
                    flow,
                    &localized("settings.shortcuts_reset_all", language),
                    metrics,
                    |card| {
                        let painter = card.painter().clone();
                        let row = account_row(card, &painter, metrics, false);
                        let size = metrics.settings_row_label_size_desktop;
                        let width = pill_text_width(&painter, &label, size, metrics);
                        let button = super::page::aligned(
                            row,
                            Align2::RIGHT_CENTER,
                            Vec2::new(
                                width,
                                metrics
                                    .screen_control_height
                                    .min(row.height() - Space::S4.px(&metrics)),
                            ),
                            Space::S4.px(&metrics),
                        );
                        pill_button(
                            context,
                            layout,
                            NODE_SETTINGS_SHORTCUT_RESET_ALL,
                            button,
                            &label,
                            size,
                        );
                    },
                );
                if !settings.gamepad_active {
                    flow.add_space(metrics.settings_group_gap);
                    draw_binding_group(
                        flow,
                        &localized("settings.group.controller", language),
                        &CONTROLLER_BINDINGS,
                        language,
                        metrics,
                    );
                }
            });
        }
        "Posters" => {
            flow_area(
                context,
                section,
                rect,
                below_groups(cards, rect, metrics),
                |flow| {
                    draw_field_group(
                        context,
                        settings,
                        flow,
                        &localized("settings.group.custom_poster", language),
                        &[0],
                        metrics,
                        viewport.is_compact(),
                        layout,
                    );
                },
            );
        }
        "Subtitles" => {
            flow_area(
                context,
                section,
                rect,
                below_groups(cards, rect, metrics),
                |flow| {
                    account::source_group(
                        context,
                        viewport,
                        settings,
                        flow,
                        (
                            account::EXTERNAL_AUDIO_SOURCE_INDEX,
                            "settings.group.external_audio_source",
                        ),
                        metrics,
                        layout,
                    );
                    flow.add_space(metrics.settings_group_gap);
                    draw_subtitle_preview(settings, flow, language, metrics);
                },
            );
        }
        "Previews" => {
            flow_area(
                context,
                section,
                rect,
                below_groups(cards, rect, metrics),
                |flow| {
                    account::source_group(
                        context,
                        viewport,
                        settings,
                        flow,
                        (3, "settings.group.seek_preview_source"),
                        metrics,
                        layout,
                    );
                    flow.add_space(metrics.settings_group_gap);
                    draw_field_group(
                        context,
                        settings,
                        flow,
                        &localized("settings.group.seek_preview_filter", language),
                        &[6, 7],
                        metrics,
                        viewport.is_compact(),
                        layout,
                    );
                },
            );
        }
        "Badges" => {
            let top = below_groups(cards, rect, metrics);
            flow_area(context, section, rect, top, |flow| {
                account::source_group(
                    context,
                    viewport,
                    settings,
                    flow,
                    (
                        account::HERO_BADGE_SOURCE_INDEX,
                        "settings.group.hero_badge_source",
                    ),
                    metrics,
                    layout,
                );
                flow.add_space(metrics.settings_group_gap);
                badges::draw_badges(context, settings, assets, language, flow, metrics, layout);
            });
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
