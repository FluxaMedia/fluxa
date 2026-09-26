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

const GENERAL_SETTINGS: [SettingsRow; 10] = [
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
        label: "Discord Rich Presence",
        key: "discordRichPresenceEnabled",
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
        label: "Content warnings",
        key: "contentWarningsEnabled",
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
const PLAYBACK_SETTINGS: [SettingsRow; 44] = [
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
    SettingsRow {
        label: "Anime upscaling",
        key: "animeUpscalingMode",
        options: &["off", "auto"],
    },
    SettingsRow {
        label: "Anime4K mode",
        key: "animeUpscalingModePreset",
        options: &["a", "b", "c"],
    },
    SettingsRow {
        label: "Anime upscaling quality",
        key: "animeUpscalingQuality",
        options: &["anime4k_s", "anime4k_m", "anime4k_l"],
    },
    SettingsRow {
        label: "Frame interpolation",
        key: "frameInterpolationMode",
        options: &["off", "display_resample", "smooth"],
    },
    SettingsRow {
        label: "Hold speed",
        key: "holdSpeed",
        options: &["1.25", "1.5", "1.75", "2.0", "2.5", "3.0"],
    },
    SettingsRow {
        label: "Countdown duration",
        key: "autoPlayCountdownSecs",
        options: &["5", "7", "10", "15"],
    },
    SettingsRow {
        label: "Try binge group",
        key: "tryBingeGroup",
        options: &[],
    },
    SettingsRow {
        label: "Audio processing",
        key: "audioProcessingMode",
        options: &["reference", "balanced", "night"],
    },
    SettingsRow {
        label: "Secondary audio",
        key: "secondaryAudioLanguage",
        options: &["none", "en", "tr"],
    },
    SettingsRow {
        label: "Secondary subtitle",
        key: "secondarySubtitleLanguage",
        options: &["none", "en", "tr"],
    },
    SettingsRow {
        label: "Buffer cache",
        key: "playerBufferCacheMb",
        options: &["100", "500", "1000", "2000", "-1"],
    },
    SettingsRow {
        label: "Forward buffer",
        key: "playerForwardBufferSeconds",
        options: &["30", "60", "120", "300", "600"],
    },
    SettingsRow {
        label: "Back buffer",
        key: "playerBackBufferSeconds",
        options: &["0", "15", "30", "60", "120", "300"],
    },
    SettingsRow {
        label: "Subtitle position",
        key: "subtitlePosition",
        options: &["100", "90", "80", "70"],
    },
    SettingsRow {
        label: "Subtitle size",
        key: "subtitleSize",
        options: &["50", "75", "100", "125", "150", "200"],
    },
    SettingsRow {
        label: "Subtitle text",
        key: "subtitleColor",
        options: &["#FFFFFF", "#000000", "#FFE45C", "#FF5D5D", "#3F7CFF", "#54D17A", "#FF8A3D", "#C084FC"],
    },
    SettingsRow {
        label: "Text transparency",
        key: "subtitleTextOpacity",
        options: &["1.0", "0.75", "0.5", "0.25", "0.0"],
    },
    SettingsRow {
        label: "Subtitle background",
        key: "subtitleBackgroundColor",
        options: &["#000000", "#FFFFFF", "#FFE45C", "#FF5D5D", "#3F7CFF"],
    },
    SettingsRow {
        label: "Background transparency",
        key: "subtitleBackgroundOpacity",
        options: &["1.0", "0.75", "0.5", "0.25", "0.0"],
    },
    SettingsRow {
        label: "Subtitle outline",
        key: "subtitleOutlineColor",
        options: &["#000000", "#FFFFFF", "#FFE45C", "#FF5D5D", "#3F7CFF"],
    },
    SettingsRow {
        label: "Outline transparency",
        key: "subtitleOutlineOpacity",
        options: &["1.0", "0.75", "0.5", "0.25", "0.0"],
    },
    SettingsRow {
        label: "Outline size",
        key: "subtitleOutlineSize",
        options: &["0", "1", "2", "3", "4", "5"],
    },
    SettingsRow {
        label: "Bold subtitles",
        key: "subtitleBold",
        options: &[],
    },
    SettingsRow {
        label: "Subtitle shadow",
        key: "subtitleShadow",
        options: &[],
    },
    SettingsRow {
        label: "Force subtitle style",
        key: "subtitleForceStyle",
        options: &[],
    },
];
const APPEARANCE_SETTINGS: [SettingsRow; 42] = [
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
        label: "AMOLED black",
        key: "amoledMode",
        options: &[],
    },
    SettingsRow {
        label: "Animations",
        key: "animationsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Reduced effects",
        key: "reducedEffects",
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
        label: "Interface density",
        key: "interfaceDensity",
        options: &["small", "medium", "large"],
    },
    SettingsRow {
        label: "Card corners",
        key: "cardCornerPreset",
        options: &["sharp", "classic", "soft", "rounded", "pill"],
    },
    SettingsRow {
        label: "Card layout",
        key: "cardLayout",
        options: &["vertical", "horizontal"],
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
        label: "Hide poster titles",
        key: "posterHideTitles",
        options: &[],
    },
    SettingsRow {
        label: "Poster preview",
        key: "posterHoverPreview",
        options: &[],
    },
    SettingsRow {
        label: "Catalog type suffix",
        key: "catalogTypeSuffixEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Continue Watching shelf",
        key: "continueWatchingEnabled",
        options: &[],
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
        label: "Show hero section",
        key: "showHeroSection",
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
        label: "Home trailer delay",
        key: "homeHeroAutoplayTrailerDelaySecs",
        options: &["2", "4", "6", "10"],
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
        label: "Detail hero season posters",
        key: "detailSeasonPostersOnHero",
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
        label: "Detail trailer delay",
        key: "detailHeroAutoplayTrailerDelaySecs",
        options: &["2", "4", "6", "10"],
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
        label: "Blur unwatched episodes",
        key: "blurUnwatchedEpisodes",
        options: &[],
    },
    SettingsRow {
        label: "Hide episode info spoilers",
        key: "spoilerHideEpisodeInfo",
        options: &[],
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

const APPEARANCE_GROUPS: [(usize, usize, &str); 6] = [
    (0, 6, "settings.group.color_and_motion"),
    (6, 10, "settings.group.navigation"),
    (10, 17, "settings.group.cards"),
    (17, 25, "settings.group.continue_watching"),
    (25, 29, "settings.group.home"),
    (29, 42, "settings.group.detail_page"),
];
const GENERAL_GROUPS: [(usize, usize, &str); 3] = [
    (0, 2, "settings.group.general_startup"),
    (2, 8, "settings.group.general_behavior"),
    (8, 10, "settings.group.general_calendar"),
];
const PLAYBACK_GROUPS: [(usize, usize, &str); 7] = [
    (0, 6, "settings.group.playback"),
    (6, 9, "settings.group.audio_subtitles"),
    (9, 12, "settings.group.sources_video"),
    (12, 19, "settings.group.skip_controls"),
    (19, 23, "settings.group.upscaling_motion"),
    (23, 32, "settings.group.player_advanced"),
    (32, 44, "settings.group.subtitle_style"),
];
const CONTENT_GROUPS: [(usize, usize, &str); 2] = [
    (0, 14, "settings.group.metadata"),
    (14, 18, "settings.group.discovery"),
];
const POSTER_GROUPS: [(usize, usize, &str); 3] = [
    (0, 4, "settings.group.poster_overlays"),
    (4, 7, "settings.group.poster_rating"),
    (7, 16, "settings.group.poster_badges"),
];
const DOWNLOAD_GROUPS: [(usize, usize, &str); 1] = [(0, 5, "settings.group.downloads")];
const APPEARANCE_PAGE_HEADER_HEIGHT: f32 = 66.0;
const APPEARANCE_GROUP_HEADING_HEIGHT: f32 = 22.0;
const APPEARANCE_GROUP_GAP: f32 = 22.0;
const FIELD_HEIGHT: f32 = 118.0;

fn settings_group_card_height(row_count: usize, metrics: UiMetrics) -> f32 {
    metrics.settings_row_height
        + row_count.saturating_sub(1) as f32 * metrics.settings_row_spacing
        + metrics.settings_card_padding
}

fn settings_groups(section: &str) -> Option<&'static [(usize, usize, &'static str)]> {
    match section {
        "General" => Some(&GENERAL_GROUPS),
        "Appearance" => Some(&APPEARANCE_GROUPS),
        "Posters" => Some(&POSTER_GROUPS),
        "Playback" => Some(&PLAYBACK_GROUPS),
        "Content" => Some(&CONTENT_GROUPS),
        "Downloads" => Some(&DOWNLOAD_GROUPS),
        _ => None,
    }
}

