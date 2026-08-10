package com.fluxa.app.shared.feature.catalog

import androidx.compose.ui.unit.dp

internal fun CatalogItemUiModel.withProminentContinueWatchingCard(
    isDesktop: Boolean = false,
    hideLabels: Boolean = false
): CatalogItemUiModel {
    val width = if (isDesktop) 260.dp else 192.dp
    val height = if (isDesktop) 130.dp else 96.dp
    return copy(
        card = card.copy(
            requestWidthPx = if (isDesktop) 860 else 640,
            requestHeightPx = if (isDesktop) 484 else 360,
            width = width,
            imageHeight = height,
            outerWidth = width,
            // "Hide labels" keeps the metadata, but moves it into the artwork so the
            // Continue Watching row stays compact instead of throwing information away.
            showTitleBar = !hideLabels,
            overlayTitleBar = hideLabels
        )
    )
}
