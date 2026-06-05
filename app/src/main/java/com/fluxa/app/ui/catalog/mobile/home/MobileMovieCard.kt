@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.foundation.ExperimentalFoundationApi::class, androidx.compose.ui.ExperimentalComposeUiApi::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Intent
import android.graphics.drawable.BitmapDrawable
import android.net.Uri
import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerDefaults
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.*
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.key.*
import androidx.compose.ui.layout.*
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items as mobileGridItems
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import coil3.decode.StaticImageDecoder
import coil3.imageLoader
import coil3.request.ImageRequest
import coil3.request.allowHardware
import coil3.request.crossfade
import coil3.size.Size
import coil3.size.Precision
import coil3.size.Scale
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.distinctUntilChanged
import androidx.compose.runtime.snapshotFlow
import java.text.SimpleDateFormat
import java.util.*
import kotlin.math.abs

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
    val deviceType = LocalDeviceType.current
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
    val isHorizontal = effectiveCardLayout == "horizontal"
    val isSquare = effectiveCardLayout == "square"
    val isEpisodeStyle = effectiveCardLayout == "episode"
    val isCatalogFolder = meta.type == "catalog_folder"
    val isProgressCard = meta.isUpNextContinueItem() || ((meta.timeOffset ?: 0L) > 0L && (meta.duration ?: 0L) > 0L)

    if (deviceType == DeviceType.Mobile && isCatalogFolder && !isContinueWatchingCard && topTenRank == null && !isProgressCard) {
        LeanCatalogFolderCard(
            meta = meta,
            profile = profile,
            cardLayout = effectiveCardLayout,
            cardScale = cardScale,
            loadArtwork = loadArtwork,
            onClick = onClick
        )
        return
    }

    if (deviceType == DeviceType.Mobile && isContinueWatchingCard && profile?.safeContinueWatchingHideTitles != true && topTenRank == null) {
        LeanContinueWatchingCard(
            meta = meta,
            profile = profile,
            cardLayout = effectiveCardLayout,
            artworkPreference = artworkPreference,
            cardScale = cardScale,
            loadArtwork = loadArtwork,
            onClick = onClick,
            onProgressActions = onProgressActions
        )
        return
    }

    if (deviceType == DeviceType.Mobile && topTenRank != null && !isHorizontal && !isSquare && !isEpisodeStyle &&
        !isCatalogFolder && !isContinueWatchingCard && !isProgressCard) {
        LeanTopTenPosterCard(
            meta = meta,
            profile = profile,
            rank = topTenRank,
            cardScale = cardScale,
            loadArtwork = loadArtwork,
            onClick = onClick
        )
        return
    }

    // Fast path for the common case on mobile: plain movie/show cards do not need the rich
    // MovieCardContent overlay/logo stack. This also covers landscape poster mode.
    if (deviceType == DeviceType.Mobile && !isEpisodeStyle &&
        !isCatalogFolder && !isContinueWatchingCard && topTenRank == null && !isProgressCard) {
        LeanPlainMovieCard(
            meta = meta,
            profile = profile,
            cardLayout = effectiveCardLayout,
            cardScale = cardScale,
            loadArtwork = loadArtwork,
            onClick = onClick
        )
        return
    }

    val density = LocalDensity.current
    var isFocused by remember { mutableStateOf(false) }
    var showForgetOverlay by remember { mutableStateOf(false) }
    val radius = if (isContinueWatchingCard) 0.dp else posterCornerRadius(profile?.safeCardCornerPreset ?: "soft")
    val isUpcomingRelease = remember(meta.released) { isUpcoming(meta.released) }
    val animationDuration = when {
        profile?.safeAnimationsEnabled == false -> 0
        else -> FluxaDimensions.AnimDuration.cardFocusScale
    }
    val cardInteractionSource = remember { MutableInteractionSource() }
    val focusedScale = FluxaDimensions.cardFocusedScale
    val hidePosterTitles = profile?.safePosterHideTitles == true || meta.hideTitle == true
    val hideContinueWatchingTitles = isContinueWatchingCard && profile?.safeContinueWatchingHideTitles == true
    val showBottomTitle = deviceType == DeviceType.Mobile &&
        !hideContinueWatchingTitles &&
        (isProgressCard || (!isEpisodeStyle && !hidePosterTitles))
    val lang = profile?.safeLanguage ?: "en"
    val cardIndication = if (deviceType == DeviceType.Mobile) null else LocalIndication.current
    // Continue-watching labels (continueWatchingEpisodeTitle compiles a Regex per call) are computed
    // only for progress/continue-watching cards; skipped entirely for plain posters.
    val needsContinueLabels = isProgressCard || isContinueWatchingCard
    val episodeLabel = remember(meta.id, meta.lastVideoId, meta.lastEpisodeName, needsContinueLabels) {
        if (needsContinueLabels) continueWatchingEpisodeLabel(meta) else null
    }
    val continueEpisodeTitle = remember(meta.id, meta.lastEpisodeName, needsContinueLabels) {
        if (needsContinueLabels) continueWatchingEpisodeTitle(meta) else null
    }
    val continueProgressLabel = remember(meta.id, meta.timeOffset, meta.duration, lang, needsContinueLabels) {
        if (needsContinueLabels) continueWatchingEpisodeProgressLabel(meta, lang) else null
    }

    val widthPreset = profile?.safePosterWidthPreset ?: "medium"

    val width = remember(effectiveCardLayout, cardScale, widthPreset, deviceType) {
        (if (deviceType == DeviceType.TV) {
            when { isEpisodeStyle -> FluxaDimensions.EpisodeCard.tvWidth; isHorizontal -> horizontalCardWidth(widthPreset, DeviceType.TV); else -> FluxaDimensions.TvPosterCard.width }
        } else {
            when { isEpisodeStyle -> FluxaDimensions.EpisodeCard.mobileWidth; isHorizontal -> horizontalCardWidth(widthPreset, DeviceType.Mobile); else -> posterCardWidth(widthPreset) }
        }) * cardScale
    }

    val imageHeight = remember(effectiveCardLayout, cardScale, widthPreset, deviceType) {
        (if (deviceType == DeviceType.TV) {
            when { isEpisodeStyle -> FluxaDimensions.EpisodeCard.tvHeight; isHorizontal -> horizontalCardHeight(widthPreset, DeviceType.TV); else -> FluxaDimensions.TvPosterCard.height }
        } else {
            when { isEpisodeStyle -> FluxaDimensions.EpisodeCard.mobileHeight; isHorizontal -> horizontalCardHeight(widthPreset, DeviceType.Mobile); isSquare -> posterCardWidth(widthPreset); else -> posterCardHeight(widthPreset) }
        }) * cardScale
    }

    val metaHeight = if (showBottomTitle) { if (isProgressCard && !episodeLabel.isNullOrBlank()) FluxaDimensions.cardMetaBarWithEpisodeLabelHeight else FluxaDimensions.cardMetaBarHeight } else 0.dp
    val height = imageHeight + metaHeight
    val rankBase = if (isHorizontal || isEpisodeStyle) imageHeight else width
    val rankNumberBoxWidth = topTenRank?.let { r -> when { r >= 10 -> rankBase * 1.24f; r == 1 -> rankBase * 0.62f; else -> rankBase * 0.82f } } ?: 0.dp
    val rankPosterOverlap = topTenRank?.let { r -> when { r >= 10 -> rankBase * 0.16f; r == 1 -> rankBase * 0.13f; else -> rankBase * 0.24f } } ?: 0.dp
    val outerWidth = if (topTenRank != null) rankNumberBoxWidth + width - rankPosterOverlap else width
    val rankFontSize = remember(topTenRank, imageHeight, isHorizontal, isEpisodeStyle, density) {
        if (topTenRank != null) with(density) { (imageHeight.toPx() * if (isHorizontal || isEpisodeStyle) 0.86f else 0.90f).toSp() } else 0.sp
    }

    // Focus scale is TV-only. Calling animateFloatAsState only on TV avoids per-card animation
    // state on mobile. deviceType is constant per app, so the conditional @Composable call is safe.
    val focusLayerModifier = if (deviceType == DeviceType.TV) {
        val scale by animateFloatAsState(
            targetValue = if (isFocused) focusedScale else 1.0f,
            animationSpec = tween(animationDuration),
            label = "scale"
        )
        Modifier
            .graphicsLayer {
                scaleX = scale
                scaleY = scale
            }
            .zIndex(if (isFocused) 10f else 1f)
    } else {
        Modifier
    }

    Box(
        modifier = Modifier
            .width(outerWidth)
            .height(height)
            .then(focusLayerModifier)
            .combinedClickable(
                interactionSource = cardInteractionSource,
                indication = cardIndication,
                onClick = {
                    if (showForgetOverlay) {
                        showForgetOverlay = false
                    } else {
                        onClick()
                    }
                },
                onLongClick = {
                    if (onProgressActions != null && (meta.timeOffset ?: 0L) > 0L) {
                        onProgressActions()
                    } else if (onForgetProgress != null && (meta.timeOffset ?: 0L) > 0L) {
                        showForgetOverlay = true
                    }
                }
            )
    ) {
        topTenRank?.let { rank ->
            Box(
                modifier = Modifier
                    .align(Alignment.TopStart)
                    .width(rankNumberBoxWidth)
                    .height(imageHeight)
                    .zIndex(0f),
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
                        y = if (isHorizontal || isEpisodeStyle) 1.dp else 2.dp
                    )
                )
            }
        }

        Column(
            modifier = Modifier
                .width(width)
                .height(height)
                .align(Alignment.TopEnd)
                .zIndex(1f)
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(imageHeight)
                // No clip here: MovieCardContent already clips with the same radius (avoid a double graphics layer).
            ) {
                MovieCardContent(
                    movie = meta,
                    isUpcoming = isUpcomingRelease,
                    isFocused = isFocused,
                    cardLayout = effectiveCardLayout,
                    artworkPreference = artworkPreference,
                    cornerRadius = radius,
                    hideTitles = hidePosterTitles,
                    showHorizontalLogo = showHorizontalLogo,
                    lang = profile?.safeLanguage,
                    isContinueWatchingCard = isContinueWatchingCard,
                    hideContinueWatchingTitles = hideContinueWatchingTitles,
                    contentWidth = width,
                    contentHeight = imageHeight,
                    loadArtwork = loadArtwork
                )
                if (isFocused && meta.focusGlowEnabled == true) {
                    Box(
                        modifier = Modifier
                            .matchParentSize()
                            .border(2.dp, Color(profile?.colorArgb ?: 0xFFFFFFFF.toInt()), RoundedCornerShape(radius))
                    )
                }
            }

            if (showBottomTitle) {
                Text(
                    text = meta.name,
                    color = Color.White,
                    fontSize = FluxaDimensions.CardText.titleSize,
                    fontWeight = FontWeight.Black,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 6.dp)
                )
                val secondaryText = when {
                    isCatalogFolder -> ""
                    isProgressCard -> episodeLabel.orEmpty()
                    else -> meta.releaseInfo?.take(4) ?: meta.released?.take(4) ?: ""
                }
                if (!secondaryText.isNullOrBlank()) {
                    Text(
                        text = secondaryText,
                        color = Color.White.copy(alpha = FluxaDimensions.Alpha.cardSubtitle),
                        fontSize = FluxaDimensions.CardText.subtitleSize,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(top = 1.dp)
                    )
                }
            }
        }

        AnimatedVisibility(
            visible = showForgetOverlay,
            enter = fadeIn(),
            exit = fadeOut(),
            modifier = Modifier
                .align(Alignment.TopEnd)
                .padding(8.dp)
                .zIndex(2f)
        ) {
            Row(
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(Color(0xE0181B20))
                    .border(1.dp, Color.White.copy(alpha = 0.16f), RoundedCornerShape(999.dp))
                    .clickable {
                        showForgetOverlay = false
                        onForgetProgress?.invoke()
                    }
                    .padding(horizontal = 10.dp, vertical = 7.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                Icon(FluxaIcons.DeleteOutline, null, tint = Color.White, modifier = Modifier.size(16.dp))
                Text(AppStrings.t(lang, "common.forget"), color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.Bold)
            }
        }
    }
}

