package com.fluxa.app.ui.catalog

import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaDimensions
import com.fluxa.app.ui.catalog.horizontalCardHeight
import com.fluxa.app.ui.catalog.horizontalCardWidth
import com.fluxa.app.ui.catalog.posterCardHeight
import com.fluxa.app.ui.catalog.posterCardWidth

const val CONTINUE_WATCHING_CATEGORY_ID = "continue_watching"

fun HomeCategory.isContinueWatchingCategory(): Boolean = id == CONTINUE_WATCHING_CATEGORY_ID

fun Meta.matchesFilter(filter: String): Boolean = when (filter) {
    "movie" -> type == "movie"
    "series" -> type == "series" || type == "tv" || type == "anime"
    else -> true
}

fun orderHomeCategories(categories: List<HomeCategory>, filter: String = "all"): List<HomeCategory> {
    return categories
        .mapNotNull { category ->
            val items = when {
                category.isContinueWatchingCategory() || category.id == "library" -> {
                    if (filter == "all") category.items else {
                        category.items.filter { it.matchesFilter(filter) }.ifEmpty { category.items }
                    }
                }
                filter == "all" -> category.items
                else -> category.items.filter { it.matchesFilter(filter) }
            }
            when {
                items.isEmpty() && category.type != "collection_folder" -> null
                items === category.items -> category
                else -> category.copy(items = items)
            }
        }
}

fun resolveHomeCardLayout(category: HomeCategory, profile: UserProfile?): String {
    return if (category.isContinueWatchingCategory()) {
        profile?.resolvedContinueWatchingLayout ?: "horizontal"
    } else if (profile?.safePosterLandscapeMode == true) {
        "horizontal"
    } else {
        profile?.safeCardLayout ?: "vertical"
    }
}

fun resolveContinueWatchingArtworkPreference(category: HomeCategory, profile: UserProfile?): String? {
    return if (category.isContinueWatchingCategory()) profile?.safeContinueWatchingArtwork ?: "episode" else null
}

private fun preferredHorizontalArtwork(meta: Meta): String? {
    return meta.background
        ?.takeIf { it.isNotBlank() && it != meta.poster && !it.contains("/poster/", ignoreCase = true) }
        ?: meta.continueWatchingBackground?.takeIf { it.isNotBlank() }
}

fun Meta.homeHeroBackdrop(seasonPostersOnHero: Boolean = true): String? {
    val seasonPoster = seasonPosters
        ?.maxByOrNull { it.key.toIntOrNull() ?: 0 }
        ?.value
        ?.takeIf { it.isNotBlank() }
    return if (seasonPostersOnHero) seasonPoster ?: background else background?.takeUnless { it == seasonPoster }
}

