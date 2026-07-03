package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.AddonCatalog
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.domain.discovery.hasRequiredCatalogExtraExcept
import com.fluxa.app.domain.discovery.supportsCatalogExtra
import com.fluxa.app.common.AppStrings
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import java.util.Locale

private const val MAX_CONCURRENT_ADDON_SEARCH_REQUESTS = 6

internal class AddonCatalogSearch(
    private val addonRepository: AddonRepository
) {
    suspend fun searchRows(
        query: String,
        language: String,
        authKey: String,
        localAddons: List<String>
    ): List<SearchResultRow> = supervisorScope {
        val addons = addonRepository.getUserAddons(authKey, localAddons)
        val searchSemaphore = Semaphore(MAX_CONCURRENT_ADDON_SEARCH_REQUESTS)
        addons.flatMap { addon ->
            val addonName = addon.manifest.name.takeIf { it.isNotBlank() }
                ?: addon.manifest.id.takeIf { it.isNotBlank() }
                ?: AppStrings.t(language, "auto.metadata")
            addon.manifest.catalogs.orEmpty()
                .filter { catalog ->
                    val type = catalog.type
                    (type == "movie" || type == "series") &&
                        catalog.supportsCatalogExtra("search") &&
                        !catalog.hasRequiredCatalogExtraExcept(setOf("search"))
                }
                .map { catalog ->
                    async {
                        searchSemaphore.withPermit {
                            runCatching {
                                val type = catalog.type ?: return@runCatching null
                                val catalogId = catalog.id ?: return@runCatching null
                                val rawItems = addonRepository.getAddonCatalog(
                                    transportUrl = addon.transportUrl,
                                    type = type,
                                    id = catalogId,
                                    search = query
                                )
                                val items = rawItems
                                    .map { meta -> meta.copy(type = type) }
                                    .distinctBy { it.id }
                                    .take(24)
                                if (items.isEmpty()) {
                                    null
                                } else {
                                    SearchResultRow(
                                        title = "${searchCatalogLabel(catalog, language)} - ${searchTypeLabel(type, language)}: $addonName",
                                        items = items,
                                        id = "addon:${addon.transportUrl.hashCode()}:$type:$catalogId",
                                        type = type,
                                        sourceAddonTransportUrl = addon.transportUrl,
                                        sourceAddonCatalogType = type
                                    )
                                }
                            }.getOrNull()
                        }
                    }
                }
        }.awaitAll().filterNotNull()
    }

    private fun searchTypeLabel(type: String, language: String): String {
        return when (type.lowercase(Locale.ROOT)) {
            "movie" -> AppStrings.t(language, "auto.movies")
            "series", "tv", "anime" -> AppStrings.t(language, "auto.series")
            else -> type.replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
        }
    }

    private fun searchCatalogLabel(catalog: AddonCatalog, language: String): String {
        return catalog.name?.takeIf { it.isNotBlank() }
            ?: catalog.id?.split('_', '-', ' ')?.filter { it.isNotBlank() }?.joinToString(" ") { word ->
                word.replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
            }
            ?: AppStrings.t(language, "auto.search")
    }
}
