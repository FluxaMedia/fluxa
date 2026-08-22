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

val UserProfile.safeAmbientLight: Boolean get() = safePrefs.ambientLight
val UserProfile.safeForceSoftwareAudio: Boolean get() = safePrefs.forceSoftwareAudio
val UserProfile.safeCardLayout: String get() = safePrefs.cardLayout
val UserProfile.safeContinueWatchingArtwork: String get() = safePrefs.continueWatchingArtwork
val UserProfile.resolvedContinueWatchingLayout: String get() = safePrefs.resolvedContinueWatchingLayout
val UserProfile.safeTimezoneConversionEnabled: Boolean get() = safePrefs.timezoneConversionEnabled
val UserProfile.safeTorrentMaxConnections: Int get() = safePrefs.torrentMaxConnections.toInt()
val UserProfile.safeTorrentCachePreset: String get() = safePrefs.torrentCachePreset
val UserProfile.safeAppTheme: String get() = safePrefs.appTheme
val UserProfile.safeReduceMotion: Boolean get() = safePrefs.reduceMotion
val UserProfile.safePictureInPicture: Boolean get() = safePrefs.pictureInPicture
val UserProfile.safeDefaultQuality: String get() = safePrefs.defaultQuality
val UserProfile.safeMobileDataUsage: String get() = safePrefs.mobileDataUsage
val UserProfile.safeHdrPlayback: Boolean get() = safePrefs.hdrPlayback
val UserProfile.safeResumePlayback: Boolean get() = safePrefs.resumePlayback
val UserProfile.safeStreamSourceRegexPattern: String get() = safePrefs.streamSourceRegexPattern
val UserProfile.safeTraktTokenExpiresAt: Long get() = safePrefs.traktTokenExpiresAt
val UserProfile.safeTraktLastSyncAt: Long get() = safePrefs.traktLastSyncAt
