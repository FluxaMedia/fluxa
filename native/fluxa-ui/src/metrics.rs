use super::*;

/// Geometry consumed by every shared Rust screen.
///
/// These values are read from `shared/contracts/ui-tokens.json` through the
/// same parser used by the low-level renderer. The small fallbacks are only a
/// malformed-token safety net; they are not a second design system. Keeping
/// the lookup here prevents platform hosts from growing their own fixed UI
/// dimensions while still allowing the renderer to paint when a user ships a
/// partially migrated custom token document.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMetrics {
    pub background: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub border: Color32,
    pub danger: Color32,
    pub dialog_radius: f32,
    pub navigation: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_foreground: Color32,
    pub focus: Color32,
    pub page_padding: f32,
    pub screen_padding: f32,
    pub content_header_top_mobile: f32,
    pub content_header_top: f32,
    pub detail_header_top_mobile: f32,
    pub detail_header_top: f32,
    pub detail_poster_width_mobile: f32,
    pub detail_poster_width_desktop: f32,
    pub detail_poster_width_tv: f32,
    pub detail_content_gap: f32,
    pub detail_mobile_content_gap: f32,
    pub detail_action_gap: f32,
    pub detail_action_top_mobile_offset: f32,
    pub detail_action_top_offset: f32,
    pub detail_similar_top_mobile_offset: f32,
    pub detail_similar_top_offset: f32,
    pub detail_play_width: f32,
    pub detail_watchlist_width: f32,
    pub detail_back_width: f32,
    pub home_hero_content_max_width_desktop: f32,
    pub home_hero_logo_max_width_desktop: f32,
    pub home_hero_logo_height_desktop: f32,
    pub home_hero_synopsis_height_desktop: f32,
    pub home_hero_synopsis_size_desktop: f32,
    pub library_search_reserved_width: f32,
    pub library_search_min_width: f32,
    pub library_search_max_width: f32,
    pub library_sort_width: f32,
    pub library_sort_min_width: f32,
    pub calendar_panel_width_desktop: f32,
    pub calendar_panel_width_tv: f32,
    pub screen_card_min_width: f32,
    pub discover_card_title_offset: f32,
    pub discover_card_subtitle_offset: f32,
    pub settings_content_offset: f32,
    pub settings_nav_width: f32,
    pub settings_screen_padding_desktop: f32,
    pub settings_content_max_width_desktop: f32,
    pub settings_nav_item_height_desktop: f32,
    pub settings_nav_item_gap_desktop: f32,
    pub settings_section_title_size_desktop: f32,
    pub settings_description_size_desktop: f32,
    pub settings_row_label_size_desktop: f32,
    pub settings_row_value_size_desktop: f32,
    pub settings_extended_top: f32,
    pub settings_extended_line_spacing: f32,
    pub settings_extended_line_spacing_tv: f32,
    pub settings_extended_line_inset: f32,
    pub settings_extended_input_height: f32,
    pub settings_extended_action_height: f32,
    pub settings_extended_action_gap: f32,
    pub settings_extended_addon_action_width: f32,
    pub settings_extended_plugin_action_width: f32,
    pub settings_extended_small_action_width: f32,
    pub settings_extended_small_action_height: f32,
    pub settings_extended_scraper_action_width: f32,
    pub settings_extended_repo_right_inset: f32,
    pub settings_extended_scraper_right_inset: f32,
    pub screen_control_height: f32,
    pub screen_control_radius: f32,
    pub focus_ring_expand: f32,
    pub focus_ring_radius: f32,
    pub focus_ring_width: f32,
    pub card_overlay_height: f32,
    pub card_content_padding: f32,
    pub settings_card_gap: f32,
    pub settings_card_height_mobile: f32,
    pub settings_card_height_desktop: f32,
    pub settings_card_padding: f32,
    pub settings_title_top: f32,
    pub settings_description_top: f32,
    pub settings_row_inset: f32,
    pub settings_row_top: f32,
    pub settings_toggle_right_inset: f32,
    pub settings_toggle_top_inset: f32,
    pub settings_toggle_on_knob_offset: f32,
    pub settings_toggle_off_knob_offset: f32,
    pub settings_row_height: f32,
    pub settings_row_spacing: f32,
    pub settings_toggle_width: f32,
    pub settings_toggle_height: f32,
    pub settings_toggle_knob_radius: f32,
    pub calendar_grid_gap_mobile: f32,
    pub calendar_grid_gap: f32,
    pub calendar_cell_height_mobile: f32,
    pub calendar_cell_height_desktop: f32,
    pub calendar_cell_height_tv: f32,
    pub calendar_cell_radius: f32,
    pub calendar_day_thumb_width: f32,
    pub calendar_day_thumb_height: f32,
    pub library_empty_offset: f32,
    pub discover_empty_offset: f32,
    pub calendar_empty_offset: f32,
    pub calendar_day_padding: f32,
    pub calendar_day_top: f32,
    pub calendar_entry_top: f32,
    pub calendar_entry_line_height: f32,
    pub calendar_thumb_right_inset: f32,
    pub calendar_thumb_top: f32,
    pub detail_error_gap: f32,
    pub detail_stream_gap: f32,
    pub similar_overlay_height: f32,
    pub similar_text_offset: f32,
    pub section_gap: f32,
    pub control_gap: f32,
    pub card_radius: f32,
    pub card_progress_height: f32,
    pub card_title_size: f32,
    pub card_subtitle_size: f32,
    pub horizontal_card_width: f32,
    pub horizontal_card_height: f32,
    pub home_continue_card_width: f32,
    pub collection_poster_width: f32,
    pub collection_poster_height: f32,
    pub collection_wide_width: f32,
    pub collection_wide_height: f32,
    pub collection_square_size: f32,
    pub screen_margin: f32,
    pub grid_min_columns: f32,
    pub grid_max_gap: f32,
    pub settings_page_card_height: f32,
    pub settings_subpage_header_offset: f32,
    pub home_continue_card_height: f32,
    pub poster_card_width: f32,
    pub poster_card_height: f32,
    pub mobile_billboard_height: f32,
    pub nav_horizontal_padding: f32,
    pub nav_vertical_padding: f32,
    pub nav_item_gap: f32,
    pub nav_icon_size: f32,
    pub nav_item_horizontal_padding: f32,
    pub nav_item_vertical_padding: f32,
    pub nav_label_size: f32,
    pub screen_title_size: f32,
    pub screen_body_size: f32,
    pub screen_section_title_size: f32,
    pub screen_section_title_size_tv: f32,
    pub screen_card_title_size: f32,
    pub screen_card_subtitle_size: f32,
    pub catalog_title_size: f32,
    pub horizontal_spacing: f32,
    pub vertical_spacing: f32,
    pub minimum_card_width: f32,
    pub poster_height_ratio: f32,
    pub horizontal_card_height_ratio: f32,
    pub focused_scale: f32,
    pub gutter: f32,
    pub text: TypeScale,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeScale {
    pub display: f32,
    pub title: f32,
    pub subtitle: f32,
    pub body: f32,
    pub meta: f32,
    pub label: f32,
}

pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
    pub const XXXL: f32 = 48.0;
}

impl UiMetrics {
    pub fn for_viewport(viewport: Viewport) -> Self {
        let tokens = active_shared_ui_tokens().expect("shared Fluxa tokens must be valid");
        let dark = active_theme("fluxa-dark").expect("shared Fluxa theme must be valid");
        let number = |path: &[&str], fallback: f32| {
            let mut value = &tokens.layout;
            for key in path {
                let Some(next) = value.get(*key) else {
                    return fallback;
                };
                value = next;
            }
            value.as_f64().map(|value| value as f32).unwrap_or(fallback)
        };
        let platform = if viewport.is_compact() {
            "mobile"
        } else if viewport.is_tv() {
            "tv"
        } else {
            "desktop"
        };
        let window_class = match viewport.size_class() {
            SizeClass::Compact => "compact",
            SizeClass::Medium => "medium",
            SizeClass::Expanded => "expanded",
        };
        let roomy = !viewport.is_compact() && !viewport.is_tv();
        let pick = |roomy_value: f32, compact_value: f32| {
            if roomy { roomy_value } else { compact_value }
        };
        let common_dp = |key, fallback| number(&["common", "dp", key], fallback);
        let common_sp = |key, fallback| number(&["common", "sp", key], fallback);
        let platform_dp = |key, fallback| number(&["platforms", platform, "dp", key], fallback);
        let platform_sp = |key, fallback| number(&["platforms", platform, "sp", key], fallback);
        let window_dp =
            |key, fallback| number(&["windowClasses", window_class, "dp", key], fallback);
        let window_sp =
            |key, fallback| number(&["windowClasses", window_class, "sp", key], fallback);
        let color = |token: &str, fallback: Color32| {
            dark.color(token)
                .map(|value| {
                    Color32::from_rgba_unmultiplied(
                        (value.r * 255.0).round() as u8,
                        (value.g * 255.0).round() as u8,
                        (value.b * 255.0).round() as u8,
                        (value.a * 255.0).round() as u8,
                    )
                })
                .unwrap_or(fallback)
        };
        let mut metrics = Self {
            background: color("background", Color32::from_rgb(7, 7, 9)),
            surface: color("surface", Color32::from_rgb(20, 20, 22)),
            surface_raised: color("surfaceRaised", Color32::from_rgb(28, 28, 31)),
            border: color("border", Color32::from_white_alpha(20)),
            danger: color("error", Color32::from_rgb(255, 107, 107)),
            navigation: color("navigation", Color32::from_rgb(20, 20, 22)),
            text_primary: color("textPrimary", Color32::WHITE),
            text_secondary: color("textSecondary", Color32::from_rgb(184, 184, 190)),
            text_muted: color("textMuted", Color32::from_rgb(138, 138, 145)),
            accent: color("accent", Color32::from_rgb(239, 78, 52)),
            accent_foreground: color("accentForeground", Color32::WHITE),
            focus: color("focus", Color32::WHITE),
            page_padding: match viewport.size_class() {
                _ if viewport.is_tv() => number(
                    &["windowClasses", "expanded", "dp", "pageHorizontalPadding"],
                    48.0,
                ),
                SizeClass::Expanded => dark.spacing.screen_padding,
                _ => window_dp("pageHorizontalPadding", 16.0),
            },
            screen_padding: dark.spacing.screen_padding,
            content_header_top_mobile: common_dp("contentHeaderTopMobile", 48.0),
            content_header_top: common_dp("contentHeaderTop", 110.0),
            detail_header_top_mobile: common_dp("detailHeaderTopMobile", 48.0),
            detail_header_top: common_dp("detailHeaderTop", 116.0),
            detail_poster_width_mobile: common_dp("detailPosterWidthMobile", 132.0),
            detail_poster_width_desktop: common_dp("detailPosterWidthDesktop", 220.0),
            detail_poster_width_tv: common_dp("detailPosterWidthTv", 270.0),
            detail_content_gap: common_dp("detailContentGap", 28.0),
            detail_mobile_content_gap: common_dp("detailMobileContentGap", 22.0),
            detail_action_gap: common_dp("detailActionGap", 8.0),
            detail_action_top_mobile_offset: common_dp("detailActionTopMobileOffset", 182.0),
            detail_action_top_offset: common_dp("detailActionTopOffset", 188.0),
            detail_similar_top_mobile_offset: common_dp("detailSimilarTopMobileOffset", 300.0),
            detail_similar_top_offset: common_dp("detailSimilarTopOffset", 34.0),
            detail_play_width: common_dp("detailPlayWidth", 118.0),
            detail_watchlist_width: common_dp("detailWatchlistWidth", 148.0),
            detail_back_width: common_dp("detailBackWidth", 92.0),
            home_hero_content_max_width_desktop: common_dp("homeHeroContentMaxWidthDesktop", 980.0),
            home_hero_logo_max_width_desktop: common_dp("homeHeroLogoMaxWidthDesktop", 820.0),
            home_hero_logo_height_desktop: common_dp("homeHeroLogoHeightDesktop", 148.0),
            home_hero_synopsis_height_desktop: common_dp("homeHeroSynopsisHeightDesktop", 88.0),
            home_hero_synopsis_size_desktop: common_sp("homeHeroSynopsisSizeDesktop", 16.0),
            library_search_reserved_width: common_dp("librarySearchReservedWidth", 150.0),
            library_search_min_width: common_dp("librarySearchMinWidth", 100.0),
            library_search_max_width: common_dp("librarySearchMaxWidth", 360.0),
            library_sort_width: common_dp("librarySortWidth", 130.0),
            library_sort_min_width: common_dp("librarySortMinWidth", 50.0),
            calendar_panel_width_desktop: common_dp("calendarPanelWidthDesktop", 360.0),
            calendar_panel_width_tv: common_dp("calendarPanelWidthTv", 440.0),
            screen_card_min_width: common_dp("screenCardMinWidth", 120.0),
            discover_card_title_offset: common_dp("discoverCardTitleOffset", 46.0),
            discover_card_subtitle_offset: common_dp("discoverCardSubtitleOffset", 25.0),
            settings_content_offset: pick(
                common_dp("settingsNavWidthDesktop", 268.0)
                    + common_dp("settingsPanelGapDesktop", 32.0),
                common_dp("settingsContentOffset", 220.0),
            ),
            settings_nav_width: pick(
                common_dp("settingsNavWidthDesktop", 268.0),
                common_dp("settingsNavWidth", 196.0),
            ),
            settings_screen_padding_desktop: common_dp("settingsScreenPaddingDesktop", 48.0),
            settings_content_max_width_desktop: common_dp("settingsContentMaxWidthDesktop", 2400.0),
            settings_nav_item_height_desktop: common_dp("settingsNavItemHeightDesktop", 44.0),
            settings_nav_item_gap_desktop: common_dp("settingsNavItemGapDesktop", 6.0),
            settings_section_title_size_desktop: common_sp("settingsSectionTitleSizeDesktop", 24.0),
            settings_description_size_desktop: common_sp("settingsDescriptionSizeDesktop", 14.0),
            settings_row_label_size_desktop: common_sp("settingsRowLabelSizeDesktop", 18.0),
            settings_row_value_size_desktop: common_sp("settingsRowValueSizeDesktop", 15.0),
            settings_extended_top: common_dp("settingsExtendedTop", 76.0),
            settings_extended_line_spacing: common_dp("settingsExtendedLineSpacing", 34.0),
            settings_extended_line_spacing_tv: common_dp("settingsExtendedLineSpacingTv", 42.0),
            settings_extended_line_inset: common_dp("settingsExtendedLineInset", 20.0),
            settings_extended_input_height: common_dp("settingsExtendedInputHeight", 40.0),
            settings_extended_action_height: common_dp("settingsExtendedActionHeight", 38.0),
            settings_extended_action_gap: common_dp("settingsExtendedActionGap", 10.0),
            settings_extended_addon_action_width: common_dp("settingsAddonActionWidth", 148.0),
            settings_extended_plugin_action_width: common_dp("settingsPluginActionWidth", 156.0),
            settings_extended_small_action_width: common_dp("settingsSmallActionWidth", 76.0),
            settings_extended_small_action_height: common_dp("settingsSmallActionHeight", 34.0),
            settings_extended_scraper_action_width: common_dp("settingsScraperActionWidth", 96.0),
            settings_extended_repo_right_inset: common_dp("settingsRepoRightInset", 80.0),
            settings_extended_scraper_right_inset: common_dp("settingsScraperRightInset", 104.0),
            screen_control_height: common_dp("screenControlHeight", 36.0),
            screen_control_radius: common_dp("screenControlRadius", 9.0),
            focus_ring_expand: common_dp("focusRingExpand", 3.0),
            focus_ring_radius: common_dp("focusRingRadius", 10.0),
            focus_ring_width: common_dp("focusRingWidth", 2.5),
            card_overlay_height: common_dp("cardOverlayHeight", 82.0),
            card_content_padding: common_dp("cardContentPadding", 10.0),
            settings_card_gap: common_dp("settingsCardGap", 18.0),
            settings_card_height_mobile: common_dp("settingsCardHeightMobile", 168.0),
            settings_card_height_desktop: common_dp("settingsCardHeightDesktop", 154.0),
            settings_card_padding: pick(
                common_dp("settingsCardPaddingDesktop", 24.0),
                common_dp("settingsCardPadding", 16.0),
            ),
            settings_title_top: pick(
                common_dp("settingsTitleTopDesktop", 18.0),
                common_dp("settingsTitleTop", 14.0),
            ),
            settings_description_top: pick(
                common_dp("settingsDescriptionTopDesktop", 54.0),
                common_dp("settingsDescriptionTop", 39.0),
            ),
            settings_row_inset: pick(
                common_dp("settingsRowInsetDesktop", 24.0),
                common_dp("settingsRowInset", 14.0),
            ),
            settings_row_top: pick(
                common_dp("settingsRowTopDesktop", 88.0),
                common_dp("settingsRowTop", 67.0),
            ),
            settings_toggle_right_inset: pick(
                common_dp("settingsToggleWidthDesktop", 48.0) + 4.0,
                common_dp("settingsToggleRightInset", 44.0),
            ),
            settings_toggle_top_inset: pick(11.0, common_dp("settingsToggleTopInset", 10.0)),
            settings_toggle_on_knob_offset: pick(
                common_dp("settingsToggleOnKnobOffsetDesktop", 18.0),
                common_dp("settingsToggleOnKnobOffset", 14.0),
            ),
            settings_toggle_off_knob_offset: pick(
                common_dp("settingsToggleOffKnobOffsetDesktop", 38.0),
                common_dp("settingsToggleOffKnobOffset", 34.0),
            ),
            settings_row_height: pick(
                common_dp("settingsRowHeightDesktop", 44.0),
                common_dp("settingsRowHeight", 32.0),
            ),
            settings_row_spacing: pick(
                common_dp("settingsRowSpacingDesktop", 56.0),
                common_dp("settingsRowSpacing", 38.0),
            ),
            settings_toggle_width: pick(
                common_dp("settingsToggleWidthDesktop", 48.0),
                common_dp("settingsToggleWidth", 40.0),
            ),
            settings_toggle_height: pick(
                common_dp("settingsToggleHeightDesktop", 26.0),
                common_dp("settingsToggleHeight", 20.0),
            ),
            settings_toggle_knob_radius: pick(
                common_dp("settingsToggleKnobRadiusDesktop", 9.0),
                common_dp("settingsToggleKnobRadius", 7.0),
            ),
            calendar_grid_gap_mobile: common_dp("calendarGridGapMobile", 5.0),
            calendar_grid_gap: common_dp("calendarGridGap", 8.0),
            calendar_cell_height_mobile: common_dp("calendarCellHeightMobile", 58.0),
            calendar_cell_height_desktop: common_dp("calendarCellHeightDesktop", 96.0),
            calendar_cell_height_tv: common_dp("calendarCellHeightTv", 112.0),
            calendar_cell_radius: common_dp("calendarCellRadius", 9.0),
            calendar_day_thumb_width: common_dp("calendarDayThumbWidth", 23.0),
            calendar_day_thumb_height: common_dp("calendarDayThumbHeight", 32.0),
            library_empty_offset: common_dp("libraryEmptyOffset", 55.0),
            discover_empty_offset: common_dp("discoverEmptyOffset", 70.0),
            calendar_empty_offset: common_dp("calendarEmptyOffset", 150.0),
            calendar_day_padding: common_dp("calendarDayPadding", 8.0),
            calendar_day_top: common_dp("calendarDayTop", 7.0),
            calendar_entry_top: common_dp("calendarEntryTop", 31.0),
            calendar_entry_line_height: common_dp("calendarEntryLineHeight", 19.0),
            calendar_thumb_right_inset: common_dp("calendarThumbRightInset", 30.0),
            calendar_thumb_top: common_dp("calendarThumbTop", 7.0),
            detail_error_gap: common_dp("detailErrorGap", 12.0),
            detail_stream_gap: common_dp("detailStreamGap", 18.0),
            similar_overlay_height: common_dp("similarOverlayHeight", 30.0),
            similar_text_offset: common_dp("similarTextOffset", 20.0),
            section_gap: dark.spacing.section_gap,
            control_gap: dark.spacing.control_gap,
            card_radius: dark.shape.card_radius,
            dialog_radius: dark.shape.dialog_radius,
            card_progress_height: common_dp("cardProgressBarHeight", 4.0),
            card_title_size: common_sp("cardTitleSize", 12.0),
            card_subtitle_size: common_sp("cardSubtitleSize", 10.0),
            horizontal_card_width: platform_dp("episodeCardWidth", 300.0),
            horizontal_card_height: platform_dp("episodeCardHeight", 180.0),
            // Home's Continue Watching row uses the same responsive size
            // presets as Compose horizontalCardWidth/Height, not the larger
            // episode-card dimensions used by other surfaces.
            home_continue_card_width: window_dp("horizontalCardBase", 196.0),
            home_continue_card_height: window_dp("horizontalCardBase", 196.0)
                * number(&["common", "number", "horizontalCardHeightRatio"], 0.56),
            poster_card_width: platform_dp("posterCardWidth", 160.0),
            poster_card_height: platform_dp("posterCardHeight", 240.0),
            mobile_billboard_height: common_dp("mobileBillboardHeight", 540.0),
            nav_horizontal_padding: window_dp("navigationHorizontalPadding", 32.0),
            nav_vertical_padding: window_dp("navigationVerticalPadding", 12.0),
            nav_item_gap: window_dp("navigationItemSpacing", 8.0),
            nav_icon_size: window_dp("navigationIconSize", 26.0),
            nav_item_horizontal_padding: window_dp("navigationItemHorizontalPadding", 14.0),
            nav_item_vertical_padding: window_dp("navigationItemVerticalPadding", 9.0),
            nav_label_size: window_sp("navigationLabelSize", 14.0),
            screen_title_size: match platform {
                "mobile" => common_sp("screenTitleSizeMobile", 32.0),
                "tv" => common_sp("screenTitleSizeTv", 42.0),
                _ => common_sp("screenTitleSize", 34.0),
            },
            screen_body_size: if platform == "tv" {
                common_sp("screenBodySizeTv", 18.0)
            } else {
                common_sp("screenBodySize", 15.0)
            },
            screen_section_title_size: common_sp("screenSectionTitleSize", 20.0),
            screen_section_title_size_tv: common_sp("screenSectionTitleSizeTv", 24.0),
            screen_card_title_size: if platform == "tv" {
                common_sp("screenCardTitleSizeTv", 17.0)
            } else {
                common_sp("screenCardTitleSize", 15.0)
            },
            screen_card_subtitle_size: common_sp("screenCardSubtitleSize", 12.0),
            catalog_title_size: window_sp("catalogTitleSize", 26.0),
            horizontal_spacing: window_dp("horizontalSpacing", 16.0),
            vertical_spacing: window_dp("verticalSpacing", 20.0),
            minimum_card_width: window_dp("minimumCardWidth", 170.0),
            poster_height_ratio: number(&["common", "number", "posterHeightRatio"], 1.5),
            horizontal_card_height_ratio: number(
                &["common", "number", "horizontalCardHeightRatio"],
                0.56,
            ),
            focused_scale: number(&["common", "number", "cardFocusedScale"], 1.12),
            gutter: platform_dp("gutter", 16.0),
            text: TypeScale {
                display: platform_sp("display", 28.0),
                title: platform_sp("title", 20.0),
                subtitle: platform_sp("subtitle", 16.0),
                body: platform_sp("body", 14.5),
                meta: platform_sp("meta", 13.0),
                label: platform_sp("label", 12.0),
            },
            collection_poster_width: platform_dp("collectionPosterWidth", 156.0),
            collection_poster_height: platform_dp("collectionPosterHeight", 234.0),
            collection_wide_width: platform_dp("collectionWideWidth", 280.0),
            collection_wide_height: platform_dp("collectionWideHeight", 158.0),
            collection_square_size: platform_dp("collectionSquareSize", 150.0),
            screen_margin: 0.0,
            grid_min_columns: platform_dp("gridMinColumns", 1.0),
            grid_max_gap: platform_dp("gridMaxGap", 0.0),
            settings_page_card_height: platform_dp("settingsPageCardHeight", 72.0),
            settings_subpage_header_offset: platform_dp("settingsSubpageHeaderOffset", 66.0),
        };
        metrics.screen_margin = match platform {
            "mobile" => metrics.page_padding,
            "tv" => metrics.screen_padding.max(32.0),
            _ => metrics.screen_padding,
        };
        let visible = platform_dp("posterVisibleCards", 0.0);
        if visible > 0.0 {
            let gap = metrics.horizontal_spacing;
            let content = viewport.width - metrics.page_padding * 2.0;
            let width = ((content - gap * (visible.ceil() - 1.0)) / visible).clamp(
                platform_dp("posterCardMin", 120.0),
                platform_dp("posterCardMax", 200.0),
            );
            metrics.poster_card_width = width;
            metrics.poster_card_height = width * metrics.poster_height_ratio;
        }
        let continue_visible = platform_dp("continueVisibleCards", 0.0);
        if continue_visible > 0.0 {
            let gap = metrics.horizontal_spacing;
            let content = viewport.width - metrics.page_padding * 2.0;
            let width = ((content - gap * (continue_visible.ceil() - 1.0)) / continue_visible)
                .clamp(
                    platform_dp("continueCardMin", 180.0),
                    platform_dp("continueCardMax", 320.0),
                );
            metrics.home_continue_card_width = width;
            metrics.home_continue_card_height = width * metrics.horizontal_card_height_ratio;
        }
        if poster_overlay::landscape() {
            metrics.poster_card_width = metrics.home_continue_card_width;
            metrics.poster_card_height = metrics.poster_card_width * 0.5625;
        }
        metrics
    }

