@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)

package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.getValue
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.request.crossfade
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.flow.distinctUntilChanged

@Composable
internal fun MobileHomeHeroItem(
    activeProfile: UserProfile?,
    viewModel: HomeViewModel,
    lang: String,
    onMovieClick: (Meta) -> Unit,
    onToggleWatchlist: () -> Unit
) {
    val billboardState = rememberHomeBillboardState(activeProfile, viewModel)
    val pool = billboardState.filteredBillboardPool.ifEmpty { billboardState.billboardPool }
    val poolSize = pool.size

    if (poolSize <= 1) {
        val heroMovie = billboardState.displayBillboardMovie ?: pool.firstOrNull() ?: return
        MobileStreamingHero(
            movie = heroMovie,
            logoUrl = billboardState.billboardLogo,
            lang = lang,
            isWatchlisted = billboardState.billboardWatchlist,
            onPlayClick = { onMovieClick(heroMovie) },
            onInfoClick = { onMovieClick(heroMovie) },
            onToggleWatchlist = onToggleWatchlist,
            index = billboardState.billboardIndex,
            poolSize = poolSize,
            seasonPostersOnHero = activeProfile?.safeHomeSeasonPostersOnHero ?: true
        )
        return
    }

    val startPage = remember(poolSize) {
        Int.MAX_VALUE / 2 - (Int.MAX_VALUE / 2).floorMod(poolSize)
    }
    val pagerState = rememberPagerState(
        initialPage = startPage + billboardState.billboardIndex.coerceIn(0, poolSize - 1),
        pageCount = { Int.MAX_VALUE }
    )

    LaunchedEffect(billboardState.billboardIndex, poolSize) {
        if (poolSize <= 0) return@LaunchedEffect
        val currentIndex = pagerState.currentPage.floorMod(poolSize)
        val delta = shortestPageDelta(currentIndex, billboardState.billboardIndex, poolSize)
        if (delta != 0 && !pagerState.isScrollInProgress) {
            pagerState.animateScrollToPage(pagerState.currentPage + delta)
        }
    }

    LaunchedEffect(pagerState, poolSize) {
        snapshotFlow { pagerState.currentPage }
            .distinctUntilChanged()
            .collect { page ->
                viewModel.syncBillboardIndex(page.floorMod(poolSize))
            }
    }

    val currentPoolPage = pagerState.currentPage.floorMod(poolSize)
    MobileStreamingHeroPager(
        movies = pool,
        currentMovie = billboardState.displayBillboardMovie,
        currentLogoUrl = billboardState.billboardLogo,
        pagerState = pagerState,
        lang = lang,
        isWatchlisted = billboardState.billboardWatchlist,
        onPlayClick = { pool.getOrNull(currentPoolPage)?.let(onMovieClick) },
        onInfoClick = { pool.getOrNull(currentPoolPage)?.let(onMovieClick) },
        onToggleWatchlist = onToggleWatchlist,
        seasonPostersOnHero = activeProfile?.safeHomeSeasonPostersOnHero ?: true
    )
}

