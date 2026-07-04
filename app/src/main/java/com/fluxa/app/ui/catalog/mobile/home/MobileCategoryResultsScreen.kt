package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.itemsIndexed
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.request.crossfade
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.distinctUntilChanged

@Composable
internal fun MobileCategoryResultsContent(
    activeProfile: UserProfile?,
    title: String,
    category: HomeCategory?,
    items: List<Meta>,
    layout: String,
    onMovieClick: (Meta) -> Unit,
    onBack: () -> Unit,
    viewModel: HomeViewModel
) {
    val gridState = rememberLazyGridState()
    var loadArtwork by remember { mutableStateOf(true) }
    LaunchedEffect(gridState) {
        snapshotFlow { gridState.isScrollInProgress }
            .distinctUntilChanged()
            .collectLatest { scrolling ->
                if (scrolling) {
                    delay(80)
                    loadArtwork = false
                } else {
                    delay(120)
                    loadArtwork = true
                }
            }
    }
    LaunchedEffect(category?.id, items.size, gridState) {
        if (category == null || !category.canLoadMore) return@LaunchedEffect
        snapshotFlow {
            gridState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0
        }
            .distinctUntilChanged()
            .collect { lastVisibleIndex ->
                if (lastVisibleIndex >= (items.size - 6).coerceAtLeast(0)) {
                    viewModel.loadMore(category.id)
                }
            }
    }

    val widthPreset = activeProfile?.safePosterWidthPreset ?: "medium"
    val isHorizontal = layout == "horizontal"
    val isSquare = layout == "square"
    val cardWidth: Dp = remember(layout, widthPreset) {
        if (isHorizontal) horizontalCardWidth(widthPreset, DeviceType.Mobile)
        else posterCardWidth(widthPreset)
    }
    val cardImageHeight: Dp = remember(layout, widthPreset) {
        when {
            isHorizontal -> horizontalCardHeight(widthPreset, DeviceType.Mobile)
            isSquare -> posterCardWidth(widthPreset)
            else -> posterCardHeight(widthPreset)
        }
    }
    val reqWidth = if (isHorizontal) 512 else 288
    val reqHeight = if (isHorizontal) 288 else if (isSquare) 288 else 432
    val hideTitles = activeProfile?.safePosterHideTitles == true
    val lang = activeProfile?.safeLanguage ?: "en"
    val upNextLabel = remember(lang) { AppStrings.t(lang, "auto.up_next") }
    val currentOnMovieClick = rememberUpdatedState(onMovieClick)

    Column(modifier = Modifier.fillMaxSize().background(Color(0xFF040508))) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 18.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            IconButton(
                onClick = onBack,
                modifier = Modifier.size(44.dp).background(Color.White.copy(alpha = 0.06f), CircleShape)
            ) {
                Icon(FluxaIcons.ArrowBack, AppStrings.t(lang, "common.back"), tint = Color.White)
            }
            Text(
                text = title,
                color = Color.White,
                fontSize = 22.sp,
                fontWeight = FontWeight.Black,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )
        }

        val skeletonBrush = rememberShimmerBrush()
        LazyVerticalGrid(
            columns = GridCells.Fixed(3),
            state = gridState,
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, bottom = 32.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            if (items.isEmpty()) {
                items(count = 9) {
                    PosterCardSkeleton(cardWidth, cardImageHeight, !hideTitles, skeletonBrush)
                }
            }
            itemsIndexed(
                items = items,
                key = { _, movie -> "${movie.type}:${movie.id}" },
                contentType = { _, movie -> layoutForGridContentType(movie, layout) }
            ) { _, movie ->
                val isUpNext = remember { movie.isUpNextContinueItem() }
                val isProgressCard = remember {
                    isUpNext || ((movie.timeOffset ?: 0L) > 0L && (movie.duration ?: 0L) > 0L)
                }
                val artwork = remember(isHorizontal) {
                    when {
                        isProgressCard -> movie.continueWatchingBackground ?: movie.background ?: movie.poster
                        isHorizontal -> preferredHorizontalArtwork(movie) ?: movie.poster
                        else -> movie.poster
                    }
                }
                val displayTitle = remember {
                    if (isProgressCard)
                        continueWatchingEpisodeTitle(movie) ?: continueWatchingEpisodeLabel(movie) ?: movie.name
                    else movie.name
                }
                val secondary = remember(lang) {
                    if (isProgressCard) continueWatchingEpisodeProgressLabel(movie, lang).orEmpty()
                    else movie.releaseInfo?.take(4) ?: movie.released?.take(4) ?: ""
                }
                val progress = remember {
                    ((movie.timeOffset ?: 0L).toFloat() / (movie.duration ?: 1L).toFloat()).coerceIn(0f, 1f)
                }
                val clickFn: () -> Unit = remember { { currentOnMovieClick.value(movie) } }

                GridCatalogCard(
                    artwork = artwork,
                    width = cardWidth,
                    imageHeight = cardImageHeight,
                    requestWidth = reqWidth,
                    requestHeight = reqHeight,
                    showTitle = !hideTitles && movie.hideTitle != true,
                    title = displayTitle,
                    secondary = secondary,
                    showProgressBar = !isUpNext && isProgressCard && progress > 0f,
                    progress = progress,
                    isUpNext = isUpNext,
                    upNextLabel = upNextLabel,
                    coverEmoji = movie.coverEmoji,
                    name = movie.name,
                    loadArtwork = loadArtwork,
                    onClick = clickFn
                )
            }
        }
    }
}