// Lean card for plain movie/show cards: bypasses the heavy MovieCardContent/CardBody chain with
// just Column + Box + AsyncImage + 2 Text, cutting per-card compose/allocation cost on fling.
@Composable
private fun LeanPlainMovieCard(
    meta: Meta,
    profile: UserProfile?,
    cardLayout: String,
    cardScale: Float,
    loadArtwork: Boolean,
    onClick: () -> Unit
) {
    val context = LocalContext.current
    val widthPreset = profile?.safePosterWidthPreset ?: "medium"
    val isHorizontal = cardLayout == "horizontal"
    val isSquare = cardLayout == "square"
    val width = (if (isHorizontal) horizontalCardWidth(widthPreset, DeviceType.Mobile) else posterCardWidth(widthPreset)) * cardScale
    val imageHeight = (when {
        isHorizontal -> horizontalCardHeight(widthPreset, DeviceType.Mobile)
        isSquare -> posterCardWidth(widthPreset)
        else -> posterCardHeight(widthPreset)
    }) * cardScale
    val showTitle = !(profile?.safePosterHideTitles == true || meta.hideTitle == true)
    val artwork = remember(meta.id, meta.poster, meta.background, cardLayout) {
        if (isHorizontal) preferredHorizontalArtwork(meta) ?: meta.poster else meta.poster
    }
    val requestWidth = if (isHorizontal) 512 else 288
    val requestHeight = when {
        isHorizontal -> 288
        isSquare -> 288
        else -> 432
    }
    val request = remember(artwork, requestWidth, requestHeight) {
        ImageRequest.Builder(context)
            .data(artwork)
            .crossfade(false)
            .memoryCacheKey(homeArtworkMemoryCacheKey(artwork, requestWidth, requestHeight))
            .diskCacheKey(artwork)
            .size(requestWidth, requestHeight)
            .build()
    }
    var failed by remember(meta.id, artwork) { mutableStateOf(artwork.isNullOrBlank()) }

    Column(
        modifier = Modifier
            .width(width)
            .clickable(
                interactionSource = null,
                indication = null,
                onClick = onClick
            )
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(imageHeight)
                .background(Color.White.copy(alpha = FluxaDimensions.Alpha.emptyCardBackground)),
            contentAlignment = Alignment.Center
        ) {
            if (loadArtwork && !failed) {
                AsyncImage(
                    model = request,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                    onError = { failed = true }
                )
            } else {
                Text(
                    text = meta.coverEmoji?.takeIf { it.isNotBlank() } ?: meta.name.take(1).uppercase(),
                    color = Color.White.copy(alpha = if (meta.coverEmoji.isNullOrBlank()) FluxaDimensions.Alpha.coverFallbackText else FluxaDimensions.Alpha.coverEmoji),
                    fontSize = if (meta.coverEmoji.isNullOrBlank()) FluxaDimensions.CardText.coverFallbackSize else FluxaDimensions.CardText.coverEmojiSize,
                    fontWeight = FontWeight.Black
                )
            }
        }
        if (showTitle) {
            Text(
                text = meta.name,
                color = Color.White,
                fontSize = FluxaDimensions.CardText.titleSize,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 6.dp)
            )
            val secondary = meta.releaseInfo?.take(4) ?: meta.released?.take(4) ?: ""
            if (secondary.isNotBlank()) {
                Text(
                    text = secondary,
                    color = Color.White.copy(alpha = FluxaDimensions.Alpha.cardSubtitle),
                    fontSize = FluxaDimensions.CardText.subtitleSize,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 1.dp)
                )
            }
        }
    }
}

