package com.fluxa.app.shared.feature.library

import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel

@Composable
fun TvLibraryScreen(
    state: LibraryUiState,
    language: String?,
    onAction: (LibraryAction) -> Unit,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    modifier: Modifier = Modifier
) {
    LibraryScreen(
        state = state,
        language = language,
        onAction = onAction,
        onItemSelected = onItemSelected,
        modifier = modifier.padding(start = 38.dp, top = 56.dp, end = 38.dp, bottom = 24.dp)
    )
}