fn section_label(title: &str, language: &str) -> String {
    localized(&format!("settings.section.{}", title.to_lowercase()), language)
}

pub(super) fn visible_groups(settings: &SettingsModel) -> Vec<(String, Vec<usize>)> {
    let language = settings.language();
    let query = settings.search.trim().to_lowercase();
    let mut offset = 0;
    let mut groups = Vec::new();
    if !query.is_empty() {
        for section in &SETTINGS_SECTIONS {
            let rows: Vec<usize> = section
                .rows
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    settings_row_label(row, language).to_lowercase().contains(&query)
                        || row.label.to_lowercase().contains(&query)
                })
                .map(|(index, _)| offset + index)
                .collect();
            if !rows.is_empty() {
                groups.push((section_label(section.title, language), rows));
            }
            offset += section.rows.len();
        }
        return groups;
    }
    let active = settings.active_section.min(SETTINGS_SECTIONS.len() - 1);
    offset = SETTINGS_SECTIONS[..active]
        .iter()
        .map(|section| section.rows.len())
        .sum();
    for &(start, end, key) in settings_groups(SETTINGS_SECTIONS[active].title).unwrap_or_default() {
        groups.push((localized(key, language), (offset + start..offset + end).collect()));
    }
    groups
}

pub(super) fn group_cards(
    rect: Rect,
    groups: &[(String, Vec<usize>)],
    metrics: UiMetrics,
) -> Vec<Rect> {
    let mut top = rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT;
    groups
        .iter()
        .map(|(_, rows)| {
            let card = Rect::from_min_size(
                Pos2::new(rect.left(), top + APPEARANCE_GROUP_HEADING_HEIGHT),
                Vec2::new(rect.width(), settings_group_card_height(rows.len(), metrics)),
            );
            top = card.bottom() + APPEARANCE_GROUP_GAP;
            card
        })
        .collect()
}