@Composable
private fun LeanCatalogFolderCard(
    meta: Meta,
    profile: UserProfile?,
    cardLayout: String,
    cardScale: Float,
    loadArtwork: Boolean,
    onClick: () -> Unit
) {
    val context = LocalContext.current
    val widthPreset = profile?.safePosterWidthPreset ?: "medium"
    val isHorizontal = cardLayout == "horizontal"
    val isSquare = cardLayout == "square"
    val width = when {
        isHorizontal -> horizontalCardWidth(widthPreset, DeviceType.Mobile)
        else -> posterCardWidth(widthPreset)
    } * cardScale
    val imageHeight = when {
        isHorizontal -> horizontalCardHeight(widthPreset, DeviceType.Mobile)
        isSquare -> posterCardWidth(widthPreset)
        else -> posterCardHeight(widthPreset)
    } * cardScale
    val showTitle = !(profile?.safePosterHideTitles == true || meta.hideTitle == true)
    val requestWidth = if (isHorizontal) 384 else 224
    val requestHeight = when {
        isHorizontal -> 216
        isSquare -> 224
        else -> 336
    }
    val request = remember(meta.poster, requestWidth, requestHeight) {
        val builder = ImageRequest.Builder(context)
            .data(meta.poster)
            .crossfade(false)
            .allowHardware(false)
            .memoryCacheKey(homeArtworkMemoryCacheKey(meta.poster, requestWidth, requestHeight))
            .diskCacheKey(meta.poster)
            .precision(Precision.INEXACT)
            .scale(Scale.FILL)
            .size(requestWidth, requestHeight)
        if (!meta.poster.isNullOrSvgArtwork()) {
            builder.decoderFactory(StaticImageDecoder.Factory())
        }
        builder.build()
    }
    var failed by remember(meta.id, meta.poster) { mutableStateOf(meta.poster.isNullOrBlank()) }

    Column(
        modifier = Modifier
            .width(width)
            .clickable(
                interactionSource = null,
                indication = null,
                onClick = onClick
            )
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(imageHeight)
                .background(Color.White.copy(alpha = FluxaDimensions.Alpha.emptyCardBackground)),
            contentAlignment = Alignment.Center
        ) {
            if (loadArtwork && !failed) {
                AsyncImage(
                    model = request,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                    filterQuality = FilterQuality.Low,
                    onError = { failed = true }
                )
            } else {
                Text(
                    text = meta.coverEmoji?.takeIf { it.isNotBlank() } ?: meta.name.take(1).uppercase(),
                    color = Color.White.copy(alpha = if (meta.coverEmoji.isNullOrBlank()) FluxaDimensions.Alpha.coverFallbackText else FluxaDimensions.Alpha.coverEmoji),
                    fontSize = if (meta.coverEmoji.isNullOrBlank()) FluxaDimensions.CardText.coverFallbackSize else FluxaDimensions.CardText.coverEmojiSize,
                    fontWeight = FontWeight.Black
                )
            }
        }
        if (showTitle) {
            Text(
                text = meta.name,
                color = Color.White,
                fontSize = FluxaDimensions.CardText.titleSize,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 6.dp)
            )
            val secondary = meta.releaseInfo?.take(4) ?: ""
            if (secondary.isNotBlank()) {
                Text(
                    text = secondary,
                    color = Color.White.copy(alpha = FluxaDimensions.Alpha.cardSubtitle),
                    fontSize = FluxaDimensions.CardText.subtitleSize,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 1.dp)
                )
            }
        }
    }
}

