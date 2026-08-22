package com.fluxa.app.shared.platform

import androidx.compose.ui.unit.dp
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel

internal fun appleCatalogItem(
    id: String,
    type: String,
    title: String,
    subtitle: String = "",
    artworkUrl: String?,
    logoUrl: String? = null,
    addonTransportUrl: String? = null,
    catalogType: String? = null,
    cacheNamespace: String,
    progress: Float? = null,
    topTenRank: Int? = null,
): CatalogItemUiModel {
    val cacheKey = artworkUrl?.let { "$cacheNamespace:$it" }
    return CatalogItemUiModel(
        id = id,
        type = type,
        source = CatalogSourceUiModel(
            addonTransportUrl = addonTransportUrl,
            catalogType = catalogType,
        ),
        card = CatalogCardUiModel(
            title = title,
            subtitle = subtitle,
            showTitleBar = true,
            artworkUrl = artworkUrl,
            artworkMemoryCacheKey = cacheKey,
            artworkDiskCacheKey = cacheKey,
            requestWidthPx = 264,
            requestHeightPx = 396,
            logoUrl = logoUrl,
            logoMemoryCacheKey = null,
            showLogo = false,
            allowCoverFallback = true,
            coverFallbackText = title,
            coverFallbackIsEmoji = false,
            width = 132.dp,
            imageHeight = 198.dp,
            outerWidth = 132.dp,
            cardBackgroundIsSurfaceCard = true,
            progress = progress?.coerceIn(0f, 1f) ?: 0f,
            showProgressBar = progress != null,
            showUpNextBadge = false,
            upNextLabel = "",
            topTenRank = topTenRank,
            rankNumberBoxWidth = 0.dp,
            rankOffsetX = 0.dp,
            rankOffsetY = 0.dp,
            rankFontSizeRatio = 1f,
            loadArtwork = true,
        ),
    )
}
