@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.CompositingStrategy
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.LayoutCoordinates
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import coil3.imageLoader
import coil3.request.ImageRequest
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import androidx.tv.material3.Text
import coil3.compose.AsyncImage
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta

@Composable
internal fun ContinueWatchingHighlight(
    movie: Meta,
    lang: String,
    label: String,
    focusRequester: FocusRequester? = null,
    upFocusRequester: FocusRequester? = null,
    onClick: () -> Unit,
    onFocus: () -> Unit
) {
    Column(modifier = Modifier.padding(start = 42.dp, top = 12.dp, bottom = 8.dp)) {
        Text(
            text = label,
            color = Color.White,
            fontSize = 30.sp,
            fontFamily = FluxaDisplay,
            fontWeight = FontWeight.SemiBold
        )
        Spacer(modifier = Modifier.height(14.dp))
        WideContinueCard(
            movie = movie,
            lang = lang,
            focusRequester = focusRequester,
            upFocusRequester = upFocusRequester,
            onClick = onClick,
            onFocus = onFocus
        )
    }
}

@Composable
private fun WideContinueCard(
    movie: Meta,
    lang: String,
    focusRequester: FocusRequester? = null,
    upFocusRequester: FocusRequester? = null,
    onClick: () -> Unit,
    onFocus: () -> Unit
) {
    var isFocused by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isFocused) 1.035f else 1f, tween(FluxaDimensions.AnimDuration.scaleAlpha), label = "continueScale")

    Row(
        modifier = Modifier
            .width(360.dp)
            .clip(RoundedCornerShape(18.dp))
            .background(Color.White.copy(alpha = 0.07f))
            .border(
                width = if (isFocused) 2.dp else 1.dp,
                color = if (isFocused) Color(0xFF9AD4FF) else Color.White.copy(alpha = 0.08f),
                shape = RoundedCornerShape(18.dp)
            )
            .graphicsLayer {
                scaleX = scale
                scaleY = scale
            }
            .then(
                if (focusRequester != null || upFocusRequester != null) {
                    Modifier
                        .let { base -> if (focusRequester != null) base.focusRequester(focusRequester) else base }
                        .focusProperties {
                            if (upFocusRequester != null) up = upFocusRequester
                        }
                } else {
                    Modifier
                }
            )
            .onFocusChanged {
                isFocused = it.isFocused
                if (it.isFocused) onFocus()
            }
            .clickable { onClick() }
            .padding(8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        AsyncImage(
            model = movie.continueWatchingBackground,
            contentDescription = movie.name,
            modifier = Modifier
                .width(156.dp)
                .height(88.dp)
                .clip(RoundedCornerShape(14.dp)),
            contentScale = ContentScale.Crop
        )
        Spacer(modifier = Modifier.width(14.dp))
        Column(modifier = Modifier.weight(1f)) {
            movie.lastEpisodeName?.takeIf { it.isNotBlank() }?.let {
                Text(
                    text = it,
                    color = Color.White.copy(alpha = 0.72f),
                    fontSize = 13.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Spacer(modifier = Modifier.height(2.dp))
            }
            Text(
                text = movie.name,
                color = Color.White,
                fontSize = 23.sp,
                fontWeight = FontWeight.Medium,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )
            Spacer(modifier = Modifier.height(8.dp))
            val remaining = movie.remainingWatchLabel(lang)
            if (remaining != null) {
                HeroBadge(text = remaining, background = Color(0xFF4A6AFB), content = Color.White)
            }
        }
    }
}

@Composable
internal fun HomeShelfRow(
    title: String,
    items: List<Meta>,
    cardLayout: String,
    artworkPreference: String?,
    activeProfile: UserProfile?,
    topTenEnabled: Boolean,
    onItemClick: (Meta) -> Unit,
    onItemFocus: (Meta) -> Unit,
    firstItemFocusRequester: FocusRequester? = null,
    upFocusRequester: FocusRequester? = null,
    onNavigateUp: (() -> Unit)? = null,
    onNeedMore: () -> Unit,
    onResolveTrailer: (suspend (Meta) -> String?)? = null,
    titleStartPadding: Dp = 42.dp
) {
    var expandedMeta by remember { mutableStateOf<Meta?>(null) }
    var rowCoordinates by remember { mutableStateOf<LayoutCoordinates?>(null) }
    var expandedOffsetX by remember { mutableStateOf(42.dp) }
    var rowWidth by remember { mutableStateOf(0.dp) }
    val density = LocalDensity.current
    val instantExpand = cardLayout != "horizontal" && cardLayout != "episode" &&
        activeProfile?.safeExpandedPostersEnabled == true &&
        (activeProfile.safeExpandedPostersDelaySeconds) == 0
    val expandedPostersEnabled = cardLayout != "horizontal" && cardLayout != "episode" &&
        activeProfile?.safeExpandedPostersEnabled == true
    val lazyListState = rememberLazyListState()
    val internalFirstCardFocusRequester = remember { FocusRequester() }
    val firstCardFocusRequester = firstItemFocusRequester ?: internalFirstCardFocusRequester
    var focusedIndex by remember { mutableStateOf(-1) }
    LaunchedEffect(focusedIndex, instantExpand) {
        if (focusedIndex < 0) return@LaunchedEffect
        snapshotFlow { expandedMeta }.first { it == null }
        if (instantExpand) {
            lazyListState.scrollToItem(focusedIndex, 0)
        } else {
            lazyListState.animateScrollToItem(focusedIndex, 0)
        }
    }
    val context = LocalContext.current
    LaunchedEffect(focusedIndex, expandedPostersEnabled) {
        if (!expandedPostersEnabled || focusedIndex < 0) return@LaunchedEffect
        listOf(focusedIndex - 1, focusedIndex + 1, focusedIndex + 2).forEach { neighborIndex ->
            val neighbor = items.getOrNull(neighborIndex) ?: return@forEach
            val artwork = preferredHorizontalArtwork(neighbor) ?: neighbor.poster
            if (!artwork.isNullOrBlank()) {
                launch {
                    context.imageLoader.execute(
                        ImageRequest.Builder(context)
                            .data(artwork)
                            .memoryCacheKey(homeArtworkMemoryCacheKey(artwork, 640, 360))
                            .diskCacheKey(artwork)
                            .size(640, 360)
                            .build()
                    )
                }
            }
        }
    }
    Column(
        modifier = Modifier
            .padding(top = 18.dp)
            .then(
                if (onNavigateUp != null) Modifier.onPreviewKeyEvent { event ->
                    if (event.type == KeyEventType.KeyDown && event.key == Key.DirectionUp) {
                        onNavigateUp()
                        true
                    } else false
                } else Modifier
            )
    ) {
        Text(
            text = title,
            color = Color.White,
            fontSize = 22.sp,
            fontFamily = FluxaDisplay,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier
                .height(32.dp)
                .padding(start = titleStartPadding)
        )
        Spacer(modifier = Modifier.height(20.dp))
        val rowHeight = when (cardLayout) {
            "episode" -> 226.dp
            "horizontal" -> horizontalCardHeight(activeProfile?.safePosterWidthPreset ?: "medium", DeviceType.TV) + 24.dp
            else -> posterCardHeight(activeProfile?.safePosterWidthPreset ?: "medium") + 82.dp
        }
        Box(modifier = Modifier.fillMaxWidth().height(rowHeight)) {
            val reentryIndex = focusedIndex.coerceAtLeast(0)
            LazyRow(
                state = lazyListState,
                modifier = Modifier
                    .fillMaxSize()
                    .onGloballyPositioned { coordinates ->
                        rowCoordinates = coordinates
                        rowWidth = with(density) { coordinates.size.width.toDp() }
                    }
                    .graphicsLayer {
                        compositingStrategy = if (lazyListState.canScrollBackward) CompositingStrategy.Offscreen else CompositingStrategy.Auto
                    }
                    .drawWithContent {
                        drawContent()
                        if (lazyListState.canScrollBackward) {
                            drawRect(
                                brush = Brush.horizontalGradient(
                                    0f to Color.Transparent,
                                    1f to Color.Black,
                                    endX = titleStartPadding.toPx()
                                ),
                                size = androidx.compose.ui.geometry.Size(titleStartPadding.toPx(), size.height),
                                blendMode = BlendMode.DstIn
                            )
                        }
                    },
                contentPadding = PaddingValues(start = titleStartPadding, end = 50.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.Top
            ) {
                itemsIndexed(items, key = { _, movie -> "${movie.type}:${movie.id}" }) { index, movie ->
                    if (index >= items.lastIndex - 4) {
                        LaunchedEffect(title, index) { onNeedMore() }
                    }
                    TvMovieCard(
                        meta = movie,
                        onFocus = {
                            onItemFocus(movie)
                            focusedIndex = index
                        },
                        onClick = { onItemClick(movie) },
                        cardLayout = cardLayout,
                        artworkPreference = artworkPreference,
                        profile = activeProfile,
                        topTenRank = if (topTenEnabled && index < 10) index + 1 else null,
                        isShelfStyle = true,
                        focusRequester = if (index == reentryIndex) firstCardFocusRequester else null,
                        upFocusRequester = if (index == 0) upFocusRequester else null,
                        onExpandedChange = { isExpanded ->
                            expandedMeta = when {
                                isExpanded -> movie
                                expandedMeta?.id == movie.id && expandedMeta?.type == movie.type -> null
                                else -> expandedMeta
                            }
                        },
                        onExpandedPositioned = { coordinates ->
                            rowCoordinates?.let { rc ->
                                val xPx = rc.localPositionOf(coordinates, Offset.Zero).x
                                expandedOffsetX = with(density) { xPx.toDp() }
                            }
                        },
                        onResolveTrailer = onResolveTrailer
                    )
                }
            }
        }
        ShelfExpandedDescription(expandedMeta, expandedOffsetX, rowWidth)
    }
}

@Composable
private fun ShelfExpandedDescription(expandedMeta: Meta?, expandedOffsetX: Dp, rowWidth: Dp) {
    val animatedOffsetX by animateDpAsState(expandedOffsetX, tween(FluxaDimensions.AnimDuration.contentExpand), label = "expandedInfoOffset")
    Box(modifier = Modifier.fillMaxWidth().height(90.dp)) {
        AnimatedVisibility(
            visible = expandedMeta != null,
            enter = fadeIn(tween(FluxaDimensions.AnimDuration.fadeIn)),
            exit = fadeOut(tween(FluxaDimensions.AnimDuration.fadeOut))
        ) {
            val info = expandedMeta
            if (info != null) {
                val metaLine = listOfNotNull(
                    info.releaseInfo?.takeIf { it.isNotBlank() },
                    info.genres?.takeIf { it.isNotEmpty() }?.take(3)?.joinToString(" • "),
                    info.ageRating?.takeIf { it.isNotBlank() }
                ).joinToString("  •  ")
                Column(
                    modifier = Modifier
                        .offset(x = animatedOffsetX)
                        .width((rowWidth - animatedOffsetX - 50.dp).coerceAtLeast(200.dp))
                        .padding(top = 2.dp, bottom = 6.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    if (metaLine.isNotBlank()) {
                        Text(metaLine, color = Color.White.copy(alpha = 0.7f), fontSize = 13.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    }
                    info.description?.takeIf { it.isNotBlank() }?.let {
                        Text(it, color = Color.White.copy(alpha = 0.78f), fontSize = 13.sp, lineHeight = 18.sp, maxLines = 3, overflow = TextOverflow.Ellipsis)
                    }
                }
            }
        }
    }
}

@Composable
internal fun TvHomeShelfSkeleton(titleStartPadding: Dp) {
    val transition = rememberInfiniteTransition(label = "shelf-skeleton")
    val alpha by transition.animateFloat(
        initialValue = 0.05f,
        targetValue = 0.13f,
        animationSpec = infiniteRepeatable(
            animation = tween(FluxaDimensions.AnimDuration.ambientColor, easing = LinearEasing),
            repeatMode = RepeatMode.Reverse
        ),
        label = "shelf-skeleton-alpha"
    )
    Column(modifier = Modifier.padding(bottom = 28.dp)) {
        Box(
            modifier = Modifier
                .padding(start = titleStartPadding, bottom = 12.dp)
                .width(140.dp)
                .height(18.dp)
                .clip(androidx.compose.foundation.shape.RoundedCornerShape(5.dp))
                .background(Color.White.copy(alpha = alpha))
        )
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            contentPadding = PaddingValues(horizontal = titleStartPadding)
        ) {
            items(5) {
                Box(
                    modifier = Modifier
                        .size(200.dp, 120.dp)
                        .clip(androidx.compose.foundation.shape.RoundedCornerShape(10.dp))
                        .background(Color.White.copy(alpha = alpha))
                )
            }
        }
    }
}

internal fun Meta.remainingWatchLabel(lang: String? = null): String? {
    val total = duration ?: return null
    val watched = timeOffset ?: return null
    val remainingMs = (total - watched).coerceAtLeast(0L)
    val mins = (remainingMs / 60000L).toInt()
    if (mins <= 0) return null
    return if (mins >= 60) {
        val hours = mins / 60
        val rest = mins % 60
        if (rest == 0) {
            AppStrings.format(lang, "format.remaining_hours", hours)
        } else {
            AppStrings.format(lang, "format.remaining_hours_minutes", hours, rest)
        }
    } else {
        AppStrings.format(lang, "format.remaining_minutes", mins)
    }
}