@Composable
private fun GridCatalogCard(
    artwork: String?,
    width: Dp,
    imageHeight: Dp,
    requestWidth: Int,
    requestHeight: Int,
    showTitle: Boolean,
    title: String,
    secondary: String,
    showProgressBar: Boolean,
    progress: Float,
    isUpNext: Boolean,
    upNextLabel: String,
    coverEmoji: String?,
    name: String,
    loadArtwork: Boolean,
    onClick: () -> Unit
) {
    val context = LocalContext.current
    val request = remember(artwork, requestWidth, requestHeight) {
        ImageRequest.Builder(context)
            .data(artwork)
            .crossfade(false)
            .memoryCacheKey(homeArtworkMemoryCacheKey(artwork, requestWidth, requestHeight))
            .diskCacheKey(artwork)
            .size(requestWidth, requestHeight)
            .build()
    }
    var failed by remember(artwork) { mutableStateOf(artwork.isNullOrBlank()) }

    Column(
        modifier = Modifier
            .width(width)
            .height(imageHeight + if (showTitle) 42.dp else 0.dp)
            .clickable(interactionSource = null, indication = null, onClick = onClick)
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(imageHeight)
                .background(Color.White.copy(alpha = 0.05f)),
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
                    text = coverEmoji?.takeIf { it.isNotBlank() } ?: name.take(1).uppercase(),
                    color = Color.White.copy(alpha = if (coverEmoji.isNullOrBlank()) 0.2f else 0.82f),
                    fontSize = if (coverEmoji.isNullOrBlank()) 48.sp else 42.sp,
                    fontWeight = FontWeight.Black
                )
            }
            if (showProgressBar) {
                Box(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .height(4.dp)
                        .background(Color.White.copy(alpha = 0.20f))
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth(progress)
                            .fillMaxHeight()
                            .background(FluxaColors.progressFill)
                    )
                }
            }
            if (isUpNext) {
                Text(
                    text = upNextLabel,
                    color = Color.White,
                    fontSize = 10.sp,
                    fontWeight = FontWeight.Black,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .padding(8.dp)
                        .background(Color.Black.copy(alpha = 0.72f))
                        .padding(horizontal = 8.dp, vertical = 5.dp)
                )
            }
        }
        if (showTitle) {
            Text(
                text = title,
                color = Color.White,
                fontSize = 12.sp,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 6.dp)
            )
            if (secondary.isNotBlank()) {
                Text(
                    text = secondary,
                    color = Color.White.copy(alpha = 0.58f),
                    fontSize = 10.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 1.dp)
                )
            }
        }
    }
}

private fun layoutForGridContentType(movie: Meta, layout: String): String {
    return when {
        movie.type == "catalog_folder" -> "folder:${movie.reason ?: layout}"
        movie.isUpNextContinueItem() || ((movie.timeOffset ?: 0L) > 0L && (movie.duration ?: 0L) > 0L) -> "progress:$layout"
        else -> "plain:$layout"
    }
}
