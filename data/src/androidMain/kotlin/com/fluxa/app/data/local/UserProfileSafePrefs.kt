package com.fluxa.app.data.local

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.domain.discovery.StremioAddonUrls

// UserProfile can no longer hold this cache as an instance field now that it lives in
// shared commonMain. A structural (equals/hashCode) map cache is too slow here — this
// class has ~150 fields including nested lists, so hashing it on every property read is
// itself the bottleneck. Caching the single last instance by reference identity matches
// the actual access pattern (the same profile object gets read repeatedly between saves)
// without paying that cost.
private object SafePrefsCache {
    @Volatile private var lastProfile: UserProfile? = null
    @Volatile private var lastPrefs: com.fluxa.app.core.rust.models.NativeProfileSafePrefs? = null

    @Synchronized
    fun get(profile: UserProfile): com.fluxa.app.core.rust.models.NativeProfileSafePrefs {
        lastPrefs?.let { if (profile === lastProfile) return it }
        val prefs = FluxaCoreNative.profileSafePrefs(profile)
        lastProfile = profile
        lastPrefs = prefs
        return prefs
    }
}

private val UserProfile.safePrefs: com.fluxa.app.core.rust.models.NativeProfileSafePrefs
    get() = SafePrefsCache.get(this)

val UserProfile.safeLanguage: String get() = safePrefs.language
val UserProfile.safeSubtitleSizePercent: Float get() = safePrefs.subtitleSizePercent
val UserProfile.safeSubtitleSize: Float get() = safePrefs.subtitleSize
val UserProfile.safeSubtitleColor: Int get() = safePrefs.subtitleColor
val UserProfile.safeSubtitleBackgroundColor: Int get() = safePrefs.subtitleBackgroundColor
val UserProfile.safeSubtitleOutlineColor: Int get() = safePrefs.subtitleOutlineColor
val UserProfile.safeSubtitleTextOpacity: Float get() = safePrefs.subtitleTextOpacity
val UserProfile.safeSubtitleBackgroundOpacity: Float get() = safePrefs.subtitleBackgroundOpacity
val UserProfile.safeSubtitleOutlineOpacity: Float get() = safePrefs.subtitleOutlineOpacity
val UserProfile.safePreferredSubtitleLanguage: String get() = safePrefs.preferredSubtitleLanguage
val UserProfile.safePreferredAudioLanguage: String get() = safePrefs.preferredAudioLanguage
val UserProfile.safeSecondarySubtitleLanguage: String get() = safePrefs.secondarySubtitleLanguage
val UserProfile.safeSecondaryAudioLanguage: String get() = safePrefs.secondaryAudioLanguage
val UserProfile.safeAmbientLight: Boolean get() = safePrefs.ambientLight
val UserProfile.safeForceSoftwareAudio: Boolean get() = safePrefs.forceSoftwareAudio
val UserProfile.safePreferredPlayer: String get() = safePrefs.preferredPlayer
val UserProfile.safeContinueWatchingSource: String get() = safePrefs.continueWatchingSource
val UserProfile.safeCardLayout: String get() = safePrefs.cardLayout
val UserProfile.safeContinueWatchingLayout: String get() = safePrefs.continueWatchingLayout
val UserProfile.safeContinueWatchingArtwork: String get() = safePrefs.continueWatchingArtwork
val UserProfile.safeContinueWatchingEnabled: Boolean get() = safePrefs.continueWatchingEnabled
val UserProfile.resolvedContinueWatchingLayout: String get() = safePrefs.resolvedContinueWatchingLayout
val UserProfile.safeSubtitleShadow: Boolean get() = safePrefs.subtitleShadow
val UserProfile.safeAutoEnableSubtitles: Boolean get() = safePrefs.autoEnableSubtitles
val UserProfile.safeAutoSkipIntro: Boolean get() = safePrefs.autoSkipIntro
val UserProfile.safeAutoPlayNextEpisode: Boolean get() = safePrefs.autoPlayNextEpisode
val UserProfile.safeNextEpisodeThresholdPercent: Float get() = safePrefs.nextEpisodeThresholdPercent
val UserProfile.safeWatchedThresholdPercent: Float get() = safePrefs.watchedThresholdPercent
val UserProfile.safeSeekForwardSeconds: Int get() = safePrefs.seekForwardSeconds.toInt()
val UserProfile.safeSeekBackwardSeconds: Int get() = safePrefs.seekBackwardSeconds.toInt()
val UserProfile.safePlayerBufferCacheMb: Int get() = FluxaCoreNative.safePlayerBufferCacheMb(playerBufferCacheMb)
val UserProfile.safePlayerForwardBufferSeconds: Int get() = safePrefs.playerForwardBufferSeconds.toInt()
val UserProfile.safePlayerBackBufferSeconds: Int get() = safePrefs.playerBackBufferSeconds.toInt()
val UserProfile.safeTimezoneConversionEnabled: Boolean get() = safePrefs.timezoneConversionEnabled
val UserProfile.safeTorrentWifiOnly: Boolean get() = safePrefs.torrentWifiOnly
val UserProfile.safeTorrentMaxConnections: Int get() = safePrefs.torrentMaxConnections.toInt()
val UserProfile.safeTorrentSpeedPreset: String get() = safePrefs.torrentSpeedPreset
val UserProfile.safeTorrentCachePreset: String get() = safePrefs.torrentCachePreset
val UserProfile.safeAppTheme: String get() = safePrefs.appTheme
val UserProfile.safeAccentColorArgb: Int get() = safePrefs.accentColorArgb
val UserProfile.safeCardCornerPreset: String get() = safePrefs.cardCornerPreset
val UserProfile.safeInterfaceDensity: String get() = safePrefs.interfaceDensity
val UserProfile.safeAmoledMode: Boolean get() = safePrefs.amoledMode
val UserProfile.safePosterWidthPreset: String get() = safePrefs.posterWidthPreset
val UserProfile.safePosterLandscapeMode: Boolean get() = safePrefs.posterLandscapeMode
val UserProfile.safePosterHideTitles: Boolean get() = safePrefs.posterHideTitles
val UserProfile.safeAnimationsEnabled: Boolean get() = safePrefs.animationsEnabled
val UserProfile.safeReduceMotion: Boolean get() = safePrefs.reduceMotion
val UserProfile.safeStartPage: String get() = safePrefs.startPage
val UserProfile.safeNotificationsEnabled: Boolean get() = safePrefs.notificationsEnabled
val UserProfile.safeAlertNewEpisodes: Boolean get() = safePrefs.alertNewEpisodes
val UserProfile.safeAutomaticUpdates: Boolean get() = safePrefs.automaticUpdates
val UserProfile.safeBackgroundPlayback: Boolean get() = safePrefs.backgroundPlayback
val UserProfile.safePictureInPicture: Boolean get() = safePrefs.pictureInPicture
val UserProfile.safePlaybackSpeed: Float get() = safePrefs.playbackSpeed
val UserProfile.safeHoldToSpeedEnabled: Boolean get() = safePrefs.holdToSpeedEnabled
val UserProfile.safeHoldSpeed: Float get() = safePrefs.holdSpeed
val UserProfile.safeTunneledPlayback: Boolean get() = safePrefs.tunneledPlayback
val UserProfile.safeUseIntroDb: Boolean get() = safePrefs.useIntroDb
val UserProfile.safeUseAniSkip: Boolean get() = safePrefs.useAniSkip
val UserProfile.safeDefaultQuality: String get() = safePrefs.defaultQuality
val UserProfile.safeMobileDataUsage: String get() = safePrefs.mobileDataUsage
val UserProfile.safeHdrPlayback: Boolean get() = safePrefs.hdrPlayback
val UserProfile.safeResumePlayback: Boolean get() = safePrefs.resumePlayback
val UserProfile.safeAutoplayMode: String get() = safePrefs.autoplayMode
val UserProfile.safeStreamSourceSelectionMode: String get() = FluxaCoreNative.safeStreamSourceSelectionMode(streamSourceSelectionMode)
val UserProfile.safeStreamSourceRegexPattern: String get() = safePrefs.streamSourceRegexPattern
val UserProfile.safeDownloadSourceSelectionMode: String get() = FluxaCoreNative.safeStreamSourceSelectionMode(downloadSourceSelectionMode)
val UserProfile.safeTryBingeGroup: Boolean get() = safePrefs.tryBingeGroup
val UserProfile.safeShowHeroSection: Boolean get() = safePrefs.showHeroSection
val UserProfile.safeTraktTokenExpiresAt: Long get() = safePrefs.traktTokenExpiresAt
val UserProfile.safeTraktLastSyncAt: Long get() = safePrefs.traktLastSyncAt
val UserProfile.safeTraktLastSyncedItems: Int get() = safePrefs.traktLastSyncedItems.toInt()
val UserProfile.safeTraktLastContinueWatchingCount: Int get() = safePrefs.traktLastContinueWatchingCount.toInt()
val UserProfile.safeTraktLastWatchlistCount: Int get() = safePrefs.traktLastWatchlistCount.toInt()

val UserProfile.safeDisabledLocalAddonIds: Set<String> get() = disabledLocalAddons.orEmpty()
    .map(StremioAddonUrls::identity)
    .toSet()
val UserProfile.safeLocalAddons: List<String> get() = safeInstalledLocalAddons
    .filterNot { StremioAddonUrls.identity(it) in safeDisabledLocalAddonIds }
