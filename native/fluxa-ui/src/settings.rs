use super::*;

#[derive(Clone, Debug)]
pub struct SettingsRow {
    pub label: &'static str,
    pub key: &'static str,
    pub options: &'static [&'static str],
}

#[derive(Clone, Debug)]
pub struct SettingsSection {
    pub title: &'static str,
    pub description: &'static str,
    pub rows: &'static [SettingsRow],
}

const GENERAL_SETTINGS: [SettingsRow; 15] = [
    SettingsRow {
        label: "Language",
        key: "language",
        options: &["en", "tr"],
    },
    SettingsRow {
        label: "Start page",
        key: "startPage",
        options: &["home", "library", "discover", "calendar"],
    },
    SettingsRow {
        label: "Notifications",
        key: "notificationsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Background playback",
        key: "backgroundPlayback",
        options: &[],
    },
    SettingsRow {
        label: "Automatic updates",
        key: "automaticUpdates",
        options: &[],
    },
    SettingsRow {
        label: "Search opens details",
        key: "searchSuggestionsOpenDetail",
        options: &[],
    },
    SettingsRow {
        label: "AMOLED black",
        key: "amoledMode",
        options: &[],
    },
    SettingsRow {
        label: "Content warnings",
        key: "contentWarningsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Detail hero collapses",
        key: "detailCollapsingHero",
        options: &[],
    },
    SettingsRow {
        label: "Prefer clear logo",
        key: "detailPreferClearlogo",
        options: &[],
    },
    SettingsRow {
        label: "Show cast",
        key: "detailShowCast",
        options: &[],
    },
    SettingsRow {
        label: "Episode descriptions",
        key: "detailShowEpisodeDescriptions",
        options: &[],
    },
    SettingsRow {
        label: "Recommendations",
        key: "detailShowRecommendations",
        options: &[],
    },
    SettingsRow {
        label: "Upcoming episodes row",
        key: "upcomingRowEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Timezone conversion",
        key: "timezoneConversionEnabled",
        options: &[],
    },
];
const PLAYBACK_SETTINGS: [SettingsRow; 19] = [
    SettingsRow {
        label: "Playback destination",
        key: "preferredPlayer",
        options: &["mpv", "exoplayer", "external"],
    },
    SettingsRow {
        label: "Stream selection",
        key: "streamSourceSelectionMode",
        options: &["manual", "first", "best", "regex"],
    },
    SettingsRow {
        label: "Autoplay next episode",
        key: "autoPlayNextEpisode",
        options: &[],
    },
    SettingsRow {
        label: "Auto skip intro",
        key: "autoSkipIntro",
        options: &[],
    },
    SettingsRow {
        label: "Retry next source",
        key: "autoRetryNextSource",
        options: &[],
    },
    SettingsRow {
        label: "Picture in picture",
        key: "pictureInPicture",
        options: &[],
    },
    SettingsRow {
        label: "Preferred audio",
        key: "preferredAudioLanguage",
        options: &["none", "en", "tr"],
    },
    SettingsRow {
        label: "Preferred subtitle language",
        key: "preferredSubtitleLanguage",
        options: &["none", "en", "tr"],
    },
    SettingsRow {
        label: "Auto-enable subtitles",
        key: "autoEnableSubtitles",
        options: &[],
    },
    SettingsRow {
        label: "P2P playback",
        key: "p2pEnabled",
        options: &[],
    },
    SettingsRow {
        label: "HDR output",
        key: "hdrEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Anime Japanese audio",
        key: "animePreferJapaneseAudio",
        options: &[],
    },
    SettingsRow {
        label: "Anime skip integration",
        key: "useAnimeSkip",
        options: &[],
    },
    SettingsRow {
        label: "Chapter skip",
        key: "useChapterSkip",
        options: &[],
    },
    SettingsRow {
        label: "Skip segments",
        key: "useSkipSegments",
        options: &[],
    },
    SettingsRow {
        label: "Seek thumbnails",
        key: "seekThumbnailEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Hold to speed",
        key: "holdToSpeedEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Playback speed",
        key: "playbackSpeed",
        options: &["0.75", "1.0", "1.25", "1.5", "2.0"],
    },
    SettingsRow {
        label: "Seek interval",
        key: "seekSeconds",
        options: &["5", "10", "15", "30"],
    },
];
const APPEARANCE_SETTINGS: [SettingsRow; 35] = [
    SettingsRow {
        label: "Accent color",
        key: "accentColorArgb",
        options: &[
            "#FFFFFF", "#E50914", "#3F7CFF", "#54D17A", "#FF8A3D", "#C084FC",
        ],
    },
    SettingsRow {
        label: "UI scale",
        key: "uiScale",
        options: &[
            "75", "80", "85", "90", "95", "100", "105", "110", "115", "120", "125", "130", "135",
            "140", "145", "150",
        ],
    },
    SettingsRow {
        label: "Animations",
        key: "animationsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "GIF autoplay",
        key: "gifAutoplayEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Navigation layout",
        key: "navLayout",
        options: &["sidebar", "topbar"],
    },
    SettingsRow {
        label: "Navigation mode",
        key: "navMode",
        options: &["compact", "classic", "adaptive"],
    },
    SettingsRow {
        label: "Sidebar behavior",
        key: "navSidebarMode",
        options: &["hover", "expanded", "collapsed"],
    },
    SettingsRow {
        label: "Card corners",
        key: "cardCornerPreset",
        options: &["sharp", "classic", "soft", "rounded", "pill"],
    },
    SettingsRow {
        label: "Poster width",
        key: "posterWidthPreset",
        options: &["xsmall", "small", "medium", "large", "xlarge"],
    },
    SettingsRow {
        label: "Landscape posters",
        key: "posterLandscapeMode",
        options: &[],
    },
    SettingsRow {
        label: "Catalog type suffix",
        key: "catalogTypeSuffixEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Hide poster titles",
        key: "posterHideTitles",
        options: &[],
    },
    SettingsRow {
        label: "Reduced effects",
        key: "reducedEffects",
        options: &[],
    },
    SettingsRow {
        label: "Poster preview",
        key: "posterHoverPreview",
        options: &[],
    },
    SettingsRow {
        label: "Card layout",
        key: "cardLayout",
        options: &["vertical", "horizontal"],
    },
    SettingsRow {
        label: "Interface density",
        key: "interfaceDensity",
        options: &["small", "medium", "large"],
    },
    SettingsRow {
        label: "Continue Watching layout",
        key: "continueWatchingLayout",
        options: &["vertical", "horizontal", "inherit"],
    },
    SettingsRow {
        label: "Continue Watching artwork",
        key: "continueWatchingArtwork",
        options: &["episode", "poster", "background"],
    },
    SettingsRow {
        label: "Remaining time display",
        key: "continueWatchingRemainingFormat",
        options: &["time", "percent"],
    },
    SettingsRow {
        label: "Progress direction",
        key: "continueWatchingProgressDirection",
        options: &["remaining", "watched"],
    },
    SettingsRow {
        label: "Continue Watching shelf",
        key: "continueWatchingEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Hide Continue Watching titles",
        key: "continueWatchingHideTitles",
        options: &[],
    },
    SettingsRow {
        label: "Keep scheduled titles",
        key: "continueWatchingKeepScheduled",
        options: &[],
    },
    SettingsRow {
        label: "Show This Week",
        key: "continueWatchingShowThisWeek",
        options: &[],
    },
    SettingsRow {
        label: "Hero season posters",
        key: "homeSeasonPostersOnHero",
        options: &[],
    },
    SettingsRow {
        label: "Autoplay home hero trailer",
        key: "homeHeroAutoplayTrailer",
        options: &[],
    },
    SettingsRow {
        label: "Trailer on detail hero",
        key: "trailerOnHero",
        options: &[],
    },
    SettingsRow {
        label: "Autoplay detail hero trailer",
        key: "detailHeroAutoplayTrailer",
        options: &[],
    },
    SettingsRow {
        label: "Blur unwatched episodes",
        key: "blurUnwatchedEpisodes",
        options: &[],
    },
    SettingsRow {
        label: "Hide episode info spoilers",
        key: "spoilerHideEpisodeInfo",
        options: &[],
    },
    SettingsRow {
        label: "Detail hero season posters",
        key: "detailSeasonPostersOnHero",
        options: &[],
    },
    SettingsRow {
        label: "Season selector style",
        key: "detailSeasonSelectorMode",
        options: &["tabs", "slider", "compact"],
    },
    SettingsRow {
        label: "Episode card layout",
        key: "episodeCardsLayout",
        options: &["standard", "wide", "compact", "horizontal"],
    },
    SettingsRow {
        label: "Home trailer delay",
        key: "homeHeroAutoplayTrailerDelaySecs",
        options: &["2", "4", "6", "10"],
    },
    SettingsRow {
        label: "Detail trailer delay",
        key: "detailHeroAutoplayTrailerDelaySecs",
        options: &["2", "4", "6", "10"],
    },
];

