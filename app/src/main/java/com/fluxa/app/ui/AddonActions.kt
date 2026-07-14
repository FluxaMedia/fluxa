package com.fluxa.app.ui

import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.ui.catalog.HomeViewModel
import java.util.Collections

internal fun installLocalAddonForProfile(
    activeProfile: UserProfile?,
    addonUrl: String,
    profileManager: ProfileManager,
    homeViewModel: HomeViewModel,
    onProfileChanged: (UserProfile) -> Unit
) {
    activeProfile?.let { profile ->
        val normalizedUrl = normalizeAddonUrlForProfile(addonUrl)
        val updated = profile.copy(
            localAddons = (profile.safeInstalledLocalAddons.map(::normalizeAddonUrlForProfile) + normalizedUrl)
                .distinctBy(::addonUrlIdentity),
            disabledLocalAddons = profile.disabledLocalAddons.orEmpty()
                .filterNot { addonUrlIdentity(it) == addonUrlIdentity(normalizedUrl) }
        )
        onProfileChanged(updated)
        profileManager.saveProfileReplacingLocalAddons(updated)
        profileManager.setLastActiveProfile(updated)
        homeViewModel.pushNuvioAddons(updated)
        homeViewModel.loadInitialData(updated, force = true)
    }
}

internal fun removeLocalAddonForProfile(
    activeProfile: UserProfile?,
    addonUrl: String,
    profileManager: ProfileManager,
    homeViewModel: HomeViewModel,
    onProfileChanged: (UserProfile) -> Unit
) {
    activeProfile?.let { profile ->
        val normalizedUrl = normalizeAddonUrlForProfile(addonUrl)
        val updated = profile.copy(
            localAddons = profile.safeInstalledLocalAddons
                .map(::normalizeAddonUrlForProfile)
                .filterNot { addonUrlIdentity(it) == addonUrlIdentity(normalizedUrl) },
            disabledLocalAddons = profile.disabledLocalAddons.orEmpty()
                .filterNot { addonUrlIdentity(it) == addonUrlIdentity(normalizedUrl) }
        )
        onProfileChanged(updated)
        profileManager.saveProfileReplacingLocalAddons(updated)
        profileManager.setLastActiveProfile(updated)
        homeViewModel.pushNuvioAddons(updated)
        homeViewModel.loadInitialData(updated, force = true)
    }
}

internal fun moveLocalAddonForProfile(
    activeProfile: UserProfile?,
    addonUrl: String,
    direction: Int,
    profileManager: ProfileManager,
    homeViewModel: HomeViewModel,
    onProfileChanged: (UserProfile) -> Unit
) {
    activeProfile?.let { profile ->
        val normalizedUrl = normalizeAddonUrlForProfile(addonUrl)
        val addons = profile.safeInstalledLocalAddons.map(::normalizeAddonUrlForProfile).toMutableList()
        val from = addons.indexOfFirst { addonUrlIdentity(it) == addonUrlIdentity(normalizedUrl) }
        val to = (from + direction).coerceIn(0, addons.lastIndex)
        if (from >= 0 && from != to) {
            Collections.swap(addons, from, to)
            val updated = profile.copy(localAddons = addons)
            onProfileChanged(updated)
            profileManager.saveProfileReplacingLocalAddons(updated)
            profileManager.setLastActiveProfile(updated)
            homeViewModel.pushNuvioAddons(updated)
            homeViewModel.loadInitialData(updated, force = true)
        }
    }
}

internal fun setLocalAddonEnabledForProfile(
    activeProfile: UserProfile?,
    addonUrl: String,
    enabled: Boolean,
    profileManager: ProfileManager,
    homeViewModel: HomeViewModel,
    onProfileChanged: (UserProfile) -> Unit
) {
    activeProfile?.let { profile ->
        val normalizedUrl = normalizeAddonUrlForProfile(addonUrl)
        val addonId = addonUrlIdentity(normalizedUrl)
        val disabled = profile.disabledLocalAddons.orEmpty()
            .map(::normalizeAddonUrlForProfile)
        val updatedDisabled = if (enabled) {
            disabled.filterNot { addonUrlIdentity(it) == addonId }
        } else {
            (disabled + normalizedUrl).distinctBy(::addonUrlIdentity)
        }
        val updated = profile.copy(disabledLocalAddons = updatedDisabled)
        onProfileChanged(updated)
        profileManager.saveProfileReplacingLocalAddons(updated)
        profileManager.setLastActiveProfile(updated)
        homeViewModel.pushNuvioAddons(updated)
        homeViewModel.loadInitialData(updated, force = true)
    }
}
