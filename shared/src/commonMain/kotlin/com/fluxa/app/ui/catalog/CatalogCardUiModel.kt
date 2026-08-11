package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Immutable
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

@Immutable
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
    val upNextBadgeAccent: Boolean = false,
    val progressLabel: String? = null,
    val topTenRank: Int?,
    val rankNumberBoxWidth: Dp,
    val rankOffsetX: Dp,
    val rankOffsetY: Dp,
    val rankFontSizeRatio: Float,
    val loadArtwork: Boolean,
    val animatedArtworkUrl: String? = null,
    val animatedArtworkMemoryCacheKey: String? = null,
    /** Draw title/subtitle inside the artwork, immediately above the progress bar. */
    val overlayTitleBar: Boolean = false,
    val cornerRadius: Dp = 8.dp
)