fn settings_category_icon(
    assets: &impl HomeAssets,
    painter: &egui::Painter,
    center: Pos2,
    name: &str,
    color: Color32,
) {
    if let Some(icon) = assets.icon(name) {
        let rect = Rect::from_center_size(center, Vec2::splat(19.0));
        painter.image(
            icon,
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            color,
        );
    }
}

const APPEARANCE_GROUPS: [(usize, usize, &str); 5] = [
    (0, 4, "native.settings.group.color_and_motion"),
    (4, 8, "native.settings.group.navigation"),
    (8, 16, "native.settings.group.posters"),
    (16, 24, "native.settings.group.continue_watching"),
    (24, 35, "native.settings.group.home_and_detail"),
];
const GENERAL_GROUPS: [(usize, usize, &str); 4] = [
    (0, 2, "native.settings.group.general_startup"),
    (2, 8, "native.settings.group.general_behavior"),
    (8, 13, "native.settings.group.general_details"),
    (13, 15, "native.settings.group.general_calendar"),
];
const PLAYBACK_GROUPS: [(usize, usize, &str); 4] = [
    (0, 6, "native.settings.group.playback"),
    (6, 9, "native.settings.group.audio_subtitles"),
    (9, 12, "native.settings.group.sources_video"),
    (12, 19, "native.settings.group.skip_controls"),
];
const CONTENT_GROUPS: [(usize, usize, &str); 3] = [
    (0, 1, "native.settings.group.home"),
    (1, 8, "native.settings.group.metadata"),
    (8, 12, "native.settings.group.discovery"),
];
const DOWNLOAD_GROUPS: [(usize, usize, &str); 1] = [(0, 5, "native.settings.group.downloads")];
const APPEARANCE_PAGE_HEADER_HEIGHT: f32 = 66.0;
const APPEARANCE_GROUP_HEADING_HEIGHT: f32 = 22.0;
const APPEARANCE_GROUP_GAP: f32 = 22.0;

fn settings_group_card_height(row_count: usize, metrics: UiMetrics) -> f32 {
    metrics.settings_row_height
        + row_count.saturating_sub(1) as f32 * metrics.settings_row_spacing
        + metrics.settings_card_padding
}

fn settings_groups(section: &str) -> Option<&'static [(usize, usize, &'static str)]> {
    match section {
        "General" => Some(&GENERAL_GROUPS),
        "Appearance" => Some(&APPEARANCE_GROUPS),
        "Playback" => Some(&PLAYBACK_GROUPS),
        "Content" => Some(&CONTENT_GROUPS),
        "Downloads" => Some(&DOWNLOAD_GROUPS),
        _ => None,
    }
}

fn settings_groups_total_height(groups: &[(usize, usize, &str)], metrics: UiMetrics) -> f32 {
    APPEARANCE_PAGE_HEADER_HEIGHT
        + groups
            .iter()
            .map(|(start, end, _)| {
                APPEARANCE_GROUP_HEADING_HEIGHT
                    + settings_group_card_height(end - start, metrics)
                    + APPEARANCE_GROUP_GAP
            })
            .sum::<f32>()
}

fn settings_group_layout(
    index: usize,
    rect: Rect,
    metrics: UiMetrics,
    groups: &[(usize, usize, &str)],
) -> Option<(Rect, usize)> {
    for (group_index, (start, end, _)) in groups.iter().enumerate() {
        if (*start..*end).contains(&index) {
            let mut heading_y = rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT;
            for (previous_start, previous_end, _) in groups.iter().take(group_index) {
                heading_y += APPEARANCE_GROUP_HEADING_HEIGHT
                    + settings_group_card_height(previous_end - previous_start, metrics)
                    + APPEARANCE_GROUP_GAP;
            }
            let card_top = heading_y + APPEARANCE_GROUP_HEADING_HEIGHT;
            let card = Rect::from_min_size(
                Pos2::new(rect.left(), card_top),
                Vec2::new(
                    rect.width(),
                    settings_group_card_height(end - start, metrics),
                ),
            );
            return Some((card, index - start));
        }
    }
    None
}

pub(super) fn appearance_group_layout(
    index: usize,
    rect: Rect,
    metrics: UiMetrics,
) -> Option<(Rect, usize)> {
    settings_group_layout(index, rect, metrics, &APPEARANCE_GROUPS)
}
const CONTENT_SETTINGS: [SettingsRow; 12] = [
    SettingsRow {
        label: "Show hero section",
        key: "showHeroSection",
        options: &[],
    },
    SettingsRow {
        label: "TMDB artwork enrichment",
        key: "tmdbEnrichArtworkEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Prefer TMDB over add-ons",
        key: "tmdbPreferOverAddons",
        options: &[],
    },
    SettingsRow {
        label: "Show TMDB ratings",
        key: "tmdbRatingsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB cast images",
        key: "tmdbCastImagesEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB descriptions",
        key: "tmdbEnrichDescriptionEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB genres and keywords",
        key: "tmdbEnrichGenresKeywordsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB logos and backdrops",
        key: "tmdbLogosBackdropsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB recommendations",
        key: "tmdbRecommendationsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB similar titles",
        key: "tmdbSimilarResultsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB trailers",
        key: "tmdbTrailersEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB watch providers",
        key: "tmdbEnrichWatchProvidersEnabled",
        options: &[],
    },
];
const EMPTY_SETTINGS: [SettingsRow; 0] = [];
const STORAGE_SETTINGS: [SettingsRow; 5] = [
    SettingsRow {
        label: "Torrent speed",
        key: "torrentSpeedPreset",
        options: &["default", "fast", "ultra_fast"],
    },
    SettingsRow {
        label: "Torrent cache",
        key: "torrentCachePreset",
        options: &["auto", "2gb", "5gb", "10gb", "unlimited"],
    },
    SettingsRow {
        label: "Download source",
        key: "downloadSourceSelectionMode",
        options: &["manual", "first", "best", "regex"],
    },
    SettingsRow {
        label: "Download subtitle",
        key: "downloadSubtitleLanguage",
        options: &["off", "preferred"],
    },
    SettingsRow {
        label: "Torrent on Wi-Fi only",
        key: "torrentWifiOnly",
        options: &[],
    },
];

pub const SETTINGS_SECTIONS: [SettingsSection; 11] = [
    SettingsSection {
        title: "Account",
        description: "Profile and connected services",
        rows: &EMPTY_SETTINGS,
    },
    SettingsSection {
        title: "General",
        description: "How Fluxa starts and behaves",
        rows: &GENERAL_SETTINGS,
    },
    SettingsSection {
        title: "Appearance",
        description: "Visual preferences",
        rows: &APPEARANCE_SETTINGS,
    },
    SettingsSection {
        title: "Playback",
        description: "Defaults for watching content",
        rows: &PLAYBACK_SETTINGS,
    },
    SettingsSection {
        title: "Device",
        description: "Renderer and playback device",
        rows: &EMPTY_SETTINGS,
    },
    SettingsSection {
        title: "Shortcuts",
        description: "Keyboard shortcuts",
        rows: &EMPTY_SETTINGS,
    },
    SettingsSection {
        title: "Controller",
        description: "Remote and gamepad navigation",
        rows: &EMPTY_SETTINGS,
    },
    SettingsSection {
        title: "Content",
        description: "Catalog and metadata preferences",
        rows: &CONTENT_SETTINGS,
    },
    SettingsSection {
        title: "Add-ons",
        description: "Installed metadata and stream add-ons",
        rows: &EMPTY_SETTINGS,
    },
    SettingsSection {
        title: "Plugins",
        description: "Scraper repositories and plugins",
        rows: &EMPTY_SETTINGS,
    },
    SettingsSection {
        title: "Downloads",
        description: "Torrent and download defaults",
        rows: &STORAGE_SETTINGS,
    },
];

