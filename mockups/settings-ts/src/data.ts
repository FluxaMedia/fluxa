export type Kind = "toggle" | "choice" | "color" | "range";

export interface Def {
  key: string;
  label: string;
  kind: Kind;
  def: string | number | boolean;
  desc?: string;
  options?: string[];
  unit?: string;
  min?: number;
  max?: number;
  step?: number;
  needs?: string;
}

export interface Group {
  title: string;
  items: Def[];
}

export interface Section {
  id: string;
  title: string;
  desc: string;
  groups: Group[];
  preview?: "theme" | "poster" | "subs";
}

export interface Category {
  id: string;
  title: string;
  desc: string;
  icon: string;
  sections: Section[];
}

const t = (key: string, label: string, def = false, desc?: string, needs?: string): Def => ({
  key, label, kind: "toggle", def, desc, needs,
});
const c = (key: string, label: string, options: string[], def: string, desc?: string, unit?: string, needs?: string): Def => ({
  key, label, kind: "choice", options, def, desc, unit, needs,
});
const col = (key: string, label: string, options: string[], def: string): Def => ({
  key, label, kind: "color", options, def,
});
const r = (key: string, label: string, min: number, max: number, step: number, def: number, unit = ""): Def => ({
  key, label, kind: "range", min, max, step, def, unit,
});

const langs = [
  "en", "tr", "ar", "he", "af", "sq", "am", "hy", "az", "eu", "be", "bn", "bs", "bg", "my", "ca", "zh", "hr", "cs",
  "da", "nl", "et", "tl", "fi", "fr", "gl", "ka", "de", "el", "gu", "hi", "hu", "is", "id", "ga", "it", "ja", "kn",
  "kk", "km", "ko", "lo", "lv", "lt", "mk", "ms", "ml", "mt", "mr", "mn", "ne", "no", "pa", "fa", "pl", "pt", "ro",
  "ru", "sr", "si", "sk", "sl", "es", "sw", "sv", "ta", "te", "th", "uk", "ur", "uz", "vi", "cy", "zu",
];
const audioLangs = ["default", "device", "original", ...langs];
const subLangs = ["none", "device", "forced", ...langs];
const swatches = ["#FFFFFF", "#000000", "#FFE45C", "#FF5D5D", "#3F7CFF", "#54D17A", "#FF8A3D", "#C084FC"];
const opacity = ["1.0", "0.75", "0.5", "0.25", "0.0"];
const corners = ["sharp", "classic", "soft", "rounded", "pill"];
const corner4 = ["top_left", "top_right", "bottom_left", "bottom_right"];

