package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.NuvioProfileDto
import com.fluxa.app.data.remote.NuvioService
import com.google.gson.JsonArray
import com.google.gson.JsonObject

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
    val profileValues = JsonArray().apply {
        profiles.forEach { profile ->
            add(JsonObject().apply {
                addProperty("profile_index", profile.profileIndex)
                addProperty("uses_primary_addons", profile.usesPrimaryAddons)
                addProperty("uses_primary_plugins", profile.usesPrimaryPlugins)
            })
        }
    }
    val scopes = NuvioCoreBridge.effectiveProfileScopes(profileIndex, profileValues)
    return NuvioEffectiveProfileScopes(
        addons = scopes.get("addons").asInt,
        plugins = scopes.get("plugins").asInt,
    )
}
