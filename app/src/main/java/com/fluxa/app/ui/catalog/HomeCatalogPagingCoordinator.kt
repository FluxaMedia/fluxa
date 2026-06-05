package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.AddonRepository
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit

internal class HomeCatalogPagingCoordinator(
    private val addonRepository: AddonRepository,
    private val scope: CoroutineScope,
    private val categories: () -> List<HomeCategory>,
    private val setCategories: (List<HomeCategory>) -> Unit,
    private val activeProfile: () -> UserProfile?,
    private val normalizeCatalogItems: suspend (List<Meta>, String, String, String?) -> List<Meta>
) {
    private val loadMoreInFlight = mutableSetOf<String>()

    fun loadMore(categoryId: String) {
        val category = categories().find { it.id == categoryId } ?: return
        if (!category.canLoadMore || !loadMoreInFlight.add(category.id)) return

        scope.launch {
            try {
                val nextSkip = if (category.items.isEmpty()) 0 else category.skip + 20
                val lang = activeProfile()?.safeLanguage ?: "en"
                val catalogSources = category.catalogSources.orEmpty()
                val nextItems = if (catalogSources.isNotEmpty()) {
                    val semaphore = Semaphore(permits = 4)
                    coroutineScope {
                        catalogSources.map { source ->
                            async {
                                semaphore.withPermit {
                                    normalizeCatalogItems(
                                        addonRepository.getAddonCatalog(
                                            source.transportUrl,
                                            source.type,
                                            source.catalogId,
                                            skip = nextSkip,
                                            genre = source.genre
                                        ),
                                        source.catalogId,
                                        lang,
                                        source.genre
                                    )
                                }
                            }
                        }.awaitAll().flatten()
                    }
                } else {
                    val transportUrl = category.addonTransportUrl
                    if (transportUrl != null) {
                        normalizeCatalogItems(
                            addonRepository.getAddonCatalog(
                                transportUrl,
                                category.type,
                                category.catalogId,
                                skip = nextSkip,
                                genre = category.addonGenre
                            ),
                            category.catalogId,
                            lang,
                            category.addonGenre
                        )
                    } else {
                        emptyList()
                    }
                }
                val currentCategories = categories().toMutableList()
                val index = currentCategories.indexOf(category)
                if (index != -1) {
                    currentCategories[index] = category.copy(
                        items = (category.items + nextItems).distinctBy { "${it.type}:${it.id}" },
                        skip = nextSkip,
                        canLoadMore = nextItems.isNotEmpty()
                    )
                    setCategories(currentCategories.toList())
                }
            } catch (e: Exception) {
                e.printStackTrace()
            } finally {
                loadMoreInFlight.remove(category.id)
            }
        }
    }
}