pub fn settings_row_by_index(index: usize) -> Option<&'static SettingsRow> {
    SETTINGS_SECTIONS
        .iter()
        .flat_map(|section| section.rows.iter())
        .nth(index)
}

pub(super) fn settings_card_height(
    viewport: Viewport,
    metrics: UiMetrics,
    section: &SettingsSection,
    settings: &SettingsModel,
) -> f32 {
    let base = if viewport.is_compact() {
        metrics.settings_card_height_mobile
    } else {
        metrics.settings_card_height_desktop
    };
    let row_content = || {
        metrics.settings_row_top
            + section.rows.len().saturating_sub(1) as f32 * metrics.settings_row_spacing
            + metrics.settings_row_height
            + metrics.settings_card_padding * 2.0
    };
    let line_spacing = if viewport.is_tv() {
        metrics.settings_extended_line_spacing_tv
    } else {
        metrics.settings_extended_line_spacing
    };
    let extended_lines = match section.title {
        "Account" => 9,
        "Device" => 6,
        "Shortcuts" => 10,
        "Controller" => 8,
        "Add-ons" => {
            1 + usize::from(settings.addon_error.is_some()) + settings.addons.len().min(6).max(1)
        }
        "Plugins" => {
            settings
                .plugins
                .get("repositories")
                .and_then(serde_json::Value::as_array)
                .map_or(0, |items| items.len().min(4))
                + settings
                    .plugins
                    .get("scrapers")
                    .and_then(serde_json::Value::as_array)
                    .map_or(0, |items| items.len().min(4))
                + 1
        }
        _ => 0,
    };
    let extended_controls = match section.title {
        "Add-ons" | "Plugins" => {
            metrics.settings_extended_input_height
                + metrics.settings_extended_action_gap
                + metrics.settings_extended_action_height
                + metrics.control_gap
        }
        _ => 0.0,
    };
    let extended_content = metrics.settings_extended_top
        + extended_lines as f32 * line_spacing
        + extended_controls
        + metrics.settings_card_padding * 2.0;
    let content_height = if section.rows.is_empty() {
        extended_content
    } else if let Some(groups) = settings_groups(section.title) {
        settings_groups_total_height(groups, metrics)
    } else {
        row_content()
    };
    base.max(content_height)
}

#[derive(Clone, Debug, Default)]
pub struct SettingsModel {
    pub values: serde_json::Value,
    pub last_write_error: Option<String>,
    pub active_section: usize,
    pub profile: serde_json::Value,
    pub addons: Vec<serde_json::Value>,
    pub addon_error: Option<String>,
    pub plugins: serde_json::Value,
    pub addon_url: String,
    pub plugin_url: String,
}

impl SettingsModel {
    fn value(&self, key: &str) -> Option<&serde_json::Value> {
        self.values
            .get(key)
            .filter(|value| !value.is_null())
            .or_else(|| setting_default(key))
    }

    pub fn bool_value(&self, key: &str) -> bool {
        self.value(key)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
    }

    pub fn display_value(&self, row: &SettingsRow) -> String {
        if row.options.is_empty() {
            if self.bool_value(row.key) {
                "On"
            } else {
                "Off"
            }
            .to_owned()
        } else {
            self.value(row.key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or(row.options[0])
                .replace('_', " ")
        }
    }

    pub fn next_value(&self, row: &SettingsRow) -> serde_json::Value {
        self.next_value_for(row, UiFormFactor::Mobile)
    }

    pub fn next_value_for(
        &self,
        row: &SettingsRow,
        form_factor: UiFormFactor,
    ) -> serde_json::Value {
        if row.options.is_empty() {
            serde_json::Value::Bool(!self.bool_value(row.key))
        } else {
            let options: Vec<&str> = row
                .options
                .iter()
                .copied()
                .filter(|option| {
                    !(row.key == "preferredPlayer"
                        && form_factor == UiFormFactor::Desktop
                        && *option == "exoplayer")
                })
                .collect();
            let current = self.value(row.key).and_then(serde_json::Value::as_str);
            let next = current
                .and_then(|value| options.iter().position(|option| *option == value))
                .map(|index| (index + 1) % options.len())
                .unwrap_or(1 % options.len());
            serde_json::Value::String(options[next].to_owned())
        }
    }
}

pub fn settings_model_from_core_snapshot(snapshot: &serde_json::Value) -> SettingsModel {
    let settings = snapshot.get("settings").unwrap_or(&serde_json::Value::Null);
    let addon_state = snapshot.get("addons").unwrap_or(&serde_json::Value::Null);
    SettingsModel {
        values: settings
            .get("values")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
        last_write_error: settings
            .get("lastWriteError")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        active_section: 0,
        profile: snapshot
            .pointer("/profile/active")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        addons: addon_state
            .get("installed")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default(),
        addon_error: addon_state
            .get("error")
            .and_then(|v| v.get("message"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        plugins: snapshot
            .get("plugins")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        addon_url: String::new(),
        plugin_url: String::new(),
    }
}

/// Pure geometry used by input adapters. It intentionally has no egui state
/// or asset dependency, so Android touch/D-pad and desktop mouse hit testing

fn settings_panel_line(
    painter: &egui::Painter,
    rect: Rect,
    y: f32,
    text: impl AsRef<str>,
    color: Color32,
    metrics: UiMetrics,
) {
    let text = text.as_ref();
    let font = FontId::proportional(metrics.screen_body_size);
    let row_left = rect.left() + metrics.settings_extended_line_inset;
    let row_right = rect.right() - metrics.settings_extended_line_inset;
    let baseline_y = y + metrics.settings_extended_line_spacing * 0.5;
    if let Some((label, value)) = text.split_once("  ·  ") {
        painter.text(
            Pos2::new(row_left + 2.0, baseline_y),
            Align2::LEFT_CENTER,
            truncate_to_width(painter, label, &font, (row_right - row_left) * 0.58),
            font.clone(),
            color,
        );
        let value_font = FontId::proportional(metrics.screen_card_subtitle_size);
        let value_width = painter
            .layout_no_wrap(value.to_owned(), value_font.clone(), Color32::WHITE)
            .size()
            .x
            .min((row_right - row_left) * 0.40);
        let chip = Rect::from_center_size(
            Pos2::new(row_right - value_width * 0.5 - 10.0, baseline_y),
            Vec2::new(value_width + 20.0, metrics.screen_control_height.min(30.0)),
        );
        painter.rect_filled(
            chip,
            metrics.screen_control_radius,
            Color32::from_rgb(35, 35, 35),
        );
        painter.rect_stroke(
            chip,
            metrics.screen_control_radius,
            egui::Stroke::new(1.0, Color32::from_white_alpha(18)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            chip.center(),
            Align2::CENTER_CENTER,
            truncate_to_width(painter, value, &value_font, chip.width() - 12.0),
            value_font,
            Color32::from_white_alpha(220),
        );
        painter.line_segment(
            [
                Pos2::new(row_left, y + metrics.settings_extended_line_spacing - 1.0),
                Pos2::new(row_right, y + metrics.settings_extended_line_spacing - 1.0),
            ],
            egui::Stroke::new(1.0, Color32::from_white_alpha(12)),
        );
    } else {
        painter.text(
            Pos2::new(row_left + 2.0, baseline_y),
            Align2::LEFT_CENTER,
            truncate_to_width(painter, text, &font, row_right - row_left - 4.0),
            font,
            color,
        );
    }
}

fn settings_panel_heading(
    painter: &egui::Painter,
    rect: Rect,
    y: f32,
    label: &str,
    metrics: UiMetrics,
) {
    painter.text(
        Pos2::new(
            rect.left() + metrics.settings_extended_line_inset + 2.0,
            y + metrics.settings_extended_line_spacing * 0.5,
        ),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(metrics.screen_card_subtitle_size),
        Color32::from_white_alpha(132),
    );
}

fn settings_panel_group(
    painter: &egui::Painter,
    outer: Rect,
    top: f32,
    height: f32,
    metrics: UiMetrics,
) -> Rect {
    let group = Rect::from_min_size(
        Pos2::new(outer.left() + 8.0, top),
        Vec2::new((outer.width() - 16.0).max(1.0), height.max(1.0)),
    );
    painter.rect_filled(group, metrics.card_radius, Color32::from_rgb(24, 24, 26));
    painter.rect_stroke(
        group,
        metrics.card_radius,
        egui::Stroke::new(1.0, Color32::from_white_alpha(16)),
        egui::StrokeKind::Inside,
    );
    group
}

pub(super) fn settings_row_label(setting: &SettingsRow, language: &str) -> String {
    let legacy_key = match setting.key {
        "autoSkipIntro" => "settings.auto_skip",
        "useAnimeSkip" => "settings.use_animeskip",
        "seekThumbnailEnabled" => "settings.seek_thumbnails",
        "holdToSpeedEnabled" => "settings.hold_to_speed",
        "playbackSpeed" => "auto.playback_speed",
        _ => "",
    };
    if !legacy_key.is_empty() {
        let translated = localized(legacy_key, language);
        if translated != legacy_key {
            return translated;
        }
    }

    let mut snake_case = String::with_capacity(setting.key.len() + 8);
    for character in setting.key.chars() {
        if character.is_ascii_uppercase() {
            snake_case.push('_');
            snake_case.push(character.to_ascii_lowercase());
        } else {
            snake_case.push(character);
        }
    }
    let legacy_key = format!("settings.{snake_case}");
    let translated = localized(&legacy_key, language);
    if translated != legacy_key {
        translated
    } else {
        // Some newer Core preferences have no legacy web translation yet.
        // Show the row's human-readable source label instead of leaking an i18n key.
        setting.label.to_owned()
    }
}

fn settings_panel_button(
    context: &egui::Context,
    layout: &mut HomeLayout,
    id: u64,
    rect: Rect,
    label: impl AsRef<str>,
    selected: bool,
    metrics: UiMetrics,
) {
    layout.focusable.push((id, rect));
    egui::Area::new(Id::new(("fluxa-settings-panel-action", id)))
        .fixed_pos(rect.min)
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let response = components::button(
                ui,
                label.as_ref(),
                rect.width(),
                rect.height(),
                if selected {
                    components::ButtonKind::Selected
                } else {
                    components::ButtonKind::Secondary
                },
                metrics,
            );
            if response.clicked() {
                layout.activated = Some(id);
            }
        });
}

fn settings_panel_input(
    context: &egui::Context,
    layout: &mut HomeLayout,
    id: u64,
    rect: Rect,
    value: &str,
    hint: impl AsRef<str>,
) {
    layout.focusable.push((id, rect));
    let mut next_value = value.to_owned();
    egui::Area::new(Id::new(("fluxa-settings-panel-input", id)))
        .fixed_pos(rect.min)
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let response = ui.add_sized(
                rect.size(),
                egui::TextEdit::singleline(&mut next_value).hint_text(hint.as_ref()),
            );
            if response.changed() {
                layout.text_input = Some(next_value.clone());
                layout.text_input_node = Some(id);
            }
        });
}