@Composable
private fun LeanContinueWatchingCard(
    meta: Meta,
    profile: UserProfile?,
    cardLayout: String,
    artworkPreference: String?,
    cardScale: Float,
    loadArtwork: Boolean,
    onClick: () -> Unit,
    onProgressActions: (() -> Unit)?
) {
    val context = LocalContext.current
    val widthPreset = profile?.safePosterWidthPreset ?: "medium"
    val isEpisodeStyle = cardLayout == "episode"
    val isHorizontal = cardLayout == "horizontal" || isEpisodeStyle
    val width = (if (isEpisodeStyle) FluxaDimensions.EpisodeCard.mobileWidth else if (isHorizontal) horizontalCardWidth(widthPreset, DeviceType.Mobile) else posterCardWidth(widthPreset)) * cardScale
    val imageHeight = (if (isEpisodeStyle) FluxaDimensions.EpisodeCard.mobileHeight else if (isHorizontal) horizontalCardHeight(widthPreset, DeviceType.Mobile) else posterCardHeight(widthPreset)) * cardScale
    val lang = profile?.safeLanguage ?: "en"
    val progress = ((meta.timeOffset ?: 0L).toFloat() / (meta.duration ?: 1L).toFloat()).coerceIn(0f, 1f)
    val isProgressCard = meta.isUpNextContinueItem() || ((meta.timeOffset ?: 0L) > 0L && (meta.duration ?: 0L) > 0L)
    val title = meta.name
    val secondary = remember(meta.id, meta.lastVideoId, meta.lastEpisodeName, isProgressCard) {
        if (isProgressCard) continueWatchingEpisodeLabel(meta).orEmpty() else meta.releaseInfo?.take(4).orEmpty()
    }
    val artwork = remember(meta.id, meta.poster, meta.background, meta.continueWatchingBackground, artworkPreference) {
        when (artworkPreference) {
            "poster" -> meta.poster ?: meta.continueWatchingBackground ?: meta.background
            "background" -> meta.background ?: meta.continueWatchingBackground ?: meta.poster
            else -> meta.continueWatchingBackground ?: meta.background ?: meta.poster
        }
    }
    val requestWidth = if (isHorizontal) 512 else 288
    val requestHeight = if (isHorizontal) 288 else 432
    val request = remember(context, artwork, requestWidth, requestHeight) {
        ImageRequest.Builder(context)
            .data(artwork)
            .crossfade(false)
            .memoryCacheKey(homeArtworkMemoryCacheKey(artwork, requestWidth, requestHeight))
            .diskCacheKey(artwork)
            .size(requestWidth, requestHeight)
            .build()
    }
    Column(
        modifier = Modifier
            .width(width)
            .combinedClickable(
                interactionSource = null,
                indication = null,
                onClick = onClick,
                onLongClick = {
                    if ((meta.timeOffset ?: 0L) > 0L) onProgressActions?.invoke()
                }
            )
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(imageHeight)
                .background(Color(0xFF141922)),
            contentAlignment = Alignment.Center
        ) {
            if (loadArtwork) {
                AsyncImage(
                    model = request,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
            }
            if (!meta.isUpNextContinueItem() && progress > 0f) {
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
                            .background(Color(0xFFE50914))
                    )
                }
            }
            if (meta.isUpNextContinueItem()) {
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
        Text(
            text = title,
            color = Color.White,
            fontSize = FluxaDimensions.CardText.titleSize,
            fontWeight = FontWeight.Black,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(top = 6.dp)
        )
        if (secondary.isNotBlank()) {
            Text(
                text = secondary,
                color = Color.White.copy(alpha = FluxaDimensions.Alpha.cardSubtitle),
                fontSize = FluxaDimensions.CardText.subtitleSize,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(top = 1.dp)
            )
        }
    }
}