fn groups_height(groups: &[(String, Vec<usize>)], metrics: UiMetrics) -> f32 {
    APPEARANCE_PAGE_HEADER_HEIGHT
        + groups
            .iter()
            .map(|(_, rows)| {
                APPEARANCE_GROUP_HEADING_HEIGHT
                    + settings_group_card_height(rows.len(), metrics)
                    + APPEARANCE_GROUP_GAP
            })
            .sum::<f32>()
}
const CONTENT_SETTINGS: [SettingsRow; 18] = [
    SettingsRow {
        label: "TMDB artwork enrichment",
        key: "tmdbEnrichArtworkEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB logos and backdrops",
        key: "tmdbLogosBackdropsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Prefer TMDB over add-ons",
        key: "tmdbPreferOverAddons",
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
        label: "TMDB cast and crew",
        key: "tmdbEnrichCastCrewEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB cast images",
        key: "tmdbCastImagesEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB episode stills",
        key: "tmdbEpisodeImagesEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB trailers",
        key: "tmdbTrailersEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB collections",
        key: "tmdbCollectionInfoEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB networks",
        key: "tmdbEnrichNetworkEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB status and schedule",
        key: "tmdbEnrichStatusScheduleEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB original titles",
        key: "tmdbEnrichOriginTitlesEnabled",
        options: &[],
    },
    SettingsRow {
        label: "TMDB watch providers",
        key: "tmdbEnrichWatchProvidersEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Show TMDB ratings",
        key: "tmdbRatingsEnabled",
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
        label: "Trakt comments",
        key: "traktCommentsEnabled",
        options: &[],
    },
];
const POSTER_SETTINGS: [SettingsRow; 16] = [
    SettingsRow {
        label: "Poster overlays",
        key: "posterOverlaysEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Badge size",
        key: "posterBadgeSize",
        options: &["default", "small", "large"],
    },
    SettingsRow {
        label: "Tinted fade",
        key: "posterFadeTint",
        options: &[],
    },
    SettingsRow {
        label: "Fade strength",
        key: "posterFadeStrength",
        options: &["high", "medium", "low"],
    },
    SettingsRow {
        label: "Rating badge",
        key: "posterRatingBadge",
        options: &[],
    },
    SettingsRow {
        label: "Rating source",
        key: "posterRatingSource",
        options: &["imdb", "mdblist"],
    },
    SettingsRow {
        label: "Rating position",
        key: "posterRatingPosition",
        options: &[
            "bar",
            "number",
            "minimal",
            "frosted",
            "top_left",
            "top_right",
            "bottom_left",
            "bottom_right",
        ],
    },
    SettingsRow {
        label: "Release status badge",
        key: "posterStatusBadge",
        options: &[],
    },
    SettingsRow {
        label: "Status position",
        key: "posterStatusPosition",
        options: &["sash", "banner", "top_left", "top_right", "bottom_left", "bottom_right"],
    },
    SettingsRow {
        label: "Watched badge",
        key: "posterWatchedBadge",
        options: &[],
    },
    SettingsRow {
        label: "Progress bar",
        key: "posterProgressBar",
        options: &[],
    },
    SettingsRow {
        label: "Library badge",
        key: "posterSavedBadge",
        options: &[],
    },
    SettingsRow {
        label: "Trending sash",
        key: "posterTrendingBadge",
        options: &[],
    },
    SettingsRow {
        label: "Age rating",
        key: "posterAgeRating",
        options: &[],
    },
    SettingsRow {
        label: "Quality badges",
        key: "posterQualityBadges",
        options: &[],
    },
    SettingsRow {
        label: "Award sash",
        key: "posterAwardBadge",
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

pub const SETTINGS_SECTIONS: [SettingsSection; 10] = [
    SettingsSection {
        title: "Account",
        description: "Profile, connected services and API keys",
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
        title: "Posters",
        description: "Badges drawn on top of poster artwork",
        rows: &POSTER_SETTINGS,
    },
    SettingsSection {
        title: "Playback",
        description: "Defaults for watching content",
        rows: &PLAYBACK_SETTINGS,
    },
    SettingsSection {
        title: "Content",
        description: "Catalog and metadata preferences",
        rows: &CONTENT_SETTINGS,
    },
    SettingsSection {
        title: "Downloads",
        description: "Torrent and download defaults",
        rows: &STORAGE_SETTINGS,
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
        title: "Shortcuts",
        description: "Keyboard and controller bindings",
        rows: &EMPTY_SETTINGS,
    },
];

pub fn settings_row_by_index(index: usize) -> Option<&'static SettingsRow> {
    SETTINGS_SECTIONS
        .iter()
        .flat_map(|section| section.rows.iter())
        .nth(index)
}

const KEYBOARD_SHORTCUTS: [(&str, &str); 8] = [
    ("settings.shortcut_navigation", "1 / 2 / 3 / 4 / 5"),
    ("settings.shortcut_search", "Ctrl + F"),
    ("settings.shortcut_back", "Backspace / Escape"),
    ("settings.shortcut_fullscreen", "F11"),
    ("settings.shortcut_play_pause", "K / Space"),
    ("settings.shortcut_seek", "← / →"),
    ("settings.shortcut_volume", "↓ / ↑"),
    ("settings.shortcut_mute", "M"),
];
const CONTROLLER_BINDINGS: [(&str, &str); 6] = [
    ("settings.controller_navigate", "D-pad / left stick"),
    ("settings.controller_select", "Enter / A"),
    ("settings.controller_back", "Escape / Back / B"),
    ("settings.controller_scroll", "D-pad / left stick"),
    ("settings.controller_seek", "Left / Right"),
    ("settings.controller_volume", "Up / Down"),
];

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
    let line_spacing = if viewport.is_tv() {
        metrics.settings_extended_line_spacing_tv
    } else {
        metrics.settings_extended_line_spacing
    };
    let groups = visible_groups(settings);
    if !settings.search.trim().is_empty() {
        return base.max(groups_height(&groups, metrics));
    }
    let content_height = match section.title {
        "Account" => account_height(metrics),
        "Shortcuts" => {
            APPEARANCE_PAGE_HEADER_HEIGHT
                + (APPEARANCE_GROUP_HEADING_HEIGHT + APPEARANCE_GROUP_GAP) * 2.0
                + settings_group_card_height(KEYBOARD_SHORTCUTS.len(), metrics)
                + settings_group_card_height(CONTROLLER_BINDINGS.len(), metrics)
        }
        "Posters" => {
            groups_height(&groups, metrics)
                + APPEARANCE_GROUP_HEADING_HEIGHT
                + FIELD_HEIGHT
                + APPEARANCE_GROUP_GAP
        }
        "Add-ons" | "Plugins" => {
            let lines = if section.title == "Add-ons" {
                1 + usize::from(settings.addon_error.is_some()) + settings.addons.len().min(6).max(1)
            } else {
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
            };
            metrics.settings_extended_top
                + lines as f32 * line_spacing
                + metrics.settings_extended_input_height
                + metrics.settings_extended_action_gap
                + metrics.settings_extended_action_height
                + metrics.control_gap
                + metrics.settings_card_padding * 2.0
        }
        _ => groups_height(&groups, metrics),
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
    pub poster_fields: [String; 3],
    pub search: String,
}

impl SettingsModel {
    fn language(&self) -> &str {
        self.values
            .get("language")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("en")
    }
    fn value(&self, key: &str) -> Option<&serde_json::Value> {
        self.values
            .get(key)
            .filter(|value| !value.is_null())
            .or_else(|| setting_default(key))
            .or_else(|| poster_overlay::setting_default(key))
    }

    pub fn ui_scale(&self) -> f32 {
        self.value("uiScale")
            .and_then(|value| {
                value
                    .as_f64()
                    .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
            })
            .map(|percent| (percent as f32 / 100.0).clamp(0.5, 2.0))
            .unwrap_or(1.0)
    }

    pub fn number_value(&self, key: &str) -> Option<f64> {
        self.value(key).and_then(|value| {
            value
                .as_f64()
                .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
        })
    }

    pub fn str_value(&self, key: &str) -> Option<&str> {
        self.value(key).and_then(serde_json::Value::as_str)
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
        poster_fields: POSTER_FIELDS.map(|field| {
            settings
                .pointer(&format!("/values/{}", field.key))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_owned()
        }),
        search: String::new(),
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

fn color_name(value: &str) -> Option<&'static str> {
    Some(match value {
        "#FFFFFF" => "auto.white",
        "#000000" => "auto.black",
        "#E50914" | "#FF5D5D" => "auto.red",
        "#3F7CFF" => "auto.blue",
        "#54D17A" => "auto.green",
        "#FF8A3D" => "auto.orange",
        "#C084FC" => "auto.purple",
        "#FFE45C" => "auto.yellow",
        _ => return None,
    })
}

fn option_label(key: &str, value: &str, language: &str) -> String {
    let named = match (key, value) {
        (_, "off") => Some("settings.off"),
        ("animeUpscalingMode", "auto") => Some("settings.auto"),
        ("animeUpscalingQuality", _) => Some(match value {
            "anime4k_s" => "settings.anime4k_s",
            "anime4k_l" => "settings.anime4k_l",
            _ => "settings.anime4k_m",
        }),
        ("animeUpscalingModePreset", "b") => Some("player.anime4k_mode_b"),
        ("animeUpscalingModePreset", "c") => Some("player.anime4k_mode_c"),
        ("animeUpscalingModePreset", _) => Some("player.anime4k_mode_a"),
        ("frameInterpolationMode", "display_resample") => {
            Some("settings.frame_interpolation_display_resample")
        }
        ("frameInterpolationMode", "smooth") => Some("settings.frame_interpolation_smooth"),
        ("audioProcessingMode", _) => Some(match value {
            "balanced" => "settings.audio_processing_balanced",
            "night" => "settings.audio_processing_night",
            _ => "settings.audio_processing_reference",
        }),
        ("playerBufferCacheMb", "-1") => Some("settings.buffer_cache_infinite"),
        ("subtitlePosition", _) => Some(match value {
            "90" => "settings.subtitle_position_low",
            "80" => "settings.subtitle_position_middle",
            "70" => "settings.subtitle_position_high",
            _ => "settings.subtitle_position_bottom",
        }),
        ("accentColorArgb" | "subtitleColor" | "subtitleBackgroundColor" | "subtitleOutlineColor", _) => {
            color_name(value)
        }
        _ => None,
    };
    if let Some(named) = named {
        return localized(named, language);
    }
    match key {
        "uiScale" | "subtitleSize" => format!("{value}%"),
        "subtitleTextOpacity" | "subtitleBackgroundOpacity" | "subtitleOutlineOpacity" => {
            format!("{}%", (value.parse::<f64>().unwrap_or(1.0) * 100.0).round())
        }
        "homeHeroAutoplayTrailerDelaySecs"
        | "detailHeroAutoplayTrailerDelaySecs"
        | "autoPlayCountdownSecs"
        | "playerForwardBufferSeconds"
        | "playerBackBufferSeconds" => format!("{value}s"),
        "holdSpeed" => format!("{value}x"),
        "playerBufferCacheMb" => format!("{value} MB"),
        _ => localized_or(&format!("settings.option.{value}"), value, language),
    }
}

pub(super) fn settings_row_label(setting: &SettingsRow, language: &str) -> String {
    let legacy_key = match setting.key {
        "autoSkipIntro" => "settings.auto_skip",
        "useAnimeSkip" => "settings.use_animeskip",
        "seekThumbnailEnabled" => "settings.seek_thumbnails",
        "holdToSpeedEnabled" => "settings.hold_to_speed",
        "playbackSpeed" => "auto.playback_speed",
        "discordRichPresenceEnabled" => "settings.discord_rich_presence_enable",
        "animeUpscalingMode" => "settings.anime_upscaling",
        "animeUpscalingModePreset" => "player.anime4k",
        "frameInterpolationMode" => "settings.frame_interpolation",
        "autoPlayCountdownSecs" => "settings.auto_play_countdown",
        "playerBufferCacheMb" => "settings.buffer_cache",
        "playerForwardBufferSeconds" => "settings.forward_buffer",
        "playerBackBufferSeconds" => "settings.back_buffer",
        "subtitleColor" => "settings.subtitle_text",
        "subtitleBackgroundColor" => "settings.subtitle_background",
        "subtitleBackgroundOpacity" => "auto.background_transparency",
        "subtitleOutlineColor" => "settings.subtitle_outline",
        "subtitleOutlineOpacity" => "settings.subtitle.outline_opacity",
        "tmdbCollectionInfoEnabled" => "settings.tmdb_enrich_collection",
        "tmdbEnrichCastCrewEnabled" => "settings.tmdb_enrich_cast_crew",
        "tmdbEnrichNetworkEnabled" => "settings.tmdb_enrich_network",
        "tmdbEnrichOriginTitlesEnabled" => "settings.tmdb_enrich_origin_titles",
        "tmdbEnrichStatusScheduleEnabled" => "settings.tmdb_enrich_status_schedule",
        "tmdbEpisodeImagesEnabled" => "settings.tmdb_enrich_episode_stills",
        "traktCommentsEnabled" => "settings.trakt_comments",
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
    metrics: UiMetrics,
) {
    let mut next_value = value.to_owned();
    egui::Area::new(Id::new(("fluxa-settings-panel-input", id)))
        .fixed_pos(rect.min)
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let response = components::text_field(
                ui,
                &mut next_value,
                hint.as_ref(),
                rect.size(),
                metrics.settings_row_value_size_desktop,
            );
            layout.focusable.push((id, response.rect));
            if response.changed() {
                layout.text_input = Some(next_value.clone());
                layout.text_input_node = Some(id);
            }
        });
}

fn pill_button(
    context: &egui::Context,
    layout: &mut HomeLayout,
    id: u64,
    rect: Rect,
    label: &str,
    size: f32,
) {
    layout.focusable.push((id, rect));
    egui::Area::new(Id::new(("fluxa-settings-pill", id)))
        .fixed_pos(rect.min)
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let (area, response) = ui.allocate_exact_size(rect.size(), Sense::click());
            ui.painter().rect_filled(
                area,
                area.height() * 0.5,
                Color32::from_white_alpha(if response.hovered() { 34 } else { 20 }),
            );
            ui.painter().text(
                area.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(size),
                Color32::WHITE,
            );
            if response.clicked() {
                layout.activated = Some(id);
            }
        });
}