@Composable
internal fun HomeUserCollectionRow(
    collection: LibraryUserCollection,
    isFirstRow: Boolean,
    horizontalPadding: androidx.compose.ui.unit.Dp,
    lang: String,
    cardLayout: String,
    widthPreset: String,
    hidePosterTitles: Boolean,
    onMovieClick: (Meta) -> Unit
) {
    val folders = collection.folders.orEmpty()
    val titleSlotHeight = collectionFolderTitleSlotHeight(hidePosterTitles)
    val rowHeight = remember(folders, cardLayout, widthPreset, titleSlotHeight) {
        val imageHeight = folders.maxOfOrNull { folder ->
            collectionFolderCardImageHeight(
                fallbackLayout = collectionFolderLayout(collection.effectiveFolderShape(folder), cardLayout),
                widthPreset = widthPreset
            )
        } ?: collectionFolderCardImageHeight(cardLayout, widthPreset)
        imageHeight + titleSlotHeight
    }
    val currentOnMovieClick = rememberUpdatedState(onMovieClick)

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 30.dp)
            .then(if (isFirstRow) Modifier.offset(y = 12.dp) else Modifier)
    ) {
        HomeCollectionHeader(
            title = collection.title,
            lang = lang,
            horizontalPadding = horizontalPadding,
        )
        LazyRow(
            modifier = Modifier
                .fillMaxWidth()
                .height(rowHeight),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            contentPadding = PaddingValues(start = horizontalPadding, end = 24.dp)
        ) {
            items(
                count = folders.size,
                key = { index ->
                    val folder = folders[index]
                    "${collection.id}_${folder.id}_$index"
                },
                contentType = { index ->
                    "collection-folder-card:${collectionFolderLayout(folders[index].shape, cardLayout)}"
                }
            ) { index ->
                val folder = folders[index]
                val poster = folder.effectiveImageUrl()
                val displayArtwork = folder.focusGifUrl
                    ?.takeIf { folder.focusGifEnabled != false }
                    ?.takeIf { it.isNotBlank() }
                    ?: poster
                val reason = collection.effectiveFolderShape(folder)
                MobileCollectionFolderCard(
                    title = folder.title,
                    poster = displayArtwork,
                    coverEmoji = folder.coverEmoji,
                    hideTitle = folder.hideTitle == true,
                    reason = reason,
                    hideTitlesByProfile = hidePosterTitles,
                    fallbackLayout = cardLayout,
                    widthPreset = widthPreset,
                    titleSlotHeight = titleSlotHeight,
                    onClick = {
                        currentOnMovieClick.value(folder.toCatalogFolderMeta(collection, poster, reason))
                    }
                )
            }
        }
    }
}

private fun LibraryUserCollectionFolder.toCatalogFolderMeta(
    collection: LibraryUserCollection,
    poster: String?,
    reason: String?
): Meta {
    return Meta(
        id = id,
        name = title,
        type = "catalog_folder",
        poster = poster,
        background = heroBackdropUrl ?: poster,
        logo = titleLogoUrl,
        releaseInfo = catalogTitle,
        reason = reason,
        focusGifUrl = focusGifUrl.takeIf { focusGifEnabled != false },
        coverEmoji = coverEmoji,
        hideTitle = hideTitle,
        focusGlowEnabled = collection.focusGlowEnabled
    )
}

@Composable
private fun HomeCollectionHeader(
    title: String,
    lang: String,
    horizontalPadding: androidx.compose.ui.unit.Dp
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = horizontalPadding, end = horizontalPadding, bottom = 8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        androidx.compose.material3.Text(
            text = title,
            fontFamily = FluxaDisplay,
            fontWeight = FontWeight.Bold,
            color = Color.White,
            fontSize = 18.sp,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
    }
}

