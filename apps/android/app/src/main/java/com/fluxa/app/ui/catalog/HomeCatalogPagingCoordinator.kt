package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.NativeHeadlessEngineResult
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit

internal class HomeCatalogPagingCoordinator(
    private val scope: CoroutineScope,
    private val platformContentGateway: HomePlatformContentGateway,
    private val activeProfile: () -> UserProfile?,
    private val categories: () -> List<HomeCategory>,
    private val setCategories: (List<HomeCategory>) -> Unit,
    private val folderCategories: () -> Map<String, HomeCategory>,
    private val setFolderCategories: (Map<String, HomeCategory>) -> Unit,
    private val normalizeItems: suspend (List<Meta>, String, String, String?) -> List<Meta>,
    private val dispatch: suspend (Any) -> NativeHeadlessEngineResult,
    private val decodeItems: suspend (Any?) -> List<Meta>
) {
    private val inFlight = mutableSetOf<String>()

    fun loadMore(categoryId: String) {
        val category = categories().firstOrNull { it.id == categoryId } ?: folderCategories()[categoryId] ?: return
        if (!category.canLoadMore || !inFlight.add(categoryId)) return
        scope.launch {
            try {
                val catalogSources = category.catalogSources.orEmpty()
                val remoteSources = category.remoteSources.orEmpty()
                if (catalogSources.isNotEmpty() || remoteSources.isNotEmpty()) {
                    loadCollectionPage(category, catalogSources, remoteSources)
                } else {
                    loadCatalogPage(category)
                }
            } finally {
                if (category.catalogSources.orEmpty().isNotEmpty() || category.remoteSources.orEmpty().isNotEmpty()) {
                    update(categoryId) { it.copy(folderSourcesLoading = false) }
                }
                inFlight.remove(categoryId)
            }
        }
    }

    private suspend fun loadCollectionPage(
        category: HomeCategory,
        catalogSources: List<HomeCatalogSource>,
        remoteSources: List<LibraryRemoteSource>
    ) {
        update(category.id) { it.copy(folderSourcesLoading = true) }
        val nextSkip = if (category.items.isEmpty()) 0 else category.skip + 20
        val language = activeProfile()?.safeLanguage ?: "en"
        val sourceResults = Channel<Pair<HomeCatalogSource, List<Meta>>>(catalogSources.size)
        coroutineScope {
            val semaphore = Semaphore(4)
            catalogSources.forEach { source ->
                launch(Dispatchers.IO) {
                    semaphore.withPermit {
                        val items = platformContentGateway.addonCatalog(
                            source.transportUrl,
                            source.type,
                            source.catalogId,
                            nextSkip,
                            source.genre
                        )
                        sourceResults.send(source to normalizeItems(items, source.catalogId, language, source.genre))
                    }
                }
            }
            repeat(catalogSources.size) {
                val (source, items) = sourceResults.receive()
                val sourceMap = items.flatMap { listOf("${it.type}:${it.id}" to source, it.id to source) }.toMap()
                update(category.id) { existing ->
                    existing.copy(
                        items = (existing.items + items).distinctBy { "${it.type}:${it.id}" },
                        resultSources = existing.resultSources + sourceMap
                    )
                }
            }
        }
        val remoteItems = if (remoteSources.isEmpty()) emptyList() else fetchPage(
            category,
            skip = nextSkip,
            transportUrl = null,
            remoteSources = remoteSources
        )
        update(category.id) { existing ->
            existing.copy(
                items = if (remoteItems.isEmpty()) existing.items else (existing.items + remoteItems).distinctBy { "${it.type}:${it.id}" },
                skip = nextSkip,
                canLoadMore = existing.items.isNotEmpty() || remoteItems.isNotEmpty()
            )
        }
    }

    private suspend fun loadCatalogPage(category: HomeCategory) {
        val items = fetchPage(
            category,
            skip = category.skip + category.items.size,
            transportUrl = category.addonTransportUrl,
            remoteSources = category.remoteSources.orEmpty()
        )
        update(category.id) { existing ->
            existing.copy(
                items = if (items.isEmpty()) existing.items else (existing.items + items).distinctBy { "${it.type}:${it.id}" },
                canLoadMore = items.isNotEmpty()
            )
        }
    }

    private suspend fun fetchPage(
        category: HomeCategory,
        skip: Int,
        transportUrl: String?,
        remoteSources: List<LibraryRemoteSource>
    ): List<Meta> {
        val result = dispatch(
            mapOf(
                "type" to "catalogPageRequested",
                "categoryId" to category.id,
                "transportUrl" to transportUrl,
                "contentType" to category.type,
                "catalogId" to category.catalogId,
                "skip" to skip,
                "genre" to category.addonGenre,
                "search" to null,
                "remoteSource" to remoteSources,
                "profile" to activeProfile()
            )
        )
        val home = result.state["home"] as? Map<*, *> ?: return emptyList()
        val paging = home["paging"] as? Map<*, *> ?: return emptyList()
        return decodeItems(paging["items"])
    }

    private fun update(categoryId: String, transform: (HomeCategory) -> HomeCategory) {
        val hidden = folderCategories()[categoryId]
        if (hidden != null) {
            setFolderCategories(folderCategories() + (categoryId to transform(hidden)))
        } else {
            setCategories(categories().map { if (it.id == categoryId) transform(it) else it })
        }
    }
}