pub struct PosterField {
    pub key: &'static str,
    pub label: &'static str,
    help: &'static str,
    hint: &'static str,
    pub input: u64,
    pub save: u64,
    pub clear: u64,
}

pub const POSTER_FIELDS: [PosterField; 3] = [
    PosterField {
        key: "posterUrlTemplate",
        label: "settings.poster_url_template",
        help: "settings.poster_url_template_help",
        hint: "settings.poster_url_template_hint",
        input: NODE_SETTINGS_POSTER_URL,
        save: NODE_SETTINGS_POSTER_URL_SAVE,
        clear: NODE_SETTINGS_POSTER_URL_CLEAR,
    },
    PosterField {
        key: "tmdbApiKey",
        label: "settings.tmdb_api_key",
        help: "settings.poster_tmdb_help",
        hint: "settings.tmdb_api_key_placeholder",
        input: NODE_SETTINGS_POSTER_TMDB_KEY,
        save: NODE_SETTINGS_POSTER_TMDB_KEY_SAVE,
        clear: NODE_SETTINGS_POSTER_TMDB_KEY_CLEAR,
    },
    PosterField {
        key: "mdblistApiKey",
        label: "settings.mdblist_api_key",
        help: "settings.poster_mdblist_help",
        hint: "settings.mdblist_api_key_placeholder",
        input: NODE_SETTINGS_POSTER_MDBLIST_KEY,
        save: NODE_SETTINGS_POSTER_MDBLIST_KEY_SAVE,
        clear: NODE_SETTINGS_POSTER_MDBLIST_KEY_CLEAR,
    },
];

