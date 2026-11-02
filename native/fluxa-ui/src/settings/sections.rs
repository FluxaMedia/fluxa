#[derive(Clone, Debug)]
pub struct SettingsRow {
    pub label: &'static str,
    pub key: &'static str,
    pub fixed: &'static [&'static str],
}

impl SettingsRow {
    pub fn choices(&self) -> &'static [&'static str] {
        if self.key == "themeId" {
            fluxa_theme::theme::choice_keys()
        } else {
            self.fixed
        }
    }
}

const TTL_HOURS: [f64; 53] = [
    1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0,
    18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 48.0, 72.0, 96.0, 120.0, 144.0, 168.0, 192.0, 216.0,
    240.0, 264.0, 288.0, 312.0, 336.0, 360.0, 384.0, 408.0, 432.0, 456.0, 480.0, 504.0, 528.0,
    552.0, 576.0, 600.0, 624.0, 648.0, 672.0, 696.0, 720.0,
];

#[derive(Clone, Copy, Debug)]
pub struct SettingsRange {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub stops: &'static [f64],
    pub marks: &'static [f64],
}

impl SettingsRange {
    fn nearest(&self, value: f64) -> usize {
        self.stops
            .iter()
            .enumerate()
            .min_by(|a, b| (a.1 - value).abs().total_cmp(&(b.1 - value).abs()))
            .map_or(0, |(index, _)| index)
    }

    pub fn snap(&self, ratio: f32) -> f64 {
        if !self.stops.is_empty() {
            let last = self.stops.len() - 1;
            return self.stops[(ratio.clamp(0.0, 1.0) * last as f32).round() as usize];
        }
        let raw = self.min + f64::from(ratio.clamp(0.0, 1.0)) * (self.max - self.min);
        ((raw / self.step).round() * self.step).clamp(self.min, self.max)
    }

    pub fn ratio(&self, value: f64) -> f32 {
        if !self.stops.is_empty() {
            return self.nearest(value) as f32 / (self.stops.len() - 1) as f32;
        }
        ((value.clamp(self.min, self.max) - self.min) / (self.max - self.min)) as f32
    }

    pub fn shift(&self, value: f64, direction: i32) -> Option<f64> {
        if !self.stops.is_empty() {
            let index = self.nearest(value) as i32 + direction;
            return self.stops.get(usize::try_from(index).ok()?).copied();
        }
        let target = value + f64::from(direction) * self.step;
        (target >= self.min - 1e-9 && target <= self.max + 1e-9).then_some(target)
    }

    pub fn format(&self, value: f64) -> String {
        if self.step >= 1.0 {
            format!("{}", value.round() as i64)
        } else {
            format!("{}", (value * 100.0).round() / 100.0)
        }
    }
}

