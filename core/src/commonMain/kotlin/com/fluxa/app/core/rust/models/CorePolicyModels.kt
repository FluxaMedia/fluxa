package com.fluxa.app.core.rust.models

data class NativeCacheEntryPolicy(
    val key: String = "",
    val storedAtMillis: Long = 0L,
    val expiresAtMillis: Long = 0L,
    val isExpired: Boolean = false
)

data class NativeCacheTrimPolicy(
    val expiredKeys: List<String> = emptyList(),
    val evictedKeys: List<String> = emptyList()
)

data class NativeAddonStoreSearchPolicy(
    val normalizedQuery: String = "",
    val url: String = "",
    val useCache: Boolean = false,
    val shouldFetch: Boolean = false
)

data class NativeDataFailurePolicy(
    val operation: String = "",
    val kind: String = "",
    val message: String = "",
    val retryable: Boolean = false,
    val staleFallbackAllowed: Boolean = false
)

data class NativeCalendarNotificationContent(
    val items: List<Map<String, Any?>> = emptyList(),
    val keys: List<String> = emptyList()
)

data class NativeWatchlistTogglePlan(
    val command: String = "add",
    val itemId: String = "",
    val optimisticIsInWatchlist: Boolean = false,
    val profileId: String? = null
)

data class NativeLibraryCollectionImportValidation(
    val isValid: Boolean = false,
    val validCollections: List<Map<String, Any?>> = emptyList(),
    val issues: List<String> = emptyList()
)

data class NativeLibraryOfflineGrouping(
    val ready: List<Map<String, Any?>> = emptyList(),
    val downloading: List<Map<String, Any?>> = emptyList(),
    val queued: List<Map<String, Any?>> = emptyList(),
    val failed: List<Map<String, Any?>> = emptyList()
)

data class NativeOfflineDownloadPlan(
    val supported: Boolean = false,
    val reason: String? = null,
    val playbackUrl: String = "",
    val baseName: String = "",
    val videoFileName: String = "",
    val subtitleFileName: String? = null,
    val posterFileName: String = "",
    val backgroundFileName: String = "",
    val logoFileName: String = "",
    val videoId: String? = null,
    val streamTitle: String? = null
)

data class NativePlayerBackendSelection(
    val backend: String = "exoplayer",
    val reason: String = "default"
)

data class NativeTorrentFallbackFilePolicy(
    val fallbackFileIndexes: List<Int> = emptyList(),
    val rejectedIndex: Int? = null
)

data class NativePlayerBufferTargets(
    val forwardBufferMs: Long = 120_000L,
    val backBufferMs: Long = 30_000L,
    val cacheSizeBytes: Long = 100_000_000L
)

data class NativePlayerRetryPolicy(
    val shouldRetry: Boolean = false,
    val fallbackAction: String = "show_error",
    val delayMs: Long = 0L,
    val retryCount: Int = 0
)

data class NativeActiveProfilePlan(
    val activeId: String = "guest",
    val shouldCreateDefault: Boolean = false,
    val activeProfile: Map<String, Any?> = emptyMap()
)

data class NativeProfileSettingsMigration(
    val migratedProfile: Map<String, Any?> = emptyMap(),
    val appliedMigrations: List<String> = emptyList(),
    val schemaVersion: Int = 2
)

data class NativeProfileAvatarDefault(
    val avatarUrl: String? = null,
    val fromCatalog: Boolean = false
)

data class NativeProfileSafePrefs(
    val language: String = "en",
    val subtitleSizePercent: Float = 100f,
    val subtitleSize: Float = 20f,
    val subtitleColor: Int = 0xFFFFFFFF.toInt(),
    val subtitleBackgroundColor: Int = 0x80000000.toInt(),
    val subtitleOutlineColor: Int = 0xFF000000.toInt(),
    val subtitleTextOpacity: Float = 1f,
    val subtitleBackgroundOpacity: Float = 0.75f,
    val subtitleOutlineOpacity: Float = 0f,
    val preferredSubtitleLanguage: String = "none",
    val preferredAudioLanguage: String = "none",
    val secondarySubtitleLanguage: String = "none",
    val secondaryAudioLanguage: String = "none",
    val ambientLight: Boolean = true,
    val forceSoftwareAudio: Boolean = false,
    val preferredPlayer: String = "exoplayer",
    val cardLayout: String = "vertical",
    val continueWatchingLayout: String = "horizontal",
    val continueWatchingArtwork: String = "episode",
    val continueWatchingEnabled: Boolean = true,
    val resolvedContinueWatchingLayout: String = "horizontal",
    val subtitleShadow: Boolean = false,
    val autoEnableSubtitles: Boolean = true,
    val autoSkipIntro: Boolean = false,
    val autoPlayNextEpisode: Boolean = true,
    val nextEpisodeThresholdPercent: Float = 90f,
    val watchedThresholdPercent: Float = 80f,
    val seekForwardSeconds: Long = 10L,
    val seekBackwardSeconds: Long = 10L,
    val playerBufferCacheMb: Int = 100,
    val playerForwardBufferSeconds: Long = 120L,
    val playerBackBufferSeconds: Long = 30L,
    val timezoneConversionEnabled: Boolean = true,
    val torrentWifiOnly: Boolean = false,
    val torrentMaxConnections: Long = 60L,
    val torrentSpeedPreset: String = "default",
    val torrentCachePreset: String = "auto",
    val appTheme: String = "dark",
    val accentColorArgb: Int = 0xFFFFFFFF.toInt(),
    val cardCornerPreset: String = "medium",
    val interfaceDensity: String = "medium",
    val amoledMode: Boolean = false,
    val posterWidthPreset: String = "medium",
    val posterLandscapeMode: Boolean = false,
    val posterHideTitles: Boolean = false,
    val animationsEnabled: Boolean = true,
    val reduceMotion: Boolean = false,
    val startPage: String = "home",
    val notificationsEnabled: Boolean = true,
    val alertNewEpisodes: Boolean = true,
    val automaticUpdates: Boolean = true,
    val backgroundPlayback: Boolean = false,
    val pictureInPicture: Boolean = true,
    val playbackSpeed: Float = 1f,
    val holdToSpeedEnabled: Boolean = true,
    val holdSpeed: Float = 2f,
    val dolbyVisionFallbackMode: String = "auto",
    val tunneledPlayback: Boolean = false,
    val useIntroDb: Boolean = true,
    val useAniSkip: Boolean = true,
    val defaultQuality: String = "1080p",
    val mobileDataUsage: String = "medium",
    val hdrPlayback: Boolean = true,
    val resumePlayback: Boolean = true,
    val autoplayMode: String = "next_episode",
    val streamSourceSelectionMode: String = "manual",
    val streamSourceRegexPattern: String = "",
    val tryBingeGroup: Boolean = false,
    val showHeroSection: Boolean = true,
    val traktTokenExpiresAt: Long = 0L,
    val traktLastSyncAt: Long = 0L,
    val traktLastSyncedItems: Long = 0L,
    val traktLastContinueWatchingCount: Long = 0L,
    val traktLastWatchlistCount: Long = 0L
)

data class NativeSearchResultGrouping(
    val groups: List<Map<String, Any?>> = emptyList(),
    val totalCount: Int = 0,
    val query: String = ""
)