internal fun Meta.toCatalogCardUiModel(
    cardLayout: String,
    artworkPreference: String?,
    profile: UserProfile?,
    cardScale: Float,
    showHorizontalLogo: Boolean,
    topTenRank: Int?,
    isContinueWatchingCard: Boolean,
    loadArtwork: Boolean
): CatalogCardUiModel {
    val language = profile?.safeLanguage ?: "en"
    val widthPreset = profile?.safePosterWidthPreset ?: "medium"
    val effectiveLayout = if (type == "catalog_folder") {
        when (reason) {
            "wide" -> "horizontal"
            "square" -> "square"
            "poster" -> "vertical"
            else -> cardLayout
        }
    } else cardLayout
    val episodeStyle = effectiveLayout == "episode"
    val horizontal = effectiveLayout == "horizontal" || episodeStyle
    val square = effectiveLayout == "square"
    val folder = type == "catalog_folder"
    val progressCard = isUpNextContinueItem() || ((timeOffset ?: 0L) > 0L && (duration ?: 0L) > 0L)
    val showTitleBar = !(isContinueWatchingCard && profile?.safeContinueWatchingHideTitles == true) &&
        !(profile?.safePosterHideTitles == true || hideTitle == true)
    val width = (when {
        episodeStyle -> FluxaDimensions.EpisodeCard.mobileWidth
        horizontal -> horizontalCardWidth(widthPreset, DeviceType.Mobile)
        else -> posterCardWidth(widthPreset)
    }) * cardScale
    val imageHeight = (when {
        episodeStyle -> FluxaDimensions.EpisodeCard.mobileHeight
        horizontal -> horizontalCardHeight(widthPreset, DeviceType.Mobile)
        square -> posterCardWidth(widthPreset)
        else -> posterCardHeight(widthPreset)
    }) * cardScale
    val artwork = when {
        episodeStyle -> when (artworkPreference) {
            "poster" -> poster
            "background" -> background
            else -> continueWatchingBackground
        } ?: background
        isContinueWatchingCard -> when (artworkPreference) {
            "poster" -> poster ?: continueWatchingBackground ?: background
            "background" -> background ?: continueWatchingBackground ?: poster
            else -> continueWatchingBackground ?: background ?: poster
        }
        horizontal -> preferredHorizontalArtwork(this) ?: poster
        else -> poster
    }
    val requestWidth = if (episodeStyle) 512 else if (folder) { if (horizontal) 384 else 224 } else { if (horizontal) 512 else 288 }
    val requestHeight = if (episodeStyle) 288 else if (folder) {
        when { horizontal -> 216; square -> 224; else -> 336 }
    } else when { horizontal -> 288; square -> 288; else -> 432 }
    val progress = if (progressCard) {
        ((timeOffset ?: 0L).toFloat() / (duration ?: 1L).toFloat()).coerceIn(0f, 1f)
    } else 0f
    val rankBase = if (horizontal) imageHeight else width
    val rankBoxWidth = topTenRank?.let {
        when { it >= 10 -> rankBase * 1.24f; it == 1 -> rankBase * 0.62f; else -> rankBase * 0.82f }
    } ?: 0.dp
    val rankOverlap = topTenRank?.let {
        when { it >= 10 -> rankBase * 0.16f; it == 1 -> rankBase * 0.13f; else -> rankBase * 0.24f }
    } ?: 0.dp
    val upNext = isUpNextContinueItem()
    return CatalogCardUiModel(
        title = name,
        subtitle = if (progressCard) continueWatchingEpisodeLabel(this).orEmpty() else releaseInfo?.take(4) ?: released?.take(4).orEmpty(),
        showTitleBar = showTitleBar,
        artworkUrl = artwork,
        artworkMemoryCacheKey = artwork?.takeIf { it.isNotBlank() }?.let { "home-artwork:${requestWidth}x$requestHeight:$it" },
        artworkDiskCacheKey = artwork,
        requestWidthPx = requestWidth,
        requestHeightPx = requestHeight,
        logoUrl = logo,
        logoMemoryCacheKey = "home-logo:$logo",
        showLogo = horizontal && !episodeStyle && showHorizontalLogo && !folder && !isContinueWatchingCard && (timeOffset ?: 0L) <= 0L && !logo.isNullOrBlank(),
        allowCoverFallback = !isContinueWatchingCard && !episodeStyle,
        coverFallbackText = coverEmoji?.takeIf { it.isNotBlank() } ?: name.take(1).uppercase(),
        coverFallbackIsEmoji = !coverEmoji.isNullOrBlank(),
        width = width,
        imageHeight = imageHeight,
        outerWidth = if (topTenRank != null) rankBoxWidth + width - rankOverlap else width,
        cardBackgroundIsSurfaceCard = isContinueWatchingCard || episodeStyle,
        progress = progress,
        showProgressBar = progressCard && !upNext && progress > 0f,
        showUpNextBadge = progressCard && upNext,
        upNextLabel = AppStrings.t(language, "auto.up_next"),
        topTenRank = topTenRank,
        rankNumberBoxWidth = rankBoxWidth,
        rankOffsetX = when { topTenRank == 1 -> 8.dp; (topTenRank ?: 0) >= 10 -> 0.dp; else -> 3.dp },
        rankOffsetY = if (horizontal) 1.dp else 2.dp,
        rankFontSizeRatio = if (topTenRank != null) { if (horizontal) 0.86f else 0.90f } else 0f,
        loadArtwork = loadArtwork
    )
}