pub fn slider_range(key: &str) -> Option<SettingsRange> {
    let (min, max, step, marks): (f64, f64, f64, &'static [f64]) = match key {
        "uiScale" => (75.0, 150.0, 5.0, &[75.0, 100.0, 150.0]),
        "subtitleSize" => (50.0, 200.0, 5.0, &[50.0, 100.0, 200.0]),
        "scrobbleWatchedPercent" => (50.0, 100.0, 1.0, &[50.0, 75.0, 100.0]),
        "autoPlayCountdownSecs" => (1.0, 30.0, 1.0, &[1.0, 10.0, 20.0, 30.0]),
        "reuseLastStreamTtlHours" => (1.0, 720.0, 1.0, &[1.0, 12.0, 24.0, 168.0, 720.0]),
        "homeHeroAutoplayTrailerDelaySecs" | "detailHeroAutoplayTrailerDelaySecs" => {
            (0.0, 10.0, 1.0, &[0.0, 5.0, 10.0])
        }
        "trickplayInterval" => (5.0, 60.0, 5.0, &[5.0, 30.0, 60.0]),
        "playerForwardBufferSeconds" => (10.0, 600.0, 10.0, &[10.0, 300.0, 600.0]),
        "playerBackBufferSeconds" => (0.0, 300.0, 5.0, &[0.0, 60.0, 300.0]),
        "subtitleTextOpacity" | "subtitleBackgroundOpacity" | "subtitleOutlineOpacity" => {
            (0.0, 1.0, 0.05, &[0.0, 0.5, 1.0])
        }
        "posterHoverPreviewDelay" => (0.0, 5.0, 0.1, &[0.0, 2.5, 5.0]),
        _ => return None,
    };
    let stops: &'static [f64] = if key == "reuseLastStreamTtlHours" {
        &TTL_HOURS
    } else {
        &[]
    };
    Some(SettingsRange {
        min,
        max,
        step,
        stops,
        marks,
    })
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
        fixed: &["en", "tr", "ar", "he"],
    },
    SettingsRow {
        label: "Notifications",
        key: "notificationsEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Discord Rich Presence",
        key: "discordRichPresenceEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Save recent searches",
        key: "saveRecentSearches",
        fixed: &[],
    },
];
pub(super) const ACCOUNT_GROUPS: [(usize, &str); 1] = [(4, "settings.group.preferences")];
pub(super) const ABOUT_SETTINGS: [SettingsRow; 2] = [
    SettingsRow {
        label: "Update checks",
        key: "automaticUpdates",
        fixed: &[],
    },
    SettingsRow {
        label: "Crash reports",
        key: "crashReports",
        fixed: &[],
    },
];
pub(super) const ABOUT_GROUPS: [(usize, &str); 1] = [(2, "settings.group.updates_privacy")];
pub(super) const APPEARANCE_SETTINGS: [SettingsRow; 13] = [
    SettingsRow {
        label: "Theme",
        key: "themeId",
        fixed: &["fluxa-dark", "amoled", "midnight-blue"],
    },
    SettingsRow {
        label: "Accent color",
        key: "accentColorArgb",
        fixed: &[
            "glass", "#FFFFFF", "#E50914", "#3F7CFF", "#54D17A", "#FF8A3D", "#C084FC",
        ],
    },
    SettingsRow {
        label: "UI scale",
        key: "uiScale",
        fixed: &[
            "75", "80", "85", "90", "95", "100", "105", "110", "115", "120", "125", "130", "135",
            "140", "145", "150",
        ],
    },
    SettingsRow {
        label: "AMOLED black",
        key: "amoledMode",
        fixed: &[],
    },
    SettingsRow {
        label: "GIF autoplay",
        key: "gifAutoplayEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Show catalog type",
        key: "showCatalogType",
        fixed: &[],
    },
    SettingsRow {
        label: "App icon",
        key: "appIcon",
        fixed: &[
            "ember", "sunset", "ocean", "dusk", "lagoon", "ruby", "citrus", "peach", "lime",
            "aurora", "neon", "flare", "ice", "rose", "silver", "gold", "jade", "blaze", "orange",
        ],
    },
    SettingsRow {
        label: "Navigation layout",
        key: "navLayout",
        fixed: &["sidebar", "topbar"],
    },
    SettingsRow {
        label: "Navigation mode",
        key: "navMode",
        fixed: &["compact", "classic", "adaptive"],
    },
    SettingsRow {
        label: "Navigation labels",
        key: "navSidebarMode",
        fixed: &["hover", "expanded", "collapsed"],
    },
    SettingsRow {
        label: "Home categories",
        key: "homeCategoryBar",
        fixed: &[],
    },
    SettingsRow {
        label: "Interface density",
        key: "interfaceDensity",
        fixed: &["small", "medium", "large"],
    },
    SettingsRow {
        label: "Navigation labels",
        key: "navLabels",
        fixed: &[],
    },
];
pub(super) const APPEARANCE_GROUPS: [(usize, &str); 2] = [
    (6, "settings.group.color_and_motion"),
    (7, "settings.group.navigation"),
];
pub(super) const HOME_SETTINGS: [SettingsRow; 19] = [
    SettingsRow {
        label: "Show hero section",
        key: "showHeroSection",
        fixed: &[],
    },
    SettingsRow {
        label: "Hero follows focus",
        key: "followFocusHero",
        fixed: &[],
    },
    SettingsRow {
        label: "Hero artwork",
        key: "homeHeroArtwork",
        fixed: &["backdrop", "poster"],
    },
    SettingsRow {
        label: "Hero season posters",
        key: "homeSeasonPostersOnHero",
        fixed: &[],
    },
    SettingsRow {
        label: "Hero ratings",
        key: "homeHeroRatings",
        fixed: &[],
    },
    SettingsRow {
        label: "Autoplay home hero trailer",
        key: "homeHeroAutoplayTrailer",
        fixed: &[],
    },
    SettingsRow {
        label: "Home trailer delay",
        key: "homeHeroAutoplayTrailerDelaySecs",
        fixed: &["2", "4", "6", "10"],
    },
    SettingsRow {
        label: "Shorten synopsis",
        key: "shortenSynopsis",
        fixed: &[],
    },
    SettingsRow {
        label: "Catalog type suffix",
        key: "catalogTypeSuffixEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Continue Watching shelf",
        key: "continueWatchingEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Continue Watching layout",
        key: "continueWatchingLayout",
        fixed: &["vertical", "horizontal", "inherit"],
    },
    SettingsRow {
        label: "Continue Watching artwork",
        key: "continueWatchingArtwork",
        fixed: &["episode", "poster", "background"],
    },
    SettingsRow {
        label: "Remaining time display",
        key: "continueWatchingRemainingFormat",
        fixed: &["time", "percent"],
    },
    SettingsRow {
        label: "Progress direction",
        key: "continueWatchingProgressDirection",
        fixed: &["remaining", "watched"],
    },
    SettingsRow {
        label: "Hide Continue Watching titles",
        key: "continueWatchingHideTitles",
        fixed: &[],
    },
    SettingsRow {
        label: "Keep scheduled titles",
        key: "continueWatchingKeepScheduled",
        fixed: &[],
    },
    SettingsRow {
        label: "Show This Week",
        key: "continueWatchingShowThisWeek",
        fixed: &[],
    },
    SettingsRow {
        label: "Continue watching badge",
        key: "continueProgressLabel",
        fixed: &["remaining", "percent"],
    },
    SettingsRow {
        label: "Upcoming row",
        key: "showUpcomingRow",
        fixed: &[],
    },
];
pub(super) const AMBIENT_SETTINGS: [SettingsRow; 2] = [
    SettingsRow {
        label: "Enabled",
        key: "ambientModeEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Start After",
        key: "ambientModeTimeoutMinutes",
        fixed: &["1", "2", "5", "10", "15", "30"],
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
        fixed: &[],
    },
    SettingsRow {
        label: "Prefer clear logo",
        key: "detailPreferClearlogo",
        fixed: &[],
    },
    SettingsRow {
        label: "Detail hero season posters",
        key: "detailSeasonPostersOnHero",
        fixed: &[],
    },
    SettingsRow {
        label: "Trailer on detail hero",
        key: "trailerOnHero",
        fixed: &[],
    },
    SettingsRow {
        label: "Autoplay detail hero trailer",
        key: "detailHeroAutoplayTrailer",
        fixed: &[],
    },
    SettingsRow {
        label: "Detail trailer delay",
        key: "detailHeroAutoplayTrailerDelaySecs",
        fixed: &["2", "4", "6", "10"],
    },
    SettingsRow {
        label: "Episode descriptions",
        key: "detailShowEpisodeDescriptions",
        fixed: &[],
    },
    SettingsRow {
        label: "Trakt comments",
        key: "traktCommentsEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Season selector style",
        key: "detailSeasonSelectorMode",
        fixed: &["posters", "chips", "dropdown"],
    },
    SettingsRow {
        label: "Episode card layout",
        key: "episodeCardsLayout",
        fixed: &["auto", "cards", "list", "grid"],
    },
    SettingsRow {
        label: "Blur unwatched episodes",
        key: "blurUnwatchedEpisodes",
        fixed: &[],
    },
    SettingsRow {
        label: "Hide episode info spoilers",
        key: "spoilerHideEpisodeInfo",
        fixed: &[],
    },
];
pub(super) const DETAILS_GROUPS: [(usize, &str); 2] = [
    (8, "settings.group.detail_page"),
    (4, "settings.group.episodes"),
];
pub(super) const STREAMKIT_SETTINGS: [SettingsRow; 11] = [
    SettingsRow {
        label: "Enable Streamkit",
        key: "streamkitEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Sort order",
        key: "streamSortOrder",
        fixed: &[
            "addon", "quality", "size", "seeders", "language", "newest", "bitrate",
        ],
    },
    SettingsRow {
        label: "Minimum resolution",
        key: "streamMinResolution",
        fixed: &["any", "480p", "720p", "1080p", "2160p"],
    },
    SettingsRow {
        label: "Hide low quality releases",
        key: "streamHideLowQuality",
        fixed: &[],
    },
    SettingsRow {
        label: "Remove duplicates",
        key: "streamDedupe",
        fixed: &[],
    },
    SettingsRow {
        label: "Duplicate matching",
        key: "streamDedupeBy",
        fixed: &["hash", "filename", "similar", "combined"],
    },
    SettingsRow {
        label: "Hide uncached sources",
        key: "streamHideUncached",
        fixed: &[],
    },
    SettingsRow {
        label: "Cached sources first",
        key: "streamPreferCached",
        fixed: &[],
    },
    SettingsRow {
        label: "Source limit",
        key: "streamLimit",
        fixed: &["off", "5", "10", "20", "50"],
    },
    SettingsRow {
        label: "Formatted source text",
        key: "streamPipelineFormat",
        fixed: &[],
    },
    SettingsRow {
        label: "Debrid service",
        key: "debridProvider",
        fixed: &[
            "none",
            "torbox",
            "realdebrid",
            "premiumize",
            "alldebrid",
            "debridlink",
            "easydebrid",
            "offcloud",
            "putio",
        ],
    },
];
pub(super) const STREAMKIT_GROUPS: [(usize, &str); 4] = [
    (1, "settings.group.kit_status"),
    (8, "settings.group.source_rules"),
    (1, "settings.group.source_text"),
    (1, "settings.group.debrid_service"),
];
pub(super) const METADATAKIT_SETTINGS: [SettingsRow; 18] = [
    SettingsRow {
        label: "Enable Metadatakit",
        key: "metadatakitEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Metadata language",
        key: "metadatakitLanguage",
        fixed: &["en-US", "tr-TR", "de-DE", "fr-FR", "es-ES", "ja-JP"],
    },
    SettingsRow {
        label: "Age rating limit",
        key: "metadatakitAgeRating",
        fixed: &["none", "G", "PG", "PG-13", "R"],
    },
    SettingsRow {
        label: "Anime id mapping",
        key: "metadatakitAnimeMapping",
        fixed: &[],
    },
    SettingsRow {
        label: "TVDB episode order",
        key: "metadatakitEpisodeOrder",
        fixed: &["official", "dvd", "absolute", "alternate"],
    },
    SettingsRow {
        label: "Season numbering",
        key: "metadatakitSeasonNumbering",
        fixed: &["sequential", "years"],
    },
    SettingsRow {
        label: "Metadata cache",
        key: "metadatakitCacheLength",
        fixed: &["short", "normal", "long"],
    },
    SettingsRow {
        label: "Anime season numbering",
        key: "metadatakitAnimeRenumber",
        fixed: &[],
    },
    SettingsRow {
        label: "English artwork only",
        key: "metadatakitEnglishArt",
        fixed: &[],
    },
    SettingsRow {
        label: "Artwork source",
        key: "metadatakitArtSource",
        fixed: &["fanart", "provider"],
    },
    SettingsRow {
        label: "Movie provider",
        key: "metadatakitMovieProvider",
        fixed: &["tmdb", "imdb"],
    },
    SettingsRow {
        label: "Series provider",
        key: "metadatakitSeriesProvider",
        fixed: &["tmdb", "tvdb", "tvmaze", "imdb"],
    },
    SettingsRow {
        label: "Anime provider",
        key: "metadatakitAnimeProvider",
        fixed: &["mal", "kitsu", "anilist", "anidb", "tvdb", "tmdb", "imdb"],
    },
    SettingsRow {
        label: "Movie search",
        key: "metadatakitMovieSearch",
        fixed: &["tmdb", "tvdb"],
    },
    SettingsRow {
        label: "Series search",
        key: "metadatakitSeriesSearch",
        fixed: &["tmdb", "tvdb"],
    },
    SettingsRow {
        label: "Anime search",
        key: "metadatakitSearchAnime",
        fixed: &[],
    },
    SettingsRow {
        label: "AI search",
        key: "metadatakitAiSearch",
        fixed: &[],
    },
    SettingsRow {
        label: "People search",
        key: "metadatakitPeopleSearch",
        fixed: &[],
    },
];
pub(super) const METADATAKIT_CATALOG_SETTINGS: [SettingsRow; 27] = [
    SettingsRow {
        label: "Trending",
        key: "catalogTrending",
        fixed: &[],
    },
    SettingsRow {
        label: "Popular",
        key: "catalogPopular",
        fixed: &[],
    },
    SettingsRow {
        label: "Top rated",
        key: "catalogTopRated",
        fixed: &[],
    },
    SettingsRow {
        label: "Now playing",
        key: "catalogNowPlaying",
        fixed: &[],
    },
    SettingsRow {
        label: "Streaming region",
        key: "metadatakitRegion",
        fixed: &[
            "US", "GB", "CA", "AU", "DE", "FR", "ES", "IT", "NL", "SE", "BR", "MX", "IN", "JP",
            "TR",
        ],
    },
    SettingsRow {
        label: "Netflix",
        key: "catalogNetflix",
        fixed: &[],
    },
    SettingsRow {
        label: "Prime Video",
        key: "catalogPrime",
        fixed: &[],
    },
    SettingsRow {
        label: "Disney+",
        key: "catalogDisney",
        fixed: &[],
    },
    SettingsRow {
        label: "Max",
        key: "catalogMax",
        fixed: &[],
    },
    SettingsRow {
        label: "Apple TV+",
        key: "catalogAppleTv",
        fixed: &[],
    },
    SettingsRow {
        label: "Hulu",
        key: "catalogHulu",
        fixed: &[],
    },
    SettingsRow {
        label: "Paramount+",
        key: "catalogParamount",
        fixed: &[],
    },
    SettingsRow {
        label: "Anime trending",
        key: "catalogAnimeTrending",
        fixed: &[],
    },
    SettingsRow {
        label: "Anime popular",
        key: "catalogAnimePopular",
        fixed: &[],
    },
    SettingsRow {
        label: "Anime airing",
        key: "catalogAnimeAiring",
        fixed: &[],
    },
    SettingsRow {
        label: "Anime top rated",
        key: "catalogAnimeTop",
        fixed: &[],
    },
    SettingsRow {
        label: "AniList watching",
        key: "catalogAnilistWatching",
        fixed: &[],
    },
    SettingsRow {
        label: "AniList planned",
        key: "catalogAnilistPlanning",
        fixed: &[],
    },
    SettingsRow {
        label: "AniList completed",
        key: "catalogAnilistCompleted",
        fixed: &[],
    },
    SettingsRow {
        label: "MAL watching",
        key: "catalogMalWatching",
        fixed: &[],
    },
    SettingsRow {
        label: "MAL planned",
        key: "catalogMalPlanning",
        fixed: &[],
    },
    SettingsRow {
        label: "MAL completed",
        key: "catalogMalCompleted",
        fixed: &[],
    },
    SettingsRow {
        label: "MAL top anime",
        key: "catalogMalTop",
        fixed: &[],
    },
    SettingsRow {
        label: "MAL airing now",
        key: "catalogMalAiring",
        fixed: &[],
    },
    SettingsRow {
        label: "MAL upcoming",
        key: "catalogMalUpcoming",
        fixed: &[],
    },
    SettingsRow {
        label: "TVDB top rated",
        key: "catalogTvdbTop",
        fixed: &[],
    },
    SettingsRow {
        label: "TVDB newest",
        key: "catalogTvdbNew",
        fixed: &[],
    },
];
pub(super) const METADATAKIT_CATALOG_GROUPS: [(usize, &str); 7] = [
    (4, "settings.group.catalogs_general"),
    (8, "settings.group.catalogs_streaming"),
    (4, "settings.group.catalogs_anime"),
    (3, "settings.group.catalogs_anilist"),
    (3, "settings.group.catalogs_mal"),
    (3, "settings.group.catalogs_mal_charts"),
    (2, "settings.group.catalogs_tvdb"),
];
pub(super) const METADATAKIT_GROUPS: [(usize, &str); 4] = [
    (1, "settings.group.kit_status"),
    (9, "settings.group.metadata_options"),
    (3, "settings.group.metadata_providers"),
    (5, "settings.group.metadata_search"),
];
pub(super) const STREAM_BADGES_SETTINGS: [SettingsRow; 9] = [
    SettingsRow {
        label: "Stream badges",
        key: "streamBadgesEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Hero badges",
        key: "heroBadgesEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Built-in badges",
        key: "streamBadgeBuiltIn",
        fixed: &[],
    },
    SettingsRow {
        label: "File size badge",
        key: "streamBadgeFileSize",
        fixed: &[],
    },
    SettingsRow {
        label: "Badge theme",
        key: "streamBadgeTheme",
        fixed: &[
            "vivid", "classic", "soft", "neon", "gold", "sunset", "cinema",
        ],
    },
    SettingsRow {
        label: "Badge placement",
        key: "streamBadgePlacement",
        fixed: &["bottom", "top"],
    },
    SettingsRow {
        label: "Show badges in player",
        key: "playerBadgesEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Card layout",
        key: "streamLayout",
        fixed: &["off", "detailed", "compact", "minimal", "custom"],
    },
    SettingsRow {
        label: "Text formatter",
        key: "streamFormatterEnabled",
        fixed: &[],
    },
];

