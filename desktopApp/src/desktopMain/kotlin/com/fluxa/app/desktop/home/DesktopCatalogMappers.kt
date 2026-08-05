package com.fluxa.app.desktop.home

import androidx.compose.ui.unit.dp
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import com.fluxa.app.ui.catalog.posterCardHeight
import com.fluxa.app.ui.catalog.posterCardWidth

internal fun Meta.toDesktopCatalogItemUiModel(
    catalogType: String,
    transportUrl: String = CINEMETA_TRANSPORT_URL
): CatalogItemUiModel {
    val width = posterCardWidth("medium")
    val height = posterCardHeight("medium")
    val card = CatalogCardUiModel(
        title = name,
        subtitle = releaseInfo?.take(4) ?: released?.take(4).orEmpty(),
        showTitleBar = true,
        artworkUrl = poster,
        artworkMemoryCacheKey = poster?.takeIf { it.isNotBlank() }?.let { "desktop-home-artwork:${width.value.toInt()}x${height.value.toInt()}:$it" },
        artworkDiskCacheKey = poster,
        requestWidthPx = 288,
        requestHeightPx = 432,
        logoUrl = null,
        logoMemoryCacheKey = null,
        showLogo = false,
        allowCoverFallback = true,
        coverFallbackText = coverEmoji?.takeIf { it.isNotBlank() } ?: name.take(1).uppercase(),
        coverFallbackIsEmoji = !coverEmoji.isNullOrBlank(),
        width = width,
        imageHeight = height,
        outerWidth = width,
        cardBackgroundIsSurfaceCard = false,
        progress = 0f,
        showProgressBar = false,
        showUpNextBadge = false,
        upNextLabel = "",
        topTenRank = null,
        rankNumberBoxWidth = 0.dp,
        rankOffsetX = 0.dp,
        rankOffsetY = 0.dp,
        rankFontSizeRatio = 0f,
        loadArtwork = true
    )
    return CatalogItemUiModel(
        id = id,
        type = type,
        card = card,
        source = CatalogSourceUiModel(addonTransportUrl = transportUrl, catalogType = catalogType),
        description = description,
        ageRating = ageRating,
        seasonsCount = seasonsCount,
        runtimeLabel = runtime
    )
}
