package com.fluxa.app.shared.feature.catalog

import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.cardCornerRadius
import com.fluxa.app.ui.catalog.horizontalCardHeight
import com.fluxa.app.ui.catalog.horizontalCardWidth
import com.fluxa.app.ui.catalog.posterCardHeight
import com.fluxa.app.ui.catalog.posterCardWidth

internal fun CatalogItemUiModel.withProminentContinueWatchingCard(
    deviceType: DeviceType,
    isDesktop: Boolean = false,
    hideLabels: Boolean = false,
    widthPreset: String = "medium",
    cornerPreset: String = "medium",
    landscapeMode: Boolean = true
): CatalogItemUiModel {
    val width = if (landscapeMode) horizontalCardWidth(widthPreset, deviceType) else posterCardWidth(widthPreset)
    val height = if (landscapeMode) horizontalCardHeight(widthPreset, deviceType) else posterCardHeight(widthPreset)
    return copy(
        card = card.copy(
            requestWidthPx = if (!landscapeMode) 288 else if (isDesktop) 860 else 640,
            requestHeightPx = if (!landscapeMode) 432 else if (isDesktop) 484 else 360,
            width = width,
            imageHeight = height,
            outerWidth = width,
            // "Hide labels" keeps the metadata, but moves it into the artwork so the
            // Continue Watching row stays compact instead of throwing information away.
            showTitleBar = !hideLabels,
            overlayTitleBar = hideLabels,
            cornerRadius = cardCornerRadius(cornerPreset)
        )
    )
}