pub(super) const POSTERS_SETTINGS: [SettingsRow; 53] = [
    SettingsRow {
        label: "Card corners",
        key: "cardCornerPreset",
        fixed: &["sharp", "classic", "soft", "rounded", "pill"],
    },
    SettingsRow {
        label: "Card layout",
        key: "cardLayout",
        fixed: &["vertical", "horizontal"],
    },
    SettingsRow {
        label: "Poster width",
        key: "posterWidthPreset",
        fixed: &["xsmall", "small", "medium", "large", "xlarge"],
    },
    SettingsRow {
        label: "Landscape posters",
        key: "posterLandscapeMode",
        fixed: &[],
    },
    SettingsRow {
        label: "Hide poster titles",
        key: "posterHideTitles",
        fixed: &[],
    },
    SettingsRow {
        label: "Poster preview",
        key: "posterHoverPreview",
        fixed: &[],
    },
    SettingsRow {
        label: "Preview delay",
        key: "posterHoverPreviewDelay",
        fixed: &["0", "0.5", "1", "2", "3", "5"],
    },
    SettingsRow {
        label: "Poster lift",
        key: "posterLift",
        fixed: &[],
    },
    SettingsRow {
        label: "Poster overlays",
        key: "posterOverlaysEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Overlay preset",
        key: "posterPreset",
        fixed: &["custom", "clean", "classic", "cinema"],
    },
    SettingsRow {
        label: "Badge size",
        key: "posterBadgeSize",
        fixed: &["default", "small", "large"],
    },
    SettingsRow {
        label: "Tinted fade",
        key: "posterFadeTint",
        fixed: &[],
    },
    SettingsRow {
        label: "Fade strength",
        key: "posterFadeStrength",
        fixed: &["high", "medium", "low"],
    },
    SettingsRow {
        label: "Textless posters",
        key: "posterTextless",
        fixed: &[],
    },
    SettingsRow {
        label: "Top fade",
        key: "posterTopGradient",
        fixed: &["off", "low", "medium", "high"],
    },
    SettingsRow {
        label: "Mute unreleased art",
        key: "posterMuteUnreleased",
        fixed: &[],
    },
    SettingsRow {
        label: "Rating badge",
        key: "posterRatingBadge",
        fixed: &[],
    },
    SettingsRow {
        label: "Rating source",
        key: "posterRatingSource",
        fixed: &["imdb", "mdblist", "weighted"],
    },
    SettingsRow {
        label: "Score weighting",
        key: "posterWeights",
        fixed: &["balanced", "critics", "audience"],
    },
    SettingsRow {
        label: "Rating logos",
        key: "posterRatingBadges",
        fixed: &["0", "1", "2", "3"],
    },
    SettingsRow {
        label: "Rating position",
        key: "posterRatingPosition",
        fixed: &[
            "bar",
            "band",
            "clean",
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
        label: "Score order",
        key: "posterMetaOrder",
        fixed: &["year_genre_score", "genre_year_score"],
    },
    SettingsRow {
        label: "Score scale",
        key: "posterScoreScale",
        fixed: &["ten", "hundred"],
    },
    SettingsRow {
        label: "Release status badge",
        key: "posterStatusBadge",
        fixed: &[],
    },
    SettingsRow {
        label: "Status position",
        key: "posterStatusPosition",
        fixed: &[
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
        fixed: &[],
    },
    SettingsRow {
        label: "Progress bar",
        key: "posterProgressBar",
        fixed: &[],
    },
    SettingsRow {
        label: "Library badge",
        key: "posterSavedBadge",
        fixed: &[],
    },
    SettingsRow {
        label: "Trending sash",
        key: "posterTrendingBadge",
        fixed: &[],
    },
    SettingsRow {
        label: "Band style",
        key: "posterBandStyle",
        fixed: &["light", "dark"],
    },
    SettingsRow {
        label: "Release date disc",
        key: "posterReleaseDisc",
        fixed: &[],
    },
    SettingsRow {
        label: "Age rating country",
        key: "posterAgeCountry",
        fixed: &[
            "auto", "US", "GB", "DE", "FR", "ES", "IT", "NL", "TR", "BR", "JP", "KR", "AU", "CA",
            "IN",
        ],
    },
    SettingsRow {
        label: "Trending source",
        key: "posterTrendingSource",
        fixed: &["week", "day", "popular"],
    },
    SettingsRow {
        label: "Age rating",
        key: "posterAgeRating",
        fixed: &[],
    },
    SettingsRow {
        label: "Quality badges",
        key: "posterQualityBadges",
        fixed: &[],
    },
    SettingsRow {
        label: "Award sash",
        key: "posterAwardBadge",
        fixed: &[],
    },
    SettingsRow {
        label: "Sash style",
        key: "posterSashStyle",
        fixed: &["diagonal", "notch", "chip", "edge"],
    },
    SettingsRow {
        label: "Notch style",
        key: "posterNotchStyle",
        fixed: &["frosted", "black", "silver", "gold", "tinted"],
    },
    SettingsRow {
        label: "Sash priority",
        key: "posterSashPriority",
        fixed: &["status_first", "awards_first", "trending_first"],
    },
    SettingsRow {
        label: "Ended and cancelled",
        key: "posterLifecycleSash",
        fixed: &[],
    },
    SettingsRow {
        label: "Foreign language",
        key: "posterForeignSash",
        fixed: &[],
    },
    SettingsRow {
        label: "Must see",
        key: "posterMustSeeSash",
        fixed: &[],
    },
    SettingsRow {
        label: "Top numbers",
        key: "posterTopNumbers",
        fixed: &[],
    },
    SettingsRow {
        label: "Numbered rows",
        key: "posterTopNumbersRows",
        fixed: &["trending", "all"],
    },
    SettingsRow {
        label: "Numbered titles",
        key: "posterTopNumbersCount",
        fixed: &["3", "5", "10", "20"],
    },
    SettingsRow {
        label: "Number style",
        key: "posterTopNumbersStyle",
        fixed: &["outline", "solid", "ghost"],
    },
    SettingsRow {
        label: "Number color",
        key: "posterTopNumbersColor",
        fixed: &["white", "gray", "accent"],
    },
    SettingsRow {
        label: "Number size",
        key: "posterTopNumbersSize",
        fixed: &["default", "small", "large"],
    },
    SettingsRow {
        label: "Number position",
        key: "posterTopNumbersPosition",
        fixed: &["beside", "overlay"],
    },
    SettingsRow {
        label: "Expanded posters",
        key: "expandedPosters",
        fixed: &[],
    },
    SettingsRow {
        label: "Expand delay",
        key: "expandedPostersDelay",
        fixed: &["0", "1", "2", "3"],
    },
    SettingsRow {
        label: "Trailer on expanded posters",
        key: "trailerOnExpandedPosters",
        fixed: &[],
    },
    SettingsRow {
        label: "Expanded poster trailer delay",
        key: "trailerOnExpandedPostersDelay",
        fixed: &["0", "2", "4", "6"],
    },
];
pub(super) const POSTERS_GROUPS: [(usize, &str); 7] = [
    (8, "settings.group.cards"),
    (8, "settings.group.poster_overlays"),
    (7, "settings.group.poster_rating"),
    (13, "settings.group.poster_badges"),
    (6, "settings.group.poster_sashes"),
    (7, "settings.group.poster_top_numbers"),
    (4, "settings.group.expanded_posters"),
];
pub(super) const PREVIEWS_SETTINGS: [SettingsRow; 4] = [
    SettingsRow {
        label: "Keyframes only",
        key: "trickplayKeyframesOnly",
        fixed: &[],
    },
    SettingsRow {
        label: "Generation mode",
        key: "trickplayMode",
        fixed: &["async", "pregenerate"],
    },
    SettingsRow {
        label: "Frame interval",
        key: "trickplayInterval",
        fixed: &["5", "10", "20", "30", "60"],
    },
    SettingsRow {
        label: "Preview sheet",
        key: "trickplaySheet",
        fixed: &[],
    },
];
pub(super) const BADGES_GROUPS: [(usize, &str); 1] = [(9, "settings.group.badges_general")];
pub(super) const PREVIEWS_GROUPS: [(usize, &str); 1] = [(4, "settings.group.seek_previews")];
pub(super) const PLAYBACK_SETTINGS: [SettingsRow; 35] = [
    SettingsRow {
        label: "Playback destination",
        key: "preferredPlayer",
        fixed: &["mpv", "external"],
    },
    SettingsRow {
        label: "Stream selection",
        key: "streamSourceSelectionMode",
        fixed: &["manual", "first", "regex"],
    },
    SettingsRow {
        label: "Autoplay next episode",
        key: "autoPlayNextEpisode",
        fixed: &[],
    },
    SettingsRow {
        label: "Mark watched at",
        key: "scrobbleWatchedPercent",
        fixed: &["50", "60", "70", "80", "90", "95", "100"],
    },
    SettingsRow {
        label: "Countdown duration",
        key: "autoPlayCountdownSecs",
        fixed: &["5", "7", "10", "15"],
    },
    SettingsRow {
        label: "Prefer binge group",
        key: "tryBingeGroup",
        fixed: &[],
    },
    SettingsRow {
        label: "Retry next source",
        key: "autoRetryNextSource",
        fixed: &[],
    },
    SettingsRow {
        label: "Background playback",
        key: "backgroundPlayback",
        fixed: &[],
    },
    SettingsRow {
        label: "Picture in picture",
        key: "pictureInPicture",
        fixed: &[],
    },
    SettingsRow {
        label: "Reuse last stream",
        key: "reuseLastStream",
        fixed: &[],
    },
    SettingsRow {
        label: "Reuse last stream TTL",
        key: "reuseLastStreamTtlHours",
        fixed: &[
            "1", "2", "3", "4", "5", "6", "24", "48", "72", "96", "120", "144",
        ],
    },
    SettingsRow {
        label: "Preload next episode sources",
        key: "preloadNextEpisodeSources",
        fixed: &[],
    },
    SettingsRow {
        label: "Prefetch streams on detail screen",
        key: "prefetchDetailStreams",
        fixed: &[],
    },
    SettingsRow {
        label: "Reuse binge group",
        key: "reuseBingeGroup",
        fixed: &[],
    },
    SettingsRow {
        label: "P2P playback",
        key: "p2pEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Dolby Vision",
        key: "dolbyVisionMode",
        fixed: &["auto", "native", "convert", "base_layer", "compose"],
    },
    SettingsRow {
        label: "Dolby Vision tone mapping",
        key: "dolbyVisionTonemap",
        fixed: &["auto", "no", "yes", "sdr"],
    },
    SettingsRow {
        label: "Display peak brightness",
        key: "displayPeakNits",
        fixed: &["0", "300", "400", "600", "1000", "1500", "4000"],
    },
    SettingsRow {
        label: "Auto skip intro",
        key: "autoSkipIntro",
        fixed: &[],
    },
    SettingsRow {
        label: "Anime skip integration",
        key: "useAnimeSkip",
        fixed: &[],
    },
    SettingsRow {
        label: "Chapter skip",
        key: "useChapterSkip",
        fixed: &[],
    },
    SettingsRow {
        label: "Skip segments",
        key: "useSkipSegments",
        fixed: &[],
    },
    SettingsRow {
        label: "Seek thumbnails",
        key: "seekThumbnailEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Hold to speed",
        key: "holdToSpeedEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Hold speed",
        key: "holdSpeed",
        fixed: &["1.25", "1.5", "1.75", "2.0", "2.5", "3.0"],
    },
    SettingsRow {
        label: "Playback speed",
        key: "playbackSpeed",
        fixed: &["0.75", "1.0", "1.25", "1.5", "2.0"],
    },
    SettingsRow {
        label: "Seek interval",
        key: "seekSeconds",
        fixed: &["5", "10", "15", "30"],
    },
    SettingsRow {
        label: "Anime upscaling",
        key: "animeUpscalingMode",
        fixed: &["off", "auto"],
    },
    SettingsRow {
        label: "Anime4K mode",
        key: "animeUpscalingModePreset",
        fixed: &["a", "b", "c"],
    },
    SettingsRow {
        label: "Anime upscaling quality",
        key: "animeUpscalingQuality",
        fixed: &["anime4k_s", "anime4k_m", "anime4k_l"],
    },
    SettingsRow {
        label: "Frame interpolation",
        key: "frameInterpolationMode",
        fixed: &["off", "display_resample", "smooth"],
    },
    SettingsRow {
        label: "Audio processing",
        key: "audioProcessingMode",
        fixed: &["reference", "balanced", "night"],
    },
    SettingsRow {
        label: "Buffer cache",
        key: "playerBufferCacheMb",
        fixed: &["100", "500", "1000", "2000", "-1"],
    },
    SettingsRow {
        label: "Forward buffer",
        key: "playerForwardBufferSeconds",
        fixed: &["30", "60", "120", "300", "600"],
    },
    SettingsRow {
        label: "Back buffer",
        key: "playerBackBufferSeconds",
        fixed: &["0", "15", "30", "60", "120", "300"],
    },
];
pub(super) const PLAYBACK_GROUPS: [(usize, &str); 6] = [
    (8, "settings.group.playback"),
    (5, "settings.group.stream_reuse"),
    (4, "settings.group.sources_video"),
    (9, "settings.group.skip_controls"),
    (4, "settings.group.upscaling_motion"),
    (4, "settings.group.player_advanced"),
];
pub(super) const LANGUAGE_OPTIONS: &[&str] = &[
    "none", "af", "am", "hy", "az", "id", "ms", "be", "bn", "bs", "bg", "my", "ca", "zh", "cy",
    "da", "de", "et", "en", "es", "eu", "tl", "fr", "ga", "gl", "ka", "el", "gu", "hi", "hr", "zu",
    "it", "ja", "kn", "kk", "km", "sw", "ko", "lo", "lv", "lt", "mk", "hu", "ml", "mt", "mr", "mn",
    "nl", "ne", "no", "uz", "pl", "pt", "pa", "ro", "ru", "sr", "sq", "si", "sk", "sl", "fi", "sv",
    "ta", "te", "th", "tr", "uk", "vi", "is", "cs", "he", "ur", "ar", "fa",
];
pub(super) const AUDIO_LANGUAGE_OPTIONS: &[&str] = &[
    "default", "device", "original", "af", "am", "hy", "az", "id", "ms", "be", "bn", "bs", "bg",
    "my", "ca", "zh", "cy", "da", "de", "et", "en", "es", "eu", "tl", "fr", "ga", "gl", "ka", "el",
    "gu", "hi", "hr", "zu", "it", "ja", "kn", "kk", "km", "sw", "ko", "lo", "lv", "lt", "mk", "hu",
    "ml", "mt", "mr", "mn", "nl", "ne", "no", "uz", "pl", "pt", "pa", "ro", "ru", "sr", "sq", "si",
    "sk", "sl", "fi", "sv", "ta", "te", "th", "tr", "uk", "vi", "is", "cs", "he", "ur", "ar", "fa",
];
pub(super) const SUBTITLE_LANGUAGE_OPTIONS: &[&str] = &[
    "none", "device", "forced", "af", "am", "hy", "az", "id", "ms", "be", "bn", "bs", "bg", "my",
    "ca", "zh", "cy", "da", "de", "et", "en", "es", "eu", "tl", "fr", "ga", "gl", "ka", "el", "gu",
    "hi", "hr", "zu", "it", "ja", "kn", "kk", "km", "sw", "ko", "lo", "lv", "lt", "mk", "hu", "ml",
    "mt", "mr", "mn", "nl", "ne", "no", "uz", "pl", "pt", "pa", "ro", "ru", "sr", "sq", "si", "sk",
    "sl", "fi", "sv", "ta", "te", "th", "tr", "uk", "vi", "is", "cs", "he", "ur", "ar", "fa",
];
pub(super) const SUBTITLES_SETTINGS: [SettingsRow; 18] = [
    SettingsRow {
        label: "Preferred audio",
        key: "preferredAudioLanguage",
        fixed: AUDIO_LANGUAGE_OPTIONS,
    },
    SettingsRow {
        label: "Secondary audio",
        key: "secondaryAudioLanguage",
        fixed: LANGUAGE_OPTIONS,
    },
    SettingsRow {
        label: "Anime Japanese audio",
        key: "animePreferJapaneseAudio",
        fixed: &[],
    },
    SettingsRow {
        label: "Preferred subtitle language",
        key: "preferredSubtitleLanguage",
        fixed: SUBTITLE_LANGUAGE_OPTIONS,
    },
    SettingsRow {
        label: "Secondary subtitle",
        key: "secondarySubtitleLanguage",
        fixed: LANGUAGE_OPTIONS,
    },
    SettingsRow {
        label: "Auto-enable subtitles",
        key: "autoEnableSubtitles",
        fixed: &[],
    },
    SettingsRow {
        label: "Subtitle position",
        key: "subtitlePosition",
        fixed: &["100", "90", "80", "70"],
    },
    SettingsRow {
        label: "Subtitle size",
        key: "subtitleSize",
        fixed: &["50", "75", "100", "125", "150", "200"],
    },
    SettingsRow {
        label: "Subtitle text",
        key: "subtitleColor",
        fixed: &[
            "#FFFFFF", "#000000", "#FFE45C", "#FF5D5D", "#3F7CFF", "#54D17A", "#FF8A3D", "#C084FC",
        ],
    },
    SettingsRow {
        label: "Text transparency",
        key: "subtitleTextOpacity",
        fixed: &["1.0", "0.75", "0.5", "0.25", "0.0"],
    },
    SettingsRow {
        label: "Subtitle background",
        key: "subtitleBackgroundColor",
        fixed: &["#000000", "#FFFFFF", "#FFE45C", "#FF5D5D", "#3F7CFF"],
    },
    SettingsRow {
        label: "Background transparency",
        key: "subtitleBackgroundOpacity",
        fixed: &["1.0", "0.75", "0.5", "0.25", "0.0"],
    },
    SettingsRow {
        label: "Subtitle outline",
        key: "subtitleOutlineColor",
        fixed: &["#000000", "#FFFFFF", "#FFE45C", "#FF5D5D", "#3F7CFF"],
    },
    SettingsRow {
        label: "Outline transparency",
        key: "subtitleOutlineOpacity",
        fixed: &["1.0", "0.75", "0.5", "0.25", "0.0"],
    },
    SettingsRow {
        label: "Outline size",
        key: "subtitleOutlineSize",
        fixed: &["0", "1", "2", "3", "4", "5"],
    },
    SettingsRow {
        label: "Bold subtitles",
        key: "subtitleBold",
        fixed: &[],
    },
    SettingsRow {
        label: "Subtitle shadow",
        key: "subtitleShadow",
        fixed: &[],
    },
    SettingsRow {
        label: "Force subtitle style",
        key: "subtitleForceStyle",
        fixed: &[],
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
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB logos and backdrops",
        key: "tmdbLogosBackdropsEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB cast images",
        key: "tmdbCastImagesEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB episode stills",
        key: "tmdbEpisodeImagesEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB descriptions",
        key: "tmdbEnrichDescriptionEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB genres and keywords",
        key: "tmdbEnrichGenresKeywordsEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB cast and crew",
        key: "tmdbEnrichCastCrewEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB collections",
        key: "tmdbCollectionInfoEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB networks",
        key: "tmdbEnrichNetworkEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB status and schedule",
        key: "tmdbEnrichStatusScheduleEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB original titles",
        key: "tmdbEnrichOriginTitlesEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB watch providers",
        key: "tmdbEnrichWatchProvidersEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Show TMDB ratings",
        key: "tmdbRatingsEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB trailers",
        key: "tmdbTrailersEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB recommendations",
        key: "tmdbRecommendationsEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "TMDB similar titles",
        key: "tmdbSimilarResultsEnabled",
        fixed: &[],
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
        fixed: &["default", "fast", "ultra_fast"],
    },
    SettingsRow {
        label: "Torrent cache",
        key: "torrentCachePreset",
        fixed: &["auto", "2gb", "5gb", "10gb", "unlimited"],
    },
    SettingsRow {
        label: "Download source",
        key: "downloadSourceSelectionMode",
        fixed: &["manual", "first", "regex"],
    },
    SettingsRow {
        label: "Download subtitle",
        key: "downloadSubtitleLanguage",
        fixed: &["off", "preferred"],
    },
    SettingsRow {
        label: "Torrent on Wi-Fi only",
        key: "torrentWifiOnly",
        fixed: &[],
    },
];
pub(super) const DOWNLOADS_GROUPS: [(usize, &str); 1] = [(5, "settings.group.downloads")];
pub(super) const PLAYER_SETTINGS: [SettingsRow; 18] = [
    SettingsRow {
        label: "Pause screen info",
        key: "pauseMetadataOverlayEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Content warnings",
        key: "contentWarningsEnabled",
        fixed: &[],
    },
    SettingsRow {
        label: "Content info",
        key: "playerShowTitle",
        fixed: &[],
    },
    SettingsRow {
        label: "Up next card",
        key: "playerShowUpNextCard",
        fixed: &[],
    },
    SettingsRow {
        label: "Audio and subtitles",
        key: "playerShowAudioSubtitles",
        fixed: &[],
    },
    SettingsRow {
        label: "Speed",
        key: "playerShowSpeed",
        fixed: &[],
    },
    SettingsRow {
        label: "Episode picker",
        key: "playerShowEpisodes",
        fixed: &[],
    },
    SettingsRow {
        label: "Next episode button",
        key: "playerShowNextEpisode",
        fixed: &[],
    },
    SettingsRow {
        label: "Player settings",
        key: "playerShowSettings",
        fixed: &[],
    },
    SettingsRow {
        label: "Volume",
        key: "playerShowVolume",
        fixed: &[],
    },
    SettingsRow {
        label: "Fullscreen",
        key: "playerShowFullscreen",
        fixed: &[],
    },
    SettingsRow {
        label: "Cast",
        key: "playerShowCast",
        fixed: &[],
    },
    SettingsRow {
        label: "Mark segment",
        key: "playerShowMarkSegment",
        fixed: &[],
    },
    SettingsRow {
        label: "Segments on seek bar",
        key: "playerShowSegments",
        fixed: &[],
    },
    SettingsRow {
        label: "Chapters on seek bar",
        key: "playerShowChapters",
        fixed: &[],
    },
    SettingsRow {
        label: "Brightness and volume gestures",
        key: "playerGestures",
        fixed: &[],
    },
    SettingsRow {
        label: "Double-tap to seek",
        key: "playerDoubleTapSeek",
        fixed: &[],
    },
    SettingsRow {
        label: "Controls in the middle",
        key: "playerCenterControls",
        fixed: &[],
    },
];
pub(super) const PLAYER_GROUPS: [(usize, &str); 3] = [
    (4, "settings.group.player_overlay"),
    (11, "settings.group.player_controls"),
    (3, "settings.group.player_touch"),
];
pub(super) const EMPTY_SETTINGS: [SettingsRow; 0] = [];
pub const SETTINGS_SECTIONS: [SettingsSection; 29] = [
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
        groups: &BADGES_GROUPS,
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
        title: "Streamkit",
        category: "Streamkit",
        description: "Aggregate, filter, sort and format the streams of your add-ons",
        rows: &STREAMKIT_SETTINGS,
        groups: &STREAMKIT_GROUPS,
    },
    SettingsSection {
        title: "Streamkit Addons",
        category: "Streamkit",
        description: "Stream add-ons that Streamkit filters and sorts",
        rows: &EMPTY_SETTINGS,
        groups: &[],
    },
    SettingsSection {
        title: "Metadatakit",
        category: "Metadatakit",
        description: "Merged metadata from TMDB with anime id mapping",
        rows: &METADATAKIT_SETTINGS,
        groups: &METADATAKIT_GROUPS,
    },
    SettingsSection {
        title: "Metadatakit Catalogs",
        category: "Metadatakit",
        description: "Home catalogs served by Metadatakit",
        rows: &METADATAKIT_CATALOG_SETTINGS,
        groups: &METADATAKIT_CATALOG_GROUPS,
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