pub fn poster_field(input: u64) -> Option<usize> {
    POSTER_FIELDS.iter().position(|field| field.input == input)
}

fn draw_field(
    context: &egui::Context,
    settings: &SettingsModel,
    index: usize,
    area: Rect,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let language = settings.language();
    let field = &POSTER_FIELDS[index];
    let painter = context.layer_painter(egui::LayerId::background());
    let label_size = metrics.settings_row_label_size_desktop;
    let inner = area.shrink2(Vec2::new(metrics.settings_row_inset + 4.0, 16.0));
    painter.text(
        inner.left_top(),
        Align2::LEFT_TOP,
        localized(field.label, language),
        FontId::proportional(label_size),
        Color32::from_white_alpha(230),
    );
    let help_font = FontId::proportional(metrics.screen_card_subtitle_size);
    painter.text(
        inner.left_top() + Vec2::new(0.0, label_size + 6.0),
        Align2::LEFT_TOP,
        truncate_to_width(&painter, &localized(field.help, language), &help_font, inner.width()),
        help_font,
        Color32::from_white_alpha(130),
    );
    let height = 36.0;
    let button_width = 84.0;
    let gap = 8.0;
    let row_top = inner.bottom() - height;
    let input = Rect::from_min_max(
        Pos2::new(inner.left(), row_top),
        Pos2::new(inner.right() - (button_width + gap) * 2.0, inner.bottom()),
    );
    settings_panel_input(
        context,
        layout,
        field.input,
        input,
        &settings.poster_fields[index],
        localized(field.hint, language),
        metrics,
    );
    for (slot, (node, label)) in [
        (field.save, "settings.poster_url_save"),
        (field.clear, "settings.poster_url_clear"),
    ]
    .into_iter()
    .enumerate()
    {
        pill_button(
            context,
            layout,
            node,
            Rect::from_min_size(
                Pos2::new(input.right() + gap + slot as f32 * (button_width + gap), row_top),
                Vec2::new(button_width, height),
            ),
            &localized(label, language),
            label_size - 1.0,
        );
    }
}

fn draw_field_group(
    context: &egui::Context,
    settings: &SettingsModel,
    rect: Rect,
    top: f32,
    title: &str,
    fields: &[usize],
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) -> Rect {
    let painter = context.layer_painter(egui::LayerId::background());
    let card = account_group(&painter, rect, top, fields.len() as f32 * FIELD_HEIGHT, title);
    for (slot, &index) in fields.iter().enumerate() {
        let area = Rect::from_min_size(
            card.left_top() + Vec2::new(0.0, slot as f32 * FIELD_HEIGHT),
            Vec2::new(card.width(), FIELD_HEIGHT),
        );
        if slot > 0 {
            painter.line_segment(
                [
                    Pos2::new(area.left() + metrics.settings_row_inset, area.top()),
                    Pos2::new(area.right() - metrics.settings_row_inset, area.top()),
                ],
                egui::Stroke::new(1.0, Color32::from_white_alpha(12)),
            );
        }
        draw_field(context, settings, index, area, metrics, layout);
    }
    card
}

