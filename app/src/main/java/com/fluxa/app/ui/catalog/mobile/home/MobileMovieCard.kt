package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta

@Composable
internal fun MobileMovieCard(
    meta: Meta,
    onFocus: (Meta) -> Unit,
    onClick: () -> Unit,
    cardLayout: String,
    onForgetProgress: (() -> Unit)? = null,
    onProgressActions: (() -> Unit)? = null,
    artworkPreference: String? = null,
    profile: UserProfile? = null,
    cardScale: Float = 1f,
    showHorizontalLogo: Boolean = true,
    topTenRank: Int? = null,
    isContinueWatchingCard: Boolean = false,
    loadArtwork: Boolean = true
) {
    val model = remember(
        meta, cardLayout, artworkPreference, profile, cardScale,
        showHorizontalLogo, topTenRank, isContinueWatchingCard, loadArtwork
    ) {
        meta.toCatalogCardUiModel(
            cardLayout = cardLayout,
            artworkPreference = artworkPreference,
            profile = profile,
            cardScale = cardScale,
            showHorizontalLogo = showHorizontalLogo,
            topTenRank = topTenRank,
            isContinueWatchingCard = isContinueWatchingCard,
            loadArtwork = loadArtwork
        )
    }
    CatalogCard(
        model = model,
        onClick = onClick,
        onLongClick = {
            if ((meta.timeOffset ?: 0L) > 0L) onProgressActions?.invoke()
        }
    )
}

