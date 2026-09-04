package com.fluxa.app.ui

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.ui.catalog.HomeViewModel

internal fun installLocalAddonForProfile(
    activeProfile: UserProfile?,
    addonUrl: String,
    profileManager: ProfileManager,
    homeViewModel: HomeViewModel,
    onProfileChanged: (UserProfile) -> Unit
) {
    activeProfile?.let { profile ->
        val updated = FluxaCoreNative.addonProfileMutationPlan(
            profile = profile,
            command = "install",
            addonKey = addonUrl,
            type = UserProfile::class.java
        ) ?: return
        onProfileChanged(updated)
        profileManager.saveProfileReplacingLocalAddons(updated)
        profileManager.setLastActiveProfile(updated)
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
        val updated = FluxaCoreNative.addonProfileMutationPlan(
            profile = profile,
            command = "remove",
            addonKey = addonUrl,
            type = UserProfile::class.java
        ) ?: return
        onProfileChanged(updated)
        profileManager.saveProfileReplacingLocalAddons(updated)
        profileManager.setLastActiveProfile(updated)
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
        val updated = FluxaCoreNative.addonProfileMutationPlan(
            profile = profile,
            command = "move",
            addonKey = addonUrl,
            direction = direction,
            type = UserProfile::class.java
        ) ?: return
        if (updated == profile) return
        onProfileChanged(updated)
        profileManager.saveProfileReplacingLocalAddons(updated)
        profileManager.setLastActiveProfile(updated)
        homeViewModel.loadInitialData(updated, force = true)
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
        val updated = FluxaCoreNative.addonProfileMutationPlan(
            profile = profile,
            command = if (enabled) "enable" else "disable",
            addonKey = addonUrl,
            type = UserProfile::class.java
        ) ?: return
        onProfileChanged(updated)
        profileManager.saveProfileReplacingLocalAddons(updated)
        profileManager.setLastActiveProfile(updated)
        homeViewModel.loadInitialData(updated, force = true)
    }
}
