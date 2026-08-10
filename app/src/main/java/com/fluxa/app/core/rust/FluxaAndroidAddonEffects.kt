@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.core.rust

import android.util.Log
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.AddonDescriptor
import kotlinx.coroutines.withTimeoutOrNull

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchAddonManifest(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val manifest = addonRepository.getAddonManifest(
        transportUrl = effect.payload.string("transportUrl"),
        forceRefresh = effect.payload.boolean("forceRefresh")
    )
    return ok(effect, manifest)
}

internal fun FluxaAndroidHeadlessEnvironment.fetchPluginManifest(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val manifestUrl = effect.payload.string("manifestUrl")
    val result = httpEffectExecutor.execute(pluginScraperHttpClient, manifestUrl)
    val body = result.body
    val statusCode = result.statusCode
    if (result.error != null || statusCode == null || statusCode !in 200..299 || body == null) {
        return error(effect, result.error ?: "http_${statusCode ?: 0}")
    }
    val manifest = FluxaCoreUniFfi.coreInvokeValue("pluginManifestParse", body)
    return ok(effect, mapOf("manifestUrl" to manifestUrl, "manifest" to manifest))
}

internal suspend fun FluxaAndroidHeadlessEnvironment.refreshInstalledAddons(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val profile = payload.profile() ?: return ok(effect, mapOf("addons" to emptyList<AddonDescriptor>()))
    val addons = withTimeoutOrNull(10_000L) {
        repository.getUserAddons(
            authKey = profile.authKey,
            localAddons = profile.safeLocalAddons,
            forceRefresh = payload.boolean("forceRefresh", true)
        )
    }.orEmpty()
    return ok(effect, mapOf("addons" to addons))
}

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchAddonResource(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val transportUrl = payload.string("transportUrl")
    val type = payload.string("contentType")
    val id = payload.string("id")
    val resource = payload.string("resource")
    Log.d("MetaFetch", "fetchAddonResource: resource=$resource type=$type id=${id.take(30)} url=$transportUrl")
    val value = when (resource) {
        "stream" -> addonRepository.getStreamsFromAddon(transportUrl, payload.string("addonName", ""), type, id)
        "catalog" -> addonRepository.getAddonCatalog(
            transportUrl = transportUrl,
            type = type,
            id = id,
            skip = payload.extraNumber("skip")?.toInt() ?: 0,
            genre = payload.extraString("genre"),
            search = payload.extraString("search")
        )
        "meta" -> addonRepository.getMetaDetailFromSpecificAddon(transportUrl, type, id)
        "subtitles" -> addonRepository.getSubtitlesFromAddon(
            transportUrl,
            type,
            id,
            payload.stringOrNull("extraArgs") ?: payload.extraString("extraArgs").orEmpty()
        )
        else -> emptyList<Any>()
    }
    return ok(effect, value)
}
