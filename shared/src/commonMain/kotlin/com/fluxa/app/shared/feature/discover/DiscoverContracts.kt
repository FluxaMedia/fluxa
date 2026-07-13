package com.fluxa.app.shared.feature.discover

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import kotlinx.coroutines.flow.Flow

data class DiscoverFilterOptionUiModel(
    val id: String?,
    val label: String
)

data class DiscoverFiltersUiModel(
    val contentType: String = "movie",
    val catalogKey: String? = null,
    val genre: String? = null
)

data class DiscoverUiState(
    val filters: DiscoverFiltersUiModel = DiscoverFiltersUiModel(),
    val catalogOptions: List<DiscoverFilterOptionUiModel> = emptyList(),
    val genreOptions: List<DiscoverFilterOptionUiModel> = emptyList(),
    val results: List<CatalogItemUiModel> = emptyList(),
    val isLoading: Boolean = false
)

sealed interface DiscoverAction {
    data class FiltersChanged(val filters: DiscoverFiltersUiModel) : DiscoverAction
    data class ItemSelected(val item: CatalogItemUiModel) : DiscoverAction
}

interface DiscoverDataSource {
    fun observeDiscover(): Flow<DiscoverUiState>
    suspend fun updateFilters(filters: DiscoverFiltersUiModel)
}
