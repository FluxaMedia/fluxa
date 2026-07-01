@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.foundation.ExperimentalFoundationApi::class, androidx.compose.ui.ExperimentalComposeUiApi::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.core.tween
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.*
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.*
import androidx.compose.ui.layout.*
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import coil3.imageLoader
import coil3.request.ImageRequest
import coil3.request.crossfade
import coil3.request.transformations
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

@Composable
internal fun TvMovieCard(
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
    isShelfStyle: Boolean = false,
    focusRequester: FocusRequester? = null,
    upFocusRequester: FocusRequester? = null,
    onExpandedChange: ((Boolean) -> Unit)? = null,
    onExpandedPositioned: ((LayoutCoordinates) -> Unit)? = null,
    onFocusedPositioned: ((LayoutCoordinates) -> Unit)? = null,
    onFocusChanged: ((Boolean) -> Unit)? = null,
    onResolveTrailer: (suspend (Meta) -> String?)? = null
) {
    val context = LocalContext.current
    var isFocused by remember { mutableStateOf(false) }
    var lastCardCoordinates by remember { mutableStateOf<LayoutCoordinates?>(null) }
    var pressStartTime by remember { mutableStateOf(0L) }
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
    val isEpisodeStyle = effectiveCardLayout == "episode"
    val radius = posterCornerRadius(profile?.safeCardCornerPreset ?: "soft")
    val animationDuration = when {
        profile?.safeAnimationsEnabled == false -> 0
        else -> if (isShelfStyle) 220 else 240
    }
    val hidePosterTitles = profile?.safePosterHideTitles == true || meta.hideTitle == true

    val width = (when {
        isEpisodeStyle -> if (isShelfStyle) 336.dp else 356.dp
        isHorizontal -> horizontalCardWidth(profile?.safePosterWidthPreset ?: "medium", DeviceType.TV)
        isShelfStyle -> posterCardWidth(profile?.safePosterWidthPreset ?: "medium") + 42.dp
        else -> 136.dp
    }) * cardScale
    val baseImageHeight = (when {
        isEpisodeStyle -> if (isShelfStyle) 210.dp else 208.dp
        isHorizontal -> horizontalCardHeight(profile?.safePosterWidthPreset ?: "medium", DeviceType.TV)
        isShelfStyle -> posterCardHeight(profile?.safePosterWidthPreset ?: "medium") + 64.dp
        else -> 204.dp
    }) * cardScale

    val expandedPostersEnabled = isShelfStyle && !isHorizontal && !isEpisodeStyle && profile?.safeExpandedPostersEnabled == true
    var expanded by remember { mutableStateOf(false) }
    LaunchedEffect(isFocused, expandedPostersEnabled) {
        if (isFocused && expandedPostersEnabled) {
            val expandedArtwork = preferredHorizontalArtwork(meta) ?: meta.poster
            if (!expandedArtwork.isNullOrBlank()) {
                launch {
                    context.imageLoader.execute(
                        ImageRequest.Builder(context)
                            .data(expandedArtwork)
                            .memoryCacheKey(expandedArtwork)
                            .diskCacheKey(expandedArtwork)
                            .size(640, 360)
                            .build()
                    )
                }
            }
            if (!meta.logo.isNullOrBlank()) {
                launch {
                    context.imageLoader.execute(
                        ImageRequest.Builder(context)
                            .data(meta.logo)
                            .memoryCacheKey("home-logo:${meta.logo}")
                            .diskCacheKey(meta.logo)
                            .build()
                    )
                }
            }
            val delaySeconds = profile?.safeExpandedPostersDelaySeconds ?: 2
            if (delaySeconds > 0) {
                delay(delaySeconds * 1000L)
            }
            expanded = true
        } else {
            expanded = false
        }
    }
    LaunchedEffect(expanded) { onExpandedChange?.invoke(expanded) }
    var expandedSettled by remember { mutableStateOf(false) }
    LaunchedEffect(expanded, animationDuration) {
        if (expanded) {
            delay(animationDuration.toLong() + 30L)
            expandedSettled = true
        } else {
            expandedSettled = false
        }
    }

    val trailerOnExpandedPostersEnabled = expandedPostersEnabled && profile?.safeTrailerOnExpandedPostersEnabled == true && onResolveTrailer != null
    var trailerUrl by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(expanded, trailerOnExpandedPostersEnabled) {
        if (expanded && trailerOnExpandedPostersEnabled) {
            val delaySeconds = profile?.safeTrailerOnExpandedPostersDelaySeconds ?: 3
            if (delaySeconds > 0) delay(delaySeconds * 1000L)
            trailerUrl = onResolveTrailer?.invoke(meta)
        } else {
            trailerUrl = null
        }
    }
    var trailerPlayer by remember { mutableStateOf<androidx.media3.exoplayer.ExoPlayer?>(null) }
    DisposableEffect(Unit) {
        onDispose { trailerPlayer?.release() }
    }
    LaunchedEffect(trailerUrl) {
        val url = trailerUrl
        if (url == null) {
            trailerPlayer?.stop()
            trailerPlayer?.clearMediaItems()
        } else {
            val playerInstance = trailerPlayer ?: androidx.media3.exoplayer.ExoPlayer.Builder(context).build().also { trailerPlayer = it }
            playerInstance.volume = 0f
            playerInstance.repeatMode = androidx.media3.common.Player.REPEAT_MODE_OFF
            playerInstance.setMediaItem(androidx.media3.common.MediaItem.fromUri(url))
            playerInstance.prepare()
            playerInstance.playWhenReady = true
        }
    }
    val imageHeight = baseImageHeight
    val effectiveWidth = if (expanded) imageHeight * 16f / 9f else width

    val isWideTopTenCard = isHorizontal || isEpisodeStyle
    val rankNumberBoxWidth = when {
        topTenRank == null -> 0.dp
        topTenRank >= 10 -> if (isWideTopTenCard) width * 0.98f else width * 1.18f
        topTenRank == 1 -> if (isWideTopTenCard) width * 0.46f else width * 0.54f
        else -> if (isWideTopTenCard) width * 0.72f else width * 0.86f
    }
    val rankPosterOverlap = when {
        topTenRank == null -> 0.dp
        topTenRank >= 10 -> if (isWideTopTenCard) width * 0.30f else width * 0.34f
        topTenRank == 1 -> if (isWideTopTenCard) width * 0.12f else width * 0.14f
        else -> if (isWideTopTenCard) width * 0.22f else width * 0.25f
    }
    val outerWidth = if (topTenRank != null) rankNumberBoxWidth + effectiveWidth - rankPosterOverlap else effectiveWidth
    val topTenFontSize = when {
        !isWideTopTenCard -> 226.sp
        isEpisodeStyle -> 158.sp
        else -> 150.sp
    }
    val topTenNumberYOffset = if (isWideTopTenCard) 1.dp else 2.dp

    Box(
        modifier = Modifier
            .width(outerWidth)
            .height(imageHeight)
            .zIndex(if (isFocused) 100f else 1f)
            .then(
                if (focusRequester != null || upFocusRequester != null) {
                    Modifier
                        .let { base -> if (focusRequester != null) base.focusRequester(focusRequester) else base }
                        .focusProperties { if (upFocusRequester != null) up = upFocusRequester }
                } else {
                    Modifier
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
                contentAlignment = Alignment.BottomEnd
            ) {
                TopTenRankNumber(
                    rank = rank,
                    fontSize = topTenFontSize,
                    modifier = Modifier.offset(
                        x = when {
                            rank == 1 -> 8.dp
                            rank >= 10 -> 0.dp
                            else -> 3.dp
                        },
                        y = topTenNumberYOffset
                    )
                )
            }
        }

        Surface(
            onClick = onClick,
            modifier = Modifier
                .align(Alignment.TopEnd)
                .width(effectiveWidth)
                .height(imageHeight)
                .onGloballyPositioned { coords ->
                    lastCardCoordinates = coords
                    if (expanded) onExpandedPositioned?.invoke(coords)
                    if (isFocused) onFocusedPositioned?.invoke(coords)
                }
                .onFocusChanged {
                    isFocused = it.isFocused
                    onFocusChanged?.invoke(it.isFocused)
                    if (it.isFocused) {
                        onFocus(meta)
                        lastCardCoordinates?.let { coords -> onFocusedPositioned?.invoke(coords) }
                    }
                }
                .onPreviewKeyEvent { event ->
                    val action = onProgressActions ?: onForgetProgress
                    if (action == null || (event.key != Key.DirectionCenter && event.key != Key.Enter)) return@onPreviewKeyEvent false
                    when (event.type) {
                        KeyEventType.KeyDown -> {
                            if (pressStartTime == 0L) pressStartTime = System.currentTimeMillis()
                            false
                        }
                        KeyEventType.KeyUp -> {
                            val heldMs = System.currentTimeMillis() - pressStartTime
                            pressStartTime = 0L
                            if (heldMs >= 500L) { action.invoke(); true } else false
                        }
                        else -> false
                    }
                },
            shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(radius)),
            colors = ClickableSurfaceDefaults.colors(containerColor = FluxaColors.surface, focusedContainerColor = FluxaColors.surfaceRaised),
            scale = ClickableSurfaceDefaults.scale(focusedScale = 1f)
        ) {
            Column(modifier = Modifier.fillMaxSize()) {
                val lang = profile?.safeLanguage ?: "en"
                val isUpcomingRelease = isUpcomingRelease(meta.released)
                val isProgressCard = (meta.timeOffset ?: 0L) > 0L && (meta.duration ?: 0L) > 0L
                val artwork = remember(meta.id, meta.poster, meta.background, meta.continueWatchingBackground, meta.focusGifUrl, effectiveCardLayout, artworkPreference, isFocused, expanded) {
                    when {
                        isEpisodeStyle -> {
                            val seriesArtwork = when (artworkPreference) {
                                "episode" -> meta.continueWatchingBackground
                                "poster" -> meta.poster
                                "background" -> meta.background
                                else -> meta.continueWatchingBackground
                            }
                            val resolved = if (meta.type == "series" || meta.type == "tv" || meta.type == "anime") seriesArtwork ?: meta.background else meta.background
                            resolved ?: meta.poster
                        }
                        isFocused && !meta.focusGifUrl.isNullOrBlank() && meta.type != "catalog_folder" -> meta.focusGifUrl
                        expanded -> preferredHorizontalArtwork(meta) ?: meta.poster
                        isHorizontal -> preferredHorizontalArtwork(meta) ?: meta.poster
                        else -> meta.poster
                    }
                }
                val requestWidth = if (isEpisodeStyle) 512 else if (isHorizontal || expanded) 640 else 320
                val requestHeight = if (isEpisodeStyle) 288 else if (isHorizontal || expanded) 360 else 480
                val request = remember(context, artwork, requestWidth, requestHeight) {
                    ImageRequest.Builder(context)
                        .data(artwork)
                        .crossfade(true)
                        .memoryCacheKey(artwork)
                        .diskCacheKey(artwork)
                        .placeholderMemoryCacheKey(meta.poster)
                        .size(requestWidth, requestHeight)
                        .build()
                }
                var failed by remember(meta.id, artwork) { mutableStateOf(artwork.isNullOrBlank()) }
                val showLogo = (isHorizontal || expandedSettled) && !isEpisodeStyle && showHorizontalLogo && meta.type != "catalog_folder" && (meta.timeOffset ?: 0L) <= 0L && !meta.logo.isNullOrBlank()
                val logoRequest = remember(meta.logo, showLogo) {
                    if (!showLogo) null else ImageRequest.Builder(context)
                        .data(meta.logo)
                        .crossfade(true)
                        .memoryCacheKey("home-logo:${meta.logo}")
                        .diskCacheKey(meta.logo)
                        .transformations(TrimTransparentEdgesTransformation())
                        .build()
                }

                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(imageHeight)
                        .clip(RoundedCornerShape(radius))
                        .background(if (isEpisodeStyle) FluxaColors.surfaceCard else Color.White.copy(alpha = FluxaDimensions.Alpha.emptyCardBackground))
                        .then(if (isFocused) Modifier.border(3.5.dp, Color(profile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt()), RoundedCornerShape(radius)) else Modifier)
                ) {
                    if (trailerUrl != null && trailerPlayer != null) {
                        androidx.compose.ui.viewinterop.AndroidView(
                            factory = { ctx ->
                                androidx.media3.ui.PlayerView(ctx).apply {
                                    useController = false
                                    resizeMode = androidx.media3.ui.AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                                    player = trailerPlayer
                                }
                            },
                            update = { it.player = trailerPlayer },
                            modifier = Modifier.fillMaxSize()
                        )
                    } else if (!failed) {
                        AsyncImage(
                            model = request,
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize().alpha(if (isUpcomingRelease && !isFocused) 0.6f else 1.0f),
                            contentScale = ContentScale.Crop,
                            onError = { failed = true }
                        )
                    } else {
                        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            Text(
                                text = meta.coverEmoji?.takeIf { it.isNotBlank() } ?: meta.name.take(1).uppercase(),
                                color = Color.White.copy(alpha = if (meta.coverEmoji.isNullOrBlank()) 0.2f else 0.82f),
                                fontSize = if (meta.coverEmoji.isNullOrBlank()) 48.sp else 42.sp,
                                fontWeight = FontWeight.Black
                            )
                        }
                    }

                    if (isEpisodeStyle) {
                        Box(
                            modifier = Modifier
                                .align(Alignment.BottomCenter)
                                .fillMaxWidth()
                                .height(112.dp)
                                .background(
                                    Brush.verticalGradient(
                                        0f to Color.Transparent,
                                        1f to Color.Black.copy(alpha = if (isFocused) 0.88f else 0.76f)
                                    )
                                )
                        )
                        if (!meta.isUpNextContinueItem() && isProgressCard) {
                            val progress = ((meta.timeOffset ?: 0L).toFloat() / (meta.duration ?: 1L).toFloat()).coerceIn(0f, 1f)
                            val remainingMs = ((meta.duration ?: 0L) - (meta.timeOffset ?: 0L)).coerceAtLeast(0L)
                            Column(modifier = Modifier.align(Alignment.BottomStart).fillMaxWidth().padding(12.dp)) {
                                Text(
                                    text = formatRemainingTime(remainingMs, lang),
                                    color = Color.White.copy(alpha = 0.82f),
                                    fontSize = 10.sp,
                                    fontWeight = FontWeight.Bold,
                                    modifier = Modifier.padding(bottom = 6.dp)
                                )
                                Box(modifier = Modifier.fillMaxWidth().height(4.dp).background(Color.White.copy(alpha = 0.20f))) {
                                    Box(modifier = Modifier.fillMaxWidth(progress).fillMaxHeight().background(Color.White))
                                }
                            }
                        }
                    } else {
                        if (showLogo || isProgressCard) {
                            Box(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = 0.6f)), startY = 150f))
                            )
                        }
                        if (showLogo && logoRequest != null) {
                            AsyncImage(
                                model = logoRequest,
                                contentDescription = null,
                                modifier = Modifier
                                    .align(Alignment.BottomStart)
                                    .padding(start = if (expanded) 16.dp else 9.dp, bottom = if (expanded) 16.dp else 9.dp)
                                    .widthIn(max = effectiveWidth * 0.42f)
                                    .heightIn(max = imageHeight * 0.30f),
                                contentScale = ContentScale.Fit
                            )
                        }
                        if (isProgressCard) {
                            val progress = ((meta.timeOffset ?: 0L).toFloat() / (meta.duration ?: 1L).toFloat()).coerceIn(0f, 1f)
                            val remainingMs = ((meta.duration ?: 0L) - (meta.timeOffset ?: 0L)).coerceAtLeast(0L)
                            Column(modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth().padding(8.dp)) {
                                Text(
                                    text = formatRemainingTime(remainingMs, lang),
                                    color = Color.White.copy(alpha = 0.7f),
                                    fontSize = 9.sp,
                                    fontWeight = FontWeight.Bold,
                                    modifier = Modifier.padding(bottom = 3.dp)
                                )
                                Box(modifier = Modifier.fillMaxWidth().height(3.dp).clip(RoundedCornerShape(1.5.dp)).background(Color.White.copy(alpha = 0.2f))) {
                                    Box(modifier = Modifier.fillMaxWidth(progress).fillMaxHeight().background(Color.White))
                                }
                            }
                        }
                    }
                }
                if (!isEpisodeStyle && !hidePosterTitles && !expanded) {
                    Text(
                        text = meta.name,
                        color = Color.White,
                        fontSize = 13.sp,
                        fontWeight = FontWeight.SemiBold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(top = 6.dp)
                    )
                    val secondaryText = meta.releaseInfo?.take(4) ?: meta.released?.take(4) ?: ""
                    if (secondaryText.isNotBlank()) {
                        Text(
                            text = secondaryText,
                            color = Color.White.copy(alpha = 0.62f),
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Medium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.padding(top = 1.dp)
                        )
                    }
                }
            }
        }

        if (isFocused && onFocusedPositioned == null) {
            Box(
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .width(effectiveWidth)
                    .height(imageHeight)
                    .zIndex(101f)
                    .padding(4.dp)
                    .border(3.5.dp, Color(profile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt()), RoundedCornerShape(radius))
            )
        }
    }
}
