package com.fluxa.app.desktop.home

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

internal const val CINEMETA_TRANSPORT_URL = "https://v3-cinemeta.strem.io/manifest.json"

private data class DesktopCatalogSource(
    val rowId: String,
    val title: String,
    val type: String,
    val catalogId: String
)

private val desktopHomeSources = listOf(
    DesktopCatalogSource(rowId = "cinemeta-top-movies", title = "Popular Movies", type = "movie", catalogId = "top"),
    DesktopCatalogSource(rowId = "cinemeta-top-series", title = "Popular Series", type = "series", catalogId = "top")
)

class DesktopHomeCoordinator(private val addonRepository: AddonRepository) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    private val _rows = MutableStateFlow<List<CatalogRowUiModel>>(emptyList())
    val rows: StateFlow<List<CatalogRowUiModel>> = _rows.asStateFlow()

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    private val _currentFilter = MutableStateFlow("all")
    val currentFilter: StateFlow<String> = _currentFilter.asStateFlow()

    private val metaByRow = mutableMapOf<String, MutableList<Meta>>()
    private val skipByRow = mutableMapOf<String, Int>()

    fun refresh() {
        if (_rows.value.isNotEmpty() || _isLoading.value) return
        scope.launch {
            _isLoading.value = true
            desktopHomeSources.forEach { source ->
                val metas = addonRepository.getAddonCatalog(CINEMETA_TRANSPORT_URL, source.type, source.catalogId, skip = 0)
                metaByRow[source.rowId] = metas.toMutableList()
                skipByRow[source.rowId] = metas.size
            }
            publishRows()
            _isLoading.value = false
        }
    }

    fun loadMore(rowId: String) {
        val source = desktopHomeSources.firstOrNull { it.rowId == rowId } ?: return
        scope.launch {
            val skip = skipByRow[rowId] ?: 0
            val more = addonRepository.getAddonCatalog(CINEMETA_TRANSPORT_URL, source.type, source.catalogId, skip = skip)
            if (more.isNotEmpty()) {
                metaByRow.getOrPut(rowId) { mutableListOf() }.addAll(more)
                skipByRow[rowId] = skip + more.size
                publishRows()
            }
        }
    }

    fun setFilter(filter: String) {
        _currentFilter.value = filter
    }

    private fun publishRows() {
        _rows.value = desktopHomeSources.map { source ->
            CatalogRowUiModel(
                id = source.rowId,
                title = source.title,
                items = metaByRow[source.rowId].orEmpty().map { it.toDesktopCatalogItemUiModel(source.type) },
                canLoadMore = true,
                categoryType = source.type
            )
        }
    }
}
