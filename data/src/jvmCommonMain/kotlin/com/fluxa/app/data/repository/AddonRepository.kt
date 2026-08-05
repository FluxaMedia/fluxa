package com.fluxa.app.data.repository

import com.fluxa.app.common.PlatformLog
import com.fluxa.app.data.remote.*
import com.fluxa.app.domain.discovery.supportsStremioResource
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Inject
import javax.inject.Singleton

private const val ADDON_META_TIMEOUT_MS = 3000L
private const val META_DETAIL_CACHE_TTL_MS = 5 * 60 * 1000L

@Singleton
class AddonRepository @Inject constructor(
    private val addonManifestClient: StremioAddonManifestClient,
    private val addonResourceClient: StremioAddonResourceClient
) {
    private val metaDetailRepositoryScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val metaDetailCache = ConcurrentHashMap<String, Pair<Long, MetaDetail?>>()
    private val metaDetailInFlight = ConcurrentHashMap<String, Deferred<MetaDetail?>>()

    suspend fun getAddonManifest(
        transportUrl: String,
        forceRefresh: Boolean = false
    ): AddonDescriptor? = addonManifestClient.getAddonManifest(transportUrl, forceRefresh)

    suspend fun getAddonMetaDetail(
        type: String,
        id: String,
        authKey: String,
        localAddons: List<String>? = emptyList()
    ): MetaDetail? {
        val cacheKey = "$type:$id"
        metaDetailCache[cacheKey]?.let { (cachedAt, value) ->
            if (System.currentTimeMillis() - cachedAt < META_DETAIL_CACHE_TTL_MS) return value
        }

        val deferred = metaDetailInFlight.computeIfAbsent(cacheKey) {
            metaDetailRepositoryScope.async {
                try {
                    fetchAddonMetaDetailUncached(type, id, authKey, localAddons)
                } finally {
                    metaDetailInFlight.remove(cacheKey)
                }
            }
        }
        val result = deferred.await()
        metaDetailCache[cacheKey] = System.currentTimeMillis() to result
        return result
    }

    private suspend fun fetchAddonMetaDetailUncached(
        type: String,
        id: String,
        authKey: String,
        localAddons: List<String>? = emptyList()
    ): MetaDetail? {
        val allAddons = getUserAddons(authKey, localAddons)
        val addons = allAddons.filter { it.supportsStremioResource("meta") }
        PlatformLog.d("MetaFetch", "getAddonMetaDetail type=$type id=${id.take(30)}: total=${allAddons.size} supported=${addons.size} names=${addons.map { it.manifest.name }}")
        if (addons.isEmpty()) return null

        return coroutineScope {
            fun launchLookup(transportUrl: String) = async {
                withTimeoutOrNull(ADDON_META_TIMEOUT_MS) {
                    addonResourceClient.getMetaDetailFromAddonResult(transportUrl, type, id)
                } ?: AddonResourceResult.NetworkError(url = transportUrl, cause = null)
            }

            val results = addons.map { addon -> launchLookup(addon.transportUrl) }.awaitAll()
            results.forEach { result ->
                when (result) {
                    is AddonResourceResult.Success -> return@coroutineScope result.value
                    is AddonResourceResult.Empty,
                    is AddonResourceResult.AddonUnsupported -> Unit
                    is AddonResourceResult.NetworkError -> PlatformLog.w(
                        "AddonRepository",
                        "Meta request failed: ${result.url} status=${result.statusCode}",
                        result.cause
                    )
                    is AddonResourceResult.ParseError -> PlatformLog.w(
                        "AddonRepository",
                        "Meta parse failed: ${result.url}",
                        result.cause
                    )
                }
            }
            null
        }
    }

    suspend fun getMetaDetailFromSpecificAddon(
        transportUrl: String,
        type: String,
        id: String,
        alternateTypes: List<String> = emptyList()
    ): MetaDetail? {
        val types = (listOf(type) + alternateTypes)
            .map { it.trim() }
            .filter { it.isNotBlank() }
            .distinct()
        for (candidateType in types) {
            PlatformLog.d("MetaFetch", "getMetaDetailFromSpecificAddon: url=$transportUrl type=$candidateType id=${id.take(30)}")
            when (val result = addonResourceClient.getMetaDetailFromAddonResult(transportUrl, candidateType, id)) {
                is AddonResourceResult.Success -> {
                    PlatformLog.d("MetaFetch", "getMetaDetailFromSpecificAddon SUCCESS: type=$candidateType name=${result.value.name} videos=${result.value.videos?.size}")
                    return result.value
                }
                is AddonResourceResult.Empty -> PlatformLog.d("MetaFetch", "getMetaDetailFromSpecificAddon EMPTY: ${result.url}")
                is AddonResourceResult.AddonUnsupported -> PlatformLog.d("MetaFetch", "getMetaDetailFromSpecificAddon UNSUPPORTED: addon=${result.addonName} type=${result.type}")
                is AddonResourceResult.NetworkError -> PlatformLog.w("MetaFetch", "getMetaDetailFromSpecificAddon NETWORK_ERROR: ${result.url} status=${result.statusCode}", result.cause)
                is AddonResourceResult.ParseError -> PlatformLog.w("MetaFetch", "getMetaDetailFromSpecificAddon PARSE_ERROR: ${result.url}", result.cause)
            }
        }
        return null
    }

    suspend fun getUserAddons(
        authKey: String,
        localAddons: List<String>? = emptyList(),
        forceRefresh: Boolean = false
    ): List<AddonDescriptor> = addonResourceClient.getUserAddons(authKey, localAddons, forceRefresh)

    suspend fun getSubtitlesFromAddon(baseUrl: String, type: String, id: String, extra: String = ""): List<SubtitleData> =
        addonResourceClient.getSubtitlesFromAddon(baseUrl, type, id, extra)

    suspend fun getStreamsFromAddon(addonTransportUrl: String, addonName: String, type: String, id: String): List<Stream> =
        addonResourceClient.getStreamsFromAddon(addonTransportUrl, addonName, type, id)

    suspend fun getAddonCatalog(transportUrl: String, type: String, id: String, skip: Int = 0, genre: String? = null, search: String? = null): List<Meta> =
        addonResourceClient.getAddonCatalog(transportUrl, type, id, skip, genre, search)

    suspend fun searchRows(
        query: String,
        language: String,
        authKey: String,
        localAddons: List<String>
    ) = AddonCatalogSearch(this).searchRows(query, language, authKey, localAddons)
}
