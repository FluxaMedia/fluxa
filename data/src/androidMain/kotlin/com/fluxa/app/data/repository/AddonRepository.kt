package com.fluxa.app.data.repository

import android.util.Log
import com.fluxa.app.data.remote.*
import com.fluxa.app.domain.discovery.supportsStremioResource
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import javax.inject.Inject
import javax.inject.Singleton

private const val ADDON_META_TIMEOUT_MS = 3000L

@Singleton
class AddonRepository @Inject constructor(
    private val addonManifestClient: StremioAddonManifestClient,
    private val addonResourceClient: StremioAddonResourceClient
) {
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
        val allAddons = getUserAddons(authKey, localAddons)
        val addons = allAddons.filter { it.supportsStremioResource("meta") }
        Log.d("MetaFetch", "getAddonMetaDetail type=$type id=${id.take(30)}: total=${allAddons.size} supported=${addons.size} names=${addons.map { it.manifest.name }}")
        if (addons.isEmpty()) return null

        val results = supervisorScope {
            addons.map { addon ->
                async {
                    withTimeoutOrNull(ADDON_META_TIMEOUT_MS) {
                        addonResourceClient.getMetaDetailFromAddonResult(addon.transportUrl, type, id)
                    } ?: AddonResourceResult.NetworkError(
                        url = addon.transportUrl,
                        cause = null
                    )
                }
            }.awaitAll()
        }

        results.forEach { result ->
            when (result) {
                is AddonResourceResult.Success -> return result.value
                is AddonResourceResult.Empty,
                is AddonResourceResult.AddonUnsupported -> Unit
                is AddonResourceResult.NetworkError -> Log.w(
                    "AddonRepository",
                    "Meta request failed: ${result.url} status=${result.statusCode}",
                    result.cause
                )
                is AddonResourceResult.ParseError -> Log.w(
                    "AddonRepository",
                    "Meta parse failed: ${result.url}",
                    result.cause
                )
            }
        }
        return null
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
            Log.d("MetaFetch", "getMetaDetailFromSpecificAddon: url=$transportUrl type=$candidateType id=${id.take(30)}")
            when (val result = addonResourceClient.getMetaDetailFromAddonResult(transportUrl, candidateType, id)) {
                is AddonResourceResult.Success -> {
                    Log.d("MetaFetch", "getMetaDetailFromSpecificAddon SUCCESS: type=$candidateType name=${result.value.name} videos=${result.value.videos?.size}")
                    return result.value
                }
                is AddonResourceResult.Empty -> Log.d("MetaFetch", "getMetaDetailFromSpecificAddon EMPTY: ${result.url}")
                is AddonResourceResult.AddonUnsupported -> Log.d("MetaFetch", "getMetaDetailFromSpecificAddon UNSUPPORTED: addon=${result.addonName} type=${result.type}")
                is AddonResourceResult.NetworkError -> Log.w("MetaFetch", "getMetaDetailFromSpecificAddon NETWORK_ERROR: ${result.url} status=${result.statusCode}", result.cause)
                is AddonResourceResult.ParseError -> Log.w("MetaFetch", "getMetaDetailFromSpecificAddon PARSE_ERROR: ${result.url}", result.cause)
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
