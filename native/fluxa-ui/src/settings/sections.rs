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

pub(super) const ACCOUNT_SETTINGS: [SettingsRow; 4] = [
    SettingsRow {
        label: "Language",
        key: "language",
        options: &["en", "tr", "ar", "he"],
    },
    SettingsRow {
        label: "Notifications",
        key: "notificationsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Discord Rich Presence",
        key: "discordRichPresenceEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Save recent searches",
        key: "saveRecentSearches",
        options: &[],
    },
];
pub(super) const ACCOUNT_GROUPS: [(usize, &str); 1] = [(4, "settings.group.preferences")];
pub(super) const ABOUT_SETTINGS: [SettingsRow; 2] = [
    SettingsRow {
        label: "Update checks",
        key: "automaticUpdates",
        options: &[],
    },
    SettingsRow {
        label: "Crash reports",
        key: "crashReports",
        options: &[],
    },
];
pub(super) const ABOUT_GROUPS: [(usize, &str); 2] = [
    (1, "settings.group.updates"),
    (1, "settings.group.privacy"),
];
pub(super) const APPEARANCE_SETTINGS: [SettingsRow; 12] = [
    SettingsRow {
        label: "Accent color",
        key: "accentColorArgb",
        options: &[
            "glass", "#FFFFFF", "#E50914", "#3F7CFF", "#54D17A", "#FF8A3D", "#C084FC",
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
pub(super) const APPEARANCE_GROUPS: [(usize, &str); 2] = [
    (5, "settings.group.color_and_motion"),
    (5, "settings.group.navigation"),
];
pub(super) const HOME_SETTINGS: [SettingsRow; 19] = [
    SettingsRow {
        label: "Show hero section",
        key: "showHeroSection",
        options: &[],
    },
    SettingsRow {
        label: "Hero follows focus",
        key: "followFocusHero",
        options: &[],
    },
    SettingsRow {
        label: "Hero artwork",
        key: "homeHeroArtwork",
        options: &["backdrop", "poster"],
    },
    SettingsRow {
        label: "Hero season posters",
        key: "homeSeasonPostersOnHero",
        options: &[],
    },
    SettingsRow {
        label: "Hero ratings",
        key: "homeHeroRatings",
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
        label: "Shorten synopsis",
        key: "shortenSynopsis",
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
        label: "Continue watching badge",
        key: "continueProgressLabel",
        options: &["remaining", "percent"],
    },
    SettingsRow {
        label: "Upcoming row",
        key: "showUpcomingRow",
        options: &[],
    },
];
pub(super) const AMBIENT_SETTINGS: [SettingsRow; 2] = [
    SettingsRow {
        label: "Enabled",
        key: "ambientModeEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Start After",
        key: "ambientModeTimeoutMinutes",
        options: &["1", "2", "5", "10", "15", "30"],
    },
];
pub(super) const AMBIENT_GROUPS: [(usize, &str); 1] = [(2, "settings.section.ambient_mode")];
pub(super) const HOME_GROUPS: [(usize, &str); 2] = [
    (9, "settings.group.home"),
    (10, "settings.group.continue_watching"),
];
pub(super) const DETAILS_SETTINGS: [SettingsRow; 12] = [
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
        label: "Episode descriptions",
        key: "detailShowEpisodeDescriptions",
        options: &[],
    },
    SettingsRow {
        label: "Trakt comments",
        key: "traktCommentsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Season selector style",
        key: "detailSeasonSelectorMode",
        options: &["posters", "chips", "dropdown"],
    },
    SettingsRow {
        label: "Episode card layout",
        key: "episodeCardsLayout",
        options: &["auto", "cards", "list", "grid"],
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
pub(super) const DETAILS_GROUPS: [(usize, &str); 2] = [
    (8, "settings.group.detail_page"),
    (4, "settings.group.episodes"),
];
pub(super) const STREAM_BADGES_SETTINGS: [SettingsRow; 7] = [
    SettingsRow {
        label: "Stream badges",
        key: "streamBadgesEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Built-in badges",
        key: "streamBadgeBuiltIn",
        options: &[],
    },
    SettingsRow {
        label: "File size badge",
        key: "streamBadgeFileSize",
        options: &[],
    },
    SettingsRow {
        label: "Badge theme",
        key: "streamBadgeTheme",
        options: &["vivid", "classic", "soft", "neon", "gold", "sunset"],
    },
    SettingsRow {
        label: "Badge placement",
        key: "streamBadgePlacement",
        options: &["bottom", "top"],
    },
    SettingsRow {
        label: "Card layout",
        key: "streamLayout",
        options: &["off", "detailed", "compact", "minimal", "custom"],
    },
    SettingsRow {
        label: "Text formatter",
        key: "streamFormatterEnabled",
        options: &[],
    },
];

pub(super) const POSTERS_SETTINGS: [SettingsRow; 33] = [
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
    SettingsRow {
        label: "Top numbers",
        key: "posterTopNumbers",
        options: &[],
    },
    SettingsRow {
        label: "Numbered rows",
        key: "posterTopNumbersRows",
        options: &["trending", "all"],
    },
    SettingsRow {
        label: "Numbered titles",
        key: "posterTopNumbersCount",
        options: &["3", "5", "10", "20"],
    },
    SettingsRow {
        label: "Number style",
        key: "posterTopNumbersStyle",
        options: &["outline", "solid", "ghost"],
    },
    SettingsRow {
        label: "Number color",
        key: "posterTopNumbersColor",
        options: &["white", "gray", "accent"],
    },
    SettingsRow {
        label: "Number size",
        key: "posterTopNumbersSize",
        options: &["default", "small", "large"],
    },
    SettingsRow {
        label: "Number position",
        key: "posterTopNumbersPosition",
        options: &["beside", "overlay"],
    },
    SettingsRow {
        label: "Expanded posters",
        key: "expandedPosters",
        options: &[],
    },
    SettingsRow {
        label: "Expand delay",
        key: "expandedPostersDelay",
        options: &["0", "1", "2", "3"],
    },
    SettingsRow {
        label: "Trailer on expanded posters",
        key: "trailerOnExpandedPosters",
        options: &[],
    },
    SettingsRow {
        label: "Expanded poster trailer delay",
        key: "trailerOnExpandedPostersDelay",
        options: &["0", "2", "4", "6"],
    },
];
pub(super) const POSTERS_GROUPS: [(usize, &str); 6] = [
    (6, "settings.group.cards"),
    (4, "settings.group.poster_overlays"),
    (3, "settings.group.poster_rating"),
    (9, "settings.group.poster_badges"),
    (7, "settings.group.poster_top_numbers"),
    (4, "settings.group.expanded_posters"),
];
pub(super) const PREVIEWS_SETTINGS: [SettingsRow; 3] = [
    SettingsRow {
        label: "Generation mode",
        key: "trickplayMode",
        options: &["async", "pregenerate"],
    },
    SettingsRow {
        label: "Frame interval",
        key: "trickplayInterval",
        options: &["5", "10", "20", "30", "60"],
    },
    SettingsRow {
        label: "Preview sheet",
        key: "trickplaySheet",
        options: &[],
    },
];
pub(super) const PREVIEWS_GROUPS: [(usize, &str); 1] = [(3, "settings.group.seek_previews")];
pub(super) const PLAYBACK_SETTINGS: [SettingsRow; 34] = [
    SettingsRow {
        label: "Playback destination",
        key: "preferredPlayer",
        options: &["mpv", "external"],
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
        label: "Mark watched at",
        key: "scrobbleWatchedPercent",
        options: &["50", "60", "70", "80", "90", "95", "100"],
    },
    SettingsRow {
        label: "Countdown duration",
        key: "autoPlayCountdownSecs",
        options: &["5", "7", "10", "15"],
    },
    SettingsRow {
        label: "Prefer binge group",
        key: "tryBingeGroup",
        options: &[],
    },
    SettingsRow {
        label: "Retry next source",
        key: "autoRetryNextSource",
        options: &[],
    },
    SettingsRow {
        label: "Background playback",
        key: "backgroundPlayback",
        options: &[],
    },
    SettingsRow {
        label: "Picture in picture",
        key: "pictureInPicture",
        options: &[],
    },
    SettingsRow {
        label: "Reuse last stream",
        key: "reuseLastStream",
        options: &[],
    },
    SettingsRow {
        label: "Reuse last stream TTL",
        key: "reuseLastStreamTtlHours",
        options: &[
            "1", "2", "3", "4", "5", "6", "24", "48", "72", "96", "120", "144",
        ],
    },
    SettingsRow {
        label: "Preload next episode sources",
        key: "preloadNextEpisodeSources",
        options: &[],
    },
    SettingsRow {
        label: "Reuse binge group",
        key: "reuseBingeGroup",
        options: &[],
    },
    SettingsRow {
        label: "P2P playback",
        key: "p2pEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Dolby Vision",
        key: "dolbyVisionMode",
        options: &["auto", "native", "convert", "base_layer", "compose"],
    },
    SettingsRow {
        label: "Dolby Vision tone mapping",
        key: "dolbyVisionTonemap",
        options: &["auto", "no", "yes", "sdr"],
    },
    SettingsRow {
        label: "Display peak brightness",
        key: "displayPeakNits",
        options: &["0", "300", "400", "600", "1000", "1500", "4000"],
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
pub(super) const PLAYBACK_GROUPS: [(usize, &str); 6] = [
    (8, "settings.group.playback"),
    (4, "settings.group.stream_reuse"),
    (4, "settings.group.sources_video"),
    (9, "settings.group.skip_controls"),
    (4, "settings.group.upscaling_motion"),
    (4, "settings.group.player_advanced"),
];
pub(super) const SUBTITLES_SETTINGS: [SettingsRow; 18] = [
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
pub(super) const SUBTITLES_GROUPS: [(usize, &str); 2] = [
    (6, "settings.group.audio_subtitles"),
    (12, "settings.group.subtitle_style"),
];
pub(super) const TMDB_SETTINGS: [SettingsRow; 16] = [
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
        label: "TMDB trailers",
        key: "tmdbTrailersEnabled",
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
];
pub(super) const TMDB_GROUPS: [(usize, &str); 3] = [
    (4, "settings.tmdb_group_images"),
    (9, "settings.tmdb_group_media_info"),
    (3, "settings.tmdb_group_discovery"),
];
pub(super) const DOWNLOADS_SETTINGS: [SettingsRow; 5] = [
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
pub(super) const DOWNLOADS_GROUPS: [(usize, &str); 1] = [(5, "settings.group.downloads")];
pub(super) const PLAYER_SETTINGS: [SettingsRow; 17] = [
    SettingsRow {
        label: "Pause screen info",
        key: "pauseMetadataOverlayEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Content warnings",
        key: "contentWarningsEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Content info",
        key: "playerShowTitle",
        options: &[],
    },
    SettingsRow {
        label: "Up next card",
        key: "playerShowUpNextCard",
        options: &[],
    },
    SettingsRow {
        label: "Audio and subtitles",
        key: "playerShowAudioSubtitles",
        options: &[],
    },
    SettingsRow {
        label: "Speed",
        key: "playerShowSpeed",
        options: &[],
    },
    SettingsRow {
        label: "Episode picker",
        key: "playerShowEpisodes",
        options: &[],
    },
    SettingsRow {
        label: "Next episode button",
        key: "playerShowNextEpisode",
        options: &[],
    },
    SettingsRow {
        label: "Player settings",
        key: "playerShowSettings",
        options: &[],
    },
    SettingsRow {
        label: "Volume",
        key: "playerShowVolume",
        options: &[],
    },
    SettingsRow {
        label: "Fullscreen",
        key: "playerShowFullscreen",
        options: &[],
    },
    SettingsRow {
        label: "Cast",
        key: "playerShowCast",
        options: &[],
    },
    SettingsRow {
        label: "Mark segment",
        key: "playerShowMarkSegment",
        options: &[],
    },
    SettingsRow {
        label: "Segments on seek bar",
        key: "playerShowSegments",
        options: &[],
    },
    SettingsRow {
        label: "Brightness and volume gestures",
        key: "playerGestures",
        options: &[],
    },
    SettingsRow {
        label: "Double-tap to seek",
        key: "playerDoubleTapSeek",
        options: &[],
    },
    SettingsRow {
        label: "Controls in the middle",
        key: "playerCenterControls",
        options: &[],
    },
];
pub(super) const PLAYER_GROUPS: [(usize, &str); 3] = [
    (4, "settings.group.player_overlay"),
    (10, "settings.group.player_controls"),
    (3, "settings.group.player_touch"),
];
pub(super) const EMPTY_SETTINGS: [SettingsRow; 0] = [];
pub const SETTINGS_SECTIONS: [SettingsSection; 25] = [
    SettingsSection {
        title: "Account",
        category: "Account",
        description: "Profile, language and services",
        rows: &ACCOUNT_SETTINGS,
        groups: &ACCOUNT_GROUPS,
    },
    SettingsSection {
        title: "TMDB",
        category: "Account",
        description: "API key and metadata enrichment",
        rows: &TMDB_SETTINGS,
        groups: &TMDB_GROUPS,
    },
    SettingsSection {
        title: "MDBList",
        category: "Account",
        description: "API key and library list",
        rows: &EMPTY_SETTINGS,
        groups: &[],
    },
    SettingsSection {
        title: "Skip Segments",
        category: "Account",
        description: "IntroDB, TheIntroDB and SkipDB keys",
        rows: &EMPTY_SETTINGS,
        groups: &[],
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
        title: "Ambient Mode",
        category: "Appearance",
        description: "Show artwork from your catalogs while the TV sits idle",
        rows: &AMBIENT_SETTINGS,
        groups: &AMBIENT_GROUPS,
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
        title: "Player",
        category: "Playback",
        description: "Overlay, controls and gestures while watching",
        rows: &PLAYER_SETTINGS,
        groups: &PLAYER_GROUPS,
    },
    SettingsSection {
        title: "Previews",
        category: "Playback",
        description: "Seek thumbnails, their source and how they are built",
        rows: &PREVIEWS_SETTINGS,
        groups: &PREVIEWS_GROUPS,
    },
    SettingsSection {
        title: "Badges",
        category: "Playback",
        description: "Badges on the source list, with regex rules and packs",
        rows: &STREAM_BADGES_SETTINGS,
        groups: &[],
    },
    SettingsSection {
        title: "Summary",
        category: "Playback",
        description: "Details of your last playback",
        rows: &EMPTY_SETTINGS,
        groups: &[],
    },
    SettingsSection {
        title: "Watch Party",
        category: "Watch Party",
        description: "Create or join a room to watch together",
        rows: &EMPTY_SETTINGS,
        groups: &[],
    },
    SettingsSection {
        title: "Content",
        category: "Content",
        description: "Add-ons, plugins and servers",
        rows: &EMPTY_SETTINGS,
        groups: &[],
    },
    SettingsSection {
        title: "Collections",
        category: "Content",
        description: "Folders of catalogs on the home screen",
        rows: &EMPTY_SETTINGS,
        groups: &[],
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
        title: "Servers",
        category: "Content",
        description: "Jellyfin, Emby and Plex libraries",
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
    SettingsSection {
        title: "About",
        category: "About",
        description: "Version and updates",
        rows: &ABOUT_SETTINGS,
        groups: &ABOUT_GROUPS,
    },
    SettingsSection {
        title: "Licenses",
        category: "About",
        description: "Open source and data credits",
        rows: &EMPTY_SETTINGS,
        groups: &[],
    },
];
