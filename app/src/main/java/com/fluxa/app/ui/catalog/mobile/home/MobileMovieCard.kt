@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.foundation.ExperimentalFoundationApi::class, androidx.compose.ui.ExperimentalComposeUiApi::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*

import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.layout.*
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import coil3.decode.StaticImageDecoder
import coil3.request.ImageRequest
import coil3.request.allowHardware
import coil3.request.crossfade
import coil3.size.Precision
import coil3.size.Scale

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
    val context = LocalContext.current
    val density = LocalDensity.current
    val lang = profile?.safeLanguage ?: "en"
    val widthPreset = profile?.safePosterWidthPreset ?: "medium"

    val effectiveCardLayout = if (meta.type == "catalog_folder") {
        when (meta.reason) {
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
    val isCatalogFolder = meta.type == "catalog_folder"
    val isProgressCard = meta.isUpNextContinueItem() || ((meta.timeOffset ?: 0L) > 0L && (meta.duration ?: 0L) > 0L)
    val hideContinueWatchingTitles = isContinueWatchingCard && profile?.safeContinueWatchingHideTitles == true
    val showTitle = !hideContinueWatchingTitles && !(profile?.safePosterHideTitles == true || meta.hideTitle == true)

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

    val artwork = remember(meta.id, meta.poster, meta.background, meta.continueWatchingBackground, effectiveCardLayout, artworkPreference, isContinueWatchingCard) {
        when {
            isEpisodeStyle -> {
                val seriesArtwork = when (artworkPreference) {
                    "episode" -> meta.continueWatchingBackground
                    "poster" -> meta.poster
                    "background" -> meta.background
                    else -> meta.continueWatchingBackground
                }
                if (meta.type == "series" || meta.type == "tv" || meta.type == "anime") seriesArtwork ?: meta.background else meta.background
            }
            isContinueWatchingCard -> when (artworkPreference) {
                "poster" -> meta.poster ?: meta.continueWatchingBackground ?: meta.background
                "background" -> meta.background ?: meta.continueWatchingBackground ?: meta.poster
                else -> meta.continueWatchingBackground ?: meta.background ?: meta.poster
            }
            isHorizontal -> preferredHorizontalArtwork(meta) ?: meta.poster
            else -> meta.poster
        }
    }
    val requestWidth = if (isEpisodeStyle) 512 else if (isCatalogFolder) { if (isHorizontal) 384 else 224 } else { if (isHorizontal) 512 else 288 }
    val requestHeight = if (isEpisodeStyle) 288 else if (isCatalogFolder) {
        when { isHorizontal -> 216; isSquare -> 224; else -> 336 }
    } else {
        when { isHorizontal -> 288; isSquare -> 288; else -> 432 }
    }
    val request = remember(context, artwork, requestWidth, requestHeight, isCatalogFolder) {
        val builder = ImageRequest.Builder(context)
            .data(artwork)
            .crossfade(false)
            .memoryCacheKey(homeArtworkMemoryCacheKey(artwork, requestWidth, requestHeight))
            .diskCacheKey(artwork)
            .size(requestWidth, requestHeight)
        if (isCatalogFolder) {
            builder.allowHardware(false).precision(Precision.INEXACT).scale(Scale.FILL)
            if (!artwork.isNullOrSvgArtwork()) {
                builder.decoderFactory(StaticImageDecoder.Factory())
            }
        }
        builder.build()
    }
    var failed by remember(meta.id, artwork) { mutableStateOf(artwork.isNullOrBlank()) }

    val showLogo = isHorizontal && !isEpisodeStyle && showHorizontalLogo && !isCatalogFolder && !isContinueWatchingCard && (meta.timeOffset ?: 0L) <= 0L && !meta.logo.isNullOrBlank()
    val logoRequest = remember(meta.logo, showLogo) {
        if (!showLogo) null else ImageRequest.Builder(context)
            .data(meta.logo)
            .crossfade(false)
            .memoryCacheKey("home-logo:${meta.logo}")
            .diskCacheKey(meta.logo)
            .build()
    }

    val progress = if (isProgressCard) ((meta.timeOffset ?: 0L).toFloat() / (meta.duration ?: 1L).toFloat()).coerceIn(0f, 1f) else 0f
    val secondaryText = when {
        isCatalogFolder -> meta.releaseInfo?.take(4) ?: ""
        isProgressCard -> continueWatchingEpisodeLabel(meta).orEmpty()
        else -> meta.releaseInfo?.take(4) ?: meta.released?.take(4) ?: ""
    }

    val rankBase = if (isHorizontal) imageHeight else width
    val rankNumberBoxWidth = topTenRank?.let { r -> when { r >= 10 -> rankBase * 1.24f; r == 1 -> rankBase * 0.62f; else -> rankBase * 0.82f } } ?: 0.dp
    val rankPosterOverlap = topTenRank?.let { r -> when { r >= 10 -> rankBase * 0.16f; r == 1 -> rankBase * 0.13f; else -> rankBase * 0.24f } } ?: 0.dp
    val outerWidth = if (topTenRank != null) rankNumberBoxWidth + width - rankPosterOverlap else width
    val rankFontSize = remember(topTenRank, imageHeight, isHorizontal, density) {
        if (topTenRank != null) with(density) { (imageHeight.toPx() * if (isHorizontal) 0.86f else 0.90f).toSp() } else 0.sp
    }

    Box(
        modifier = Modifier
            .width(outerWidth)
            .height(imageHeight + if (showTitle) FluxaDimensions.cardMetaBarHeight else 0.dp)
            .combinedClickable(
                interactionSource = null,
                indication = null,
                onClick = onClick,
                onLongClick = {
                    if ((meta.timeOffset ?: 0L) > 0L) onProgressActions?.invoke()
                }
            )
    ) {
        topTenRank?.let { rank ->
            Box(
                modifier = Modifier
                    .align(Alignment.TopStart)
                    .width(rankNumberBoxWidth)
                    .height(imageHeight),
                contentAlignment = Alignment.CenterEnd
            ) {
                TopTenRankNumber(
                    rank = rank,
                    fontSize = rankFontSize,
                    modifier = Modifier.offset(
                        x = when {
                            rank == 1 -> 8.dp
                            rank >= 10 -> 0.dp
                            else -> 3.dp
                        },
                        y = if (isHorizontal) 1.dp else 2.dp
                    )
                )
            }
        }

        Column(modifier = Modifier.align(Alignment.TopEnd).width(width)) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(imageHeight)
                    .background(if (isContinueWatchingCard || isEpisodeStyle) FluxaColors.surfaceCard else Color.White.copy(alpha = FluxaDimensions.Alpha.emptyCardBackground)),
                contentAlignment = Alignment.Center
            ) {
                if (loadArtwork && !failed) {
                    AsyncImage(
                        model = request,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop,
                        filterQuality = if (isCatalogFolder) FilterQuality.Low else FilterQuality.High,
                        onError = { failed = true }
                    )
                } else if (!isContinueWatchingCard && !isEpisodeStyle) {
                    Text(
                        text = meta.coverEmoji?.takeIf { it.isNotBlank() } ?: meta.name.take(1).uppercase(),
                        color = Color.White.copy(alpha = if (meta.coverEmoji.isNullOrBlank()) FluxaDimensions.Alpha.coverFallbackText else FluxaDimensions.Alpha.coverEmoji),
                        fontSize = if (meta.coverEmoji.isNullOrBlank()) FluxaDimensions.CardText.coverFallbackSize else FluxaDimensions.CardText.coverEmojiSize,
                        fontWeight = FontWeight.Black
                    )
                }
                if (loadArtwork && showLogo && logoRequest != null) {
                    Box(modifier = Modifier.fillMaxSize().background(MOBILE_CARD_BOTTOM_GRADIENT))
                    AsyncImage(
                        model = logoRequest,
                        contentDescription = null,
                        modifier = Modifier
                            .align(Alignment.BottomStart)
                            .padding(start = 9.dp, bottom = 9.dp)
                            .widthIn(max = width * 0.42f)
                            .heightIn(max = imageHeight * 0.30f),
                        contentScale = ContentScale.Fit
                    )
                }
                if (isProgressCard && !meta.isUpNextContinueItem() && progress > 0f) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.BottomCenter)
                            .fillMaxWidth()
                            .padding(horizontal = 8.dp, vertical = 6.dp)
                            .height(FluxaDimensions.cardProgressBarHeight)
                            .clip(RoundedCornerShape(999.dp))
                            .background(Color.Black.copy(alpha = 0.50f))
                    ) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(progress)
                                .fillMaxHeight()
                                .background(FluxaColors.progressFill)
                        )
                    }
                }
                if (isProgressCard && meta.isUpNextContinueItem()) {
                    Text(
                        text = AppStrings.t(lang, "auto.up_next"),
                        color = Color.White,
                        fontSize = FluxaDimensions.CardText.subtitleSize,
                        fontWeight = FontWeight.Black,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier
                            .align(Alignment.TopEnd)
                            .padding(8.dp)
                            .background(Color.Black.copy(alpha = FluxaDimensions.Alpha.upNextBadge))
                            .padding(horizontal = 8.dp, vertical = 5.dp)
                    )
                }
            }
            if (showTitle) {
                Text(
                    text = meta.name,
                    color = Color.White,
                    fontSize = FluxaDimensions.CardText.titleSize,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 6.dp)
                )
                if (secondaryText.isNotBlank()) {
                    Text(
                        text = secondaryText,
                        color = Color.White.copy(alpha = FluxaDimensions.Alpha.cardSubtitle),
                        fontSize = FluxaDimensions.CardText.subtitleSize,
                        fontWeight = FontWeight.Medium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(top = 1.dp)
                    )
                }
            }
        }
    }
}

private val MOBILE_CARD_BOTTOM_GRADIENT = Brush.verticalGradient(
    colors = listOf(Color.Transparent, Color.Black.copy(alpha = 0.6f)),
    startY = 150f
)


internal fun homeArtworkMemoryCacheKey(url: String?, width: Int, height: Int): String? {
    return url?.takeIf { it.isNotBlank() }?.let { "home-artwork:${width}x$height:$it" }
}

private fun String?.isNullOrSvgArtwork(): Boolean {
    val value = this?.substringBefore('?')?.substringBefore('#') ?: return true
    return value.endsWith(".svg", ignoreCase = true)
}