fn draw_settings_extended_section(
    context: &egui::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &impl HomeAssets,
    section: &str,
    language: &str,
    rect: Rect,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    let left = rect.left() + metrics.settings_card_padding;
    let right = rect.right() - metrics.settings_card_padding;
    let mut y = rect.top() + metrics.settings_extended_top;
    let line_gap = if viewport.is_tv() {
        metrics.settings_extended_line_spacing_tv
    } else {
        metrics.settings_extended_line_spacing
    };
    let muted = Color32::from_white_alpha(165);
    match section {
        "Account" => {
            settings_panel_heading(
                &painter,
                rect,
                y,
                &localized(
                    &format!("native.settings.section.{}", section.to_lowercase()),
                    language,
                )
                .to_uppercase(),
                metrics,
            );
            y += line_gap;
            let profile_name = settings
                .profile
                .get("name")
                .or_else(|| settings.profile.get("displayName"))
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| localized("native.settings.account_empty", language));
            settings_panel_line(
                &painter,
                rect,
                y,
                &format!("Profile  ·  {profile_name}"),
                Color32::WHITE,
                metrics,
            );
            y += line_gap;
            for (label, key) in [
                ("Stremio", "stremioAuthKey"),
                ("Nuvio", "nuvioAccessToken"),
                ("Trakt", "traktAccessToken"),
                ("Simkl", "simklAccessToken"),
                ("AniList", "anilistAccessToken"),
            ] {
                let connected = settings
                    .profile
                    .get(key)
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|v| !v.is_empty());
                let status = if connected {
                    "native.settings.connected"
                } else {
                    "native.settings.not_connected"
                };
                settings_panel_line(
                    &painter,
                    rect,
                    y,
                    &format!("{label}  ·  {}", localized(status, language)),
                    muted,
                    metrics,
                );
                y += line_gap;
            }
            for (label, key) in [
                ("Library source", "integrationLibrarySource"),
                ("Continue watching source", "continueWatchingSource"),
            ] {
                settings_panel_line(&painter, rect, y + line_gap * 0.5, label, muted, metrics);
                let mut choices = vec!["local"];
                for (source, token_key) in [
                    ("nuvio", "nuvioAccessToken"),
                    ("trakt", "traktAccessToken"),
                    ("simkl", "simklAccessToken"),
                    ("anilist", "anilistAccessToken"),
                    ("stremio", "stremioAuthKey"),
                ] {
                    if settings
                        .profile
                        .get(token_key)
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|token| !token.is_empty())
                    {
                        choices.push(source);
                    }
                }
                let mut selected = settings
                    .value(key)
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("local")
                    .to_owned();
                let value_rect = Rect::from_min_size(
                    Pos2::new(right - (right - left).min(240.0).max(140.0), y + 2.0),
                    Vec2::new(
                        (right - left).min(240.0).max(140.0),
                        (line_gap - 4.0).min(metrics.screen_control_height),
                    ),
                );
                egui::Area::new(Id::new(("fluxa-settings-account-choice", key)))
                    .fixed_pos(value_rect.min)
                    .order(egui::Order::Foreground)
                    .show(context, |ui| {
                        ui.set_min_size(value_rect.size());
                        ui.set_max_size(value_rect.size());
                        style_settings_dropdown(ui, metrics);
                        ui.spacing_mut().button_padding = Vec2::new(12.0, 7.0);
                        let chevron = assets.icon("ChevronDown");
                        let response = components::dropdown_frame()
                            .show(ui, |ui| {
                                egui::ComboBox::from_id_salt(("settings-account", key))
                                    .width(value_rect.width())
                                    .height(240.0)
                                    .popup_style(components::dropdown_popup_style(metrics))
                                    .selected_text(
                                        RichText::new(selected.replace('_', " "))
                                            .color(Color32::from_rgb(242, 243, 246))
                                            .size(metrics.settings_row_value_size_desktop)
                                            .strong(),
                                    )
                                    .icon(move |ui, rect, visuals, _| {
                                        if let Some(icon) = chevron {
                                            ui.painter().image(
                                                icon,
                                                rect.shrink(2.0),
                                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                                visuals.fg_stroke.color,
                                            );
                                        }
                                    })
                                    .show_ui(ui, |ui| {
                                        for choice in choices {
                                            ui.selectable_value(
                                                &mut selected,
                                                choice.to_owned(),
                                                localized(
                                                    &format!("native.settings.option.{choice}"),
                                                    language,
                                                ),
                                            );
                                        }
                                    })
                            })
                            .inner;
                        layout.focusable.push((
                            NODE_SETTINGS_ROW_BASE
                                + 10_000
                                + if key == "integrationLibrarySource" {
                                    0
                                } else {
                                    1
                                },
                            response.response.rect,
                        ));
                    });
                if settings.value(key).and_then(serde_json::Value::as_str)
                    != Some(selected.as_str())
                {
                    layout.setting_change =
                        Some((key.to_owned(), serde_json::Value::String(selected)));
                }
                y += line_gap;
            }
        }
        "Device" => {
            settings_panel_heading(
                &painter,
                rect,
                y,
                &localized(
                    &format!("native.settings.section.{}", section.to_lowercase()),
                    language,
                )
                .to_uppercase(),
                metrics,
            );
            y += line_gap;
            let form = match viewport.form_factor {
                UiFormFactor::Mobile => "Mobile",
                UiFormFactor::Tv => "TV",
                UiFormFactor::Desktop => "Desktop",
            };
            let rows = [
                ("native.settings.device_layout", form),
                ("native.settings.ui_renderer", "Fluxa Rust · egui · wgpu"),
                (
                    "native.settings.playback_engine",
                    settings
                        .value("playerEngine")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("mpv"),
                ),
                (
                    "native.settings.video_output",
                    settings
                        .value("renderBackend")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("Vulkan"),
                ),
                ("native.settings.device_reported_by_host", ""),
            ];
            let group =
                settings_panel_group(&painter, rect, y, rows.len() as f32 * line_gap, metrics);
            for (index, (label, value)) in rows.into_iter().enumerate() {
                let value = if value.is_empty() {
                    localized(label, language)
                } else {
                    value.to_owned()
                };
                settings_panel_line(
                    &painter,
                    group,
                    y + index as f32 * line_gap,
                    &format!("{}  ·  {value}", localized(label, language)),
                    muted,
                    metrics,
                );
            }
        }
        "Shortcuts" => {
            settings_panel_heading(
                &painter,
                rect,
                y,
                &localized(
                    &format!("native.settings.section.{}", section.to_lowercase()),
                    language,
                )
                .to_uppercase(),
                metrics,
            );
            y += line_gap;
            let rows = [
                ("native.settings.shortcut_navigation", "1 / 2 / 3 / 4 / 5"),
                ("native.settings.shortcut_search", "Ctrl + F"),
                ("native.settings.shortcut_back", "Backspace / Escape"),
                ("native.settings.shortcut_fullscreen", "F11"),
                ("native.settings.shortcut_play_pause", "K / Space"),
                ("native.settings.shortcut_seek", "← / →"),
                ("native.settings.shortcut_volume", "↓ / ↑"),
                ("native.settings.shortcut_mute", "M"),
            ];
            let group =
                settings_panel_group(&painter, rect, y, rows.len() as f32 * line_gap, metrics);
            for (index, (action, keys)) in rows.into_iter().enumerate() {
                settings_panel_line(
                    &painter,
                    group,
                    y + index as f32 * line_gap,
                    &format!("{}  ·  {keys}", localized(action, language)),
                    muted,
                    metrics,
                );
            }
            y += 8.0 * line_gap;
            settings_panel_line(
                &painter,
                rect,
                y,
                localized("native.settings.bindings_shared", language),
                Color32::from_white_alpha(115),
                metrics,
            );
        }
        "Controller" => {
            settings_panel_heading(
                &painter,
                rect,
                y,
                &localized(
                    &format!("native.settings.section.{}", section.to_lowercase()),
                    language,
                )
                .to_uppercase(),
                metrics,
            );
            y += line_gap;
            let rows = [
                ("native.settings.controller_navigate", "D-pad / left stick"),
                ("native.settings.controller_select", "Enter / A"),
                ("native.settings.controller_back", "Escape / Back / B"),
                ("native.settings.controller_scroll", "D-pad / left stick"),
                ("native.settings.controller_seek", "Left / Right"),
                ("native.settings.controller_volume", "Up / Down"),
            ];
            let group =
                settings_panel_group(&painter, rect, y, rows.len() as f32 * line_gap, metrics);
            for (index, (action, binding)) in rows.into_iter().enumerate() {
                settings_panel_line(
                    &painter,
                    group,
                    y + index as f32 * line_gap,
                    &format!("{}  ·  {binding}", localized(action, language)),
                    muted,
                    metrics,
                );
            }
            y += 6.0 * line_gap;
            settings_panel_line(
                &painter,
                rect,
                y,
                localized("native.settings.controller_host_details", language),
                Color32::from_white_alpha(115),
                metrics,
            );
        }
        "Add-ons" => {
            let input_rect = Rect::from_min_max(
                Pos2::new(left, y),
                Pos2::new(right, y + metrics.settings_extended_input_height),
            );
            settings_panel_input(
                context,
                layout,
                NODE_SETTINGS_ADDON_URL,
                input_rect,
                &settings.addon_url,
                localized("native.settings.addon_url", language),
            );
            y += metrics.settings_extended_input_height + metrics.settings_extended_action_gap;
            let button_width = metrics
                .settings_extended_addon_action_width
                .min(((right - left) - metrics.settings_extended_action_gap) / 2.0);
            settings_panel_button(
                context,
                layout,
                NODE_SETTINGS_ADDON_INSTALL,
                Rect::from_min_size(
                    Pos2::new(left, y),
                    Vec2::new(button_width, metrics.settings_extended_action_height),
                ),
                localized("native.settings.addon_install", language),
                false,
                metrics,
            );
            settings_panel_button(
                context,
                layout,
                NODE_SETTINGS_ADDON_REFRESH,
                Rect::from_min_size(
                    Pos2::new(
                        left + button_width + metrics.settings_extended_action_gap,
                        y,
                    ),
                    Vec2::new(button_width, metrics.settings_extended_action_height),
                ),
                localized("native.settings.addon_refresh", language),
                false,
                metrics,
            );
            y += metrics.settings_extended_action_height + metrics.control_gap;
            let installed_count = settings.addons.len().min(6).max(1);
            let installed_lines = installed_count + usize::from(settings.addon_error.is_some()) + 1;
            let group = settings_panel_group(
                &painter,
                rect,
                y,
                installed_lines as f32 * line_gap,
                metrics,
            );
            settings_panel_heading(
                &painter,
                group,
                y,
                &localized("native.settings.section.add-ons", language).to_uppercase(),
                metrics,
            );
            y += line_gap;
            if let Some(error) = settings.addon_error.as_deref() {
                settings_panel_line(
                    &painter,
                    group,
                    y,
                    error,
                    Color32::from_rgb(255, 125, 110),
                    metrics,
                );
                y += line_gap;
            }
            if settings.addons.is_empty() {
                settings_panel_line(
                    &painter,
                    group,
                    y,
                    localized("native.settings.addon_empty", language),
                    muted,
                    metrics,
                );
            } else {
                for addon in settings.addons.iter().take(6) {
                    let label = addon
                        .get("manifest")
                        .and_then(|m| m.get("name"))
                        .and_then(serde_json::Value::as_str)
                        .or_else(|| addon.get("name").and_then(serde_json::Value::as_str))
                        .or_else(|| {
                            addon
                                .get("transportUrl")
                                .and_then(serde_json::Value::as_str)
                        })
                        .unwrap_or("Installed add-on");
                    settings_panel_line(&painter, group, y, label, muted, metrics);
                    y += line_gap;
                }
            }
        }
        "Plugins" => {
            let input_rect = Rect::from_min_max(
                Pos2::new(left, y),
                Pos2::new(right, y + metrics.settings_extended_input_height),
            );
            settings_panel_input(
                context,
                layout,
                NODE_SETTINGS_PLUGIN_URL,
                input_rect,
                &settings.plugin_url,
                localized("native.settings.plugin_url", language),
            );
            y += metrics.settings_extended_input_height + metrics.settings_extended_action_gap;
            settings_panel_button(
                context,
                layout,
                NODE_SETTINGS_PLUGIN_INSTALL,
                Rect::from_min_size(
                    Pos2::new(left, y),
                    Vec2::new(
                        metrics.settings_extended_plugin_action_width,
                        metrics.settings_extended_action_height,
                    ),
                ),
                localized("native.settings.plugin_add", language),
                false,
                metrics,
            );
            y += metrics.settings_extended_action_height + metrics.control_gap * 2.0;
            let repos = settings
                .plugins
                .get("repositories")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            let scrapers = settings
                .plugins
                .get("scrapers")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            if let Some(error) = settings
                .plugins
                .get("error")
                .and_then(|v| v.get("message"))
                .and_then(serde_json::Value::as_str)
            {
                settings_panel_line(
                    &painter,
                    rect,
                    y,
                    error,
                    Color32::from_rgb(255, 125, 110),
                    metrics,
                );
                y += line_gap;
            }
            for (index, repo) in repos.iter().take(4).enumerate() {
                let name = repo
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("Plugin repository");
                let url = repo
                    .get("manifestUrl")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                settings_panel_line(
                    &painter,
                    rect,
                    y + 18.0,
                    &format!("{name}  ·  {url}"),
                    muted,
                    metrics,
                );
                settings_panel_button(
                    context,
                    layout,
                    NODE_SETTINGS_PLUGIN_REFRESH_BASE + index as u64,
                    Rect::from_min_size(
                        Pos2::new(
                            right
                                - metrics.settings_extended_repo_right_inset * 2.0
                                - metrics.settings_extended_action_gap,
                            y,
                        ),
                        Vec2::new(
                            metrics.settings_extended_small_action_width,
                            metrics.settings_extended_small_action_height,
                        ),
                    ),
                    localized("native.settings.plugin_refresh", language),
                    false,
                    metrics,
                );
                settings_panel_button(
                    context,
                    layout,
                    NODE_SETTINGS_PLUGIN_REPOSITORY_BASE + index as u64,
                    Rect::from_min_size(
                        Pos2::new(right - metrics.settings_extended_repo_right_inset, y),
                        Vec2::new(
                            metrics.settings_extended_small_action_width,
                            metrics.settings_extended_small_action_height,
                        ),
                    ),
                    localized("native.settings.plugin_remove", language),
                    false,
                    metrics,
                );
                y += line_gap + metrics.control_gap * 0.75;
            }
            for (index, scraper) in scrapers.iter().take(4).enumerate() {
                let name = scraper
                    .get("name")
                    .or_else(|| scraper.get("id"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("Scraper");
                let enabled = scraper
                    .get("enabled")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                settings_panel_line(
                    &painter,
                    rect,
                    y + metrics.settings_extended_action_height / 2.0,
                    name,
                    muted,
                    metrics,
                );
                settings_panel_button(
                    context,
                    layout,
                    NODE_SETTINGS_PLUGIN_SCRAPER_BASE + index as u64,
                    Rect::from_min_size(
                        Pos2::new(right - metrics.settings_extended_scraper_right_inset, y),
                        Vec2::new(
                            metrics.settings_extended_scraper_action_width,
                            metrics.settings_extended_small_action_height,
                        ),
                    ),
                    localized(
                        if enabled {
                            "native.settings.plugin_enabled"
                        } else {
                            "native.settings.plugin_disabled"
                        },
                        language,
                    ),
                    enabled,
                    metrics,
                );
                y += line_gap + metrics.control_gap * 0.75;
            }
            if repos.is_empty() && scrapers.is_empty() {
                settings_panel_line(
                    &painter,
                    rect,
                    y,
                    localized("native.settings.plugin_empty", language),
                    muted,
                    metrics,
                );
            }
        }
        _ => {}
    }
}

