#[derive(Clone, Debug)]
pub struct SettingsRow {
    pub label: &'static str,
    pub key: &'static str,
    pub options: &'static [&'static str],
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
pub(super) const ABOUT_GROUPS: [(usize, &str); 1] = [(2, "settings.group.updates_privacy")];
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
pub(super) const STREAMKIT_SETTINGS: [SettingsRow; 7] = [
    SettingsRow {
        label: "Enable Streamkit",
        key: "streamkitEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Sort order",
        key: "streamSortOrder",
        options: &["addon", "quality", "size", "seeders"],
    },
    SettingsRow {
        label: "Minimum resolution",
        key: "streamMinResolution",
        options: &["any", "480p", "720p", "1080p", "2160p"],
    },
    SettingsRow {
        label: "Hide low quality releases",
        key: "streamHideLowQuality",
        options: &[],
    },
    SettingsRow {
        label: "Remove duplicates",
        key: "streamDedupe",
        options: &[],
    },
    SettingsRow {
        label: "Source limit",
        key: "streamLimit",
        options: &["off", "5", "10", "20", "50"],
    },
    SettingsRow {
        label: "Formatted source text",
        key: "streamPipelineFormat",
        options: &[],
    },
];
pub(super) const STREAMKIT_GROUPS: [(usize, &str); 3] = [
    (1, "settings.group.kit_status"),
    (5, "settings.group.source_rules"),
    (1, "settings.group.source_text"),
];
pub(super) const METADATAKIT_SETTINGS: [SettingsRow; 3] = [
    SettingsRow {
        label: "Enable Metadatakit",
        key: "metadatakitEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Metadata language",
        key: "metadatakitLanguage",
        options: &["en-US", "tr-TR", "de-DE", "fr-FR", "es-ES", "ja-JP"],
    },
    SettingsRow {
        label: "Anime id mapping",
        key: "metadatakitAnimeMapping",
        options: &[],
    },
];
pub(super) const METADATAKIT_GROUPS: [(usize, &str); 2] = [
    (1, "settings.group.kit_status"),
    (2, "settings.group.metadata_options"),
];
pub(super) const STREAM_BADGES_SETTINGS: [SettingsRow; 8] = [
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
        options: &[
            "vivid", "classic", "soft", "neon", "gold", "sunset", "cinema",
        ],
    },
    SettingsRow {
        label: "Badge placement",
        key: "streamBadgePlacement",
        options: &["bottom", "top"],
    },
    SettingsRow {
        label: "Show badges in player",
        key: "playerBadgesEnabled",
        options: &[],
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

pub(super) const POSTERS_SETTINGS: [SettingsRow; 53] = [
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
        label: "Preview delay",
        key: "posterHoverPreviewDelay",
        options: &["0", "0.5", "1", "2", "3", "5"],
    },
    SettingsRow {
        label: "Poster lift",
        key: "posterLift",
        options: &[],
    },
    SettingsRow {
        label: "Poster overlays",
        key: "posterOverlaysEnabled",
        options: &[],
    },
    SettingsRow {
        label: "Overlay preset",
        key: "posterPreset",
        options: &["custom", "clean", "classic", "cinema"],
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
        label: "Textless posters",
        key: "posterTextless",
        options: &[],
    },
    SettingsRow {
        label: "Top fade",
        key: "posterTopGradient",
        options: &["off", "low", "medium", "high"],
    },
    SettingsRow {
        label: "Mute unreleased art",
        key: "posterMuteUnreleased",
        options: &[],
    },
    SettingsRow {
        label: "Rating badge",
        key: "posterRatingBadge",
        options: &[],
    },
    SettingsRow {
        label: "Rating source",
        key: "posterRatingSource",
        options: &["imdb", "mdblist", "weighted"],
    },
    SettingsRow {
        label: "Score weighting",
        key: "posterWeights",
        options: &["balanced", "critics", "audience"],
    },
    SettingsRow {
        label: "Rating logos",
        key: "posterRatingBadges",
        options: &["0", "1", "2", "3"],
    },
    SettingsRow {
        label: "Rating position",
        key: "posterRatingPosition",
        options: &[
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
        options: &["year_genre_score", "genre_year_score"],
    },
    SettingsRow {
        label: "Score scale",
        key: "posterScoreScale",
        options: &["ten", "hundred"],
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
        label: "Band style",
        key: "posterBandStyle",
        options: &["light", "dark"],
    },
    SettingsRow {
        label: "Release date disc",
        key: "posterReleaseDisc",
        options: &[],
    },
    SettingsRow {
        label: "Age rating country",
        key: "posterAgeCountry",
        options: &[
            "auto", "US", "GB", "DE", "FR", "ES", "IT", "NL", "TR", "BR", "JP", "KR", "AU", "CA",
            "IN",
        ],
    },
    SettingsRow {
        label: "Trending source",
        key: "posterTrendingSource",
        options: &["week", "day", "popular"],
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
        label: "Sash style",
        key: "posterSashStyle",
        options: &["diagonal", "notch", "chip", "edge"],
    },
    SettingsRow {
        label: "Notch style",
        key: "posterNotchStyle",
        options: &["frosted", "black", "silver", "gold", "tinted"],
    },
    SettingsRow {
        label: "Sash priority",
        key: "posterSashPriority",
        options: &["status_first", "awards_first", "trending_first"],
    },
    SettingsRow {
        label: "Ended and cancelled",
        key: "posterLifecycleSash",
        options: &[],
    },
    SettingsRow {
        label: "Foreign language",
        key: "posterForeignSash",
        options: &[],
    },
    SettingsRow {
        label: "Must see",
        key: "posterMustSeeSash",
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
        options: &[],
    },
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
pub(super) const PREVIEWS_GROUPS: [(usize, &str); 1] = [(4, "settings.group.seek_previews")];
pub(super) const PLAYBACK_SETTINGS: [SettingsRow; 35] = [
    SettingsRow {
        label: "Playback destination",
        key: "preferredPlayer",
        options: &["mpv", "external"],
    },
    SettingsRow {
        label: "Stream selection",
        key: "streamSourceSelectionMode",
        options: &["manual", "first", "regex"],
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
        label: "Prefetch streams on detail screen",
        key: "prefetchDetailStreams",
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
        options: AUDIO_LANGUAGE_OPTIONS,
    },
    SettingsRow {
        label: "Secondary audio",
        key: "secondaryAudioLanguage",
        options: LANGUAGE_OPTIONS,
    },
    SettingsRow {
        label: "Anime Japanese audio",
        key: "animePreferJapaneseAudio",
        options: &[],
    },
    SettingsRow {
        label: "Preferred subtitle language",
        key: "preferredSubtitleLanguage",
        options: SUBTITLE_LANGUAGE_OPTIONS,
    },
    SettingsRow {
        label: "Secondary subtitle",
        key: "secondarySubtitleLanguage",
        options: LANGUAGE_OPTIONS,
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
        options: &["manual", "first", "regex"],
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
pub const SETTINGS_SECTIONS: [SettingsSection; 27] = [
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
        title: "Streamkit",
        category: "Streamkit",
        description: "Aggregate, filter, sort and format the streams of your add-ons",
        rows: &STREAMKIT_SETTINGS,
        groups: &STREAMKIT_GROUPS,
    },
    SettingsSection {
        title: "Metadatakit",
        category: "Metadatakit",
        description: "Merged metadata from TMDB with anime id mapping",
        rows: &METADATAKIT_SETTINGS,
        groups: &METADATAKIT_GROUPS,
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
