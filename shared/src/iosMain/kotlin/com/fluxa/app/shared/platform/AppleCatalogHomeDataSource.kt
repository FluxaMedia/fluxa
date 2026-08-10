package com.fluxa.app.shared.platform

import com.fluxa.app.core.apple.AppleCatalogHomeSnapshot
import com.fluxa.app.core.apple.AppleCatalogItemSnapshot
import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

class AppleCatalogHomeDataSource : CatalogHomeDataSource {
    private val state = MutableStateFlow(CatalogHomeUiState())
    private var onRefreshRequested: () -> Unit = {}

    override fun observeHome(): Flow<CatalogHomeUiState> = state.asStateFlow()

    override suspend fun refresh() {
        onRefreshRequested()
    }

    override suspend fun loadMore(rowId: String) = Unit

    override suspend fun setFilter(filter: String) {
        state.value = state.value.copy(activeFilter = filter)
    }

    fun setOnRefreshRequested(handler: () -> Unit) {
        onRefreshRequested = handler
    }

    fun update(snapshot: AppleCatalogHomeSnapshot) {
        state.value = CatalogHomeUiState(
            rows = snapshot.rows.map { row ->
                CatalogRowUiModel(
                    id = row.id,
                    title = row.title,
                    canLoadMore = row.canLoadMore,
                    items = row.items.map { item -> item.toCatalogItemUiModel() }
                )
            },
            isLoading = snapshot.isLoading
        )
    }
}

private fun AppleCatalogItemSnapshot.toCatalogItemUiModel(): CatalogItemUiModel = appleCatalogItem(
    id = id,
    type = type,
    title = title,
    subtitle = subtitle,
    artworkUrl = artworkUrl,
    logoUrl = logoUrl,
    addonTransportUrl = addonTransportUrl,
    catalogType = catalogType,
    cacheNamespace = "apple-catalog",
    progress = progress,
    topTenRank = topTenRank,
)
