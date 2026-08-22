package com.fluxa.app.core.rust

import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Stream
import com.google.gson.JsonArray
import com.google.gson.JsonElement

internal fun FluxaAndroidHeadlessEnvironment.syncWatchlistProfile(effect: NativeHeadlessEffect) {
    val profileId = effect.payload.parseProfile(gson)?.id ?: effect.payload.stringOrNull("profileId")
    profileId?.let(watchlistManager::setActiveProfile)
}

internal fun JsonElement.asJsonObjectOrNull() = takeIf { it.isJsonObject }?.asJsonObject

internal fun com.google.gson.JsonObject.getAsJsonArrayOrNull(key: String): JsonArray? = get(key)?.takeIf { it.isJsonArray }?.asJsonArray

internal fun FluxaAndroidHeadlessEnvironment.isTmdbContentId(id: String): Boolean = id.startsWith("tmdb:", ignoreCase = true) || id.toIntOrNull() != null

internal suspend fun FluxaAndroidHeadlessEnvironment.loadCsNativeMetaDetail(id: String): MetaDetail? = cloudStreamRuntime.loadMetaDetail(id)

internal suspend fun FluxaAndroidHeadlessEnvironment.loadCsNativeStreams(id: String, directTimeoutMs: Long = 30_000L): List<Stream> = cloudStreamRuntime.loadStreams(id, directTimeoutMs)

internal fun FluxaAndroidHeadlessEnvironment.csQualityScore(quality: String): Int = cloudStreamRuntime.qualityScore(quality)