@Composable
private fun LeanTopTenPosterCard(
    meta: Meta,
    profile: UserProfile?,
    rank: Int,
    cardScale: Float,
    loadArtwork: Boolean,
    onClick: () -> Unit
) {
    val context = LocalContext.current
    val density = LocalDensity.current
    val widthPreset = profile?.safePosterWidthPreset ?: "medium"
    val width = posterCardWidth(widthPreset) * cardScale
    val imageHeight = posterCardHeight(widthPreset) * cardScale
    val hideTitle = profile?.safePosterHideTitles == true || meta.hideTitle == true
    val request = remember(context, meta.poster) {
        ImageRequest.Builder(context)
            .data(meta.poster)
            .crossfade(false)
            .memoryCacheKey(homeArtworkMemoryCacheKey(meta.poster, 288, 432))
            .diskCacheKey(meta.poster)
            .size(288, 432)
            .build()
    }
    val rankBase = width
    val rankNumberBoxWidth = when {
        rank >= 10 -> rankBase * 1.24f
        rank == 1 -> rankBase * 0.62f
        else -> rankBase * 0.82f
    }
    val rankPosterOverlap = when {
        rank >= 10 -> rankBase * 0.16f
        rank == 1 -> rankBase * 0.13f
        else -> rankBase * 0.24f
    }
    val outerWidth = rankNumberBoxWidth + width - rankPosterOverlap
    val rankFontSize = remember(imageHeight, density) { with(density) { (imageHeight.toPx() * 0.90f).toSp() } }

    Box(
        modifier = Modifier
            .width(outerWidth)
            .height(imageHeight + if (hideTitle) 0.dp else 42.dp)
            .clickable(
                interactionSource = null,
                indication = null,
                onClick = onClick
            )
    ) {
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
                    y = 2.dp
                )
            )
        }
        Column(
            modifier = Modifier
                .align(Alignment.TopEnd)
                .width(width)
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(imageHeight)
                    .background(Color.White.copy(alpha = FluxaDimensions.Alpha.emptyCardBackground))
            ) {
                if (loadArtwork) {
                    AsyncImage(
                        model = request,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop
                    )
                }
            }
            if (!hideTitle) {
                Text(
                    text = meta.name,
                    color = Color.White,
                    fontSize = FluxaDimensions.CardText.titleSize,
                    fontWeight = FontWeight.Black,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 6.dp)
                )
                val secondary = meta.releaseInfo?.take(4) ?: meta.released?.take(4) ?: ""
                if (secondary.isNotBlank()) {
                    Text(
                        text = secondary,
                        color = Color.White.copy(alpha = FluxaDimensions.Alpha.cardSubtitle),
                        fontSize = FluxaDimensions.CardText.subtitleSize,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(top = 1.dp)
                    )
                }
            }
        }
    }
}

internal fun homeArtworkMemoryCacheKey(url: String?, width: Int, height: Int): String? {
    return url?.takeIf { it.isNotBlank() }?.let { "home-artwork:${width}x$height:$it" }
}

private fun String?.isNullOrSvgArtwork(): Boolean {
    val value = this?.substringBefore('?')?.substringBefore('#') ?: return true
    return value.endsWith(".svg", ignoreCase = true)
}
