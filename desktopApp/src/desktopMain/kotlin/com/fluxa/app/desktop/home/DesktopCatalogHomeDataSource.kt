package com.fluxa.app.desktop.home

import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine

class DesktopCatalogHomeDataSource(
    private val coordinator: DesktopHomeCoordinator
) : CatalogHomeDataSource {
    override fun observeHome(): Flow<CatalogHomeUiState> = combine(
        coordinator.rows,
        coordinator.isLoading,
        coordinator.currentFilter
    ) { rows, isLoading, filter ->
        CatalogHomeUiState(
            rows = rows,
            isLoading = isLoading,
            billboard = null,
            heroItems = emptyList(),
            showHeroSection = false,
            activeFilter = filter
        )
    }

    override fun initialHomeState(): CatalogHomeUiState = CatalogHomeUiState(
        rows = coordinator.rows.value,
        isLoading = coordinator.isLoading.value,
        billboard = null,
        heroItems = emptyList(),
        showHeroSection = false,
        activeFilter = coordinator.currentFilter.value
    )

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