export const categories: Category[] = [
  {
    id: "account",
    title: "Account",
    desc: "Profile, services and metadata",
    icon: "user",
    sections: [
      {
        id: "account",
        title: "General",
        desc: "Language, notifications and privacy",
        groups: [
          {
            title: "Preferences",
            items: [
              c("language", "Language", langs.slice(0, 4), "en"),
              t("notificationsEnabled", "Notifications", true),
              t("discordRichPresenceEnabled", "Discord Rich Presence", false),
              t("saveRecentSearches", "Save recent searches", true),
            ],
          },
          {
            title: "Updates and privacy",
            items: [
              t("automaticUpdates", "Update checks", true),
              t("crashReports", "Crash reports", true),
            ],
          },
        ],
      },
      {
        id: "tmdb",
        title: "TMDB",
        desc: "Metadata enrichment",
        groups: [
          {
            title: "Images",
            items: [
              t("tmdbEnrichArtworkEnabled", "Artwork enrichment", true),
              t("tmdbLogosBackdropsEnabled", "Logos and backdrops", true, undefined, "tmdbEnrichArtworkEnabled"),
              t("tmdbCastImagesEnabled", "Cast images", true),
              t("tmdbEpisodeImagesEnabled", "Episode stills", true),
            ],
          },
          {
            title: "Media info",
            items: [
              t("tmdbEnrichDescriptionEnabled", "Descriptions", true),
              t("tmdbEnrichGenresKeywordsEnabled", "Genres and keywords", true),
              t("tmdbEnrichCastCrewEnabled", "Cast and crew", true),
              t("tmdbCollectionInfoEnabled", "Collections", true),
              t("tmdbEnrichNetworkEnabled", "Networks", true),
              t("tmdbEnrichStatusScheduleEnabled", "Status and schedule", true),
              t("tmdbEnrichOriginTitlesEnabled", "Original titles", false),
              t("tmdbEnrichWatchProvidersEnabled", "Watch providers", false),
            ],
          },
          {
            title: "Discovery",
            items: [
              t("tmdbRatingsEnabled", "Ratings", true),
              t("tmdbTrailersEnabled", "Trailers", true),
              t("tmdbRecommendationsEnabled", "Recommendations", true),
              t("tmdbSimilarResultsEnabled", "Similar titles", true),
            ],
          },
        ],
      },
    ],
  },
  {
    id: "appearance",
    title: "Appearance",
    desc: "Theme, home, details and posters",
    icon: "palette",
    sections: [
      {
        id: "theme",
        title: "Theme",
        desc: "Color, motion and navigation",
        preview: "theme",
        groups: [
          {
            title: "Color and motion",
            items: [
              col("accentColorArgb", "Accent color", ["#FFFFFF", "#E50914", "#3F7CFF", "#54D17A", "#FF8A3D", "#C084FC"], "#FFFFFF"),
              r("uiScale", "UI scale", 75, 150, 5, 100, "%"),
              t("amoledMode", "AMOLED black", false),
              t("gifAutoplayEnabled", "GIF autoplay", true),
              t("showCatalogType", "Show catalog type", false),
              c("interfaceDensity", "Interface density", ["small", "medium", "large"], "medium"),
              c("appIcon", "App icon", ["ember", "sunset", "ocean", "dusk", "lagoon", "ruby", "citrus", "peach", "lime", "aurora", "neon", "flare", "ice", "rose", "silver", "gold", "jade", "blaze", "orange"], "ember"),
            ],
          },
          {
            title: "Navigation",
            items: [
              c("navLayout", "Layout", ["sidebar", "topbar"], "sidebar"),
              c("navMode", "Mode", ["compact", "classic", "adaptive"], "adaptive"),
              c("navSidebarMode", "Sidebar behavior", ["hover", "expanded", "collapsed"], "hover"),
              t("navFloating", "Floating bar", true),
              t("navLabels", "Labels", true),
            ],
          },
        ],
      },
      {
        id: "home",
        title: "Home",
        desc: "Hero and rows",
        groups: [
          {
            title: "Hero",
            items: [
              t("showHeroSection", "Show hero section", true),
              t("followFocusHero", "Hero follows focus", true, undefined, "showHeroSection"),
              c("homeHeroArtwork", "Hero artwork", ["backdrop", "poster"], "backdrop", undefined, undefined, "showHeroSection"),
              t("homeSeasonPostersOnHero", "Season posters", false, undefined, "showHeroSection"),
              t("homeHeroRatings", "Ratings", true, undefined, "showHeroSection"),
              t("homeHeroAutoplayTrailer", "Autoplay trailer", false, undefined, "showHeroSection"),
              c("homeHeroAutoplayTrailerDelaySecs", "Trailer delay", ["2", "4", "6", "10"], "4", undefined, "s", "homeHeroAutoplayTrailer"),
              t("shortenSynopsis", "Shorten synopsis", true),
              t("catalogTypeSuffixEnabled", "Catalog type suffix", false),
            ],
          },
          {
            title: "Continue Watching",
            items: [
              t("continueWatchingEnabled", "Show shelf", true),
              c("continueWatchingLayout", "Layout", ["vertical", "horizontal", "inherit"], "inherit", undefined, undefined, "continueWatchingEnabled"),
              c("continueWatchingArtwork", "Artwork", ["episode", "poster", "background"], "episode", undefined, undefined, "continueWatchingEnabled"),
              c("continueWatchingRemainingFormat", "Remaining time", ["time", "percent"], "time", undefined, undefined, "continueWatchingEnabled"),
              c("continueWatchingProgressDirection", "Progress direction", ["remaining", "watched"], "watched", undefined, undefined, "continueWatchingEnabled"),
              t("continueWatchingHideTitles", "Hide titles", false, undefined, "continueWatchingEnabled"),
              t("continueWatchingKeepScheduled", "Keep scheduled titles", true, undefined, "continueWatchingEnabled"),
              t("continueWatchingShowThisWeek", "Show This Week", true, undefined, "continueWatchingEnabled"),
              c("continueProgressLabel", "Badge", ["remaining", "percent"], "remaining", undefined, undefined, "continueWatchingEnabled"),
              t("showUpcomingRow", "Upcoming row", true),
            ],
          },
        ],
      },
      {
        id: "details",
        title: "Details",
        desc: "Layout of the detail page",
        groups: [
          {
            title: "Detail page",
            items: [
              t("detailCollapsingHero", "Hero collapses on scroll", true),
              t("detailPreferClearlogo", "Prefer clear logo", true),
              t("detailSeasonPostersOnHero", "Season posters on hero", false),
              t("trailerOnHero", "Trailer on hero", true),
              t("detailHeroAutoplayTrailer", "Autoplay trailer", false, undefined, "trailerOnHero"),
              c("detailHeroAutoplayTrailerDelaySecs", "Trailer delay", ["2", "4", "6", "10"], "4", undefined, "s", "detailHeroAutoplayTrailer"),
              t("detailShowEpisodeTitles", "Episode titles", true),
              t("detailShowEpisodeDescriptions", "Episode descriptions", true),
              t("traktCommentsEnabled", "Trakt comments", false),
            ],
          },
          {
            title: "Episodes",
            items: [
              c("detailSeasonSelectorMode", "Season selector", ["posters", "chips", "dropdown"], "chips"),
              c("episodeCardsLayout", "Episode layout", ["auto", "cards", "list", "grid"], "auto"),
              t("blurUnwatchedEpisodes", "Blur unwatched episodes", false),
              t("spoilerHideEpisodeInfo", "Hide spoiler details", false),
            ],
          },
        ],
      },
      {
        id: "posters",
        title: "Posters",
        desc: "Cards, badges and numbered rows",
        preview: "poster",
        groups: [
          {
            title: "Cards",
            items: [
              c("cardCornerPreset", "Corners", corners, "soft"),
              c("cardLayout", "Layout", ["vertical", "horizontal"], "vertical"),
              c("posterWidthPreset", "Width", ["xsmall", "small", "medium", "large", "xlarge"], "medium"),
              t("posterLandscapeMode", "Landscape posters", false),
              t("posterHideTitles", "Hide titles", false),
              t("posterHoverPreview", "Hover preview", true),
              r("posterHoverPreviewDelay", "Preview delay", 0, 5, 0.1, 1, "s"),
              t("posterLift", "Lift on hover", true),
            ],
          },
          {
            title: "Overlays",
            items: [
              t("posterOverlaysEnabled", "Poster overlays", true),
              c("posterBadgeSize", "Badge size", ["default", "small", "large"], "default", undefined, undefined, "posterOverlaysEnabled"),
              t("posterFadeTint", "Tinted fade", true, undefined, "posterOverlaysEnabled"),
              c("posterFadeStrength", "Fade strength", ["high", "medium", "low"], "medium", undefined, undefined, "posterFadeTint"),
            ],
          },
          {
            title: "Rating",
            items: [
              t("posterRatingBadge", "Rating badge", true),
              c("posterRatingSource", "Source", ["imdb", "mdblist"], "imdb", undefined, undefined, "posterRatingBadge"),
              c("posterRatingPosition", "Position", ["bar", "number", "minimal", "frosted", ...corner4], "frosted", undefined, undefined, "posterRatingBadge"),
            ],
          },
          {
            title: "Badges",
            items: [
              t("posterStatusBadge", "Release status", false),
              c("posterStatusPosition", "Status position", ["sash", "banner", ...corner4], "sash", undefined, undefined, "posterStatusBadge"),
              t("posterWatchedBadge", "Watched", true),
              t("posterProgressBar", "Progress bar", true),
              t("posterSavedBadge", "In library", true),
              t("posterTrendingBadge", "Trending sash", false),
              t("posterAgeRating", "Age rating", false),
              t("posterQualityBadges", "Quality", false),
              t("posterAwardBadge", "Award sash", false),
            ],
          },
          {
            title: "Top numbers",
            items: [
              t("posterTopNumbers", "Numbered rows", false),
              c("posterTopNumbersRows", "Rows", ["trending", "all"], "trending", undefined, undefined, "posterTopNumbers"),
              c("posterTopNumbersCount", "Titles", ["3", "5", "10", "20"], "10", undefined, undefined, "posterTopNumbers"),
              c("posterTopNumbersStyle", "Style", ["outline", "solid", "ghost"], "outline", undefined, undefined, "posterTopNumbers"),
              c("posterTopNumbersColor", "Color", ["white", "gray", "accent"], "white", undefined, undefined, "posterTopNumbers"),
              c("posterTopNumbersSize", "Size", ["default", "small", "large"], "default", undefined, undefined, "posterTopNumbers"),
              c("posterTopNumbersPosition", "Position", ["beside", "overlay"], "beside", undefined, undefined, "posterTopNumbers"),
            ],
          },
          {
            title: "Expanded posters",
            items: [
              t("expandedPosters", "Expand on focus", false),
              c("expandedPostersDelay", "Expand delay", ["0", "1", "2", "3"], "1", undefined, "s", "expandedPosters"),
              t("trailerOnExpandedPosters", "Trailer", false, undefined, "expandedPosters"),
              c("trailerOnExpandedPostersDelay", "Trailer delay", ["0", "2", "4", "6"], "2", undefined, "s", "trailerOnExpandedPosters"),
            ],
          },
        ],
      },
      {
        id: "ambient",
        title: "Ambient mode",
        desc: "Artwork while the screen sits idle",
        groups: [
          {
            title: "Ambient mode",
            items: [
              t("ambientModeEnabled", "Enabled", false),
              c("ambientModeTimeoutMinutes", "Start after", ["1", "2", "5", "10", "15", "30"], "5", undefined, "min", "ambientModeEnabled"),
            ],
          },
        ],
      },
    ],
  },
  {
    id: "playback",
    title: "Playback",
    desc: "Player, subtitles and streams",
    icon: "play",
    sections: [
      {
        id: "playback",
        title: "Playback",
        desc: "Sources, skipping and video",
        groups: [
          {
            title: "Playback",
            items: [
              c("preferredPlayer", "Playback destination", ["mpv", "external"], "mpv"),
              c("streamSourceSelectionMode", "Stream selection", ["manual", "first", "best", "regex"], "manual"),
              t("autoPlayNextEpisode", "Autoplay next episode", true),
              c("autoPlayCountdownSecs", "Countdown", ["5", "7", "10", "15"], "7", undefined, "s", "autoPlayNextEpisode"),
              r("scrobbleWatchedPercent", "Mark watched at", 50, 100, 5, 90, "%"),
              t("tryBingeGroup", "Prefer binge group", true),
              t("autoRetryNextSource", "Retry next source", true),
              t("backgroundPlayback", "Background playback", false),
              t("pictureInPicture", "Picture in picture", true),
            ],
          },
          {
            title: "Stream reuse",
            items: [
              t("reuseLastStream", "Reuse last stream", true, "Resume with the stream you last watched instead of searching again"),
              c("reuseLastStreamTtlHours", "Keep for", ["1", "2", "3", "4", "5", "6", "24", "48", "72", "96", "120", "144"], "24", "How long after watching the saved stream is still reused", "h", "reuseLastStream"),
              t("preloadNextEpisodeSources", "Preload next episode", true, "Fetch sources for the next episode before the current one ends"),
              t("prefetchDetailStreams", "Prefetch on detail screen", false, "Start searching sources as soon as a title opens"),
              t("reuseBingeGroup", "Reuse binge group", true, "Carry the current release's binge group to the next episode"),
            ],
          },
          {
            title: "Sources and video",
            items: [
              t("p2pEnabled", "P2P playback", false),
              c("dolbyVisionMode", "Dolby Vision", ["auto", "native", "convert", "base_layer", "compose"], "auto"),
              c("dolbyVisionTonemap", "Tone mapping", ["auto", "no", "yes", "sdr"], "auto"),
              c("displayPeakNits", "Display peak brightness", ["0", "300", "400", "600", "1000", "1500", "4000"], "0", "0 detects it automatically", " nits"),
            ],
          },
          {
            title: "Skipping and controls",
            items: [
              t("autoSkipIntro", "Auto skip intro", false),
              t("useAnimeSkip", "Anime skip integration", true),
              t("useChapterSkip", "Chapter skip", true),
              t("useSkipSegments", "Skip segments", true),
              t("seekThumbnailEnabled", "Seek thumbnails", true),
              t("holdToSpeedEnabled", "Hold to speed", true),
              c("holdSpeed", "Hold speed", ["1.25", "1.5", "1.75", "2.0", "2.5", "3.0"], "2.0", undefined, "×", "holdToSpeedEnabled"),
              c("playbackSpeed", "Default speed", ["0.75", "1.0", "1.25", "1.5", "2.0"], "1.0", undefined, "×"),
              c("seekSeconds", "Seek interval", ["5", "10", "15", "30"], "10", undefined, "s"),
            ],
          },
          {
            title: "Upscaling and motion",
            items: [
              c("animeUpscalingMode", "Anime upscaling", ["off", "auto"], "off"),
              c("animeUpscalingModePreset", "Anime4K mode", ["a", "b", "c"], "a", undefined, undefined, "animeUpscalingMode"),
              c("animeUpscalingQuality", "Quality", ["anime4k_s", "anime4k_m", "anime4k_l"], "anime4k_m", undefined, undefined, "animeUpscalingMode"),
              c("frameInterpolationMode", "Frame interpolation", ["off", "display_resample", "smooth"], "off"),
              c("audioProcessingMode", "Audio processing", ["reference", "balanced", "night"], "reference"),
            ],
          },
          {
            title: "Buffering",
            items: [
              c("playerBufferCacheMb", "Buffer cache", ["100", "500", "1000", "2000", "-1"], "500", undefined, " MB"),
              c("playerForwardBufferSeconds", "Forward buffer", ["30", "60", "120", "300", "600"], "120", undefined, "s"),
              c("playerBackBufferSeconds", "Back buffer", ["0", "15", "30", "60", "120", "300"], "30", undefined, "s"),
            ],
          },
        ],
      },
      {
        id: "subtitles",
        title: "Audio and subtitles",
        desc: "Languages and subtitle style",
        preview: "subs",
        groups: [
          {
            title: "Languages",
            items: [
              c("preferredAudioLanguage", "Preferred audio", audioLangs, "original"),
              c("secondaryAudioLanguage", "Secondary audio", ["none", ...langs], "none"),
              t("animePreferJapaneseAudio", "Anime in Japanese", true),
              c("preferredSubtitleLanguage", "Preferred subtitles", subLangs, "device"),
              c("secondarySubtitleLanguage", "Secondary subtitles", ["none", ...langs], "none"),
              t("autoEnableSubtitles", "Auto-enable subtitles", true),
            ],
          },
          {
            title: "Style",
            items: [
              c("subtitlePosition", "Position", ["100", "90", "80", "70"], "90", undefined, "%"),
              r("subtitleSize", "Size", 50, 200, 5, 100, "%"),
              col("subtitleColor", "Text color", swatches, "#FFFFFF"),
              c("subtitleTextOpacity", "Text opacity", opacity, "1.0"),
              col("subtitleBackgroundColor", "Background", swatches.slice(0, 5), "#000000"),
              c("subtitleBackgroundOpacity", "Background opacity", opacity, "0.5"),
              col("subtitleOutlineColor", "Outline", swatches.slice(0, 5), "#000000"),
              c("subtitleOutlineOpacity", "Outline opacity", opacity, "1.0"),
              c("subtitleOutlineSize", "Outline size", ["0", "1", "2", "3", "4", "5"], "2"),
              t("subtitleBold", "Bold", false),
              t("subtitleShadow", "Shadow", true),
              t("subtitleForceStyle", "Override styled subtitles", false),
            ],
          },
        ],
      },
      {
        id: "player",
        title: "Player",
        desc: "Overlay and controls",
        groups: [
          {
            title: "Overlay",
            items: [
              t("pauseMetadataOverlayEnabled", "Pause screen info", true),
              t("contentWarningsEnabled", "Content warnings", true),
              t("playerShowTitle", "Content info", true),
              t("playerShowUpNextCard", "Up next card", true),
            ],
          },
          {
            title: "Controls",
            items: [
              t("playerShowAudioSubtitles", "Audio and subtitles", true),
              t("playerShowSpeed", "Speed", true),
              t("playerShowEpisodes", "Episode picker", true),
              t("playerShowNextEpisode", "Next episode", true),
              t("playerShowSettings", "Player settings", true),
              t("playerShowVolume", "Volume", true),
              t("playerShowFullscreen", "Fullscreen", true),
              t("playerShowCast", "Cast", true),
              t("playerShowMarkSegment", "Mark segment", false),
              t("playerShowSegments", "Segments on seek bar", true),
            ],
          },
          {
            title: "Touch",
            items: [
              t("playerGestures", "Brightness and volume gestures", true),
              t("playerDoubleTapSeek", "Double-tap to seek", true),
              t("playerCenterControls", "Controls in the middle", true),
            ],
          },
        ],
      },
      {
        id: "previews",
        title: "Seek previews",
        desc: "Thumbnails on the seek bar",
        groups: [
          {
            title: "Seek previews",
            items: [
              c("trickplayMode", "Generation", ["async", "pregenerate"], "async"),
              c("trickplayInterval", "Frame interval", ["5", "10", "20", "30", "60"], "10", undefined, "s"),
              t("trickplaySheet", "Preview sheet", true),
            ],
          },
        ],
      },
      {
        id: "badges",
        title: "Stream badges",
        desc: "How sources are labelled",
        groups: [
          {
            title: "Badges",
            items: [
              t("streamBadgesEnabled", "Stream badges", true),
              t("streamBadgeBuiltIn", "Built-in badges", true, undefined, "streamBadgesEnabled"),
              t("streamBadgeFileSize", "File size", true, undefined, "streamBadgesEnabled"),
              c("streamBadgeTheme", "Theme", ["vivid", "classic", "soft", "neon", "gold", "sunset", "cinema"], "vivid", undefined, undefined, "streamBadgesEnabled"),
              c("streamBadgePlacement", "Placement", ["bottom", "top"], "bottom", undefined, undefined, "streamBadgesEnabled"),
              c("streamLayout", "Card layout", ["off", "detailed", "compact", "minimal", "custom"], "detailed"),
              t("streamFormatterEnabled", "Text formatter", false),
            ],
          },
        ],
      },
    ],
  },
  {
    id: "content",
    title: "Content",
    desc: "Add-ons, plugins and servers",
    icon: "package",
    sections: [
      {
        id: "content",
        title: "Library",
        desc: "Where your titles come from",
        groups: [
          {
            title: "Sources",
            items: [
              t("collectionsEnabled", "Collections", true),
              t("addonsAutoUpdate", "Update add-ons automatically", true),
              t("pluginsAutoUpdate", "Update plugins automatically", true),
            ],
          },
        ],
      },
    ],
  },
  {
    id: "downloads",
    title: "Downloads",
    desc: "Torrent and download defaults",
    icon: "download",
    sections: [
      {
        id: "downloads",
        title: "Downloads",
        desc: "Peer-to-peer and storage",
        groups: [
          {
            title: "Peer-to-peer and storage",
            items: [
              c("torrentSpeedPreset", "Torrent speed", ["default", "fast", "ultra_fast"], "default"),
              c("torrentCachePreset", "Torrent cache", ["auto", "2gb", "5gb", "10gb", "unlimited"], "auto"),
              c("downloadSourceSelectionMode", "Download source", ["manual", "first", "best", "regex"], "manual"),
              c("downloadSubtitleLanguage", "Download subtitle", ["off", "preferred"], "preferred"),
              t("torrentWifiOnly", "Torrent on Wi-Fi only", true),
            ],
          },
        ],
      },
    ],
  },
  {
    id: "shortcuts",
    title: "Shortcuts",
    desc: "Keyboard and controller",
    icon: "keyboard",
    sections: [
      {
        id: "shortcuts",
        title: "Keyboard",
        desc: "Desktop shortcuts",
        groups: [
          {
            title: "General",
            items: [
              t("shortcutsEnabled", "Keyboard shortcuts", true),
              t("controllerEnabled", "Controller navigation", true),
            ],
          },
        ],
      },
    ],
  },
  {
    id: "about",
    title: "About",
    desc: "Version, licenses and support",
    icon: "info",
    sections: [
      {
        id: "about",
        title: "About",
        desc: "Version and licenses",
        groups: [
          {
            title: "Fluxa",
            items: [t("aboutBetaChannel", "Receive beta builds", false)],
          },
        ],
      },
    ],
  },
];

export const shortcuts: [string, string][] = [
  ["Open settings", "Ctrl ,"],
  ["Search", "Ctrl K"],
  ["Play / pause", "Space"],
  ["Seek back / forward", "← →"],
  ["Volume", "↑ ↓"],
  ["Fullscreen", "F"],
  ["Next episode", "N"],
  ["Mute", "M"],
  ["Subtitles", "C"],
];
