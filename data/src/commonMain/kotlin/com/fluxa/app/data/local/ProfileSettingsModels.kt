package com.fluxa.app.data.local

private const val CINEMETA_ADDON_URL = "https://v3-cinemeta.strem.io/manifest.json"
internal val DEFAULT_ADDON_URLS = listOf(CINEMETA_ADDON_URL)

data class ExternalAccounts(
    val traktAccessToken: String? = null,
    val traktRefreshToken: String? = null,
    val traktTokenExpiresAt: Long? = null,
    val traktLastSyncAt: Long? = null,
    val traktLastSyncedItems: Int? = null,
    val traktLastContinueWatchingCount: Int? = null,
    val traktLastWatchlistCount: Int? = null,
    val malAccessToken: String? = null,
    val malRefreshToken: String? = null,
    val simklAccessToken: String? = null
)

data class AddonSettings(
    val localAddons: List<String>? = DEFAULT_ADDON_URLS,
    val disabledLocalAddons: List<String>? = emptyList()
)

data class SubtitleSettings(
    val size: Float? = 100f,
    val color: Int? = 0xFFFFFFFF.toInt(),
    val backgroundColor: Int? = 0x80000000.toInt(),
    val outlineColor: Int? = 0xFF000000.toInt(),
    val textOpacity: Float? = 1f,
    val backgroundOpacity: Float? = 0.5f,
    val outlineOpacity: Float? = 1f,
    val preferredLanguage: String? = null,
    val secondaryLanguage: String? = null,
    val shadow: Boolean? = true,
    val autoEnable: Boolean? = true
)

data class PlaybackSettings(
    val preferredAudioLanguage: String? = null,
    val secondaryAudioLanguage: String? = null,
    val ambientLight: Boolean? = true,
    val forceSoftwareAudio: Boolean? = false,
    val audioDecoderMode: String? = null,
    val preferredPlayer: String? = "internal",
    val autoSkipIntro: Boolean? = false,
    val autoPlayNextEpisode: Boolean? = true,
    val nextEpisodeThresholdPercent: Float? = 90f,
    val watchedThresholdPercent: Float? = 80f,
    val seekForwardSeconds: Int? = 10,
    val seekBackwardSeconds: Int? = 10,
    val playerBufferCacheMb: Int? = 100,
    val playerForwardBufferSeconds: Int? = 30,
    val playerBackBufferSeconds: Int? = 30,
    val playerMinBufferSeconds: Int? = 8,
    val playerPlaybackBufferMs: Int? = 1500,
    val playerRebufferBufferMs: Int? = 2500,
    val backgroundPlayback: Boolean? = false,
    val pictureInPicture: Boolean? = true,
    val playbackSpeed: Float? = 1f,
    val holdToSpeedEnabled: Boolean? = true,
    val holdSpeed: Float? = 2f,
    val dolbyVisionFallbackMode: String? = "auto",
    val dvRpuMode: Int? = null,
    val dvZeroLevel5: Boolean? = null,
    val dvHdr10PlusMode: String? = null,
    val tunneledPlayback: Boolean? = false,
    val useIntroDb: Boolean? = true,
    val useAniSkip: Boolean? = true,
    val defaultQuality: String? = "1080p",
    val mobileDataUsage: String? = "medium",
    val hdrPlayback: Boolean? = true,
    val resumePlayback: Boolean? = true,
    val autoplayMode: String? = "next_episode",
    val streamSourceSelectionMode: String? = "manual",
    val streamSourceRegexPattern: String? = "",
    val tryBingeGroup: Boolean? = false
)

data class TorrentSettings(
    val wifiOnly: Boolean? = false,
    val maxConnections: Int? = 60,
    val speedPreset: String? = "default",
    val cachePreset: String? = "auto"
)

data class AppearanceSettings(
    val language: String? = "en",
    val cardLayout: String? = "vertical",
    val continueWatchingLayout: String? = "horizontal",
    val continueWatchingArtwork: String? = "episode",
    val continueWatchingEnabled: Boolean? = true,
    val continueWatchingHideTitles: Boolean? = false,
    val appTheme: String? = "dark",
    val accentColorArgb: Int? = 0xFFFFFFFF.toInt(),
    val cardCornerPreset: String? = "medium",
    val interfaceDensity: String? = "medium",
    val amoledMode: Boolean? = false,
    val posterWidthPreset: String? = "medium",
    val posterLandscapeMode: Boolean? = false,
    val posterHideTitles: Boolean? = false,
    val detailEpisodeViewMode: String? = "modern",
    val detailSeasonSelectorMode: String? = "dropdown",
    val detailSeasonPostersOnHero: Boolean? = true,
    val homeSeasonPostersOnHero: Boolean? = true,
    val animationsEnabled: Boolean? = true,
    val reduceMotion: Boolean? = false,
    val startPage: String? = "home"
)

data class HomeFeedSettings(
    val heroFeedToggles: List<String>? = null,
    val homeFeedToggles: List<String>? = null,
    val topTenFeedToggles: List<String>? = null,
    val heroFeedOrder: List<String>? = null,
    val homeFeedOrder: List<String>? = null,
    val showHeroSection: Boolean? = true,
    val libraryCollections: List<LibraryUserCollection>? = emptyList(),
    val cs3FeedsConfigured: Boolean? = null
)
