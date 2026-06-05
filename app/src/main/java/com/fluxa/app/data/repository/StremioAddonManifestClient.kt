package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.*
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.domain.discovery.StremioAddonUrls
import com.fluxa.app.ui.catalog.AppStrings
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import java.net.URLEncoder

import javax.inject.Inject
import javax.inject.Named

class StremioAddonManifestClient @Inject constructor(
    internal val cache: RepositoryMemoryCache,
    internal val persistentCache: AddonPersistentCache,
    @param:Named("StremioClient") private val manifestClient: OkHttpClient
) {
    private val unknownName: (String?) -> String = { AppStrings.t(it, "auto.unknown") }
    private inline fun <reified T> getCached(key: String): T? = cache.get(key)
    private fun putCache(key: String, value: Any) = cache.put(key, value)

    internal fun normalizeAddonTransportUrl(rawUrl: String): String {
        return StremioAddonUrls.normalizeManifestUrl(rawUrl)
    }

    internal fun addonBaseUrl(transportUrl: String): String {
        return StremioAddonUrls.baseUrl(transportUrl)
    }

    internal fun buildAddonResourceUrl(
        transportUrl: String,
        resource: String,
        type: String,
        id: String,
        extraArgs: Map<String, String?> = emptyMap()
    ): String {
        return FluxaCoreNative.buildResourceUrl(transportUrl, resource, type, id, extraArgs)
    }

    internal fun encodePathSegment(value: String): String {
        return URLEncoder.encode(value, Charsets.UTF_8.name()).replace("+", "%20")
    }

    internal fun resolveAddonAssetUrl(assetUrl: String?, manifestUrl: String): String? {
        val descriptor = AddonDescriptor(
            manifest = AddonManifest(
                id = "",
                name = unknownName(null),
                resources = emptyList(),
                types = emptyList(),
                catalogs = emptyList(),
                logo = assetUrl
            ),
            transportUrl = manifestUrl
        )
        return FluxaCoreNative.resolveManifestAssets(descriptor)?.manifest?.logo
    }

    suspend fun getAddonManifest(
        transportUrl: String,
        forceRefresh: Boolean = false
    ): AddonDescriptor? = withContext(Dispatchers.IO) {
        val fetchPlan = FluxaCoreNative.manifestFetchPlan(transportUrl) ?: return@withContext null
        val cacheKey = fetchPlan.cacheKey
        val memoryHit = if (!forceRefresh) getCached<AddonDescriptor>(cacheKey) else null
        val persistentHit = if (!forceRefresh && memoryHit == null) persistentCache.getManifest(cacheKey) else null
        val decision = FluxaCoreNative.manifestFetchDecision(
            forceRefresh = forceRefresh,
            memoryHit = memoryHit != null,
            persistentHit = persistentHit != null
        )
        when (decision.phase) {
            "memory" -> memoryHit?.let { return@withContext it.withResolvedManifestAssets() }
            "persistent" -> persistentHit?.let {
                putCache(cacheKey, it)
                return@withContext it.withResolvedManifestAssets()
            }
        }
        fetchPlan.candidateUrls.forEach { candidateUrl ->
            try {
                val httpRequest = Request.Builder().url(candidateUrl).build()
                val httpResponse = manifestClient.newCall(httpRequest).execute()
                val statusCode = httpResponse.code
                val body = httpResponse.body.string().also { httpResponse.close() }
                if (statusCode !in 200..299) return@forEach
                val descriptor = parseAddonManifest(body, candidateUrl) ?: return@forEach
                putCache(cacheKey, descriptor)
                persistentCache.putManifest(cacheKey, descriptor)
                return@withContext descriptor
            } catch (_: Exception) {
            }
        }
        if (decision.allowStaleFallback) persistentCache.getManifest(cacheKey)?.let {
            putCache(cacheKey, it)
            return@withContext it.withResolvedManifestAssets()
        }
        null
    }

    internal fun parseAddonManifest(body: String, transportUrl: String): AddonDescriptor? {
        return FluxaCoreNative.parseManifestJson(body, transportUrl, unknownName(null))
    }

    internal fun AddonDescriptor.withResolvedManifestAssets(): AddonDescriptor {
        return FluxaCoreNative.resolveManifestAssets(this) ?: this
    }

    internal fun AddonDescriptor.mergeLiveManifest(live: AddonDescriptor?): AddonDescriptor {
        return FluxaCoreNative.mergeLiveManifest(this, live, unknownName(null)) ?: this.withResolvedManifestAssets()
    }

}
