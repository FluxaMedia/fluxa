package com.fluxa.app.shared.feature.catalog

import androidx.compose.ui.unit.dp

internal fun CatalogItemUiModel.withProminentContinueWatchingCard(): CatalogItemUiModel = copy(
    card = card.copy(
        requestWidthPx = 640,
        requestHeightPx = 360,
        width = 192.dp,
        imageHeight = 96.dp,
        outerWidth = 192.dp
    )
)