@Composable
internal fun HomeCategoryRow(
    categoryId: String,
    title: String,
    titleTail: String?,
    addonIconUrl: String?,
    addonTransportUrl: String?,
    addonCatalogType: String?,
    rowItems: List<Meta>,
    canLoadMore: Boolean,
    isContinueWatching: Boolean,
    isLibrary: Boolean,
    isFirstRow: Boolean,
    horizontalPadding: androidx.compose.ui.unit.Dp,
    showTopTenRanking: Boolean,
    lang: String,
    cardLayout: String,
    widthPreset: String,
    hidePosterTitles: Boolean,
    artworkPreference: String?,
    onViewAllClick: () -> Unit,
    onLoadMore: (String) -> Unit,
    onMovieClick: (Meta, String?, String?) -> Unit,
    onPlayDirect: (Meta) -> Unit,
    onProgressAction: (Meta) -> Unit
) {
    val currentOnMovieClick = rememberUpdatedState(onMovieClick)
    val currentAddonTransportUrl = rememberUpdatedState(addonTransportUrl)
    val currentAddonCatalogType = rememberUpdatedState(addonCatalogType)
    val currentOnPlayDirect = rememberUpdatedState(onPlayDirect)
    val currentOnProgressAction = rememberUpdatedState(onProgressAction)
    val currentOnLoadMore = rememberUpdatedState(onLoadMore)
    val listState = rememberLazyListState()
    val context = LocalContext.current
    val addonIconRequest = remember(addonIconUrl) {
        addonIconUrl?.takeIf { it.isNotBlank() }?.let {
            ImageRequest.Builder(context)
                .data(it)
                .crossfade(false)
                .memoryCacheKey("home-addon-icon:$it")
                .diskCacheKey(it)
                .build()
        }
    }
    val isActionRow = isContinueWatching || isLibrary
    val cardScale = if (isActionRow && cardLayout == "horizontal") 1.1f else 1f
    val rowIsHorizontal = cardLayout == "horizontal" || cardLayout == "episode"
    val rowIsSquare = cardLayout == "square"
    val cardWidth = remember(cardLayout, widthPreset, cardScale) {
        (if (rowIsHorizontal) horizontalCardWidth(widthPreset, DeviceType.Mobile)
        else posterCardWidth(widthPreset)) * cardScale
    }
    val cardImageHeight = remember(cardLayout, widthPreset, cardScale) {
        (when {
            cardLayout == "episode" -> 148.dp
            rowIsHorizontal -> horizontalCardHeight(widthPreset, DeviceType.Mobile)
            rowIsSquare -> posterCardWidth(widthPreset)
            else -> posterCardHeight(widthPreset)
        }) * cardScale
    }
    val profileShowTitle = !hidePosterTitles
    val upNextLabel = AppStrings.t(lang, "auto.up_next")
    LaunchedEffect(categoryId, canLoadMore, isActionRow, rowItems.size, listState) {
        if (isActionRow || !canLoadMore || rowItems.isEmpty()) return@LaunchedEffect
        snapshotFlow {
            val lastVisibleIndex = listState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: -1
            lastVisibleIndex >= rowItems.lastIndex - HOME_CATALOG_LOAD_MORE_THRESHOLD
        }
            .distinctUntilChanged()
            .collect { shouldLoadMore ->
                if (shouldLoadMore) currentOnLoadMore.value(categoryId)
            }
    }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 22.dp)
            .then(if (isFirstRow) Modifier.offset(y = 12.dp) else Modifier)
    ) {
        HomeRowHeader(
            title = title,
            titleTail = titleTail,
            horizontalPadding = horizontalPadding,
            addonIconRequest = addonIconRequest,
            viewAllLabel = AppStrings.t(lang, "common.view_all"),
            showViewAll = !isContinueWatching && !isLibrary,
            onViewAllClick = onViewAllClick
        )
        LazyRow(
            state = listState,
            modifier = Modifier
                .fillMaxWidth()
                .height(cardImageHeight + if (profileShowTitle) 42.dp else 0.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            contentPadding = PaddingValues(start = horizontalPadding, end = 24.dp)
        ) {
            items(
                count = rowItems.size,
                key = { index ->
                    val item = rowItems[index]
                    "${categoryId}_${item.type}_${item.id}_$index"
                },
                contentType = { "catalog-card:$cardLayout:$isContinueWatching:$isActionRow" }
            ) { movieIndex ->
                val movie = rowItems[movieIndex]
                val artwork = when {
                    isContinueWatching && artworkPreference == "poster" ->
                        movie.poster ?: movie.continueWatchingBackground ?: movie.background
                    isContinueWatching && artworkPreference == "background" ->
                        movie.background ?: movie.continueWatchingBackground ?: movie.poster
                    isContinueWatching ->
                        movie.continueWatchingBackground ?: movie.background ?: movie.poster
                    rowIsHorizontal -> movie.background ?: movie.poster
                    else -> movie.poster ?: movie.background
                }
                val clickFn = remember(movie.id, isActionRow) {
                    if (isActionRow) ({ currentOnPlayDirect.value(movie) })
                    else ({ currentOnMovieClick.value(movie, currentAddonTransportUrl.value, currentAddonCatalogType.value) })
                }
                val longClickFn = remember(movie.id, isActionRow) {
                    if (isActionRow && (movie.timeOffset ?: 0L) > 0L)
                        ({ currentOnProgressAction.value(movie) })
                    else null
                }
                if (isActionRow) {
                    val isUpNext = movie.isUpNextContinueItem()
                    val isProgressItem = isUpNext || ((movie.timeOffset ?: 0L) > 0L && (movie.duration ?: 0L) > 0L)
                    val title = movie.name
                    val secondary = if (isContinueWatching && isProgressItem) {
                        continueWatchingEpisodeLabel(movie).orEmpty()
                    } else {
                        movie.releaseInfo?.take(4) ?: movie.released?.take(4) ?: ""
                    }
                    val progress = ((movie.timeOffset ?: 0L).toFloat() /
                        (movie.duration ?: 1L).toFloat()).coerceIn(0f, 1f)
                    HomeCatalogCard(
                        artwork = artwork,
                        width = cardWidth,
                        imageHeight = cardImageHeight,
                        showTitle = profileShowTitle && movie.hideTitle != true,
                        title = title,
                        secondary = secondary,
                        showProgressBar = !isUpNext && isProgressItem && progress > 0f,
                        progress = progress,
                        isUpNext = isUpNext,
                        upNextLabel = upNextLabel,
                        topTenRank = null,
                        coverEmoji = movie.coverEmoji,
                        name = movie.name,
                        onClick = clickFn,
                        onLongClick = longClickFn
                    )
                } else {
                    HomeCatalogCard(
                        artwork = artwork,
                        width = cardWidth,
                        imageHeight = cardImageHeight,
                        showTitle = profileShowTitle && movie.hideTitle != true,
                        title = movie.name,
                        secondary = movie.releaseInfo?.take(4) ?: movie.released?.take(4) ?: "",
                        showProgressBar = false,
                        progress = 0f,
                        isUpNext = false,
                        upNextLabel = upNextLabel,
                        topTenRank = if (showTopTenRanking && movieIndex < 10) movieIndex + 1 else null,
                        coverEmoji = movie.coverEmoji,
                        name = movie.name,
                        onClick = clickFn,
                        onLongClick = null
                    )
                }
            }
        }
    }
}

