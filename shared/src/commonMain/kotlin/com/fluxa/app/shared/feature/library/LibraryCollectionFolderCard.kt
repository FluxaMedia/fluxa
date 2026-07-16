package com.fluxa.app.shared.feature.library

import com.fluxa.app.ui.catalog.CatalogCardUiModel
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaDimensions
import com.fluxa.app.ui.catalog.horizontalCardHeight
import com.fluxa.app.ui.catalog.horizontalCardWidth
import com.fluxa.app.ui.catalog.posterCardHeight
import com.fluxa.app.ui.catalog.posterCardWidth
import androidx.compose.ui.unit.dp

enum class FolderTileShape { Poster, Landscape, Square }

fun LibraryFolderUiModel.tileShape(): FolderTileShape = when (shape?.trim()?.lowercase()) {
    "wide", "landscape" -> FolderTileShape.Landscape
    "square" -> FolderTileShape.Square
    else -> FolderTileShape.Poster
}

fun LibraryFolderUiModel.effectiveCoverUrl(): String? =
    coverImageUrl?.takeIf { it.isNotBlank() } ?: imageUrl?.takeIf { it.isNotBlank() }

fun LibraryFolderUiModel.toCatalogCardUiModel(widthPreset: String = "medium"): CatalogCardUiModel {
    val staticArtwork = effectiveCoverUrl()
    val artwork = focusGifUrl
        ?.takeIf { it.isNotBlank() && focusGifEnabled }
        ?: staticArtwork
    val shape = tileShape()
    val width = when (shape) {
        FolderTileShape.Landscape -> horizontalCardWidth(widthPreset, DeviceType.Mobile)
        else -> posterCardWidth(widthPreset)
    }
    val imageHeight = when (shape) {
        FolderTileShape.Landscape -> horizontalCardHeight(widthPreset, DeviceType.Mobile)
        FolderTileShape.Square -> posterCardWidth(widthPreset)
        FolderTileShape.Poster -> posterCardHeight(widthPreset)
    }
    val emoji = coverEmoji?.takeIf { it.isNotBlank() }
    return CatalogCardUiModel(
        title = title,
        subtitle = catalogTitle.orEmpty(),
        showTitleBar = !hideTitle,
        artworkUrl = artwork,
        artworkMemoryCacheKey = "collection-folder:$id",
        artworkDiskCacheKey = "collection-folder:$id",
        requestWidthPx = if (shape == FolderTileShape.Landscape) 384 else 224,
        requestHeightPx = when (shape) {
            FolderTileShape.Landscape -> 216
            FolderTileShape.Square -> 224
            FolderTileShape.Poster -> 336
        },
        logoUrl = null,
        logoMemoryCacheKey = null,
        showLogo = false,
        allowCoverFallback = artwork.isNullOrBlank(),
        coverFallbackText = emoji ?: title.take(2).uppercase(),
        coverFallbackIsEmoji = emoji != null,
        width = width,
        imageHeight = imageHeight,
        outerWidth = width,
        cardBackgroundIsSurfaceCard = true,
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
}