private fun Meta.toCatalogCardUiModel(
    cardLayout: String,
    artworkPreference: String?,
    profile: UserProfile?,
    cardScale: Float,
    showHorizontalLogo: Boolean,
    topTenRank: Int?,
    isContinueWatchingCard: Boolean,
    loadArtwork: Boolean
): CatalogCardUiModel {
    val lang = profile?.safeLanguage ?: "en"
    val widthPreset = profile?.safePosterWidthPreset ?: "medium"

    val effectiveCardLayout = if (type == "catalog_folder") {
        when (reason) {
            "wide" -> "horizontal"
            "square" -> "square"
            "poster" -> "vertical"
            else -> cardLayout
        }
    } else {
        cardLayout
    }
    val isEpisodeStyle = effectiveCardLayout == "episode"
    val isHorizontal = effectiveCardLayout == "horizontal" || isEpisodeStyle
    val isSquare = effectiveCardLayout == "square"
    val isCatalogFolder = type == "catalog_folder"
    val isProgressCard = isUpNextContinueItem() || ((timeOffset ?: 0L) > 0L && (duration ?: 0L) > 0L)
    val hideContinueWatchingTitles = isContinueWatchingCard && profile?.safeContinueWatchingHideTitles == true
    val showTitleBar = !hideContinueWatchingTitles && !(profile?.safePosterHideTitles == true || hideTitle == true)

    val width = (when {
        isEpisodeStyle -> FluxaDimensions.EpisodeCard.mobileWidth
        isHorizontal -> horizontalCardWidth(widthPreset, DeviceType.Mobile)
        else -> posterCardWidth(widthPreset)
    }) * cardScale
    val imageHeight = (when {
        isEpisodeStyle -> FluxaDimensions.EpisodeCard.mobileHeight
        isHorizontal -> horizontalCardHeight(widthPreset, DeviceType.Mobile)
        isSquare -> posterCardWidth(widthPreset)
        else -> posterCardHeight(widthPreset)
    }) * cardScale

    val artwork = when {
        isEpisodeStyle -> {
            val seriesArtwork = when (artworkPreference) {
                "episode" -> continueWatchingBackground
                "poster" -> poster
                "background" -> background
                else -> continueWatchingBackground
            }
            if (type == "series" || type == "tv" || type == "anime") seriesArtwork ?: background else background
        }
        isContinueWatchingCard -> when (artworkPreference) {
            "poster" -> poster ?: continueWatchingBackground ?: background
            "background" -> background ?: continueWatchingBackground ?: poster
            else -> continueWatchingBackground ?: background ?: poster
        }
        isHorizontal -> preferredHorizontalArtwork(this) ?: poster
        else -> poster
    }
    val requestWidth = if (isEpisodeStyle) 512 else if (isCatalogFolder) { if (isHorizontal) 384 else 224 } else { if (isHorizontal) 512 else 288 }
    val requestHeight = if (isEpisodeStyle) 288 else if (isCatalogFolder) {
        when { isHorizontal -> 216; isSquare -> 224; else -> 336 }
    } else {
        when { isHorizontal -> 288; isSquare -> 288; else -> 432 }
    }

    val showLogo = isHorizontal && !isEpisodeStyle && showHorizontalLogo && !isCatalogFolder &&
        !isContinueWatchingCard && (timeOffset ?: 0L) <= 0L && !logo.isNullOrBlank()

    val progress = if (isProgressCard) {
        ((timeOffset ?: 0L).toFloat() / (duration ?: 1L).toFloat()).coerceIn(0f, 1f)
    } else {
        0f
    }
    val secondaryText = when {
        isCatalogFolder -> releaseInfo?.take(4) ?: ""
        isProgressCard -> continueWatchingEpisodeLabel(this).orEmpty()
        else -> releaseInfo?.take(4) ?: released?.take(4) ?: ""
    }

    val rankBase = if (isHorizontal) imageHeight else width
    val rankNumberBoxWidth = topTenRank?.let { r ->
        when { r >= 10 -> rankBase * 1.24f; r == 1 -> rankBase * 0.62f; else -> rankBase * 0.82f }
    } ?: 0.dp
    val rankPosterOverlap = topTenRank?.let { r ->
        when { r >= 10 -> rankBase * 0.16f; r == 1 -> rankBase * 0.13f; else -> rankBase * 0.24f }
    } ?: 0.dp
    val outerWidth = if (topTenRank != null) rankNumberBoxWidth + width - rankPosterOverlap else width
    val rankOffsetX = when {
        topTenRank == 1 -> 8.dp
        (topTenRank ?: 0) >= 10 -> 0.dp
        else -> 3.dp
    }
    val rankOffsetY = if (isHorizontal) 1.dp else 2.dp
    val rankFontSizeRatio = if (topTenRank != null) { if (isHorizontal) 0.86f else 0.90f } else 0f

    val isUpNext = isUpNextContinueItem()
    val coverFallbackIsEmoji = !coverEmoji.isNullOrBlank()
    val coverFallbackText = coverEmoji?.takeIf { it.isNotBlank() } ?: name.take(1).uppercase()

    return CatalogCardUiModel(
        title = name,
        subtitle = secondaryText,
        showTitleBar = showTitleBar,
        artworkUrl = artwork,
        artworkMemoryCacheKey = homeArtworkMemoryCacheKey(artwork, requestWidth, requestHeight),
        artworkDiskCacheKey = artwork,
        requestWidthPx = requestWidth,
        requestHeightPx = requestHeight,
        logoUrl = logo,
        logoMemoryCacheKey = "home-logo:$logo",
        showLogo = showLogo,
        allowCoverFallback = !isContinueWatchingCard && !isEpisodeStyle,
        coverFallbackText = coverFallbackText,
        coverFallbackIsEmoji = coverFallbackIsEmoji,
        width = width,
        imageHeight = imageHeight,
        outerWidth = outerWidth,
        cardBackgroundIsSurfaceCard = isContinueWatchingCard || isEpisodeStyle,
        progress = progress,
        showProgressBar = isProgressCard && !isUpNext && progress > 0f,
        showUpNextBadge = isProgressCard && isUpNext,
        upNextLabel = AppStrings.t(lang, "auto.up_next"),
        topTenRank = topTenRank,
        rankNumberBoxWidth = rankNumberBoxWidth,
        rankOffsetX = rankOffsetX,
        rankOffsetY = rankOffsetY,
        rankFontSizeRatio = rankFontSizeRatio,
        loadArtwork = loadArtwork
    )
}

internal fun homeArtworkMemoryCacheKey(url: String?, width: Int, height: Int): String? {
    return url?.takeIf { it.isNotBlank() }?.let { "home-artwork:${width}x$height:$it" }
}