fn draw_binding_group(
    painter: &egui::Painter,
    rect: Rect,
    top: f32,
    title: &str,
    rows: &[(&str, &str)],
    language: &str,
    metrics: UiMetrics,
) -> Rect {
    let card = account_group(
        painter,
        rect,
        top,
        settings_group_card_height(rows.len(), metrics),
        title,
    );
    let label_font = FontId::proportional(metrics.settings_row_label_size_desktop);
    let value_font = FontId::proportional(metrics.settings_row_value_size_desktop);
    for (index, (action, keys)) in rows.iter().enumerate() {
        let row = account_row(card, index, metrics);
        if index > 0 {
            account_divider(painter, row, metrics);
        }
        painter.text(
            row.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            localized(action, language),
            label_font.clone(),
            Color32::from_white_alpha(210),
        );
        painter.text(
            row.right_center() - Vec2::new(6.0, 0.0),
            Align2::RIGHT_CENTER,
            *keys,
            value_font.clone(),
            Color32::from_white_alpha(150),
        );
    }
    card
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
        "Account" => draw_account(context, settings, assets, language, rect, metrics, layout),
        "Shortcuts" => {
            let card = draw_binding_group(
                &painter,
                rect,
                rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT,
                &localized("settings.group.keyboard", language),
                &KEYBOARD_SHORTCUTS,
                language,
                metrics,
            );
            draw_binding_group(
                &painter,
                rect,
                card.bottom() + APPEARANCE_GROUP_GAP,
                &localized("settings.group.controller", language),
                &CONTROLLER_BINDINGS,
                language,
                metrics,
            );
        }
        "Posters" => {
            let top = group_cards(rect, &visible_groups(settings), metrics)
                .last()
                .map_or(rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT, |card| {
                    card.bottom() + APPEARANCE_GROUP_GAP
                });
            draw_field_group(
                context,
                settings,
                rect,
                top,
                &localized("settings.group.custom_poster", language),
                &[0],
                metrics,
                layout,
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
                localized("settings.addon_url", language),
                metrics,
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
                localized("settings.addon_install", language),
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
                localized("settings.addon_refresh", language),
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
                &localized("settings.section.add-ons", language).to_uppercase(),
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
                    localized("settings.addon_empty", language),
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
                localized("settings.plugin_url", language),
                metrics,
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
                localized("settings.plugin_add", language),
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
                    localized("settings.plugin_refresh", language),
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
                    localized("settings.plugin_remove", language),
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
                            "settings.plugin_enabled"
                        } else {
                            "settings.plugin_disabled"
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
                    localized("settings.plugin_empty", language),
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
    paint_ambient(&painter, screen, assets);
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
        margin.max(76.0)
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
                    RichText::new(localized("settings.description", language))
                        .size(if tv {
                            metrics.screen_body_size_tv
                        } else {
                            metrics.screen_body_size
                        })
                        .color(Color32::from_white_alpha(165)),
                );
                ui.add_space(metrics.section_gap);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = metrics.control_gap;
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
                    let switch = components::button_auto_width(
                        ui,
                        &localized("settings.switch_profiles", language),
                        components::ButtonKind::Secondary,
                        metrics.nav_label_size + 2.0,
                        metrics,
                    );
                    layout.focusable.push((NODE_SETTINGS_SWITCH_PROFILE, switch.rect));
                    if switch.clicked() {
                        layout.activated = Some(NODE_SETTINGS_SWITCH_PROFILE);
                    }
                });
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
    let searching = !settings.search.trim().is_empty();
    let search_height = metrics.screen_control_height;
    let search_field = |layout: &mut HomeLayout, rect: Rect| {
        egui::Area::new(Id::new("fluxa-settings-search"))
            .fixed_pos(rect.min)
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                let mut query = settings.search.clone();
                let response = components::search_field(
                    ui,
                    &mut query,
                    &localized("settings.search_placeholder", language),
                    rect.width(),
                    rect.height(),
                    metrics,
                );
                layout.focusable.push((NODE_SETTINGS_SEARCH, response.rect));
                if response.changed() {
                    layout.text_input = Some(query);
                    layout.text_input_node = Some(NODE_SETTINGS_SEARCH);
                }
            });
    };
    if compact {
        search_field(
            &mut layout,
            Rect::from_min_size(
                Pos2::new(margin, section_top - scroll_y),
                Vec2::new((viewport.width - margin * 2.0).max(1.0), search_height),
            ),
        );
        let nav_rect = Rect::from_min_size(
            Pos2::new(
                margin,
                section_top + search_height + metrics.control_gap - scroll_y,
            ),
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
                                        "settings.section.{}",
                                        section.title.to_lowercase()
                                    ),
                                    language,
                                );
                                let response = components::button_auto_width(
                                    ui,
                                    &label,
                                    if index == active_section && !searching {
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
        let search_top = if desktop { 58.0 } else { metrics.control_gap };
        search_field(
            &mut layout,
            Rect::from_min_size(
                Pos2::new(margin + metrics.control_gap, section_top + search_top - scroll_y),
                Vec2::new(nav_width - metrics.control_gap * 2.0, search_height),
            ),
        );
        let nav_top_inset = search_top + search_height + metrics.control_gap * 1.5;
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
                    let active = index == active_section && !searching;
                    if active {
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
                    settings_category_icon(
                        assets,
                        ui.painter(),
                        item_rect.left_center() + Vec2::new(22.0, 0.0),
                        section.title,
                        if active {
                            Color32::WHITE
                        } else {
                            Color32::from_white_alpha(150)
                        },
                    );
                    ui.painter().text(
                        item_rect.left_center() + Vec2::new(42.0, 0.0),
                        Align2::LEFT_CENTER,
                        localized(
                            &format!("settings.section.{}", section.title.to_lowercase()),
                            language,
                        ),
                        FontId::proportional(metrics.nav_label_size + if tv { 0.0 } else { 2.0 }),
                        if active {
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
        section_top
            + search_height
            + metrics.control_gap
            + metrics.screen_control_height
            + metrics.section_gap
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
    let groups = visible_groups(settings);
    let cards = group_cards(rect, &groups, metrics);
    for ((title, _), card) in groups.iter().zip(&cards) {
        painter.text(
            Pos2::new(rect.left() + 8.0, card.top() - APPEARANCE_GROUP_HEADING_HEIGHT),
            Align2::LEFT_TOP,
            title.to_uppercase(),
            FontId::proportional(metrics.screen_card_subtitle_size),
            Color32::from_white_alpha(145),
        );
        painter.rect_filled(*card, metrics.card_radius, Color32::from_rgb(19, 19, 19));
        painter.rect_stroke(
            *card,
            metrics.card_radius,
            egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
            egui::StrokeKind::Inside,
        );
    }
    if !searching && matches!(section.title, "Add-ons" | "Plugins") {
        painter.rect_filled(rect, metrics.card_radius, Color32::from_rgb(19, 19, 19));
        painter.rect_stroke(
            rect,
            metrics.card_radius,
            egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
            egui::StrokeKind::Inside,
        );
    }
    let heading = if !searching {
        section_label(section.title, language)
    } else if groups.is_empty() {
        localized("settings.search_no_results", language)
    } else {
        localized("settings.search_results", language)
    };
    painter.text(
        rect.left_top() + Vec2::new(metrics.settings_card_padding, metrics.settings_title_top),
        Align2::LEFT_TOP,
        heading,
        FontId::proportional(if tv {
            metrics.screen_section_title_size_tv - metrics.control_gap
        } else if !compact {
            metrics.settings_section_title_size_desktop
        } else {
            metrics.screen_section_title_size
        }),
        Color32::WHITE,
    );
    if !desktop && !searching {
        painter.text(
            rect.left_top()
                + Vec2::new(
                    metrics.settings_card_padding,
                    metrics.settings_description_top,
                ),
            Align2::LEFT_TOP,
            localized(
                &format!(
                    "settings.section.{}.description",
                    section.title.to_lowercase()
                ),
                language,
            ),
            FontId::proportional(metrics.screen_card_subtitle_size),
            Color32::from_white_alpha(130),
        );
    }
    if !searching {
        draw_settings_extended_section(
            context,
            viewport,
            settings,
            assets,
            section.title,
            language,
            rect,
            metrics,
            &mut layout,
        );
    }
    let rows = groups.iter().zip(&cards).flat_map(|((_, rows), card)| {
        rows.iter().enumerate().map(move |(slot, &index)| (index, slot, *card))
    });
    for (index, slot, card) in rows {
        let Some(setting) = settings_row_by_index(index) else {
            continue;
        };
        let node = NODE_SETTINGS_ROW_BASE + index as u64;
        let row_rect = Rect::from_min_size(
            card.left_top()
                + Vec2::new(
                    metrics.settings_row_inset,
                    6.0 + slot as f32 * metrics.settings_row_spacing,
                ),
            Vec2::new(
                card.width() - metrics.settings_row_inset * 2.0,
                metrics.settings_row_height,
            ),
        );
        if slot > 0 {
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
        if setting.key == "accentColorArgb" {
            let current = settings
                .value(setting.key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or(setting.options[0])
                .to_ascii_uppercase();
            let size = 28.0;
            let gap = 12.0;
            let count = setting.options.len() as f32;
            let strip = Rect::from_min_max(
                Pos2::new(
                    row_rect.right() - 6.0 - count * size - (count - 1.0) * gap,
                    row_rect.center().y - size * 0.5,
                ),
                Pos2::new(row_rect.right() - 6.0, row_rect.center().y + size * 0.5),
            );
            layout.focusable.push((node, strip.expand(6.0)));
            egui::Area::new(Id::new(("fluxa-settings-swatches", node)))
                .fixed_pos(strip.min)
                .order(egui::Order::Foreground)
                .show(context, |ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    ui.horizontal(|ui| {
                        for option in setting.options {
                            let (rect, response) =
                                ui.allocate_exact_size(Vec2::splat(size), Sense::click());
                            let color = Color32::from_hex(option).unwrap_or(Color32::WHITE);
                            let active = option.eq_ignore_ascii_case(&current);
                            let grow = ui.ctx().animate_bool_with_time(
                                Id::new(("fluxa-swatch", *option)),
                                active || response.hovered(),
                                0.12,
                            );
                            let center = rect.center();
                            ui.painter().circle_filled(center, size * 0.5 - 4.0 + 2.0 * grow, color);
                            if active {
                                ui.painter().circle_stroke(
                                    center,
                                    size * 0.5,
                                    egui::Stroke::new(2.0, Color32::WHITE),
                                );
                            } else if response.hovered() {
                                ui.painter().circle_stroke(
                                    center,
                                    size * 0.5,
                                    egui::Stroke::new(1.0, Color32::from_white_alpha(90)),
                                );
                            }
                            let name = match *option {
                                "#FFFFFF" => "auto.white",
                                "#E50914" => "auto.red",
                                "#3F7CFF" => "auto.blue",
                                "#54D17A" => "auto.green",
                                "#FF8A3D" => "auto.orange",
                                "#C084FC" => "auto.purple",
                                _ => "",
                            };
                            let response = if name.is_empty() {
                                response
                            } else {
                                response.on_hover_text(localized(name, language))
                            };
                            if response.clicked() && !active {
                                layout.setting_change = Some((
                                    setting.key.to_owned(),
                                    serde_json::Value::String((*option).to_owned()),
                                ));
                            }
                        }
                    });
                });
        } else if !is_toggle {
            let raw_value = settings
                .value(setting.key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or(setting.options[0]);
            let value = option_label(setting.key, raw_value, language);
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
            let selected_text = value;
            let mut selected = raw_value.to_owned();
            egui::Area::new(Id::new(("fluxa-settings-dropdown", node)))
                .fixed_pos(value_rect.min)
                .order(egui::Order::Foreground)
                .show(context, |ui| {
                    ui.set_min_size(value_rect.size());
                    ui.set_max_size(value_rect.size());
                    let choices = setting
                        .options
                        .iter()
                        .map(|option| {
                            let label = option_label(setting.key, option, language);
                            ((*option).to_owned(), label)
                        })
                        .collect::<Vec<_>>();
                    let response = components::choice_field(
                        ui,
                        Id::new(("settings-choice", node)),
                        value_rect.size(),
                        value_size,
                        &selected_text,
                        &choices,
                        &mut selected,
                        value_rect.width(),
                        true,
                    );
                    layout.focusable.push((node, response.rect));
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

const ACCOUNT_PROFILE_CARD_HEIGHT: f32 = 92.0;
const ACCOUNT_SERVICES: [(&str, &str); 5] = [
    ("Stremio", "stremioAuthKey"),
    ("Nuvio", "nuvioAccessToken"),
    ("Trakt", "traktAccessToken"),
    ("Simkl", "simklAccessToken"),
    ("AniList", "anilistAccessToken"),
];
const ACCOUNT_SOURCES: [(&str, &str); 2] = [
    ("settings.integration_library_source", "integrationLibrarySource"),
    ("settings.continue_watching_source", "continueWatchingSource"),
];

fn account_height(metrics: UiMetrics) -> f32 {
    APPEARANCE_PAGE_HEADER_HEIGHT
        + (APPEARANCE_GROUP_HEADING_HEIGHT + APPEARANCE_GROUP_GAP) * 4.0
        + ACCOUNT_PROFILE_CARD_HEIGHT
        + settings_group_card_height(ACCOUNT_SERVICES.len(), metrics)
        + settings_group_card_height(ACCOUNT_SOURCES.len(), metrics)
        + FIELD_HEIGHT * 2.0
}

fn account_group(painter: &egui::Painter, rect: Rect, top: f32, height: f32, title: &str) -> Rect {
    painter.text(
        Pos2::new(rect.left() + 8.0, top),
        Align2::LEFT_TOP,
        title.to_uppercase(),
        FontId::proportional(13.0),
        Color32::from_white_alpha(145),
    );
    let card = Rect::from_min_size(
        Pos2::new(rect.left(), top + APPEARANCE_GROUP_HEADING_HEIGHT),
        Vec2::new(rect.width(), height),
    );
    painter.rect_filled(card, 12.0, Color32::from_rgb(19, 19, 19));
    painter.rect_stroke(
        card,
        12.0,
        egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
        egui::StrokeKind::Inside,
    );
    card
}

fn account_row(card: Rect, index: usize, metrics: UiMetrics) -> Rect {
    Rect::from_min_size(
        card.left_top()
            + Vec2::new(
                metrics.settings_row_inset,
                6.0 + index as f32 * metrics.settings_row_spacing,
            ),
        Vec2::new(
            card.width() - metrics.settings_row_inset * 2.0,
            metrics.settings_row_height,
        ),
    )
}

fn account_divider(painter: &egui::Painter, row: Rect, metrics: UiMetrics) {
    let y = row.top() - (metrics.settings_row_spacing - metrics.settings_row_height) * 0.5;
    painter.line_segment(
        [Pos2::new(row.left(), y), Pos2::new(row.right(), y)],
        egui::Stroke::new(1.0, Color32::from_white_alpha(12)),
    );
}

fn draw_account(
    context: &egui::Context,
    settings: &SettingsModel,
    assets: &impl HomeAssets,
    language: &str,
    rect: Rect,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    let label_size = metrics.settings_row_label_size_desktop;
    let connected = |key: &str| {
        settings
            .profile
            .get(key)
            .and_then(serde_json::Value::as_str)
            .is_some_and(|token| !token.is_empty())
    };

    let mut top = rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT;
    let card = account_group(
        &painter,
        rect,
        top,
        ACCOUNT_PROFILE_CARD_HEIGHT,
        &localized("settings.group.account_profile", language),
    );
    let name = settings
        .profile
        .get("name")
        .or_else(|| settings.profile.get("displayName"))
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| localized("settings.account_empty", language));
    let avatar = Rect::from_center_size(
        Pos2::new(card.left() + 22.0 + 26.0, card.center().y),
        Vec2::splat(52.0),
    );
    let avatar_url = settings
        .profile
        .get("avatarUrl")
        .and_then(serde_json::Value::as_str)
        .filter(|url| !url.starts_with("data:"));
    match assets.cached_texture(avatar_url) {
        Some(texture) => {
            painter.add(
                egui::epaint::RectShape::filled(avatar, 26.0, Color32::WHITE)
                    .with_texture(texture, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0))),
            );
        }
        None => {
            painter.circle_filled(avatar.center(), 26.0, Color32::from_white_alpha(22));
            let initials = name
                .split_whitespace()
                .take(2)
                .filter_map(|word| word.chars().next())
                .flat_map(char::to_uppercase)
                .collect::<String>();
            painter.text(
                avatar.center(),
                Align2::CENTER_CENTER,
                initials,
                FontId::proportional(20.0),
                Color32::WHITE,
            );
        }
    }
    painter.text(
        Pos2::new(avatar.right() + 16.0, card.center().y),
        Align2::LEFT_CENTER,
        &name,
        FontId::proportional(label_size + 4.0),
        Color32::WHITE,
    );
    let switch_label = localized("settings.switch_profiles", language);
    let switch_width = painter
        .layout_no_wrap(switch_label.clone(), FontId::proportional(label_size), Color32::WHITE)
        .size()
        .x
        + 32.0;
    let switch_rect = Rect::from_center_size(
        Pos2::new(card.right() - 20.0 - switch_width * 0.5, card.center().y),
        Vec2::new(switch_width, 36.0),
    );
    pill_button(
        context,
        layout,
        NODE_SETTINGS_SWITCH_PROFILE,
        switch_rect,
        &switch_label,
        label_size,
    );

    top = card.bottom() + APPEARANCE_GROUP_GAP;
    let card = account_group(
        &painter,
        rect,
        top,
        settings_group_card_height(ACCOUNT_SERVICES.len(), metrics),
        &localized("settings.group.account_services", language),
    );
    for (index, (label, key)) in ACCOUNT_SERVICES.iter().enumerate() {
        let row = account_row(card, index, metrics);
        if index > 0 {
            account_divider(&painter, row, metrics);
        }
        painter.text(
            row.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            *label,
            FontId::proportional(label_size),
            Color32::from_white_alpha(210),
        );
        let on = connected(key);
        let status = localized(
            if on {
                "settings.connected"
            } else {
                "settings.not_connected"
            },
            language,
        );
        let font = FontId::proportional(metrics.screen_card_subtitle_size);
        let width = painter
            .layout_no_wrap(status.clone(), font.clone(), Color32::WHITE)
            .size()
            .x;
        let pill = Rect::from_center_size(
            row.right_center() - Vec2::new(width * 0.5 + 24.0, 0.0),
            Vec2::new(width + 40.0, 28.0),
        );
        painter.rect_stroke(
            pill,
            14.0,
            egui::Stroke::new(1.0, Color32::from_white_alpha(if on { 40 } else { 16 })),
            egui::StrokeKind::Inside,
        );
        painter.circle_filled(
            Pos2::new(pill.left() + 14.0, pill.center().y),
            3.5,
            if on {
                Color32::WHITE
            } else {
                Color32::from_white_alpha(60)
            },
        );
        painter.text(
            Pos2::new(pill.left() + 24.0, pill.center().y),
            Align2::LEFT_CENTER,
            status,
            font,
            Color32::from_white_alpha(if on { 230 } else { 130 }),
        );
    }

    top = card.bottom() + APPEARANCE_GROUP_GAP;
    let card = account_group(
        &painter,
        rect,
        top,
        settings_group_card_height(ACCOUNT_SOURCES.len(), metrics),
        &localized("settings.group.account_sync", language),
    );
    for (index, (label_key, key)) in ACCOUNT_SOURCES.iter().enumerate() {
        let row = account_row(card, index, metrics);
        if index > 0 {
            account_divider(&painter, row, metrics);
        }
        painter.text(
            row.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            localized(label_key, language),
            FontId::proportional(label_size),
            Color32::from_white_alpha(210),
        );
        let mut choices = Vec::new();
        for (source, token_key) in [
            ("nuvio", "nuvioAccessToken"),
            ("trakt", "traktAccessToken"),
            ("simkl", "simklAccessToken"),
            ("anilist", "anilistAccessToken"),
            ("stremio", "stremioAuthKey"),
        ] {
            if connected(token_key) {
                choices.push(source);
            }
        }
        let current = settings
            .value(key)
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let mut selected = current.clone();
        let current_label = if choices.contains(&current.as_str()) {
            localized_or(&format!("settings.option.{current}"), &current, language)
        } else if choices.is_empty() {
            localized("settings.not_connected", language)
        } else {
            localized("settings.option.none", language)
        };
        let value_rect = Rect::from_center_size(
            row.right_center() - Vec2::new(90.0, 0.0),
            Vec2::new(176.0, metrics.screen_control_height.min(row.height() - 4.0)),
        );
        egui::Area::new(Id::new(("fluxa-settings-account-choice", *key)))
            .fixed_pos(value_rect.min)
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                ui.set_min_size(value_rect.size());
                ui.set_max_size(value_rect.size());
                let options = choices
                    .iter()
                    .map(|choice| {
                        (
                            (*choice).to_owned(),
                            localized_or(&format!("settings.option.{choice}"), choice, language),
                        )
                    })
                    .collect::<Vec<_>>();
                let response = components::choice_field(
                    ui,
                    Id::new(("settings-account", *key)),
                    value_rect.size(),
                    metrics.settings_row_value_size_desktop,
                    &current_label,
                    &options,
                    &mut selected,
                    value_rect.width(),
                    !options.is_empty(),
                );
                layout.focusable.push((
                    NODE_SETTINGS_ROW_BASE + 10_000 + index as u64,
                    response.rect,
                ));
            });
        if selected != current {
            layout.setting_change = Some(((*key).to_owned(), serde_json::Value::String(selected)));
        }
    }

    draw_field_group(
        context,
        settings,
        rect,
        card.bottom() + APPEARANCE_GROUP_GAP,
        &localized("settings.group.api_keys", language),
        &[1, 2],
        metrics,
        layout,
    );
}

