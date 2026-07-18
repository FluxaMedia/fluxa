package com.fluxa.app.shared.feature.search

import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel

@Composable
fun TvSearchScreen(
    state: SearchUiState,
    language: String?,
    onQueryChanged: (String) -> Unit,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    onAddToLibrary: (CatalogItemUiModel) -> Unit = {},
    onClearHistory: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    SearchScreen(
        state = state,
        language = language,
        onQueryChanged = onQueryChanged,
        onItemSelected = onItemSelected,
        onAddToLibrary = onAddToLibrary,
        onClearHistory = onClearHistory,
        modifier = modifier.padding(start = 38.dp, top = 56.dp, end = 38.dp, bottom = 24.dp)
    )
}