pub fn draw_settings(
    context: &egui::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let language = settings
        .values
        .get("language")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("en");
    let mut metrics = UiMetrics::for_viewport(viewport);
    metrics.accent = settings
        .value("accentColorArgb")
        .and_then(serde_json::Value::as_str)
        .and_then(parse_settings_color)
        .unwrap_or(Color32::WHITE);
    metrics.accent_foreground = if accent_needs_dark_foreground(metrics.accent) {
        Color32::from_rgb(20, 20, 22)
    } else {
        Color32::WHITE
    };
    let scroll_y = resolve_screen_scroll(
        context,
        viewport,
        Id::new("fluxa-screen-scroll-settings"),
        settings_scroll_max(viewport, settings),
    );
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    painter.rect_filled(screen, 0.0, metrics.background);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 4, assets);
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let desktop = !compact && !tv;
    let margin = if compact {
        metrics.page_padding
    } else if tv {
        metrics.screen_padding.max(32.0)
    } else {
        metrics
            .settings_screen_padding_desktop
            .max(metrics.screen_padding)
    };
    let top = if compact {
        metrics.detail_header_top_mobile
    } else if desktop {
        margin
    } else {
        metrics.content_header_top
    };
    let mut header_bottom = top;
    if !desktop {
        egui::Area::new(Id::new("fluxa-shared-settings-header"))
            .fixed_pos(Pos2::new(margin, top))
            .show(context, |ui| {
                ui.label(
                    RichText::new(localized("nav.settings", language))
                        .size(if compact {
                            metrics.screen_title_size_mobile
                        } else if tv {
                            metrics.screen_title_size_tv
                        } else {
                            metrics.screen_title_size
                        })
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(metrics.control_gap);
                ui.label(
                    RichText::new(localized("native.settings.description", language))
                        .size(if tv {
                            metrics.screen_body_size_tv
                        } else {
                            metrics.screen_body_size
                        })
                        .color(Color32::from_white_alpha(165)),
                );
                ui.add_space(metrics.section_gap);
                let back = components::button_auto_width(
                    ui,
                    &format!("‹  {}", localized("common.back", language)),
                    components::ButtonKind::Secondary,
                    metrics.nav_label_size + 2.0,
                    metrics,
                );
                layout.focusable.push((NODE_SETTINGS_BACK, back.rect));
                if back.clicked() {
                    layout.activated = Some(NODE_SETTINGS_BACK);
                }
                header_bottom = ui.min_rect().bottom();
            });
    }
    let active_section = settings.active_section.min(SETTINGS_SECTIONS.len() - 1);
    let section_top = if desktop {
        top
    } else {
        header_bottom + metrics.section_gap
    };
    let content_x = if compact {
        margin
    } else {
        margin + metrics.settings_content_offset
    };
    let content_width = (viewport.width - content_x - margin)
        .min(if compact || tv {
            f32::MAX
        } else {
            metrics.settings_content_max_width_desktop
        })
        .max(1.0);
    if compact {
        let nav_rect = Rect::from_min_size(
            Pos2::new(margin, section_top - scroll_y),
            Vec2::new(
                (viewport.width - margin * 2.0).max(1.0),
                metrics.screen_control_height,
            ),
        );
        egui::Area::new(Id::new("fluxa-settings-category-strip"))
            .fixed_pos(nav_rect.min)
            .show(context, |ui| {
                egui::ScrollArea::horizontal()
                    .id_salt("fluxa-settings-category-scroll")
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (index, section) in SETTINGS_SECTIONS.iter().enumerate() {
                                let node = NODE_SETTINGS_SECTION_BASE + index as u64;
                                let label = localized(
                                    &format!(
                                        "native.settings.section.{}",
                                        section.title.to_lowercase()
                                    ),
                                    language,
                                );
                                let response = components::button_auto_width(
                                    ui,
                                    &label,
                                    if index == active_section {
                                        components::ButtonKind::Selected
                                    } else {
                                        components::ButtonKind::Secondary
                                    },
                                    metrics.nav_label_size,
                                    metrics,
                                );
                                if response.rect.intersects(nav_rect) {
                                    layout
                                        .focusable
                                        .push((node, response.rect.intersect(nav_rect)));
                                }
                                if response.clicked() {
                                    layout.activated = Some(node);
                                }
                            }
                        });
                    });
            });
    } else {
        let nav_width = metrics
            .settings_nav_width
            .min((viewport.width - margin * 2.0).max(1.0) * 0.32);
        let nav_item_height = if tv {
            metrics.screen_control_height
        } else {
            metrics.settings_nav_item_height_desktop
        };
        let nav_item_gap = if tv {
            metrics.control_gap * 0.75
        } else {
            metrics.settings_nav_item_gap_desktop
        };
        let nav_panel = Rect::from_min_size(
            Pos2::new(margin, section_top - scroll_y),
            Vec2::new(
                nav_width,
                (viewport.height - section_top + scroll_y - margin).max(nav_item_height),
            ),
        );
        painter.rect_filled(
            nav_panel,
            metrics.card_radius,
            if desktop {
                Color32::from_rgb(10, 10, 10)
            } else {
                metrics.surface
            },
        );
        painter.rect_stroke(
            nav_panel,
            metrics.card_radius,
            egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
            egui::StrokeKind::Inside,
        );
        if desktop {
            painter.text(
                nav_panel.left_top() + Vec2::new(metrics.control_gap + 2.0, 18.0),
                Align2::LEFT_TOP,
                localized("nav.settings", language),
                FontId::proportional(metrics.settings_section_title_size_desktop + 2.0),
                Color32::WHITE,
            );
        }
        let nav_top_inset = if desktop { 76.0 } else { metrics.control_gap };
        for (index, section) in SETTINGS_SECTIONS.iter().enumerate() {
            let node = NODE_SETTINGS_SECTION_BASE + index as u64;
            let rect = Rect::from_min_size(
                Pos2::new(
                    margin + metrics.control_gap,
                    section_top + nav_top_inset + index as f32 * (nav_item_height + nav_item_gap)
                        - scroll_y,
                ),
                Vec2::new(nav_width - metrics.control_gap * 2.0, nav_item_height),
            );
            layout.focusable.push((node, rect));
            let mut clicked = false;
            egui::Area::new(Id::new(("fluxa-settings-category", node)))
                .fixed_pos(rect.min)
                .show(context, |ui| {
                    let (item_rect, response) = ui.allocate_exact_size(rect.size(), Sense::click());
                    if index == active_section {
                        ui.painter().rect_filled(
                            item_rect,
                            metrics.screen_control_radius,
                            Color32::from_white_alpha(12),
                        );
                        ui.painter().rect_filled(
                            Rect::from_min_max(
                                item_rect.left_top() + Vec2::new(1.0, 9.0),
                                Pos2::new(item_rect.left() + 4.0, item_rect.bottom() - 9.0),
                            ),
                            2.0,
                            metrics.accent,
                        );
                    }
                    let icon_name = [
                        "Account",
                        "General",
                        "Appearance",
                        "Playback",
                        "Device",
                        "Shortcuts",
                        "Controller",
                        "Content",
                        "Add-ons",
                        "Plugins",
                        "Downloads",
                    ][index];
                    settings_category_icon(
                        assets,
                        ui.painter(),
                        item_rect.left_center() + Vec2::new(22.0, 0.0),
                        icon_name,
                        if index == active_section {
                            Color32::WHITE
                        } else {
                            Color32::from_white_alpha(150)
                        },
                    );
                    ui.painter().text(
                        item_rect.left_center() + Vec2::new(42.0, 0.0),
                        Align2::LEFT_CENTER,
                        localized(
                            &format!("native.settings.section.{}", section.title.to_lowercase()),
                            language,
                        ),
                        FontId::proportional(metrics.nav_label_size + if tv { 0.0 } else { 2.0 }),
                        if index == active_section {
                            Color32::WHITE
                        } else {
                            Color32::from_white_alpha(185)
                        },
                    );
                    clicked = response.clicked();
                });
            if clicked {
                layout.activated = Some(node);
            }
        }
    }
    let nav_bottom = if compact {
        section_top + metrics.screen_control_height + metrics.section_gap
    } else {
        section_top
    };
    let card_top = nav_bottom - scroll_y;
    let card_width = content_width;
    let section = &SETTINGS_SECTIONS[active_section];
    let card_height = settings_card_height(viewport, metrics, section, settings);
    let rect = Rect::from_min_size(
        Pos2::new(content_x, card_top),
        Vec2::new(card_width, card_height),
    );
    if let Some(groups) = settings_groups(section.title) {
        let mut heading_y = rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT;
        for &(start, end, title_key) in groups {
            painter.text(
                Pos2::new(rect.left() + 8.0, heading_y),
                Align2::LEFT_TOP,
                localized(title_key, language).to_uppercase(),
                FontId::proportional(metrics.screen_card_subtitle_size),
                Color32::from_white_alpha(145),
            );
            let group_rect = Rect::from_min_size(
                Pos2::new(rect.left(), heading_y + APPEARANCE_GROUP_HEADING_HEIGHT),
                Vec2::new(
                    rect.width(),
                    settings_group_card_height(end - start, metrics),
                ),
            );
            painter.rect_filled(
                group_rect,
                metrics.card_radius,
                Color32::from_rgb(19, 19, 19),
            );
            painter.rect_stroke(
                group_rect,
                metrics.card_radius,
                egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
                egui::StrokeKind::Inside,
            );
            heading_y = group_rect.bottom() + APPEARANCE_GROUP_GAP;
        }
    } else {
        painter.rect_filled(
            rect,
            metrics.card_radius,
            if desktop {
                Color32::from_rgb(16, 16, 16)
            } else {
                Color32::from_rgb(18, 19, 24)
            },
        );
        painter.rect_stroke(
            rect,
            metrics.card_radius,
            egui::Stroke::new(1.0, Color32::from_white_alpha(18)),
            egui::StrokeKind::Inside,
        );
    }
    painter.text(
        rect.left_top() + Vec2::new(metrics.settings_card_padding, metrics.settings_title_top),
        Align2::LEFT_TOP,
        localized(
            &format!("native.settings.section.{}", section.title.to_lowercase()),
            language,
        ),
        FontId::proportional(if tv {
            metrics.screen_section_title_size_tv - metrics.control_gap
        } else if !compact {
            metrics.settings_section_title_size_desktop
        } else {
            metrics.screen_section_title_size
        }),
        Color32::WHITE,
    );
    if !desktop {
        painter.text(
            rect.left_top()
                + Vec2::new(
                    metrics.settings_card_padding,
                    metrics.settings_description_top,
                ),
            Align2::LEFT_TOP,
            localized(
                &format!(
                    "native.settings.section.{}.description",
                    section.title.to_lowercase()
                ),
                language,
            ),
            FontId::proportional(metrics.screen_card_subtitle_size),
            Color32::from_white_alpha(130),
        );
    }
    let preceding_rows: usize = SETTINGS_SECTIONS[..active_section]
        .iter()
        .map(|section| section.rows.len())
        .sum();
    draw_settings_extended_section(
        context,
        viewport,
        settings,
        assets,
        &section.title,
        language,
        rect,
        metrics,
        &mut layout,
    );
    for (row_index, setting) in section.rows.iter().enumerate() {
        let node = NODE_SETTINGS_ROW_BASE + (preceding_rows + row_index) as u64;
        let (row_rect, group_row_index) = if let Some(groups) = settings_groups(section.title) {
            let (group_rect, group_row_index) =
                settings_group_layout(row_index, rect, metrics, groups)
                    .expect("settings row group");
            (
                Rect::from_min_size(
                    group_rect.left_top()
                        + Vec2::new(
                            metrics.settings_row_inset,
                            6.0 + group_row_index as f32 * metrics.settings_row_spacing,
                        ),
                    Vec2::new(
                        group_rect.width() - metrics.settings_row_inset * 2.0,
                        metrics.settings_row_height,
                    ),
                ),
                Some(group_row_index),
            )
        } else {
            (
                Rect::from_min_size(
                    rect.left_top()
                        + Vec2::new(
                            metrics.settings_row_inset,
                            metrics.settings_row_top
                                + row_index as f32 * metrics.settings_row_spacing,
                        ),
                    Vec2::new(
                        rect.width() - metrics.settings_row_inset * 2.0,
                        metrics.settings_row_height,
                    ),
                ),
                None,
            )
        };
        if group_row_index.unwrap_or(row_index) > 0 {
            let y =
                row_rect.top() - (metrics.settings_row_spacing - metrics.settings_row_height) * 0.5;
            painter.line_segment(
                [
                    Pos2::new(row_rect.left(), y),
                    Pos2::new(row_rect.right(), y),
                ],
                egui::Stroke::new(1.0, Color32::from_white_alpha(12)),
            );
        }
        let is_toggle = setting.options.is_empty();
        let mut clicked = false;
        if is_toggle {
            layout.focusable.push((node, row_rect));
            egui::Area::new(Id::new(("fluxa-settings-row", node)))
                .fixed_pos(row_rect.min)
                .order(egui::Order::Foreground)
                .show(context, |ui| {
                    let (_, response) = ui.allocate_exact_size(row_rect.size(), Sense::click());
                    clicked = response.clicked();
                });
        }
        let row_label_size = if !compact && !tv {
            metrics.settings_row_label_size_desktop
        } else {
            metrics.nav_label_size
        };
        painter.text(
            row_rect.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            truncate_to_width(
                &painter,
                &settings_row_label(setting, language),
                &FontId::proportional(row_label_size),
                row_rect.width() * if is_toggle { 0.72 } else { 0.55 },
            ),
            FontId::proportional(row_label_size),
            Color32::from_white_alpha(210),
        );
        if !is_toggle {
            let raw_value = settings
                .value(setting.key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or(setting.options[0]);
            let value = if setting.key == "uiScale" {
                format!("{raw_value}%")
            } else if matches!(
                setting.key,
                "homeHeroAutoplayTrailerDelaySecs" | "detailHeroAutoplayTrailerDelaySecs"
            ) {
                format!("{raw_value}s")
            } else if setting.key == "accentColorArgb" {
                let color_key = match raw_value {
                    "#FFFFFF" => Some("auto.white"),
                    "#E50914" => Some("auto.red"),
                    "#3F7CFF" => Some("auto.blue"),
                    "#54D17A" => Some("auto.green"),
                    "#FF8A3D" => Some("auto.orange"),
                    "#C084FC" => Some("auto.purple"),
                    _ => None,
                };
                color_key
                    .map(|key| localized(key, language))
                    .unwrap_or_else(|| raw_value.to_owned())
            } else {
                localized(&format!("native.settings.option.{raw_value}"), language)
            };
            let value_size = if !compact && !tv {
                metrics.settings_row_value_size_desktop
            } else {
                metrics.screen_card_subtitle_size
            };
            let value_width =
                (value.chars().count() as f32 * value_size * 0.56 + 28.0).clamp(92.0, 188.0);
            let value_rect = Rect::from_center_size(
                row_rect.right_center() - Vec2::new(value_width * 0.5 + 2.0, 0.0),
                Vec2::new(
                    value_width,
                    metrics.screen_control_height.min(row_rect.height() - 4.0),
                ),
            );
            let selected_text = if setting.key == "accentColorArgb" {
                format!("●  {value}")
            } else {
                value
            };
            let mut selected = raw_value.to_owned();
            egui::Area::new(Id::new(("fluxa-settings-dropdown", node)))
                .fixed_pos(value_rect.min)
                .order(egui::Order::Foreground)
                .show(context, |ui| {
                    ui.set_min_size(value_rect.size());
                    ui.set_max_size(value_rect.size());
                    style_settings_dropdown(ui, metrics);
                    ui.spacing_mut().button_padding = Vec2::new(12.0, 7.0);
                    let chevron = assets.icon("ChevronDown");
                    let response = components::dropdown_frame()
                        .show(ui, |ui| {
                            egui::ComboBox::from_id_salt(("settings-choice", node))
                                .width(value_width)
                                .height(240.0)
                                .popup_style(components::dropdown_popup_style(metrics))
                                .selected_text(
                                    RichText::new(selected_text)
                                        .color(Color32::from_rgb(242, 243, 246))
                                        .size(value_size)
                                        .strong(),
                                )
                                .icon(move |ui, rect, visuals, _| {
                                    if let Some(icon) = chevron {
                                        ui.painter().image(
                                            icon,
                                            rect.shrink(2.0),
                                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                            visuals.fg_stroke.color,
                                        );
                                    }
                                })
                                .show_ui(ui, |ui| {
                                    for option in setting.options {
                                        let label = if setting.key == "accentColorArgb" {
                                            let key = match *option {
                                                "#FFFFFF" => "auto.white",
                                                "#E50914" => "auto.red",
                                                "#3F7CFF" => "auto.blue",
                                                "#54D17A" => "auto.green",
                                                "#FF8A3D" => "auto.orange",
                                                "#C084FC" => "auto.purple",
                                                _ => "",
                                            };
                                            if key.is_empty() {
                                                (*option).to_owned()
                                            } else {
                                                localized(key, language)
                                            }
                                        } else if setting.key == "uiScale" {
                                            format!("{option}%")
                                        } else if matches!(
                                            setting.key,
                                            "homeHeroAutoplayTrailerDelaySecs"
                                                | "detailHeroAutoplayTrailerDelaySecs"
                                        ) {
                                            format!("{option}s")
                                        } else {
                                            localized(
                                                &format!("native.settings.option.{}", option),
                                                language,
                                            )
                                        };
                                        ui.selectable_value(
                                            &mut selected,
                                            (*option).to_owned(),
                                            label,
                                        );
                                    }
                                })
                        })
                        .inner;
                    layout.focusable.push((node, response.response.rect));
                });
            if selected != raw_value {
                layout.setting_change =
                    Some((setting.key.to_owned(), serde_json::Value::String(selected)));
            }
        } else {
            let enabled = settings.bool_value(setting.key);
            let track_rect = Rect::from_center_size(
                row_rect.right_center() - Vec2::new(metrics.settings_toggle_width * 0.5 + 3.0, 0.0),
                Vec2::new(
                    metrics.settings_toggle_width,
                    metrics.settings_toggle_height,
                ),
            );
            painter.rect_filled(
                track_rect,
                metrics.settings_toggle_height / 2.0,
                if enabled {
                    metrics.accent
                } else {
                    Color32::from_rgb(54, 56, 59)
                },
            );
            let knob_x = if enabled {
                track_rect.right() - metrics.settings_toggle_knob_radius - 2.0
            } else {
                track_rect.left() + metrics.settings_toggle_knob_radius + 2.0
            };
            let knob_color = if enabled && accent_needs_dark_foreground(metrics.accent) {
                Color32::from_rgb(24, 25, 28)
            } else if enabled {
                Color32::WHITE
            } else {
                Color32::from_rgb(168, 172, 178)
            };
            painter.circle_filled(
                Pos2::new(knob_x, track_rect.center().y),
                metrics
                    .settings_toggle_knob_radius
                    .min(metrics.settings_toggle_height * 0.5 - 2.0),
                knob_color,
            );
        }
        if clicked {
            layout.activated = Some(node);
        }
    }
    if let Some(error) = settings.last_write_error.as_deref() {
        painter.text(
            Pos2::new(margin, viewport.height - 24.0),
            Align2::LEFT_BOTTOM,
            error,
            FontId::proportional(13.0),
            Color32::from_rgb(255, 145, 125),
        );
    }
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    if let Some((_, rect)) = layout.focusable.iter().find(|(id, _)| Some(*id) == focused) {
        painter.rect_stroke(
            rect.expand(metrics.focus_ring_expand),
            metrics.focus_ring_radius,
            egui::Stroke::new(metrics.focus_ring_width, Color32::WHITE),
            egui::StrokeKind::Outside,
        );
    }
    layout
}

