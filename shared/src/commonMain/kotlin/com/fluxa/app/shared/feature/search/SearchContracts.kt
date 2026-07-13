package com.fluxa.app.shared.feature.search

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import kotlinx.coroutines.flow.Flow

data class SearchUiState(
    val query: String = "",
    val results: List<CatalogItemUiModel> = emptyList(),
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
    suspend fun clearHistory()
}