@Composable
private fun HomeRowHeader(
    title: String,
    titleTail: String?,
    horizontalPadding: androidx.compose.ui.unit.Dp,
    addonIconRequest: ImageRequest?,
    viewAllLabel: String,
    showViewAll: Boolean,
    onViewAllClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = horizontalPadding, end = horizontalPadding, bottom = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        val fullTitle = remember(title, titleTail) {
            if (titleTail != null) "$title  —  $titleTail" else title
        }
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            if (addonIconRequest != null) {
                AsyncImage(
                    model = addonIconRequest,
                    contentDescription = null,
                    modifier = Modifier.height(20.dp).width(20.dp).clip(RoundedCornerShape(4.dp)),
                    contentScale = ContentScale.Fit
                )
            }
            androidx.compose.material3.Text(
                text = fullTitle,
                fontFamily = FluxaDisplay,
                fontWeight = FontWeight.Bold,
                color = Color.White,
                fontSize = 18.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
        if (showViewAll) {
            val viewAllInteractionSource = remember { MutableInteractionSource() }
            androidx.compose.material3.Text(
                text = viewAllLabel,
                color = Color.White.copy(alpha = 0.68f),
                fontSize = 13.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.clickable(
                    interactionSource = viewAllInteractionSource,
                    indication = null
                ) { onViewAllClick() }
            )
        }
    }
}

@Composable
internal fun MobileHomeShelfSkeleton(horizontalPadding: androidx.compose.ui.unit.Dp) {
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
    Column(modifier = Modifier.padding(bottom = 24.dp)) {
        Box(
            modifier = Modifier
                .padding(start = horizontalPadding, bottom = 10.dp)
                .width(120.dp)
                .height(16.dp)
                .clip(RoundedCornerShape(5.dp))
                .background(Color.White.copy(alpha = alpha))
        )
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            contentPadding = PaddingValues(horizontal = horizontalPadding)
        ) {
            items(5) {
                Box(
                    modifier = Modifier
                        .size(130.dp, 195.dp)
                        .clip(RoundedCornerShape(8.dp))
                        .background(Color.White.copy(alpha = alpha))
                )
            }
        }
    }
}
