package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.NuvioProfileDto
import com.fluxa.app.data.remote.NuvioService

internal data class NuvioEffectiveProfileScopes(
    val addons: Int,
    val plugins: Int,
)

/** Applies Nuvio's `uses_primary_addons/plugins` inheritance contract. */
internal suspend fun NuvioService.resolveEffectiveProfileScopes(
    authorization: String,
    profileIndex: Int,
    knownProfiles: List<NuvioProfileDto>? = null,
): NuvioEffectiveProfileScopes {
    val profiles = knownProfiles ?: runCatching {
        pullProfiles(authorization).takeIf { it.isSuccessful }?.body().orEmpty()
    }.getOrDefault(emptyList())
    val target = profiles.firstOrNull { it.profileIndex == profileIndex }
    return NuvioEffectiveProfileScopes(
        addons = if (profileIndex != 1 && target?.usesPrimaryAddons == true) 1 else profileIndex,
        plugins = if (profileIndex != 1 && target?.usesPrimaryPlugins == true) 1 else profileIndex,
    )
}
