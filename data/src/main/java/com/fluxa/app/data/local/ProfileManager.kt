package com.fluxa.app.data.local

import com.fluxa.app.data.remote.*
import com.fluxa.app.domain.discovery.*

import android.content.Context
import android.content.SharedPreferences
import com.fluxa.app.common.Constants
import com.fluxa.app.core.rust.FluxaCoreNative
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import androidx.annotation.WorkerThread
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

data class UserProfile(
    val id: String,
    val email: String,
    val authKey: String,
    val colorArgb: Int = 0xFFFFFFFF.toInt(),
    val isGuest: Boolean = false,
    val avatarUrl: String? = null,
    val pinHash: String? = null,
    val biometricEnabled: Boolean? = false,
    val streamingServerUrl: String? = Constants.LocalServer.BASE_URL,
    val localAddons: List<String>? = DEFAULT_ADDON_URLS,
    val disabledLocalAddons: List<String>? = emptyList(),
    val language: String? = "en", // "tr" or "en"
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
    @Transient
    private var safePrefsCache: com.fluxa.app.core.rust.models.NativeProfileSafePrefs? = null
    private val safePrefs: com.fluxa.app.core.rust.models.NativeProfileSafePrefs
        get() = safePrefsCache ?: FluxaCoreNative.profileSafePrefs(this).also { safePrefsCache = it }
    val safeLanguage: String get() = safePrefs.language
    val safeSubtitleSizePercent: Float get() = safePrefs.subtitleSizePercent
    val safeSubtitleSize: Float get() = safePrefs.subtitleSize
    val safeSubtitleColor: Int get() = safePrefs.subtitleColor
    val safeSubtitleBackgroundColor: Int get() = safePrefs.subtitleBackgroundColor
    val safeSubtitleOutlineColor: Int get() = safePrefs.subtitleOutlineColor
    val safeSubtitleTextOpacity: Float get() = safePrefs.subtitleTextOpacity
    val safeSubtitleBackgroundOpacity: Float get() = safePrefs.subtitleBackgroundOpacity
    val safeSubtitleOutlineOpacity: Float get() = safePrefs.subtitleOutlineOpacity
    val safePreferredSubtitleLanguage: String get() = safePrefs.preferredSubtitleLanguage
    val safePreferredAudioLanguage: String get() = safePrefs.preferredAudioLanguage
    val safeSecondarySubtitleLanguage: String get() = safePrefs.secondarySubtitleLanguage
    val safeSecondaryAudioLanguage: String get() = safePrefs.secondaryAudioLanguage
    val safeAmbientLight: Boolean get() = safePrefs.ambientLight
    val safeForceSoftwareAudio: Boolean get() = safePrefs.forceSoftwareAudio
    val safeAudioDecoderMode: String get() {
        val mode = audioDecoderMode
        if (mode != null && mode in setOf("hw_only", "hw_prefer", "sw_only")) return mode
        return if (forceSoftwareAudio == true) "sw_only" else "hw_prefer"
    }
    val safePreferredPlayer: String get() = safePrefs.preferredPlayer
    val safeCardLayout: String get() = safePrefs.cardLayout
    val safeContinueWatchingLayout: String get() = safePrefs.continueWatchingLayout
    val safeContinueWatchingArtwork: String get() = safePrefs.continueWatchingArtwork
    val safeContinueWatchingEnabled: Boolean get() = safePrefs.continueWatchingEnabled
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
    val resolvedContinueWatchingLayout: String get() = safePrefs.resolvedContinueWatchingLayout
    val safeSubtitleShadow: Boolean get() = safePrefs.subtitleShadow
    val safeAutoEnableSubtitles: Boolean get() = safePrefs.autoEnableSubtitles
    val safeAutoSkipIntro: Boolean get() = safePrefs.autoSkipIntro
    val safeAutoPlayNextEpisode: Boolean get() = safePrefs.autoPlayNextEpisode
    val safeNextEpisodeThresholdPercent: Float get() = safePrefs.nextEpisodeThresholdPercent
    val safeWatchedThresholdPercent: Float get() = safePrefs.watchedThresholdPercent
    val safeSeekForwardSeconds: Int get() = safePrefs.seekForwardSeconds.toInt()
    val safeSeekBackwardSeconds: Int get() = safePrefs.seekBackwardSeconds.toInt()
    val safePlayerBufferCacheMb: Int get() = FluxaCoreNative.safePlayerBufferCacheMb(playerBufferCacheMb)
    val safePlayerForwardBufferSeconds: Int get() = safePrefs.playerForwardBufferSeconds.toInt()
    val safePlayerBackBufferSeconds: Int get() = safePrefs.playerBackBufferSeconds.toInt()
    val safePlayerMinBufferSeconds: Int get() = (playerMinBufferSeconds ?: 8).coerceIn(2, 30)
    val safePlayerPlaybackBufferMs: Int get() = (playerPlaybackBufferMs ?: 1500).coerceIn(500, 5000)
    val safePlayerRebufferBufferMs: Int get() = (playerRebufferBufferMs ?: 2500).coerceIn(1000, 10000)
    val safeTimezoneConversionEnabled: Boolean get() = safePrefs.timezoneConversionEnabled
    val safeTorrentWifiOnly: Boolean get() = safePrefs.torrentWifiOnly
    val safeTorrentMaxConnections: Int get() = safePrefs.torrentMaxConnections.toInt()
    val safeTorrentSpeedPreset: String get() = safePrefs.torrentSpeedPreset
    val safeTorrentCachePreset: String get() = safePrefs.torrentCachePreset
    val safeAppTheme: String get() = safePrefs.appTheme
    val safeAccentColorArgb: Int get() = safePrefs.accentColorArgb
    val safeCardCornerPreset: String get() = safePrefs.cardCornerPreset
    val safeInterfaceDensity: String get() = safePrefs.interfaceDensity
    val safeAmoledMode: Boolean get() = safePrefs.amoledMode
    val safePosterWidthPreset: String get() = safePrefs.posterWidthPreset
    val safePosterLandscapeMode: Boolean get() = safePrefs.posterLandscapeMode
    val safePosterHideTitles: Boolean get() = safePrefs.posterHideTitles
    val safeAnimationsEnabled: Boolean get() = safePrefs.animationsEnabled
    val safeReduceMotion: Boolean get() = safePrefs.reduceMotion
    val safeStartPage: String get() = safePrefs.startPage
    val safeNotificationsEnabled: Boolean get() = safePrefs.notificationsEnabled
    val safeAlertNewEpisodes: Boolean get() = safePrefs.alertNewEpisodes
    val safeAutomaticUpdates: Boolean get() = safePrefs.automaticUpdates
    val safeBackgroundPlayback: Boolean get() = safePrefs.backgroundPlayback
    val safePictureInPicture: Boolean get() = safePrefs.pictureInPicture
    val safePlaybackSpeed: Float get() = safePrefs.playbackSpeed
    val safeHoldToSpeedEnabled: Boolean get() = safePrefs.holdToSpeedEnabled
    val safeHoldSpeed: Float get() = safePrefs.holdSpeed
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
    val safeTunneledPlayback: Boolean get() = safePrefs.tunneledPlayback
    val safeUseIntroDb: Boolean get() = safePrefs.useIntroDb
    val safeUseAniSkip: Boolean get() = safePrefs.useAniSkip
    val safeDefaultQuality: String get() = safePrefs.defaultQuality
    val safeMobileDataUsage: String get() = safePrefs.mobileDataUsage
    val safeHdrPlayback: Boolean get() = safePrefs.hdrPlayback
    val safeResumePlayback: Boolean get() = safePrefs.resumePlayback
    val safeAutoplayMode: String get() = safePrefs.autoplayMode
    val safeStreamSourceSelectionMode: String get() = FluxaCoreNative.safeStreamSourceSelectionMode(streamSourceSelectionMode)
    val safeStreamSourceRegexPattern: String get() = safePrefs.streamSourceRegexPattern
    val safeDownloadSourceSelectionMode: String get() = FluxaCoreNative.safeStreamSourceSelectionMode(downloadSourceSelectionMode)
    val safeDownloadSourceRegexPattern: String get() = downloadSourceRegexPattern.orEmpty()
    val safeDownloadSubtitleLanguage: String get() = downloadSubtitleLanguage?.takeIf { it in setOf("off", "preferred", "tr", "en", "ja", "es", "fr", "de") } ?: "preferred"
    val safeTryBingeGroup: Boolean get() = safePrefs.tryBingeGroup
    val safeMpvCustomOptions: String get() = mpvCustomOptions.orEmpty()
    val safeAnimeUseMpv: Boolean get() = animeUseMpv ?: false
    val safeHeroFeedToggles: Set<String> get() = heroFeedToggles?.toSet() ?: defaultHeroFeedKeys
    val safeHomeFeedToggles: Set<String> get() = homeFeedToggles?.toSet() ?: defaultHomeFeedKeys
    val safeTopTenFeedToggles: Set<String> get() = topTenFeedToggles.orEmpty().toSet()
    val safeShowHeroSection: Boolean get() = safePrefs.showHeroSection
    val safeLibraryCollections: List<LibraryUserCollection> get() = libraryCollections ?: homeFeedSettings?.libraryCollections.orEmpty()

    val displayName: String get() = when {
        isGuest -> if (email.isBlank() || email == "guest") "Guest" else email
        email.contains("@") -> email.substringBefore("@")
        else -> email
    }
    val safeStreamingServerUrl: String get() = streamingServerUrl ?: Constants.LocalServer.BASE_URL
    val safeInstalledLocalAddons: List<String> get() = localAddons.orEmpty()
    val safeDisabledLocalAddonIds: Set<String> get() = disabledLocalAddons.orEmpty()
        .map(StremioAddonUrls::identity)
        .toSet()
    val safeLocalAddons: List<String> get() = safeInstalledLocalAddons
        .filterNot { StremioAddonUrls.identity(it) in safeDisabledLocalAddonIds }
    val safeTraktTokenExpiresAt: Long get() = safePrefs.traktTokenExpiresAt
    val safeTraktLastSyncAt: Long get() = safePrefs.traktLastSyncAt
    val safeTraktLastSyncedItems: Int get() = safePrefs.traktLastSyncedItems.toInt()
    val safeTraktLastContinueWatchingCount: Int get() = safePrefs.traktLastContinueWatchingCount.toInt()
    val safeTraktLastWatchlistCount: Int get() = safePrefs.traktLastWatchlistCount.toInt()
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
        return palette[Math.abs(h) % palette.size]
    }

    companion object {
        val defaultHeroFeedKeys = emptySet<String>()
        val defaultHomeFeedKeys = emptySet<String>()
    }
}

