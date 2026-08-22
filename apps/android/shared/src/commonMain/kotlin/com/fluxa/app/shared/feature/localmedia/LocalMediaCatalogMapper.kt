package com.fluxa.app.shared.feature.localmedia

import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.posterCardHeight
import com.fluxa.app.ui.catalog.posterCardWidth

fun LocalMediaCatalogEntry.toCatalogItemUiModel(deviceType: DeviceType): CatalogItemUiModel {
    val width = posterCardWidth("medium")
    val height = posterCardHeight("medium")
    val art = posterUrl
    val card = CatalogCardUiModel(
        title = title,
        subtitle = year?.toString() ?: releaseLabel?.take(4).orEmpty(),
        showTitleBar = true,
        artworkUrl = art,
        artworkMemoryCacheKey = art?.let { "local-media:${width.value.toInt()}x${height.value.toInt()}:$it" },
        artworkDiskCacheKey = art,
        requestWidthPx = if (deviceType == DeviceType.TV) 360 else 288,
        requestHeightPx = if (deviceType == DeviceType.TV) 540 else 432,
        logoUrl = logoUrl,
        logoMemoryCacheKey = logoUrl?.let { "local-media-logo:$it" },
        showLogo = false,
        allowCoverFallback = true,
        coverFallbackText = title.take(1).uppercase(),
        coverFallbackIsEmoji = false,
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
        loadArtwork = true,
    )
    return CatalogItemUiModel(
        id = contentId,
        type = contentType,
        card = card,
        source = CatalogSourceUiModel(
            addonTransportUrl = metadataAddonUrl,
            catalogType = contentType,
        ),
        posterUrl = posterUrl,
        backdropUrl = backdropUrl,
        description = description,
        releaseLabel = releaseLabel,
        ratingLabel = ratingLabel,
        ageRating = ageRating,
        genres = genres,
        seasonsCount = seasonsCount,
        runtimeLabel = runtimeLabel,
    )
}

fun LocalMediaLibrarySnapshot.toCatalogRows(deviceType: DeviceType, language: String? = null): List<com.fluxa.app.shared.feature.catalog.CatalogRowUiModel> = buildList {
    if (movies.isNotEmpty()) add(
        com.fluxa.app.shared.feature.catalog.CatalogRowUiModel(
            id = "local-media-movies",
            title = AppStrings.t(language, "library.local_media_movies"),
            items = movies.map { it.toCatalogItemUiModel(deviceType) },
            categoryType = "movie",
        )
    )
    if (tvShows.isNotEmpty()) add(
        com.fluxa.app.shared.feature.catalog.CatalogRowUiModel(
            id = "local-media-tv",
            title = AppStrings.t(language, "library.local_media_tv_shows"),
            items = tvShows.map { it.toCatalogItemUiModel(deviceType) },
            categoryType = "series",
        )
    )
    if (anime.isNotEmpty()) add(
        com.fluxa.app.shared.feature.catalog.CatalogRowUiModel(
            id = "local-media-anime",
            title = AppStrings.t(language, "auto.anime"),
            items = anime.map { it.toCatalogItemUiModel(deviceType) },
            categoryType = "series",
        )
    )
}
