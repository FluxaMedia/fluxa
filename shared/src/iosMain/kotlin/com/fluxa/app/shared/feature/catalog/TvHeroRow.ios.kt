package com.fluxa.app.shared.feature.catalog

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
actual fun TvHeroRow(
    items: List<CatalogItemUiModel>,
    language: String?,
    onItemClick: (CatalogItemUiModel) -> Unit,
    modifier: Modifier
) {
}