@Singleton
class ProfileManager @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson
) {
    private val prefs: SharedPreferences = context.getSharedPreferences("fluxa_profiles", Context.MODE_PRIVATE)
    private val profilesLock = Any()
    @Volatile private var cachedProfiles: List<UserProfile>? = null

    @WorkerThread
    fun saveProfile(profile: UserProfile) {
        saveProfileInternal(profile, mergeMirroredAddons = true)
    }

    @WorkerThread
    fun saveProfileReplacingLocalAddons(profile: UserProfile) {
        saveProfileInternal(profile, mergeMirroredAddons = false)
    }

    private fun saveProfileInternal(profile: UserProfile, mergeMirroredAddons: Boolean) {
        val sanitizedProfile = sanitizeProfile(profile, mergeMirroredAddons)
        // The whole read-modify-write must be one atomic unit, not just the cache swap —
        // otherwise two concurrent callers (e.g. a WorkManager worker and a coroutine
        // completion callback) can each read a stale list and clobber each other's write.
        synchronized(profilesLock) {
            val profiles = getProfiles().toMutableList()
            val existingIndex = profiles.indexOfFirst {
                (it.id == sanitizedProfile.id) || (it.email == sanitizedProfile.email && !it.isGuest)
            }
            if (existingIndex != -1) {
                profiles[existingIndex] = sanitizedProfile
            } else {
                profiles.add(sanitizedProfile)
            }
            val json = gson.toJson(profiles)
            prefs.edit()
                .putString("profiles_list", json)
                .putStringSet(localAddonsKey(sanitizedProfile), sanitizedProfile.safeInstalledLocalAddons.toSet())
                .apply()
            cachedProfiles = profiles
        }
    }

    fun getProfiles(): List<UserProfile> {
        cachedProfiles?.let { return it }
        return synchronized(profilesLock) {
            cachedProfiles ?: loadProfiles().also { cachedProfiles = it }
        }
    }

    @WorkerThread
    private fun loadProfiles(): List<UserProfile> {
        val json = prefs.getString("profiles_list", null)
        if (json == null) {
            return emptyList()
        }
        val type = object : TypeToken<List<UserProfile>>() {}.type
        val list: List<UserProfile> = gson.fromJson(json, type)
        return list
            .map { sanitizeProfile(it, mergeMirroredAddons = true) }
            .distinctBy { if (it.isGuest) it.id else it.email }
    }

    private fun sanitizeProfile(profile: UserProfile, mergeMirroredAddons: Boolean): UserProfile {
        val mirroredAddons = if (mergeMirroredAddons) {
            prefs.getStringSet(localAddonsKey(profile), emptySet()).orEmpty()
        } else {
            emptySet()
        }
        return FluxaCoreNative.sanitizeProfile(
            profile = profile,
            mirroredAddons = mirroredAddons,
            mergeMirroredAddons = mergeMirroredAddons,
            type = UserProfile::class.java
        ) ?: profile.withStructuredSettings()
    }

    private fun localAddonsKey(profile: UserProfile): String {
        return FluxaCoreNative.profileLocalAddonsKey(profile)
    }

    @WorkerThread
    fun deleteProfile(email: String) {
        synchronized(profilesLock) {
            val profiles = getProfiles().filterNot { it.email == email }
            val json = gson.toJson(profiles)
            prefs.edit().putString("profiles_list", json).apply()
            cachedProfiles = profiles
        }
    }

    @WorkerThread
    fun deleteProfileById(id: String) {
        synchronized(profilesLock) {
            val profiles = getProfiles().filterNot { it.id == id }
            val json = gson.toJson(profiles)
            prefs.edit().putString("profiles_list", json).apply()
            cachedProfiles = profiles
        }
    }

    fun getLastActiveProfileId(): String? {
        return prefs.getString("last_active_profile_id", null)
    }

    fun setLastActiveProfile(profile: UserProfile?) {
        val editor = prefs.edit()
        if (profile == null) {
            editor.remove("last_active_profile_id")
        } else {
            editor.putString("last_active_profile_id", profile.id)
        }
        editor.apply()
    }

    fun registerOnChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener) {
        prefs.registerOnSharedPreferenceChangeListener(listener)
    }

    fun unregisterOnChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener) {
        prefs.unregisterOnSharedPreferenceChangeListener(listener)
    }
}
