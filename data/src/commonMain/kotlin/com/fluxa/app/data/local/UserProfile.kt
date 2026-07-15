package com.fluxa.app.data.local

import com.fluxa.app.common.Constants

data class UserProfile(
    val id: String,
    val email: String,
    val authKey: String,
    val profileName: String? = null,
    val colorArgb: Int = 0xFFFFFFFF.toInt(),
    val isGuest: Boolean = false,
    val avatarUrl: String? = null,
    val pinHash: String? = null,
    val biometricEnabled: Boolean? = false,
    val streamingServerUrl: String? = Constants.LocalServer.BASE_URL,
    val localAddons: List<String>? = DEFAULT_ADDON_URLS,
    val disabledLocalAddons: List<String>? = emptyList(),
    val language: String? = "en",
    val buildServerUrl: String? = null,
    val subtitleSize: Float? = 100f,
    val subtitleColor: Int? = 0xFFFFFFFF.toInt(),
    val subtitleBackgroundColor: Int? = 0x80000000.toInt(),
    val subtitleOutlineColor: Int? = 0xFF000000.toInt(),
    val subtitleTextOpacity: Float? = 1f,
    val subtitleBackgroundOpacity: Float? = 0.5f,
    val subtitleOutlineOpacity: Float? = 1f,
    val preferredSubtitleLanguage: String? = null,
    val preferredAudioLanguage: String? = null,
    val secondarySubtitleLanguage: String? = null,
    val secondaryAudioLanguage: String? = null,
    val ambientLight: Boolean? = true,
    val forceSoftwareAudio: Boolean? = false,
    val audioDecoderMode: String? = null,
    val preferredPlayer: String? = "internal",
    val nuvioAccessToken: String? = null,
    val nuvioRefreshToken: String? = null,
    val nuvioTokenExpiresAt: Long? = null,
    val nuvioUserId: String? = null,
    val nuvioEmail: String? = null,
    val nuvioProfileIndex: Int? = null,
    val nuvioLastSyncAt: Long? = null,
    val traktAccessToken: String? = null,
    val traktRefreshToken: String? = null,
    val traktTokenExpiresAt: Long? = null,
    val traktLastSyncAt: Long? = null,
    val traktLastSyncedItems: Int? = null,
    val traktLastContinueWatchingCount: Int? = null,
    val traktLastWatchlistCount: Int? = null,
    val malAccessToken: String? = null,
    val malRefreshToken: String? = null,
    val malTokenExpiresAt: Long? = null,
    val malLastSyncAt: Long? = null,
    val simklAccessToken: String? = null,
    val simklLastSyncAt: Long? = null,
    val anilistAccessToken: String? = null,
    val anilistRefreshToken: String? = null,
    val anilistTokenExpiresAt: Long? = null,
    val tmdbApiKey: String? = null,
    val introDbApiKey: String? = null,
    val tmdbCastImagesEnabled: Boolean? = true,
    val tmdbSimilarResultsEnabled: Boolean? = true,
    val tmdbTrailersEnabled: Boolean? = true,
    val tmdbRecommendationsEnabled: Boolean? = true,
    val tmdbCollectionInfoEnabled: Boolean? = true,
    val tmdbEpisodeImagesEnabled: Boolean? = true,
    val tmdbLogosBackdropsEnabled: Boolean? = true,
    val tmdbRatingsEnabled: Boolean? = true,
    val tmdbBasicInfoEnabled: Boolean? = true,
    val tmdbDetailsEnabled: Boolean? = true,
    val tmdbProductionsEnabled: Boolean? = true,
    val tmdbNetworksEnabled: Boolean? = true,
    val cardLayout: String? = "vertical",
    val continueWatchingLayout: String? = "horizontal",
    val continueWatchingArtwork: String? = "episode",
    val continueWatchingEnabled: Boolean? = true,
    val continueWatchingHideTitles: Boolean? = false,
    val trailerOnHero: Boolean? = true,
    val heroFollowsFocusedItem: Boolean? = false,
    val blurUnwatchedEpisodes: Boolean? = false,
    val episodeCardsLayout: String? = "list",
    val expandedPostersEnabled: Boolean? = false,
    val expandedPostersDelaySeconds: Int? = 2,
    val trailerOnHomeHeroEnabled: Boolean? = false,
    val trailerOnHomeHeroDelaySeconds: Int? = 4,
    val trailerOnExpandedPostersEnabled: Boolean? = false,
    val trailerOnExpandedPostersDelaySeconds: Int? = 3,
    val tvNavLayout: String? = "left",
    val subtitleShadow: Boolean? = true,
    val autoEnableSubtitles: Boolean? = true,
    val autoSkipIntro: Boolean? = false,
    val autoPlayNextEpisode: Boolean? = true,
    val autoPlayCountdownSecs: Int? = 10,
    val autoRetryNextSource: Boolean? = false,
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
    val timezoneConversionEnabled: Boolean? = true,
    val torrentWifiOnly: Boolean? = false,
    val torrentMaxConnections: Int? = 60,
    val torrentSpeedPreset: String? = "default",
    val torrentCachePreset: String? = "auto",
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
    val gifAutoplayEnabled: Boolean? = true,
    val startPage: String? = "home",
    val notificationsEnabled: Boolean? = true,
    val alertNewEpisodes: Boolean? = true,
    val automaticUpdates: Boolean? = true,
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
    val useChapterSkip: Boolean? = true,
    val defaultQuality: String? = "1080p",
    val mobileDataUsage: String? = "medium",
    val hdrPlayback: Boolean? = true,
    val resumePlayback: Boolean? = true,
    val autoplayMode: String? = "next_episode",
    val streamSourceSelectionMode: String? = "manual",
    val streamSourceRegexPattern: String? = "",
    val downloadSourceSelectionMode: String? = "first",
    val downloadSourceRegexPattern: String? = "",
    val downloadSubtitleLanguage: String? = "preferred",
    val tryBingeGroup: Boolean? = false,
    val mpvCustomOptions: String? = null,
    val animeUseMpv: Boolean? = false,
    val animePreferJapaneseAudio: Boolean? = false,
    val heroFeedToggles: List<String>? = null,
    val homeFeedToggles: List<String>? = null,
    val topTenFeedToggles: List<String>? = null,
    val heroFeedOrder: List<String>? = null,
    val homeFeedOrder: List<String>? = null,
    val showHeroSection: Boolean? = true,
    val libraryCollections: List<LibraryUserCollection>? = emptyList(),
    val cs3FeedsConfigured: Boolean? = null,
    val externalAccounts: ExternalAccounts? = null,
    val addonSettings: AddonSettings? = null,
    val subtitleSettings: SubtitleSettings? = null,
    val playbackSettings: PlaybackSettings? = null,
    val torrentSettings: TorrentSettings? = null,
    val appearanceSettings: AppearanceSettings? = null,
    val homeFeedSettings: HomeFeedSettings? = null
) {
    val safeAudioDecoderMode: String get() {
        val mode = audioDecoderMode
        if (mode != null && mode in setOf("hw_only", "hw_prefer", "sw_only")) return mode
        return if (forceSoftwareAudio == true) "sw_only" else "hw_prefer"
    }
    val safeContinueWatchingHideTitles: Boolean get() = continueWatchingHideTitles ?: appearanceSettings?.continueWatchingHideTitles ?: false
    val safeTrailerOnHero: Boolean get() = trailerOnHero ?: true
    val safeHeroFollowsFocusedItem: Boolean get() = heroFollowsFocusedItem ?: false
    val safeBlurUnwatchedEpisodes: Boolean get() = blurUnwatchedEpisodes ?: false
    val safeEpisodeCardsLayout: String get() = episodeCardsLayout ?: "list"
    val safeExpandedPostersEnabled: Boolean get() = expandedPostersEnabled ?: false
    val safeExpandedPostersDelaySeconds: Int get() = (expandedPostersDelaySeconds ?: 2).coerceIn(0, 10)
    val safeTrailerOnHomeHeroEnabled: Boolean get() = trailerOnHomeHeroEnabled ?: false
    val safeTrailerOnHomeHeroDelaySeconds: Int get() = (trailerOnHomeHeroDelaySeconds ?: 4).coerceIn(0, 15)
    val safeTrailerOnExpandedPostersEnabled: Boolean get() = trailerOnExpandedPostersEnabled ?: false
    val safeTrailerOnExpandedPostersDelaySeconds: Int get() = (trailerOnExpandedPostersDelaySeconds ?: 3).coerceIn(0, 15)
    val safeTvNavLayout: String get() = if (tvNavLayout == "top") "top" else "left"
    val safeDetailSeasonSelectorMode: String get() = when (val mode = detailSeasonSelectorMode ?: appearanceSettings?.detailSeasonSelectorMode) {
        "tabs", "posters" -> mode
        else -> "dropdown"
    }
    val safeDetailSeasonPostersOnHero: Boolean get() = detailSeasonPostersOnHero ?: appearanceSettings?.detailSeasonPostersOnHero ?: true
    val safeHomeSeasonPostersOnHero: Boolean get() = homeSeasonPostersOnHero ?: appearanceSettings?.homeSeasonPostersOnHero ?: true
    val safeAutoPlayCountdownSecs: Int get() = (autoPlayCountdownSecs ?: 10).coerceIn(0, 15)
    val safeAutoRetryNextSource: Boolean get() = autoRetryNextSource ?: false
    val safePlayerMinBufferSeconds: Int get() = (playerMinBufferSeconds ?: 8).coerceIn(2, 30)
    val safePlayerPlaybackBufferMs: Int get() = (playerPlaybackBufferMs ?: 1500).coerceIn(500, 5000)
    val safePlayerRebufferBufferMs: Int get() = (playerRebufferBufferMs ?: 2500).coerceIn(1000, 10000)
    val safeGifAutoplayEnabled: Boolean get() = gifAutoplayEnabled ?: true
    val safeDolbyVisionFallbackMode: String get() = when (dolbyVisionFallbackMode) {
        "off" -> "off"
        "force_hdr10" -> "force_hdr10"
        else -> "auto"
    }
    val safeDvRpuMode: Int get() = dvRpuMode?.takeIf { it in setOf(1, 2, 4) } ?: 2
    val safeDvZeroLevel5: Boolean get() = dvZeroLevel5 ?: false
    val safeDvHdr10PlusMode: String get() = when (dvHdr10PlusMode) {
        "always", "never" -> dvHdr10PlusMode
        else -> "auto"
    }
    val safeUseChapterSkip: Boolean get() = useChapterSkip ?: true
    val safeDownloadSourceRegexPattern: String get() = downloadSourceRegexPattern.orEmpty()
    val safeDownloadSubtitleLanguage: String get() = downloadSubtitleLanguage?.takeIf { it in setOf("off", "preferred", "tr", "en", "ja", "es", "fr", "de") } ?: "preferred"
    val safeMpvCustomOptions: String get() = mpvCustomOptions.orEmpty()
    val safeAnimeUseMpv: Boolean get() = animeUseMpv ?: false
    val safeAnimePreferJapaneseAudio: Boolean get() = animePreferJapaneseAudio ?: false
    val safeHeroFeedToggles: Set<String> get() = heroFeedToggles?.toSet() ?: defaultHeroFeedKeys
    val safeHomeFeedToggles: Set<String> get() = homeFeedToggles?.toSet() ?: defaultHomeFeedKeys
    val safeTopTenFeedToggles: Set<String> get() = topTenFeedToggles.orEmpty().toSet()
    val safeLibraryCollections: List<LibraryUserCollection> get() = libraryCollections ?: homeFeedSettings?.libraryCollections.orEmpty()

    val displayName: String get() = when {
        !profileName.isNullOrBlank() -> profileName
        isGuest -> if (email.isBlank() || email == "guest") "Guest" else email
        email.contains("@") -> email.substringBefore("@")
        else -> email
    }
    val safeStreamingServerUrl: String get() = streamingServerUrl ?: Constants.LocalServer.BASE_URL
    val safeInstalledLocalAddons: List<String> get() = localAddons.orEmpty()
    val safeSimklLastSyncAt: Long get() = simklLastSyncAt ?: 0L
    val safeMalTokenExpiresAt: Long get() = malTokenExpiresAt ?: 0L
    val safeMalLastSyncAt: Long get() = malLastSyncAt ?: 0L
    val safeTmdbApiKey: String get() = tmdbApiKey.orEmpty()
    val safeIntroDbApiKey: String get() = introDbApiKey.orEmpty()
    val safeTmdbCastImagesEnabled: Boolean get() = tmdbCastImagesEnabled ?: true
    val safeTmdbSimilarResultsEnabled: Boolean get() = tmdbSimilarResultsEnabled ?: true
    val safeTmdbTrailersEnabled: Boolean get() = tmdbTrailersEnabled ?: true
    val safeTmdbRecommendationsEnabled: Boolean get() = tmdbRecommendationsEnabled ?: true
    val safeTmdbCollectionInfoEnabled: Boolean get() = tmdbCollectionInfoEnabled ?: true
    val safeTmdbEpisodeImagesEnabled: Boolean get() = tmdbEpisodeImagesEnabled ?: true
    val safeTmdbLogosBackdropsEnabled: Boolean get() = tmdbLogosBackdropsEnabled ?: true
    val safeTmdbRatingsEnabled: Boolean get() = tmdbRatingsEnabled ?: true
    val safeTmdbBasicInfoEnabled: Boolean get() = tmdbBasicInfoEnabled ?: true
    val safeTmdbDetailsEnabled: Boolean get() = tmdbDetailsEnabled ?: true
    val safeTmdbProductionsEnabled: Boolean get() = tmdbProductionsEnabled ?: true
    val safeTmdbNetworksEnabled: Boolean get() = tmdbNetworksEnabled ?: true

    fun withStructuredSettings(): UserProfile {
        return copy(
            externalAccounts = externalAccounts ?: ExternalAccounts(
                traktAccessToken = traktAccessToken,
                traktRefreshToken = traktRefreshToken,
                traktTokenExpiresAt = traktTokenExpiresAt,
                traktLastSyncAt = traktLastSyncAt,
                traktLastSyncedItems = traktLastSyncedItems,
                traktLastContinueWatchingCount = traktLastContinueWatchingCount,
                traktLastWatchlistCount = traktLastWatchlistCount,
                malAccessToken = malAccessToken,
                malRefreshToken = malRefreshToken,
                simklAccessToken = simklAccessToken
            ),
            addonSettings = addonSettings ?: AddonSettings(
                localAddons = localAddons,
                disabledLocalAddons = disabledLocalAddons
            ),
            subtitleSettings = subtitleSettings ?: SubtitleSettings(
                size = subtitleSize,
                color = subtitleColor,
                backgroundColor = subtitleBackgroundColor,
                outlineColor = subtitleOutlineColor,
                textOpacity = subtitleTextOpacity,
                backgroundOpacity = subtitleBackgroundOpacity,
                outlineOpacity = subtitleOutlineOpacity,
                preferredLanguage = preferredSubtitleLanguage,
                secondaryLanguage = secondarySubtitleLanguage,
                shadow = subtitleShadow,
                autoEnable = autoEnableSubtitles
            ),
            playbackSettings = playbackSettings ?: PlaybackSettings(
                preferredAudioLanguage = preferredAudioLanguage,
                secondaryAudioLanguage = secondaryAudioLanguage,
                ambientLight = ambientLight,
                forceSoftwareAudio = forceSoftwareAudio,
                audioDecoderMode = audioDecoderMode,
                preferredPlayer = preferredPlayer,
                autoSkipIntro = autoSkipIntro,
                autoPlayNextEpisode = autoPlayNextEpisode,
                nextEpisodeThresholdPercent = nextEpisodeThresholdPercent,
                watchedThresholdPercent = watchedThresholdPercent,
                seekForwardSeconds = seekForwardSeconds,
                seekBackwardSeconds = seekBackwardSeconds,
                playerBufferCacheMb = playerBufferCacheMb,
                playerForwardBufferSeconds = playerForwardBufferSeconds,
                playerBackBufferSeconds = playerBackBufferSeconds,
                playerMinBufferSeconds = playerMinBufferSeconds,
                playerPlaybackBufferMs = playerPlaybackBufferMs,
                playerRebufferBufferMs = playerRebufferBufferMs,
                backgroundPlayback = backgroundPlayback,
                pictureInPicture = pictureInPicture,
                playbackSpeed = playbackSpeed,
                holdToSpeedEnabled = holdToSpeedEnabled,
                holdSpeed = holdSpeed,
                dolbyVisionFallbackMode = dolbyVisionFallbackMode,
                dvRpuMode = dvRpuMode,
                dvZeroLevel5 = dvZeroLevel5,
                tunneledPlayback = tunneledPlayback,
                useIntroDb = useIntroDb,
                useAniSkip = useAniSkip,
                defaultQuality = defaultQuality,
                mobileDataUsage = mobileDataUsage,
                hdrPlayback = hdrPlayback,
                resumePlayback = resumePlayback,
                autoplayMode = autoplayMode,
                streamSourceSelectionMode = streamSourceSelectionMode,
                streamSourceRegexPattern = streamSourceRegexPattern,
                tryBingeGroup = tryBingeGroup
            ),
            torrentSettings = torrentSettings ?: TorrentSettings(
                wifiOnly = torrentWifiOnly,
                maxConnections = torrentMaxConnections,
                speedPreset = torrentSpeedPreset,
                cachePreset = torrentCachePreset
            ),
            appearanceSettings = appearanceSettings ?: AppearanceSettings(
                language = language,
                cardLayout = cardLayout,
                continueWatchingLayout = continueWatchingLayout,
                continueWatchingArtwork = continueWatchingArtwork,
                continueWatchingEnabled = continueWatchingEnabled,
                continueWatchingHideTitles = continueWatchingHideTitles,
                appTheme = appTheme,
                accentColorArgb = accentColorArgb,
                cardCornerPreset = cardCornerPreset,
                interfaceDensity = interfaceDensity,
                amoledMode = amoledMode,
                posterWidthPreset = posterWidthPreset,
                posterLandscapeMode = posterLandscapeMode,
                posterHideTitles = posterHideTitles,
                detailEpisodeViewMode = detailEpisodeViewMode,
                detailSeasonSelectorMode = detailSeasonSelectorMode,
                detailSeasonPostersOnHero = detailSeasonPostersOnHero,
                homeSeasonPostersOnHero = homeSeasonPostersOnHero,
                animationsEnabled = animationsEnabled,
                reduceMotion = reduceMotion,
                startPage = startPage
            ),
            homeFeedSettings = homeFeedSettings ?: HomeFeedSettings(
                heroFeedToggles = heroFeedToggles,
                homeFeedToggles = homeFeedToggles,
                topTenFeedToggles = topTenFeedToggles,
                heroFeedOrder = heroFeedOrder,
                homeFeedOrder = homeFeedOrder,
                showHeroSection = showHeroSection,
                libraryCollections = libraryCollections,
                cs3FeedsConfigured = cs3FeedsConfigured
            )
        )
    }

    val safeColorArgb: Int get() {
        if (colorArgb != 0xFFFFFFFF.toInt()) return colorArgb
        val seed = id.ifBlank { email }
        var h = 0
        for (ch in seed) h = h * 31 + ch.code
        val palette = intArrayOf(
            0xFFE03131.toInt(), 0xFF1971C2.toInt(), 0xFF2F9E44.toInt(), 0xFFE67700.toInt(),
            0xFF7048E8.toInt(), 0xFF0C8599.toInt(), 0xFFC2255C.toInt(), 0xFF5C940D.toInt()
        )
        return palette[kotlin.math.abs(h) % palette.size]
    }

    companion object {
        val defaultHeroFeedKeys = emptySet<String>()
        val defaultHomeFeedKeys = emptySet<String>()
    }
}
