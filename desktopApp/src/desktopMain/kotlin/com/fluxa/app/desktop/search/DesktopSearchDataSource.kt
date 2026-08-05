package com.fluxa.app.desktop.search

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.desktop.home.CINEMETA_TRANSPORT_URL
import com.fluxa.app.desktop.home.toDesktopCatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.shared.feature.search.SearchDataSource
import com.fluxa.app.shared.feature.search.SearchUiState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

private const val SEARCH_DEBOUNCE_MS = 350L
private const val MAX_RECENT_ITEMS = 10

private data class DesktopSearchSource(val type: String, val catalogId: String, val titleKey: String)

private val desktopSearchSources = listOf(
    DesktopSearchSource(type = "movie", catalogId = "top", titleKey = "auto.movies"),
    DesktopSearchSource(type = "series", catalogId = "top", titleKey = "auto.series")
)

class DesktopSearchDataSource(
    private val addonRepository: AddonRepository,
    private val language: String? = null
) : SearchDataSource {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private val state = MutableStateFlow(SearchUiState())
    private var searchJob: Job? = null

    override fun observeSearch(): Flow<SearchUiState> = state.asStateFlow()

    override suspend fun search(query: String) {
        searchJob?.cancel()
        if (query.isBlank()) {
            state.value = state.value.copy(query = query, results = emptyList(), resultRows = emptyList(), isLoading = false)
            return
        }
        state.value = state.value.copy(query = query, isLoading = true)
        searchJob = scope.launch {
            delay(SEARCH_DEBOUNCE_MS)
            val rows = desktopSearchSources.mapNotNull { source ->
                val metas = runCatching {
                    addonRepository.getAddonCatalog(CINEMETA_TRANSPORT_URL, source.type, source.catalogId, search = query)
                }.getOrDefault(emptyList())
                if (metas.isEmpty()) return@mapNotNull null
                CatalogRowUiModel(
                    id = "search:${source.type}:${source.catalogId}",
                    title = AppStrings.t(language, source.titleKey),
                    items = metas.map { it.toDesktopCatalogItemUiModel(source.type) }
                )
            }
            if (state.value.query == query) {
                state.value = state.value.copy(
                    results = rows.flatMap { it.items },
                    resultRows = rows,
                    isLoading = false
                )
            }
        }
    }

    override suspend fun recordSelection(item: CatalogItemUiModel) {
        val history = (listOf(item) + state.value.recentItems)
            .distinctBy { "${it.type}:${it.id}" }
            .take(MAX_RECENT_ITEMS)
        state.value = state.value.copy(recentItems = history)
    }

    override suspend fun clearHistory() {
        state.value = state.value.copy(recentItems = emptyList())
    }
}
