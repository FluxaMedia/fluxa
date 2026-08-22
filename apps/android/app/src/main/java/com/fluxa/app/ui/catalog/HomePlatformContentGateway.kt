package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.DetailTrailer
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.data.repository.CloudStreamCatalogClient
import com.fluxa.app.data.repository.SearchResultRow
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import com.lagradost.cloudstream3.MainAPI
import javax.inject.Inject
import javax.inject.Singleton
import kotlinx.coroutines.flow.StateFlow

@Singleton
class HomePlatformContentGateway @Inject constructor(
    private val repository: StremioRepository,
    private val addonRepository: AddonRepository,
    private val pluginManager: PluginManager,
    private val cloudStreamCatalogClient: CloudStreamCatalogClient
) {
    val loadedApis: StateFlow<List<MainAPI>> get() = pluginManager.loadedApis
    val installedPlugins: StateFlow<List<InstalledPlugin>> get() = pluginManager.installedPlugins

    suspend fun addonMetaDetail(type: String, id: String, authKey: String, localAddons: List<String>): MetaDetail? =
        addonRepository.getAddonMetaDetail(type, id, authKey, localAddons)

    suspend fun cloudFeedItems(feedKey: String): List<Meta> =
        cloudStreamCatalogClient.fetchFeedItems(loadedApis.value, feedKey)

    suspend fun cloudHomeCategories(
        apis: List<MainAPI>,
        iconsByApiName: Map<String, String>,
        enabledFeedKeys: Set<String>?
    ): List<HomeCategory> = cloudStreamCatalogClient.fetchHomeCatalogCategories(apis, iconsByApiName, enabledFeedKeys)

    suspend fun trailers(type: String, id: String, language: String, apiKey: String): List<DetailTrailer> =
        repository.getTmdbTrailers(type, id, language, apiKey)

    suspend fun searchRows(
        query: String,
        language: String,
        authKey: String,
        localAddons: List<String>
    ): List<SearchResultRow> = addonRepository.searchRows(query, language, authKey, localAddons) +
        cloudStreamCatalogClient.searchRows(loadedApis.value, query)

    suspend fun addonCatalog(
        transportUrl: String,
        type: String,
        catalogId: String,
        skip: Int,
        genre: String?
    ): List<Meta> = addonRepository.getAddonCatalog(transportUrl, type, catalogId, skip, genre)
}
