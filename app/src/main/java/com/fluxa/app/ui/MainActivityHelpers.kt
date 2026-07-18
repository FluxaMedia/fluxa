package com.fluxa.app.ui

import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.shared.FluxaDestination
import com.fluxa.app.ui.catalog.DeviceType

internal fun MetaDetail.toMeta() = Meta(id, name, type, poster, background, logo, description, imdbRating, releaseInfo = releaseInfo, originalLanguage = originalLanguage, videos = videos, trailers = trailers)

internal fun normalizeAddonUrlForProfile(rawUrl: String): String {
    return FluxaCoreNative.normalizeManifestUrl(rawUrl)
}

internal fun addonUrlIdentity(rawUrl: String): String {
    return FluxaCoreNative.identity(rawUrl)
}

internal fun UserProfile?.requiresHomeReload(next: UserProfile): Boolean {
    val previous = this ?: return true
    return previous.safeLanguage != next.safeLanguage ||
        previous.authKey != next.authKey ||
        previous.safeLocalAddons != next.safeLocalAddons ||
        previous.safeDisabledLocalAddonIds != next.safeDisabledLocalAddonIds ||
        previous.heroFeedToggles != next.heroFeedToggles ||
        previous.heroFeedOrder != next.heroFeedOrder ||
        previous.homeFeedToggles != next.homeFeedToggles ||
        previous.homeFeedOrder != next.homeFeedOrder ||
        previous.safeContinueWatchingEnabled != next.safeContinueWatchingEnabled ||
        previous.safeContinueWatchingSource != next.safeContinueWatchingSource ||
        previous.safeShowHeroSection != next.safeShowHeroSection
}

internal fun initialProfileForDevice(profileManager: ProfileManager, deviceType: DeviceType): UserProfile? {
    if (deviceType != DeviceType.Mobile) return null
    val profiles = profileManager.getProfiles()
    return profileManager.getLastActiveProfileId()
        ?.let { id -> profiles.firstOrNull { it.id == id } }
        ?: profiles.firstOrNull()
}

internal fun initialDestinationForProfile(profile: UserProfile?): FluxaDestination {
    return profile?.let {
        when (it.safeStartPage) {
            "discover" -> FluxaDestination.Discover
            "library" -> FluxaDestination.Library
            else -> FluxaDestination.Home
        }
    } ?: FluxaDestination.ProfileList
}

internal fun generateOAuthCodeVerifier(): String {
    val bytes = ByteArray(32)
    java.security.SecureRandom().nextBytes(bytes)
    return android.util.Base64.encodeToString(
        bytes,
        android.util.Base64.URL_SAFE or android.util.Base64.NO_WRAP or android.util.Base64.NO_PADDING
    )
}

internal data class TraktDeviceAuthUiState(
    val userCode: String,
    val verificationUrl: String,
    val isWaiting: Boolean = true
)
