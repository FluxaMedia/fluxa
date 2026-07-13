package com.fluxa.app.shared.feature.search

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import kotlinx.coroutines.flow.Flow

data class SearchUiState(
    val query: String = "",
    val results: List<CatalogItemUiModel> = emptyList(),
    val resultRows: List<CatalogRowUiModel> = emptyList(),
    val recentItems: List<CatalogItemUiModel> = emptyList(),
    val isLoading: Boolean = false
)

sealed interface SearchAction {
    data class QueryChanged(val value: String) : SearchAction
    data class ItemSelected(val item: CatalogItemUiModel) : SearchAction
    data object ClearHistory : SearchAction
}

interface SearchDataSource {
    fun observeSearch(): Flow<SearchUiState>
    suspend fun search(query: String)
    suspend fun recordSelection(item: CatalogItemUiModel)
    suspend fun clearHistory()
}
