use super::*;

mod addons;
pub use addons::{addon_action, addon_transport_url};

#[derive(Clone, Debug)]
pub struct SettingsRow {
    pub label: &'static str,
    pub key: &'static str,
    pub options: &'static [&'static str],
}

#[derive(Clone, Debug)]
pub struct SettingsSection {
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub rows: &'static [SettingsRow],
    pub groups: &'static [(usize, &'static str)],
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
const GENERAL_GROUPS: [(usize, &str); 3] = [
    (2, "settings.group.general_startup"),
    (6, "settings.group.general_behavior"),
    (2, "settings.group.general_calendar"),
];
const APPEARANCE_SETTINGS: [SettingsRow; 15] = [
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
        label: "Liquid glass",
        key: "liquidGlass",
        options: &[],
    },
    SettingsRow {
        label: "GIF autoplay",
        key: "gifAutoplayEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Show catalog type",
        key: "showCatalogType",
        options: &[],
    },
    SettingsRow {
        label: "App icon",
        key: "appIcon",
        options: &[
            "ember", "sunset", "ocean", "dusk", "lagoon", "ruby", "citrus", "peach", "lime",
            "aurora", "neon", "flare", "ice", "rose", "silver", "gold", "jade", "blaze", "orange",
        ],
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
        label: "Floating navigation bar",
        key: "navFloating",
        options: &[],
    },
    SettingsRow {
        label: "Navigation labels",
        key: "navLabels",
        options: &[],
    },
];
const APPEARANCE_GROUPS: [(usize, &str); 2] = [
    (8, "settings.group.color_and_motion"),
    (6, "settings.group.navigation"),
];
const HOME_SETTINGS: [SettingsRow; 13] = [
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
];
const HOME_GROUPS: [(usize, &str); 2] = [
    (5, "settings.group.home"),
    (8, "settings.group.continue_watching"),
];
const DETAILS_SETTINGS: [SettingsRow; 13] = [
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
const DETAILS_GROUPS: [(usize, &str); 2] = [
    (9, "settings.group.detail_page"),
    (4, "settings.group.episodes"),
];
const POSTERS_SETTINGS: [SettingsRow; 22] = [
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
        options: &[
            "sash",
            "banner",
            "top_left",
            "top_right",
            "bottom_left",
            "bottom_right",
        ],
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
const POSTERS_GROUPS: [(usize, &str); 4] = [
    (6, "settings.group.cards"),
    (4, "settings.group.poster_overlays"),
    (3, "settings.group.poster_rating"),
    (9, "settings.group.poster_badges"),
];
const PLAYBACK_SETTINGS: [SettingsRow; 26] = [
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
        label: "Auto skip intro",
        key: "autoSkipIntro",
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
        label: "Hold speed",
        key: "holdSpeed",
        options: &["1.25", "1.5", "1.75", "2.0", "2.5", "3.0"],
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
        label: "Audio processing",
        key: "audioProcessingMode",
        options: &["reference", "balanced", "night"],
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
];
const PLAYBACK_GROUPS: [(usize, &str); 5] = [
    (7, "settings.group.playback"),
    (2, "settings.group.sources_video"),
    (9, "settings.group.skip_controls"),
    (4, "settings.group.upscaling_motion"),
    (4, "settings.group.player_advanced"),
];
const SUBTITLES_SETTINGS: [SettingsRow; 18] = [
    SettingsRow {
        label: "Preferred audio",
        key: "preferredAudioLanguage",
        options: &["none", "en", "tr"],
    },
    SettingsRow {
        label: "Secondary audio",
        key: "secondaryAudioLanguage",
        options: &["none", "en", "tr"],
    },
    SettingsRow {
        label: "Anime Japanese audio",
        key: "animePreferJapaneseAudio",
        options: &[],
    },
    SettingsRow {
        label: "Preferred subtitle language",
        key: "preferredSubtitleLanguage",
        options: &["none", "en", "tr"],
    },
    SettingsRow {
        label: "Secondary subtitle",
        key: "secondarySubtitleLanguage",
        options: &["none", "en", "tr"],
    },
    SettingsRow {
        label: "Auto-enable subtitles",
        key: "autoEnableSubtitles",
        options: &[],
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
        options: &[
            "#FFFFFF", "#000000", "#FFE45C", "#FF5D5D", "#3F7CFF", "#54D17A", "#FF8A3D", "#C084FC",
        ],
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
const SUBTITLES_GROUPS: [(usize, &str); 2] = [
    (6, "settings.group.audio_subtitles"),
    (12, "settings.group.subtitle_style"),
];
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
const CONTENT_GROUPS: [(usize, &str); 2] = [
    (14, "settings.group.metadata"),
    (4, "settings.group.discovery"),
];
const DOWNLOADS_SETTINGS: [SettingsRow; 5] = [
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
const DOWNLOADS_GROUPS: [(usize, &str); 1] = [(5, "settings.group.downloads")];
const EMPTY_SETTINGS: [SettingsRow; 0] = [];
pub const SETTINGS_SECTIONS: [SettingsSection; 13] = [
    SettingsSection {
        title: "Account",
        category: "Account",
        description: "Profile, connected services and API keys",
        rows: &EMPTY_SETTINGS,
        groups: &[],
    },
    SettingsSection {
        title: "General",
        category: "General",
        description: "How Fluxa starts and behaves",
        rows: &GENERAL_SETTINGS,
        groups: &GENERAL_GROUPS,
    },
    SettingsSection {
        title: "Theme",
        category: "Appearance",
        description: "Visual preferences",
        rows: &APPEARANCE_SETTINGS,
        groups: &APPEARANCE_GROUPS,
    },
    SettingsSection {
        title: "Home",
        category: "Appearance",
        description: "Hero and rows on the home screen",
        rows: &HOME_SETTINGS,
        groups: &HOME_GROUPS,
    },
    SettingsSection {
        title: "Details",
        category: "Appearance",
        description: "Layout of the detail page",
        rows: &DETAILS_SETTINGS,
        groups: &DETAILS_GROUPS,
    },
    SettingsSection {
        title: "Posters",
        category: "Appearance",
        description: "Badges drawn on top of poster artwork",
        rows: &POSTERS_SETTINGS,
        groups: &POSTERS_GROUPS,
    },
    SettingsSection {
        title: "Playback",
        category: "Playback",
        description: "Defaults for watching content",
        rows: &PLAYBACK_SETTINGS,
        groups: &PLAYBACK_GROUPS,
    },
    SettingsSection {
        title: "Subtitles",
        category: "Playback",
        description: "Audio and subtitle languages and style",
        rows: &SUBTITLES_SETTINGS,
        groups: &SUBTITLES_GROUPS,
    },
    SettingsSection {
        title: "Content",
        category: "Content",
        description: "Catalog and metadata preferences",
        rows: &CONTENT_SETTINGS,
        groups: &CONTENT_GROUPS,
    },
    SettingsSection {
        title: "Add-ons",
        category: "Content",
        description: "Installed metadata and stream add-ons",
        rows: &EMPTY_SETTINGS,
        groups: &[],
    },
    SettingsSection {
        title: "Plugins",
        category: "Content",
        description: "Scraper repositories and plugins",
        rows: &EMPTY_SETTINGS,
        groups: &[],
    },
    SettingsSection {
        title: "Downloads",
        category: "Downloads",
        description: "Torrent and download defaults",
        rows: &DOWNLOADS_SETTINGS,
        groups: &DOWNLOADS_GROUPS,
    },
    SettingsSection {
        title: "Shortcuts",
        category: "Shortcuts",
        description: "Keyboard and controller bindings",
        rows: &EMPTY_SETTINGS,
        groups: &[],
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

const APPEARANCE_PAGE_HEADER_HEIGHT: f32 = 66.0;
const APPEARANCE_GROUP_HEADING_HEIGHT: f32 = 22.0;
const APPEARANCE_GROUP_GAP: f32 = 22.0;
const FIELD_HEIGHT: f32 = 118.0;

fn settings_group_card_height(row_count: usize, metrics: UiMetrics) -> f32 {
    metrics.settings_row_height
        + row_count.saturating_sub(1) as f32 * metrics.settings_row_spacing
        + metrics.settings_card_padding
}

const ROW_SUBTITLE_EXTRA: f32 = 18.0;

fn described(mut metrics: UiMetrics) -> UiMetrics {
    metrics.settings_row_height += ROW_SUBTITLE_EXTRA;
    metrics.settings_row_spacing += ROW_SUBTITLE_EXTRA;
    metrics
}

pub(super) fn row_applies(key: &str, viewport: Viewport) -> bool {
    let compact = viewport.is_compact();
    let desktop = !compact && !viewport.is_tv();
    match key {
        "navLayout" | "navMode" | "navSidebarMode" | "interfaceDensity" | "navFloating"
        | "navLabels" | "posterHoverPreview" => !compact,
        "discordRichPresenceEnabled" | "automaticUpdates" => desktop,
        "backgroundPlayback" | "pictureInPicture" | "torrentWifiOnly" => !desktop,
        _ => true,
    }
}

pub fn section_applies(title: &str, viewport: Viewport) -> bool {
    title != "Shortcuts" || !(viewport.is_compact() || viewport.is_tv())
}

pub fn settings_page_for_node(node: u64) -> Option<usize> {
    let count = SETTINGS_SECTIONS.len() as u64;
    [NODE_SETTINGS_SECTION_BASE, NODE_SETTINGS_PAGE_BASE]
        .into_iter()
        .find(|base| (*base..base + count).contains(&node))
        .map(|base| (node - base) as usize)
}

pub fn category_pages(index: usize) -> impl Iterator<Item = usize> {
    let category = SETTINGS_SECTIONS[index.min(SETTINGS_SECTIONS.len() - 1)].category;
    (0..SETTINGS_SECTIONS.len()).filter(move |&page| SETTINGS_SECTIONS[page].category == category)
}

pub fn categories(viewport: Viewport) -> impl Iterator<Item = usize> {
    (0..SETTINGS_SECTIONS.len()).filter(move |&index| {
        let section = &SETTINGS_SECTIONS[index];
        section_applies(section.title, viewport)
            && (index == 0 || SETTINGS_SECTIONS[index - 1].category != section.category)
    })
}

fn shows_hub(settings: &SettingsModel) -> bool {
    settings.search.trim().is_empty()
        && !settings.page_open
        && category_pages(settings.active_section).count() > 1
}

fn page_card_height(viewport: Viewport) -> f32 {
    if viewport.is_tv() {
        88.0
    } else if viewport.is_compact() {
        68.0
    } else {
        72.0
    }
}

fn subpages(active: usize) -> impl Iterator<Item = usize> {
    category_pages(active).filter(move |&page| page != active)
}

fn links_height(viewport: Viewport, settings: &SettingsModel) -> f32 {
    if shows_hub(settings) {
        subpages(settings.active_section).count() as f32 * page_card_height(viewport)
            + APPEARANCE_GROUP_GAP
    } else {
        0.0
    }
}

fn draw_page_cards(
    context: &egui::Context,
    viewport: Viewport,
    painter: &egui::Painter,
    active: usize,
    language: &str,
    metrics: UiMetrics,
    top_left: Pos2,
    width: f32,
    layout: &mut HomeLayout,
) {
    let height = page_card_height(viewport);
    let pages: Vec<usize> = subpages(active).collect();
    let card = Rect::from_min_size(top_left, Vec2::new(width, pages.len() as f32 * height));
    painter.rect_filled(card, metrics.card_radius, Color32::from_rgb(19, 19, 19));
    painter.rect_stroke(
        card,
        metrics.card_radius,
        egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
        egui::StrokeKind::Inside,
    );
    let title_size = if viewport.is_tv() {
        metrics.nav_label_size + 4.0
    } else {
        metrics.nav_label_size + 3.0
    };
    let subtitle_size = metrics.screen_card_subtitle_size + 2.0;
    for (slot, page) in pages.into_iter().enumerate() {
        let node = NODE_SETTINGS_PAGE_BASE + page as u64;
        let row = Rect::from_min_size(
            top_left + Vec2::new(0.0, slot as f32 * height),
            Vec2::new(width, height),
        );
        if slot > 0 {
            painter.line_segment(
                [
                    Pos2::new(row.left() + 20.0, row.top()),
                    Pos2::new(row.right(), row.top()),
                ],
                egui::Stroke::new(1.0, Color32::from_white_alpha(12)),
            );
        }
        let title = section_label(SETTINGS_SECTIONS[page].title, language);
        let description = localized(
            &format!(
                "settings.section.{}.description",
                SETTINGS_SECTIONS[page].title.to_lowercase()
            ),
            language,
        );
        let mut clicked = false;
        egui::Area::new(Id::new(("fluxa-settings-page-card", node)))
            .constrain(false)
            .fixed_pos(row.min)
            .show(context, |ui| {
                let (rect, response) = ui.allocate_exact_size(row.size(), Sense::click());
                if response.hovered() || response.is_pointer_button_down_on() {
                    ui.painter().rect_filled(
                        rect,
                        metrics.card_radius,
                        Color32::from_white_alpha(8),
                    );
                }
                let text_width = (rect.width() - 72.0).max(1.0);
                let title_font = crate::fonts::regular(title_size);
                let subtitle_font = crate::fonts::regular(subtitle_size);
                let text_top = rect.center().y - (title_size + 4.0 + subtitle_size) * 0.5;
                ui.painter().text(
                    Pos2::new(rect.left() + 20.0, text_top),
                    Align2::LEFT_TOP,
                    truncate_to_width(ui.painter(), &title, &title_font, text_width),
                    title_font,
                    Color32::WHITE,
                );
                ui.painter().text(
                    Pos2::new(rect.left() + 20.0, text_top + title_size + 4.0),
                    Align2::LEFT_TOP,
                    truncate_to_width(ui.painter(), &description, &subtitle_font, text_width),
                    subtitle_font,
                    Color32::from_white_alpha(120),
                );
                let tip = rect.right_center() - Vec2::new(22.0, 0.0);
                let stroke = egui::Stroke::new(1.6, Color32::from_white_alpha(120));
                ui.painter()
                    .line_segment([tip + Vec2::new(-5.0, -6.0), tip], stroke);
                ui.painter()
                    .line_segment([tip + Vec2::new(-5.0, 6.0), tip], stroke);
                clicked = response.clicked();
            });
        layout.focusable.push((node, row));
        if clicked {
            layout.activated = Some(node);
        }
    }
}

fn section_label(title: &str, language: &str) -> String {
    localized(
        &format!("settings.section.{}", title.to_lowercase()),
        language,
    )
}

pub(super) fn visible_groups(
    settings: &SettingsModel,
    viewport: Viewport,
) -> Vec<(String, Vec<usize>)> {
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
                .filter(|(_, row)| row_applies(row.key, viewport))
                .filter(|(_, row)| {
                    settings_row_label(row, language)
                        .to_lowercase()
                        .contains(&query)
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
    let rows = SETTINGS_SECTIONS[active].rows;
    let mut start = 0;
    for &(count, key) in SETTINGS_SECTIONS[active].groups {
        let end = start + count;
        let visible: Vec<usize> = (start..end)
            .filter(|&index| row_applies(rows[index].key, viewport))
            .map(|index| offset + index)
            .collect();
        if !visible.is_empty() {
            groups.push((localized(key, language), visible));
        }
        start = end;
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
                Vec2::new(
                    rect.width(),
                    settings_group_card_height(rows.len(), metrics),
                ),
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
    let groups = visible_groups(settings, viewport);
    let rows_metrics = described(metrics);
    if !settings.search.trim().is_empty() {
        return base.max(groups_height(&groups, rows_metrics));
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
            groups_height(&groups, rows_metrics)
                + APPEARANCE_GROUP_HEADING_HEIGHT
                + FIELD_HEIGHT
                + APPEARANCE_GROUP_GAP
        }
        "Add-ons" => {
            APPEARANCE_PAGE_HEADER_HEIGHT
                + addons::addons_height(settings, metrics, settings.language())
        }
        "Plugins" => {
            APPEARANCE_PAGE_HEADER_HEIGHT
                + addons::plugins_height(settings, metrics, settings.language())
        }
        _ => groups_height(&groups, rows_metrics),
    };
    base.max(content_height + links_height(viewport, settings))
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
    pub section_open: bool,
    pub page_open: bool,
    pub account_auth: Option<AccountPrompt>,
}

#[derive(Clone, Debug, Default)]
pub struct AccountPrompt {
    pub provider: String,
    pub code: String,
    pub url: String,
    pub failed: bool,
}

impl SettingsModel {
    pub fn language(&self) -> &str {
        self.values
            .get("language")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("en")
    }
    pub fn value(&self, key: &str) -> Option<&serde_json::Value> {
        self.values
            .get(key)
            .filter(|value| !value.is_null())
            .or_else(|| setting_default(key))
            .or_else(|| poster_overlay::setting_default(key))
    }

    pub fn app_icon(&self) -> Option<&str> {
        self.value("appIcon").and_then(|value| value.as_str())
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
        section_open: false,
        page_open: false,
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
        account_auth: None,
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
    let font = crate::fonts::regular(metrics.screen_body_size);
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
        let value_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
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

pub fn option_label(key: &str, value: &str, language: &str) -> String {
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
        (
            "accentColorArgb"
            | "subtitleColor"
            | "subtitleBackgroundColor"
            | "subtitleOutlineColor",
            _,
        ) => color_name(value),
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

pub fn settings_row_label(setting: &SettingsRow, language: &str) -> String {
    let label = raw_row_label(setting, language);
    if !language.starts_with("en") {
        return label;
    }
    label
        .split(' ')
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(chars).collect()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn raw_row_label(setting: &SettingsRow, language: &str) -> String {
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
        .constrain(false)
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
        .constrain(false)
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
        .constrain(false)
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
}

pub const POSTER_FIELDS: [PosterField; 3] = [
    PosterField {
        key: "posterUrlTemplate",
        label: "settings.poster_url_template",
        help: "settings.poster_url_template_help",
        hint: "settings.poster_url_template_hint",
        input: NODE_SETTINGS_POSTER_URL,
    },
    PosterField {
        key: "tmdbApiKey",
        label: "settings.tmdb_api_key",
        help: "settings.poster_tmdb_help",
        hint: "settings.tmdb_api_key_placeholder",
        input: NODE_SETTINGS_POSTER_TMDB_KEY,
    },
    PosterField {
        key: "mdblistApiKey",
        label: "settings.mdblist_api_key",
        help: "settings.poster_mdblist_help",
        hint: "settings.mdblist_api_key_placeholder",
        input: NODE_SETTINGS_POSTER_MDBLIST_KEY,
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
    compact: bool,
    layout: &mut HomeLayout,
) {
    let language = settings.language();
    let field = &POSTER_FIELDS[index];
    let painter = context.layer_painter(egui::LayerId::background());
    let label_size = if compact {
        metrics.nav_label_size + 2.0
    } else {
        metrics.settings_row_label_size_desktop
    };
    let inner = area.shrink2(Vec2::new(metrics.settings_row_inset + 4.0, 16.0));
    painter.text(
        inner.left_top(),
        Align2::LEFT_TOP,
        localized(field.label, language),
        crate::fonts::regular(label_size),
        Color32::from_white_alpha(230),
    );
    let help_font = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
    painter.text(
        inner.left_top() + Vec2::new(0.0, label_size + 6.0),
        Align2::LEFT_TOP,
        truncate_to_width(
            &painter,
            &localized(field.help, language),
            &help_font,
            inner.width(),
        ),
        help_font,
        Color32::from_white_alpha(130),
    );
    let height = 36.0;
    let input = Rect::from_min_max(
        Pos2::new(inner.left(), inner.bottom() - height),
        inner.right_bottom(),
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
}

fn draw_field_group(
    context: &egui::Context,
    settings: &SettingsModel,
    rect: Rect,
    top: f32,
    title: &str,
    fields: &[usize],
    metrics: UiMetrics,
    compact: bool,
    layout: &mut HomeLayout,
) -> Rect {
    let painter = context.layer_painter(egui::LayerId::background());
    let card = account_group(
        &painter,
        rect,
        top,
        fields.len() as f32 * FIELD_HEIGHT,
        title,
        compact,
    );
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
        draw_field(context, settings, index, area, metrics, compact, layout);
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
        false,
    );
    let label_font = FontId::proportional(metrics.settings_row_label_size_desktop);
    let value_font = crate::fonts::regular(metrics.settings_row_value_size_desktop);
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
    assets: &mut impl HomeAssets,
    section: &str,
    language: &str,
    rect: Rect,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    match section {
        "Account" => draw_account(
            context, viewport, settings, assets, language, rect, metrics, layout,
        ),
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
            let top = group_cards(
                rect,
                &visible_groups(settings, viewport),
                described(metrics),
            )
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
                viewport.is_compact(),
                layout,
            );
        }
        "Add-ons" => addons::draw_addons(
            context,
            settings,
            assets,
            language,
            rect,
            rect.top()
                + if viewport.is_compact() {
                    8.0
                } else {
                    APPEARANCE_PAGE_HEADER_HEIGHT
                },
            metrics,
            layout,
        ),
        "Plugins" => addons::draw_plugins(
            context,
            settings,
            assets,
            language,
            rect,
            rect.top()
                + if viewport.is_compact() {
                    8.0
                } else {
                    APPEARANCE_PAGE_HEADER_HEIGHT
                },
            metrics,
            layout,
        ),
        _ => {}
    }
}

pub const COMPACT_SETTINGS_ROW: f32 = 56.0;

pub fn compact_settings_content_top(metrics: UiMetrics, settings: &SettingsModel) -> f32 {
    let top = metrics.detail_header_top_mobile;
    if settings.section_open && settings.search.trim().is_empty() {
        top + 40.0 + metrics.section_gap
    } else {
        top + metrics.screen_title_size_mobile
            + metrics.section_gap
            + metrics.screen_control_height
            + metrics.section_gap
    }
}

pub fn compact_settings_list_height(viewport: Viewport, metrics: UiMetrics) -> f32 {
    (categories(viewport).count() + 1) as f32 * COMPACT_SETTINGS_ROW + metrics.section_gap
}

fn draw_compact_settings_list(
    context: &egui::Context,
    viewport: Viewport,
    painter: &egui::Painter,
    assets: &impl HomeAssets,
    language: &str,
    metrics: UiMetrics,
    anchor: Rect,
    layout: &mut HomeLayout,
) {
    let entries: Vec<(u64, &str, String)> = categories(viewport)
        .map(|index| {
            let section = &SETTINGS_SECTIONS[index];
            (
                NODE_SETTINGS_SECTION_BASE + index as u64,
                section.category,
                section_label(section.category, language),
            )
        })
        .chain(std::iter::once((
            NODE_SETTINGS_SWITCH_PROFILE,
            "Profile",
            localized("settings.switch_profiles", language),
        )))
        .collect();
    let sections = categories(viewport).count();
    let group = |top: f32, count: usize| {
        let card = Rect::from_min_size(
            Pos2::new(anchor.left(), top),
            Vec2::new(anchor.width(), count as f32 * COMPACT_SETTINGS_ROW),
        );
        painter.rect_filled(card, metrics.card_radius, Color32::from_rgb(19, 19, 19));
        painter.rect_stroke(
            card,
            metrics.card_radius,
            egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
            egui::StrokeKind::Inside,
        );
    };
    group(anchor.top(), sections);
    let profile_top = anchor.top() + sections as f32 * COMPACT_SETTINGS_ROW + metrics.section_gap;
    group(profile_top, 1);
    for (index, (node, icon, label)) in entries.into_iter().enumerate() {
        let top = if index < sections {
            anchor.top() + index as f32 * COMPACT_SETTINGS_ROW
        } else {
            profile_top
        };
        let rect = Rect::from_min_size(
            Pos2::new(anchor.left(), top),
            Vec2::new(anchor.width(), COMPACT_SETTINGS_ROW),
        );
        if index > 0 && index < sections {
            painter.line_segment(
                [
                    Pos2::new(rect.left() + 52.0, rect.top()),
                    Pos2::new(rect.right(), rect.top()),
                ],
                egui::Stroke::new(1.0, Color32::from_white_alpha(12)),
            );
        }
        let mut clicked = false;
        egui::Area::new(Id::new(("fluxa-settings-compact-row", node)))
            .constrain(false)
            .fixed_pos(rect.min)
            .show(context, |ui| {
                let (row, response) = ui.allocate_exact_size(rect.size(), Sense::click());
                if response.is_pointer_button_down_on() {
                    ui.painter().rect_filled(
                        row,
                        metrics.card_radius,
                        Color32::from_white_alpha(10),
                    );
                }
                settings_category_icon(
                    assets,
                    ui.painter(),
                    row.left_center() + Vec2::new(26.0, 0.0),
                    icon,
                    Color32::WHITE,
                );
                ui.painter().text(
                    row.left_center() + Vec2::new(52.0, 0.0),
                    Align2::LEFT_CENTER,
                    label,
                    crate::fonts::regular(metrics.nav_label_size + 2.0),
                    Color32::WHITE,
                );
                let tip = row.right_center() - Vec2::new(20.0, 0.0);
                let stroke = egui::Stroke::new(1.6, Color32::from_white_alpha(120));
                ui.painter()
                    .line_segment([tip + Vec2::new(-5.0, -6.0), tip], stroke);
                ui.painter()
                    .line_segment([tip + Vec2::new(-5.0, 6.0), tip], stroke);
                clicked = response.clicked();
            });
        layout.focusable.push((node, rect));
        if clicked {
            layout.activated = Some(node);
        }
    }
}

pub fn draw_settings(
    context: &egui::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
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
    layout.activated = draw_navigation_bar(context, viewport, 3, assets);
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
    if tv {
        egui::Area::new(Id::new("fluxa-shared-settings-header"))
            .constrain(false)
            .fixed_pos(Pos2::new(margin, top - scroll_y))
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
                    layout
                        .focusable
                        .push((NODE_SETTINGS_SWITCH_PROFILE, switch.rect));
                    if switch.clicked() {
                        layout.activated = Some(NODE_SETTINGS_SWITCH_PROFILE);
                    }
                });
                header_bottom = ui.min_rect().bottom() + scroll_y;
            });
    }
    let active_section = settings.active_section.min(SETTINGS_SECTIONS.len() - 1);
    let section_top = if desktop {
        top
    } else if compact {
        compact_settings_content_top(metrics, settings)
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
            .constrain(false)
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
        let width = (viewport.width - margin * 2.0).max(1.0);
        let root = !settings.section_open || searching;
        if root {
            painter.text(
                Pos2::new(margin, top - scroll_y),
                Align2::LEFT_TOP,
                localized("nav.settings", language),
                FontId::proportional(metrics.screen_title_size_mobile),
                Color32::WHITE,
            );
            search_field(
                &mut layout,
                Rect::from_min_size(
                    Pos2::new(
                        margin,
                        top + metrics.screen_title_size_mobile + metrics.section_gap - scroll_y,
                    ),
                    Vec2::new(width, search_height),
                ),
            );
        } else {
            let back_rect =
                Rect::from_min_size(Pos2::new(margin - 8.0, top - scroll_y), Vec2::splat(40.0));
            egui::Area::new(Id::new("fluxa-settings-back"))
                .constrain(false)
                .fixed_pos(back_rect.min)
                .order(egui::Order::Foreground)
                .show(context, |ui| {
                    let response = components::icon_button(
                        ui,
                        assets.icon("ArrowLeft"),
                        40.0,
                        Color32::WHITE,
                        false,
                        false,
                        true,
                    );
                    if response.clicked() {
                        layout.activated = Some(NODE_SETTINGS_BACK);
                    }
                });
            layout.focusable.push((NODE_SETTINGS_BACK, back_rect));
            painter.text(
                back_rect.right_center() + Vec2::new(4.0, 0.0),
                Align2::LEFT_CENTER,
                if settings.page_open {
                    section_label(SETTINGS_SECTIONS[active_section].title, language)
                } else {
                    section_label(SETTINGS_SECTIONS[active_section].category, language)
                },
                FontId::proportional(metrics.text.title),
                Color32::WHITE,
            );
        }
        if !settings.section_open && !searching {
            draw_compact_settings_list(
                context,
                viewport,
                &painter,
                assets,
                language,
                metrics,
                Rect::from_min_size(
                    Pos2::new(margin, section_top - scroll_y),
                    Vec2::new(width, 0.0),
                ),
                &mut layout,
            );
            return layout;
        }
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
                Pos2::new(
                    margin + metrics.control_gap,
                    section_top + search_top - scroll_y,
                ),
                Vec2::new(nav_width - metrics.control_gap * 2.0, search_height),
            ),
        );
        let nav_top_inset = search_top + search_height + metrics.control_gap * 1.5;
        let active_category = SETTINGS_SECTIONS[active_section].category;
        for (slot, index) in categories(viewport).enumerate() {
            let section = &SETTINGS_SECTIONS[index];
            let node = NODE_SETTINGS_SECTION_BASE + index as u64;
            let rect = Rect::from_min_size(
                Pos2::new(
                    margin + metrics.control_gap,
                    section_top + nav_top_inset + slot as f32 * (nav_item_height + nav_item_gap)
                        - scroll_y,
                ),
                Vec2::new(nav_width - metrics.control_gap * 2.0, nav_item_height),
            );
            layout.focusable.push((node, rect));
            let mut clicked = false;
            egui::Area::new(Id::new(("fluxa-settings-category", node)))
                .constrain(false)
                .fixed_pos(rect.min)
                .show(context, |ui| {
                    let (item_rect, response) = ui.allocate_exact_size(rect.size(), Sense::click());
                    let active = section.category == active_category && !searching;
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
                        section.category,
                        if active {
                            Color32::WHITE
                        } else {
                            Color32::from_white_alpha(150)
                        },
                    );
                    ui.painter().text(
                        item_rect.left_center() + Vec2::new(42.0, 0.0),
                        Align2::LEFT_CENTER,
                        section_label(section.category, language),
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
    let nav_bottom = section_top;
    let card_top = nav_bottom - scroll_y;
    let card_width = content_width;
    let section = &SETTINGS_SECTIONS[active_section];
    let card_height = settings_card_height(viewport, metrics, section, settings);
    let rect = Rect::from_min_size(
        Pos2::new(content_x, card_top),
        Vec2::new(card_width, card_height),
    );
    let groups = visible_groups(settings, viewport);
    let results = searching && !groups.is_empty();
    let rect = if results {
        rect.translate(Vec2::new(0.0, -APPEARANCE_PAGE_HEADER_HEIGHT))
    } else if compact && !groups.is_empty() {
        let lead = if groups[0].0 == section_label(section.title, language) {
            APPEARANCE_GROUP_HEADING_HEIGHT
        } else {
            0.0
        };
        rect.translate(Vec2::new(0.0, 8.0 - APPEARANCE_PAGE_HEADER_HEIGHT - lead))
    } else {
        rect
    };
    let header = rect;
    let cards = group_cards(rect, &groups, described(metrics));
    if shows_hub(settings) {
        let top = cards
            .last()
            .map_or(rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT, |card| {
                card.bottom() + APPEARANCE_GROUP_GAP
            });
        draw_page_cards(
            context,
            viewport,
            &painter,
            active_section,
            language,
            metrics,
            Pos2::new(rect.left(), top),
            rect.width(),
            &mut layout,
        );
    }
    let page_title = section_label(section.title, language);
    for ((title, _), card) in groups.iter().zip(&cards) {
        if !(compact && !searching && *title == page_title) {
            painter.text(
                Pos2::new(
                    rect.left() + if compact { 16.0 } else { 8.0 },
                    card.top() - APPEARANCE_GROUP_HEADING_HEIGHT,
                ),
                Align2::LEFT_TOP,
                if compact {
                    title.clone()
                } else {
                    title.to_uppercase()
                },
                crate::fonts::regular(
                    metrics.screen_card_subtitle_size + if compact { 1.0 } else { 0.0 },
                ),
                Color32::from_white_alpha(if compact { 120 } else { 145 }),
            );
        }
        painter.rect_filled(*card, metrics.card_radius, Color32::from_rgb(19, 19, 19));
        if !compact {
            painter.rect_stroke(
                *card,
                metrics.card_radius,
                egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
                egui::StrokeKind::Inside,
            );
        }
    }
    let heading = if compact && !searching {
        String::new()
    } else if !searching && settings.page_open {
        format!(
            "{}  ›  {}",
            section_label(section.category, language),
            section_label(section.title, language)
        )
    } else if !searching {
        section_label(section.category, language)
    } else if groups.is_empty() {
        localized("settings.search_no_results", language)
    } else {
        String::new()
    };
    painter.text(
        header.left_top() + Vec2::new(metrics.settings_card_padding, metrics.settings_title_top),
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
    if tv && !searching {
        painter.text(
            header.left_top()
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
            crate::fonts::regular(metrics.screen_card_subtitle_size),
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
    let row_metrics = described(metrics);
    let rows = groups.iter().zip(&cards).flat_map(|((_, rows), card)| {
        rows.iter()
            .enumerate()
            .map(move |(slot, &index)| (index, slot, *card))
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
                    6.0 + slot as f32 * row_metrics.settings_row_spacing,
                ),
            Vec2::new(
                card.width() - metrics.settings_row_inset * 2.0,
                row_metrics.settings_row_height,
            ),
        );
        if slot > 0 {
            let y = row_rect.top()
                - (row_metrics.settings_row_spacing - row_metrics.settings_row_height) * 0.5;
            painter.line_segment(
                [
                    Pos2::new(row_rect.left(), y),
                    Pos2::new(row_rect.right(), y),
                ],
                egui::Stroke::new(1.0, Color32::from_white_alpha(12)),
            );
        }
        let is_toggle = setting.options.is_empty();
        let is_swatches = setting.key == "accentColorArgb";
        let is_app_icon = setting.key == "appIcon";
        let sheet_row = (compact || tv || is_app_icon) && !is_toggle && !is_swatches;
        let value_below =
            sheet_row && !is_app_icon && compact && viewport.platform == UiPlatform::Android;
        let mut clicked = false;
        if sheet_row {
            layout.focusable.push((node, row_rect));
        }
        if is_toggle {
            layout.focusable.push((node, row_rect));
            egui::Area::new(Id::new(("fluxa-settings-row", node)))
                .constrain(false)
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
            metrics.nav_label_size + 2.0
        };
        let subtitle_size = metrics.screen_card_subtitle_size + 2.0;
        let value_size = if !compact && !tv {
            metrics.settings_row_value_size_desktop
        } else {
            metrics.screen_card_subtitle_size + 1.0
        };
        let swatch_size = if compact { 22.0 } else { 28.0 };
        let swatch_gap = if compact { 8.0 } else { 12.0 };
        let raw_value = (!is_toggle).then(|| {
            settings
                .value(setting.key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or(setting.options[0])
        });
        let value = raw_value.map(|raw| option_label(setting.key, raw, language));
        let value_font = crate::fonts::regular(value_size);
        let value_width = if value_below || is_app_icon {
            0.0
        } else if sheet_row {
            value.as_deref().map_or(0.0, |value| {
                painter
                    .layout_no_wrap(value.to_owned(), value_font.clone(), Color32::WHITE)
                    .size()
                    .x
                    .min(row_rect.width() * 0.4)
            }) + 22.0
        } else {
            setting
                .options
                .iter()
                .map(|option| {
                    painter
                        .layout_no_wrap(
                            option_label(setting.key, option, language),
                            value_font.clone(),
                            Color32::WHITE,
                        )
                        .size()
                        .x
                })
                .fold(0.0, f32::max)
                + 50.0
        }
        .clamp(0.0, row_rect.width() * 0.5)
        .max(if sheet_row { 0.0 } else { 92.0 });
        let value_width = if value_below || is_app_icon {
            0.0
        } else {
            value_width
        };
        let text_left = row_rect.left() + 4.0;
        let control_width = if is_swatches {
            let count = setting.options.len() as f32;
            count * swatch_size + (count - 1.0) * swatch_gap + 6.0
        } else if is_app_icon {
            (row_rect.height() - 16.0).min(44.0) + 4.0
        } else if is_toggle {
            metrics.settings_toggle_width + 3.0
        } else {
            value_width + 2.0
        };
        let text_width = (row_rect.right() - control_width - 12.0 - text_left).max(1.0);
        let label_font = crate::fonts::regular(row_label_size);
        let subtitle_font = crate::fonts::regular(subtitle_size);
        let text_height = row_label_size + 4.0 + subtitle_size;
        let text_top = row_rect.center().y - text_height * 0.5;
        painter.text(
            Pos2::new(text_left, text_top),
            Align2::LEFT_TOP,
            truncate_to_width(
                &painter,
                &settings_row_label(setting, language),
                &label_font,
                text_width,
            ),
            label_font.clone(),
            Color32::from_white_alpha(225),
        );
        let subtitle_top = text_top + row_label_size + 4.0;
        if is_app_icon {
            let icon = (row_rect.height() - 16.0).min(44.0);
            if let Some(texture) = raw_value.and_then(|id| assets.app_icon(id)) {
                painter.image(
                    texture,
                    Rect::from_min_size(
                        Pos2::new(
                            row_rect.right() - 4.0 - icon,
                            row_rect.center().y - icon * 0.5,
                        ),
                        Vec2::splat(icon),
                    ),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        }
        {
            painter.text(
                Pos2::new(text_left, subtitle_top),
                Align2::LEFT_TOP,
                truncate_to_width(
                    &painter,
                    &if value_below || is_app_icon {
                        value.clone().unwrap_or_default()
                    } else {
                        localized(
                            &format!("settings.row.{}.description", setting.key),
                            language,
                        )
                    },
                    &subtitle_font,
                    text_width,
                ),
                subtitle_font,
                if value_below || is_app_icon {
                    Color32::from_white_alpha(160)
                } else {
                    Color32::from_white_alpha(if compact { 100 } else { 120 })
                },
            );
        }
        if is_swatches {
            let current = raw_value.unwrap_or_default().to_ascii_uppercase();
            let size = swatch_size;
            let gap = swatch_gap;
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
                .constrain(false)
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
                            ui.painter().circle_filled(
                                center,
                                size * 0.5 - 4.0 + 2.0 * grow,
                                color,
                            );
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
        } else if sheet_row {
            if !value_below && !is_app_icon {
                let tip = row_rect.right_center() - Vec2::new(6.0, 0.0);
                let stroke = egui::Stroke::new(1.6, Color32::from_white_alpha(110));
                painter.line_segment([tip + Vec2::new(-5.0, -6.0), tip], stroke);
                painter.line_segment([tip + Vec2::new(-5.0, 6.0), tip], stroke);
            }
            if !value_below && !is_app_icon {
                let tip = row_rect.right_center() - Vec2::new(6.0, 0.0);
                painter.text(
                    tip - Vec2::new(16.0, 0.0),
                    Align2::RIGHT_CENTER,
                    truncate_to_width(
                        &painter,
                        value.as_deref().unwrap_or_default(),
                        &value_font,
                        row_rect.width() * 0.4,
                    ),
                    value_font,
                    Color32::from_white_alpha(140),
                );
            }
        } else if let (Some(raw_value), Some(value)) = (raw_value, value) {
            let value_rect = Rect::from_center_size(
                row_rect.right_center() - Vec2::new(value_width * 0.5 + 2.0, 0.0),
                Vec2::new(
                    value_width,
                    metrics.screen_control_height.min(row_rect.height() - 4.0),
                ),
            );
            let mut selected = raw_value.to_owned();
            egui::Area::new(Id::new(("fluxa-settings-dropdown", node)))
                .constrain(false)
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
                        &value,
                        &choices,
                        &mut selected,
                        value_rect.width(),
                        true,
                        false,
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
            components::toggle(
                context,
                &painter,
                Id::new(("fluxa-settings-toggle", node)),
                track_rect,
                enabled,
                metrics,
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
            crate::fonts::regular(15.0),
            Color32::from_rgb(255, 145, 125),
        );
    }
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    components::focus_ring(&painter, &layout.focusable, focused, viewport, metrics);
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
const ACCOUNT_SERVICES: [(&str, &str); 4] = [
    ("Trakt", "traktAccessToken"),
    ("Simkl", "simklAccessToken"),
    ("AniList", "anilistAccessToken"),
    ("MDBList", "mdblistApiKey"),
];
pub const ACCOUNT_PROVIDERS: [&str; 2] = ["trakt", "simkl"];
const ACCOUNT_SOURCES: [(&str, &str); 2] = [
    (
        "settings.integration_library_source",
        "integrationLibrarySource",
    ),
    (
        "settings.continue_watching_source",
        "continueWatchingSource",
    ),
];

fn account_height(metrics: UiMetrics) -> f32 {
    APPEARANCE_PAGE_HEADER_HEIGHT
        + (APPEARANCE_GROUP_HEADING_HEIGHT + APPEARANCE_GROUP_GAP) * 4.0
        + ACCOUNT_PROFILE_CARD_HEIGHT
        + settings_group_card_height(ACCOUNT_SERVICES.len(), metrics)
        + settings_group_card_height(ACCOUNT_SOURCES.len(), metrics)
        + FIELD_HEIGHT * 2.0
}

fn account_group(
    painter: &egui::Painter,
    rect: Rect,
    top: f32,
    height: f32,
    title: &str,
    compact: bool,
) -> Rect {
    if compact {
        painter.text(
            Pos2::new(rect.left() + 16.0, top),
            Align2::LEFT_TOP,
            title,
            crate::fonts::regular(15.0),
            Color32::from_white_alpha(120),
        );
    } else {
        painter.text(
            Pos2::new(rect.left() + 8.0, top),
            Align2::LEFT_TOP,
            title.to_uppercase(),
            crate::fonts::regular(13.0),
            Color32::from_white_alpha(145),
        );
    }
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

fn source_label(source: &str, language: &str) -> String {
    match source {
        "mdblist" => localized("settings.service.mdblist", language),
        _ => localized_or(&format!("settings.option.{source}"), source, language),
    }
}

pub fn account_source_state(
    settings: &SettingsModel,
    key: &str,
) -> (Vec<(String, String)>, String, String) {
    let language = settings.language();
    let options = [
        ("nuvio", "nuvioAccessToken"),
        ("trakt", "traktAccessToken"),
        ("simkl", "simklAccessToken"),
        ("anilist", "anilistAccessToken"),
        ("stremio", "stremioAuthKey"),
        ("mdblist", "mdblistApiKey"),
    ]
    .into_iter()
    .filter(|(_, token_key)| account_connected(settings, token_key))
    .map(|(source, _)| (source.to_owned(), source_label(source, language)))
    .collect::<Vec<_>>();
    let current = settings
        .value(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let label = match options.iter().find(|(source, _)| *source == current) {
        Some((_, label)) => label.clone(),
        None if options.is_empty() => localized("settings.not_connected", language),
        None => localized("settings.option.none", language),
    };
    (options, current, label)
}

pub fn account_source(index: usize) -> Option<(&'static str, &'static str)> {
    ACCOUNT_SOURCES.get(index).copied()
}

fn account_connected(settings: &SettingsModel, key: &str) -> bool {
    settings
        .profile
        .get(key)
        .or_else(|| settings.value(key))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|token| !token.is_empty())
}

fn draw_account(
    context: &egui::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &impl HomeAssets,
    language: &str,
    rect: Rect,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let compact = viewport.is_compact();
    let sheet = compact || viewport.is_tv();
    let painter = context.layer_painter(egui::LayerId::background());
    let label_size = if compact {
        metrics.screen_body_size
    } else {
        metrics.settings_row_label_size_desktop
    };
    let connected = |key: &str| account_connected(settings, key);

    let mut top = rect.top()
        + if compact {
            8.0
        } else {
            APPEARANCE_PAGE_HEADER_HEIGHT
        };
    let card = account_group(
        &painter,
        rect,
        top,
        ACCOUNT_PROFILE_CARD_HEIGHT,
        &localized("settings.group.account_profile", language),
        compact,
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
    let name_left = avatar.right() + 16.0;
    let mut name_right = card.right() - 20.0;
    let switch_label = localized("settings.switch_profiles", language);
    let switch_width = painter
        .layout_no_wrap(
            switch_label.clone(),
            FontId::proportional(label_size),
            Color32::WHITE,
        )
        .size()
        .x
        + 32.0;
    let switch_rect = Rect::from_center_size(
        Pos2::new(card.right() - 20.0 - switch_width * 0.5, card.center().y),
        Vec2::new(switch_width, 36.0),
    );
    if !compact {
        pill_button(
            context,
            layout,
            NODE_SETTINGS_SWITCH_PROFILE,
            switch_rect,
            &switch_label,
            label_size,
        );
        name_right = switch_rect.left() - 12.0;
    }
    let mut name_job = egui::text::LayoutJob::simple_singleline(
        name,
        crate::fonts::regular(label_size + 4.0),
        Color32::WHITE,
    );
    name_job.wrap.max_width = (name_right - name_left).max(1.0);
    name_job.wrap.max_rows = 1;
    name_job.wrap.break_anywhere = true;
    let name_galley = painter.layout_job(name_job);
    painter.galley(
        Pos2::new(name_left, card.center().y - name_galley.size().y * 0.5),
        name_galley,
        Color32::WHITE,
    );

    top = card.bottom() + APPEARANCE_GROUP_GAP;
    let card = account_group(
        &painter,
        rect,
        top,
        settings_group_card_height(ACCOUNT_SERVICES.len(), metrics),
        &localized("settings.group.account_services", language),
        compact,
    );
    for (index, (label, key)) in ACCOUNT_SERVICES.iter().enumerate() {
        let row = account_row(card, index, metrics);
        if index > 0 {
            account_divider(&painter, row, metrics);
        }
        let mut label_left = 4.0;
        if let Some((texture, aspect)) = assets.logo(&label.to_lowercase()) {
            let size = aspect * 22.0;
            painter.image(
                texture,
                Rect::from_min_size(row.left_center() + Vec2::new(4.0, -11.0), size),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            label_left += size.x + 12.0;
        }
        let on = connected(key);
        let provider = label.to_lowercase();
        let prompt = settings
            .account_auth
            .as_ref()
            .filter(|prompt| prompt.provider == provider);
        let status = match prompt {
            Some(prompt) if prompt.failed => localized("settings.account_sign_in_failed", language),
            Some(prompt) => format!(
                "{} {}  ·  {}",
                localized("trakt.device.enter_code", language),
                prompt.code,
                prompt.url
            ),
            None => localized(
                if on {
                    "settings.connected"
                } else {
                    "settings.not_connected"
                },
                language,
            ),
        };
        if sheet {
            let slot = ACCOUNT_PROVIDERS.iter().position(|name| *name == provider);
            if let Some(slot) = slot {
                layout
                    .focusable
                    .push((NODE_SETTINGS_ACCOUNT_BASE + slot as u64, row));
            }
            let status_font = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
            let tip = row.right_center() - Vec2::new(6.0, 0.0);
            let right = if slot.is_some() {
                tip.x - 16.0
            } else {
                row.right() - 4.0
            };
            if viewport.platform == UiPlatform::Android {
                let text_top = row.center().y - (label_size + 4.0 + status_font.size) * 0.5;
                painter.text(
                    Pos2::new(row.left() + label_left, text_top),
                    Align2::LEFT_TOP,
                    *label,
                    crate::fonts::regular(label_size),
                    Color32::from_white_alpha(225),
                );
                painter.text(
                    Pos2::new(row.left() + label_left, text_top + label_size + 4.0),
                    Align2::LEFT_TOP,
                    truncate_to_width(
                        &painter,
                        &status,
                        &status_font,
                        right - row.left() - label_left,
                    ),
                    status_font,
                    Color32::from_white_alpha(if on { 200 } else { 160 }),
                );
            } else {
                painter.text(
                    row.left_center() + Vec2::new(label_left, 0.0),
                    Align2::LEFT_CENTER,
                    *label,
                    crate::fonts::regular(label_size),
                    Color32::from_white_alpha(225),
                );
                painter.text(
                    Pos2::new(right, row.center().y),
                    Align2::RIGHT_CENTER,
                    status,
                    status_font,
                    Color32::from_white_alpha(140),
                );
            }
            if slot.is_some() && viewport.platform != UiPlatform::Android {
                let stroke = egui::Stroke::new(1.6, Color32::from_white_alpha(110));
                painter.line_segment([tip + Vec2::new(-5.0, -6.0), tip], stroke);
                painter.line_segment([tip + Vec2::new(-5.0, 6.0), tip], stroke);
            }
            continue;
        }
        painter.text(
            row.left_center() + Vec2::new(label_left, 0.0),
            Align2::LEFT_CENTER,
            *label,
            FontId::proportional(label_size),
            Color32::from_white_alpha(210),
        );
        let mut status_right = 8.0;
        if let Some(slot) = ACCOUNT_PROVIDERS.iter().position(|name| *name == provider) {
            let action = localized(
                if on {
                    "settings.account_disconnect"
                } else {
                    "settings.account_connect"
                },
                language,
            );
            let width = painter
                .layout_no_wrap(
                    action.clone(),
                    FontId::proportional(label_size),
                    Color32::WHITE,
                )
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
                NODE_SETTINGS_ACCOUNT_BASE + slot as u64,
                button,
                &action,
                label_size,
            );
            status_right += width + 12.0;
        }
        painter.text(
            row.right_center() - Vec2::new(status_right, 0.0),
            Align2::RIGHT_CENTER,
            status,
            crate::fonts::regular(metrics.screen_card_subtitle_size),
            Color32::from_white_alpha(if on { 200 } else { 110 }),
        );
    }

    top = card.bottom() + APPEARANCE_GROUP_GAP;
    let card = account_group(
        &painter,
        rect,
        top,
        settings_group_card_height(ACCOUNT_SOURCES.len(), metrics),
        &localized("settings.group.account_sync", language),
        compact,
    );
    for (index, (label_key, key)) in ACCOUNT_SOURCES.iter().enumerate() {
        let row = account_row(card, index, metrics);
        if index > 0 {
            account_divider(&painter, row, metrics);
        }
        let (options, current, current_label) = account_source_state(settings, key);
        let mut selected = current.clone();
        if sheet {
            let node = NODE_SETTINGS_ACCOUNT_SOURCE_BASE + index as u64;
            layout.focusable.push((node, row));
            let title = localized(label_key, language);
            let value_font = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
            let tip = row.right_center() - Vec2::new(6.0, 0.0);
            if viewport.platform == UiPlatform::Android {
                let text_top = row.center().y - (label_size + 4.0 + value_font.size) * 0.5;
                painter.text(
                    Pos2::new(row.left() + 4.0, text_top),
                    Align2::LEFT_TOP,
                    title,
                    crate::fonts::regular(label_size),
                    Color32::from_white_alpha(225),
                );
                painter.text(
                    Pos2::new(row.left() + 4.0, text_top + label_size + 4.0),
                    Align2::LEFT_TOP,
                    current_label,
                    value_font,
                    Color32::from_white_alpha(160),
                );
            } else {
                painter.text(
                    row.left_center() + Vec2::new(4.0, 0.0),
                    Align2::LEFT_CENTER,
                    title,
                    crate::fonts::regular(label_size),
                    Color32::from_white_alpha(225),
                );
                painter.text(
                    tip - Vec2::new(16.0, 0.0),
                    Align2::RIGHT_CENTER,
                    current_label,
                    value_font,
                    Color32::from_white_alpha(140),
                );
                let stroke = egui::Stroke::new(1.6, Color32::from_white_alpha(110));
                painter.line_segment([tip + Vec2::new(-5.0, -6.0), tip], stroke);
                painter.line_segment([tip + Vec2::new(-5.0, 6.0), tip], stroke);
            }
            continue;
        }
        painter.text(
            row.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            localized(label_key, language),
            FontId::proportional(label_size),
            Color32::from_white_alpha(210),
        );
        let value_width = if compact { 132.0 } else { 176.0 };
        let value_rect = Rect::from_min_size(
            Pos2::new(row.right() - 2.0 - value_width, row.center().y),
            Vec2::new(
                value_width,
                metrics.screen_control_height.min(row.height() - 4.0),
            ),
        )
        .translate(Vec2::new(
            0.0,
            -metrics.screen_control_height.min(row.height() - 4.0) * 0.5,
        ));
        egui::Area::new(Id::new(("fluxa-settings-account-choice", *key)))
            .constrain(false)
            .fixed_pos(value_rect.min)
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                ui.set_min_size(value_rect.size());
                ui.set_max_size(value_rect.size());
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
                    false,
                );
                layout.focusable.push((
                    NODE_SETTINGS_ACCOUNT_SOURCE_BASE + index as u64,
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
        compact,
        layout,
    );
}