    pub fn navigation_label_size(self, tv: bool) -> f32 {
        self.nav_label_size + if tv { 3.0 } else { 0.0 }
    }

    pub fn navigation_icon_size(self, tv: bool) -> f32 {
        if tv { 25.0 } else { 20.0 }
    }

    pub fn navigation_item_width(self, label: &str, tv: bool) -> f32 {
        44.0 + self.navigation_icon_size(tv)
            + 8.0
            + estimated_navigation_text_width(label, self.navigation_label_size(tv))
    }

    pub fn navigation_profile_width(self, profile_name: &str, tv: bool) -> f32 {
        (36.0
            + NAV_AVATAR_RADIUS * 2.0
            + 8.0
            + estimated_navigation_text_width(profile_name, self.navigation_label_size(tv)))
        .clamp(72.0, 200.0)
    }

    pub fn navigation_bar_width(self, viewport: Viewport, profile_name: &str) -> f32 {
        let compact = viewport.is_compact();
        let tv = viewport.is_tv();
        let margin = if compact {
            self.page_padding
        } else if tv {
            self.page_padding.max(32.0)
        } else {
            self.page_padding
        };
        let nav_width = crate::navigation::nav_labels()
            .iter()
            .map(|label| self.navigation_item_width(label, tv))
            .sum::<f32>();
        (if compact {
            (viewport.width - margin * 2.0).max(280.0)
        } else {
            nav_width + NAV_ITEM_GAP * 4.0 + self.navigation_profile_width(profile_name, tv)
        })
        .min((viewport.width - margin * 2.0).max(1.0))
    }
}
