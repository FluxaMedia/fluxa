package com.fluxa.app.desktop.home

import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.ui.catalog.CONTINUE_WATCHING_CATEGORY_ID
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine

private const val HERO_ITEM_COUNT = 6

private fun heroItemsFrom(rows: List<CatalogRowUiModel>): List<CatalogItemUiModel> =
    rows.firstOrNull { it.id != CONTINUE_WATCHING_CATEGORY_ID && it.items.isNotEmpty() }
        ?.items
        ?.take(HERO_ITEM_COUNT)
        .orEmpty()

class DesktopCatalogHomeDataSource(
    private val coordinator: DesktopHomeCoordinator
) : CatalogHomeDataSource {
    override fun observeHome(): Flow<CatalogHomeUiState> = combine(
        coordinator.rows,
        coordinator.isLoading,
        coordinator.currentFilter
    ) { rows, isLoading, filter ->
        val heroItems = heroItemsFrom(rows)
        CatalogHomeUiState(
            rows = rows,
            isLoading = isLoading,
            billboard = null,
            heroItems = heroItems,
            showHeroSection = heroItems.isNotEmpty(),
            activeFilter = filter
        )
    }

    override fun initialHomeState(): CatalogHomeUiState {
        val heroItems = heroItemsFrom(coordinator.rows.value)
        return CatalogHomeUiState(
            rows = coordinator.rows.value,
            isLoading = coordinator.isLoading.value,
            billboard = null,
            heroItems = heroItems,
            showHeroSection = heroItems.isNotEmpty(),
            activeFilter = coordinator.currentFilter.value
        )
    }

    override suspend fun refresh() {
        coordinator.refresh()
    }

    override suspend fun loadMore(rowId: String) {
        coordinator.loadMore(rowId)
    }

    override suspend fun setFilter(filter: String) {
        coordinator.setFilter(filter)
    }
}
