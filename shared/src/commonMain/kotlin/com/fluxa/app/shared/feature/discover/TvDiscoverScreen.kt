package com.fluxa.app.shared.feature.discover

import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel

@Composable
fun TvDiscoverScreen(
    state: DiscoverUiState,
    language: String?,
    onFiltersChanged: (DiscoverFiltersUiModel) -> Unit,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    onLoadMore: () -> Unit = {},
    searchQuery: String = "",
    onSearchQueryChanged: (String) -> Unit = {},
    searchResultRows: List<CatalogRowUiModel> = emptyList(),
    searchResults: List<CatalogItemUiModel> = emptyList(),
    isSearching: Boolean = false,
    modifier: Modifier = Modifier
) {
    DiscoverScreen(
        state = state,
        language = language,
        onFiltersChanged = onFiltersChanged,
        onItemSelected = onItemSelected,
        onLoadMore = onLoadMore,
        searchQuery = searchQuery,
        onSearchQueryChanged = onSearchQueryChanged,
        searchResultRows = searchResultRows,
        searchResults = searchResults,
        isSearching = isSearching,
        modifier = modifier.padding(start = 38.dp, top = 56.dp, end = 38.dp, bottom = 24.dp)
    )
}