pub(super) fn parse_settings_color(value: &str) -> Option<Color32> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 {
        return None;
    }
    let rgb = u32::from_str_radix(hex, 16).ok()?;
    Some(Color32::from_rgb(
        ((rgb >> 16) & 0xff) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
    ))
}

fn style_settings_dropdown(ui: &mut egui::Ui, metrics: UiMetrics) {
    let dropdown_surface = Color32::from_rgb(60, 60, 60);
    let visuals = ui.visuals_mut();
    visuals.window_fill = dropdown_surface;
    visuals.widgets.noninteractive.bg_fill = dropdown_surface;
    visuals.widgets.inactive.bg_fill = dropdown_surface;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(43, 45, 53);
    visuals.widgets.active.bg_fill = Color32::from_rgb(49, 51, 60);
    visuals.widgets.open.bg_fill = dropdown_surface;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Color32::from_white_alpha(28));
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, Color32::from_white_alpha(58));
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, Color32::from_white_alpha(72));
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, metrics.accent);
    visuals.widgets.open.fg_stroke.color = Color32::from_rgb(242, 243, 246);
    visuals.selection.bg_fill = metrics.accent;
    visuals.selection.stroke.color = metrics.accent_foreground;
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(9);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(9);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(9);
    visuals.widgets.open.corner_radius = egui::CornerRadius::same(9);
    ui.spacing_mut().icon_width = 18.0;
    ui.spacing_mut().icon_spacing = 8.0;
}

fn accent_needs_dark_foreground(color: Color32) -> bool {
    let linear = |channel: u8| {
        let value = channel as f32 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b()) > 0.55
}
