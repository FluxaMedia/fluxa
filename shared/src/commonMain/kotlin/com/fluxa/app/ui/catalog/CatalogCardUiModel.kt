package com.fluxa.app.ui.catalog

import androidx.compose.ui.unit.Dp

data class CatalogCardUiModel(
    val title: String,
    val subtitle: String,
    val showTitleBar: Boolean,
    val artworkUrl: String?,
    val artworkMemoryCacheKey: String?,
    val artworkDiskCacheKey: String?,
    val requestWidthPx: Int,
    val requestHeightPx: Int,
    val logoUrl: String?,
    val logoMemoryCacheKey: String?,
    val showLogo: Boolean,
    val allowCoverFallback: Boolean,
    val coverFallbackText: String,
    val coverFallbackIsEmoji: Boolean,
    val width: Dp,
    val imageHeight: Dp,
    val outerWidth: Dp,
    val cardBackgroundIsSurfaceCard: Boolean,
    val progress: Float,
    val showProgressBar: Boolean,
    val showUpNextBadge: Boolean,
    val upNextLabel: String,
    val topTenRank: Int?,
    val rankNumberBoxWidth: Dp,
    val rankOffsetX: Dp,
    val rankOffsetY: Dp,
    val rankFontSizeRatio: Float,
    val loadArtwork: Boolean
)
