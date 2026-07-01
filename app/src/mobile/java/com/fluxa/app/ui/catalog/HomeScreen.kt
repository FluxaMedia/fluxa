@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(
    androidx.compose.foundation.ExperimentalFoundationApi::class,
    androidx.compose.material3.ExperimentalMaterial3Api::class
)

package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.interaction.MutableInteractionSource
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
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.request.crossfade
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.ui.platform.LocalContext
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.flow.distinctUntilChanged

@Composable
fun HomeScreen(
    activeProfile: UserProfile?,
    onMovieClick: (Meta, String?, String?) -> Unit,
    onPlayDirect: (Meta) -> Unit,
    onWatchlistClick: () -> Unit,
    onProfileClick: () -> Unit,
    onSearchClick: () -> Unit,
    onExploreClick: (String, String?) -> Unit,
    onCategoryClick: (HomeCategory) -> Unit,
    viewModel: HomeViewModel,
    sharedPlayer: androidx.media3.exoplayer.ExoPlayer,
    activePreviewUrl: String?
) {
    val catalogState = rememberHomeCatalogState(activeProfile, viewModel)
    val horizontalPadding = 16.dp
    val profileLanguage = activeProfile?.safeLanguage ?: "en"
    val profileCardLayout = activeProfile?.safeCardLayout ?: "vertical"
    val profilePosterWidthPreset = activeProfile?.safePosterWidthPreset ?: "medium"
    val profileHidePosterTitles = activeProfile?.safePosterHideTitles == true
    var progressActionMeta by remember { mutableStateOf<Meta?>(null) }
    val onProgressAction: (Meta) -> Unit = remember { { meta -> progressActionMeta = meta } }
    val homeRowSpecs = remember(
        catalogState.orderedCategories,
        activeProfile?.safeLibraryCollections,
        profileLanguage,
        activeProfile,
        catalogState.topTenFeedKeys
    ) {
        buildMobileHomeRowSpecs(
            categories = catalogState.orderedCategories,
            collections = activeProfile?.safeLibraryCollections.orEmpty(),
            lang = profileLanguage,
            activeProfile = activeProfile,
            topTenFeedKeys = catalogState.topTenFeedKeys
        )
    }

    val stableOnCategoryClickState = rememberUpdatedState(onCategoryClick)
    val stableOnMovieClickState = rememberUpdatedState(onMovieClick)
    val stableOnPlayDirectState = rememberUpdatedState(onPlayDirect)
    val stableOnCategoryClick = remember<(HomeCategory) -> Unit> { { cat -> stableOnCategoryClickState.value(cat) } }
    val stableOnMovieClick = remember<(Meta, String?, String?) -> Unit> { { meta, addonUrl, catalogType -> stableOnMovieClickState.value(meta, addonUrl, catalogType) } }
    val stableOnPlayDirect = remember<(Meta) -> Unit> { { meta -> stableOnPlayDirectState.value(meta) } }
    val stableLoadMoreCategory = remember<(String) -> Unit> { { categoryId -> viewModel.loadMore(categoryId) } }
    val homeListState = rememberLazyListState(
        initialFirstVisibleItemIndex = viewModel.savedHomeScrollIndex,
        initialFirstVisibleItemScrollOffset = viewModel.savedHomeScrollOffset
    )

    DisposableEffect(Unit) {
        onDispose {
            viewModel.savedHomeScrollIndex = homeListState.firstVisibleItemIndex
            viewModel.savedHomeScrollOffset = homeListState.firstVisibleItemScrollOffset
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
    ) {
        androidx.compose.foundation.lazy.LazyColumn(
            state = homeListState,
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = 120.dp)
        ) {
            if (catalogState.showHeroSection) {
                item(key = "home-hero", contentType = "hero") {
                    MobileHomeHeroItem(
                        activeProfile = activeProfile,
                        viewModel = viewModel,
                        lang = activeProfile?.safeLanguage ?: "en",
                        onMovieClick = { meta -> stableOnMovieClick(meta, null, null) },
                        onToggleWatchlist = { viewModel.toggleBillboardWatchlist() }
                    )
                }
            }
            items(
                count = homeRowSpecs.size,
                key = { index -> homeRowSpecs[index].key },
                contentType = { index ->
                    when (val spec = homeRowSpecs[index]) {
                        is MobileHomeRowSpec.Collection -> "collection-row"
                        is MobileHomeRowSpec.Category ->
                            "category-row:${spec.categoryType}:${spec.cardLayout}:${spec.isContinueWatching}:${spec.isLibrary}:${spec.showTopTenRanking}"
                    }
                }
            ) { rowIndex ->
                when (val spec = homeRowSpecs[rowIndex]) {
                    is MobileHomeRowSpec.Collection -> {
                        HomeUserCollectionRow(
                            collection = spec.collection,
                            isFirstRow = rowIndex == 0,
                            horizontalPadding = horizontalPadding,
                            lang = profileLanguage,
                            cardLayout = profileCardLayout,
                            widthPreset = profilePosterWidthPreset,
                            hidePosterTitles = profileHidePosterTitles,
                            onMovieClick = { meta -> stableOnMovieClick(meta, null, null) }
                        )
                        return@items
                    }
                    is MobileHomeRowSpec.Category -> {
                        val onViewAllClick = remember(spec.categoryId, spec.navTitle, spec.categoryType) {
                            { stableOnCategoryClick(spec.toNavigationCategory()) }
                        }
                        HomeCategoryRow(
                            categoryId = spec.categoryId,
                            title = spec.title,
                            titleTail = spec.titleTail,
                            addonIconUrl = spec.addonIconUrl,
                            addonTransportUrl = spec.addonTransportUrl,
                            addonCatalogType = spec.addonCatalogType,
                            rowItems = spec.rowItems,
                            canLoadMore = spec.canLoadMore,
                            isContinueWatching = spec.isContinueWatching,
                            isLibrary = spec.isLibrary,
                            isFirstRow = rowIndex == 0,
                            horizontalPadding = horizontalPadding,
                            showTopTenRanking = spec.showTopTenRanking,
                            lang = profileLanguage,
                            cardLayout = spec.cardLayout,
                            widthPreset = profilePosterWidthPreset,
                            hidePosterTitles = profileHidePosterTitles,
                            artworkPreference = spec.artworkPreference,
                            onViewAllClick = onViewAllClick,
                            onLoadMore = stableLoadMoreCategory,
                            onMovieClick = stableOnMovieClick,
                            onPlayDirect = stableOnPlayDirect,
                            onProgressAction = onProgressAction
                        )
                    }
                }
            }
        }

        progressActionMeta?.let { meta ->
            ContinueWatchingActionsSheet(
                meta = meta,
                lang = activeProfile?.safeLanguage ?: "en",
                onDismiss = { progressActionMeta = null },
                onDetails = {
                    progressActionMeta = null
                    onMovieClick(meta, null, null)
                },
                onForget = {
                    progressActionMeta = null
                    viewModel.forgetPlaybackProgress(meta)
                }
            )
        }
    }
}

@Immutable
private sealed interface MobileHomeRowSpec {
    val key: String

    data class Collection(val collection: LibraryUserCollection) : MobileHomeRowSpec {
        override val key: String = "collection_${collection.id}"
    }

    data class Category(
        val categoryId: String,
        val categoryType: String,
        val navTitle: String,
        val title: String,
        val titleTail: String?,
        val addonIconUrl: String?,
        val addonTransportUrl: String?,
        val addonCatalogType: String?,
        val rowItems: List<Meta>,
        val canLoadMore: Boolean,
        val isContinueWatching: Boolean,
        val isLibrary: Boolean,
        val showTopTenRanking: Boolean,
        val cardLayout: String,
        val artworkPreference: String?
    ) : MobileHomeRowSpec {
        override val key: String = categoryId

        fun toNavigationCategory(): HomeCategory {
            return HomeCategory(
                name = navTitle,
                items = emptyList(),
                id = categoryId,
                type = categoryType
            )
        }
    }
}

private fun buildMobileHomeRowSpecs(
    categories: List<HomeCategory>,
    collections: List<LibraryUserCollection>,
    lang: String,
    activeProfile: UserProfile?,
    topTenFeedKeys: Set<String>
): List<MobileHomeRowSpec> {
    val visibleCollections = collections.filter { it.folders.orEmpty().isNotEmpty() }
    val aboveCollections = visibleCollections.filter { it.showOnHome == true }
    val belowCollections = visibleCollections.filter { it.showOnHome != true }
    val hasContinueWatching = categories.any { it.id == "continue_watching" }
    val rows = mutableListOf<MobileHomeRowSpec>()
    rows.addAll(aboveCollections.map(MobileHomeRowSpec::Collection))
    if (!hasContinueWatching) {
        rows.addAll(belowCollections.map(MobileHomeRowSpec::Collection))
    }
    categories.forEach { category ->
        rows.add(category.toMobileHomeCategoryRowSpec(lang, activeProfile, topTenFeedKeys))
        if (category.id == "continue_watching") {
            rows.addAll(belowCollections.map(MobileHomeRowSpec::Collection))
        }
    }
    return rows
}

private fun HomeCategory.toMobileHomeCategoryRowSpec(
    lang: String,
    activeProfile: UserProfile?,
    topTenFeedKeys: Set<String>
): MobileHomeRowSpec.Category {
    val isContinueWatching = isContinueWatchingCategory()
    val isLibrary = id == "library"
    val isActionRow = isContinueWatching || isLibrary
    val titleParts = homeCategoryTitleParts(this, lang)
    return MobileHomeRowSpec.Category(
        categoryId = id,
        categoryType = type,
        navTitle = name,
        title = titleParts.first,
        titleTail = titleParts.second,
        addonIconUrl = addonIconUrl,
        addonTransportUrl = addonTransportUrl ?: catalogSources?.firstOrNull()?.transportUrl,
        addonCatalogType = catalogSources?.firstOrNull()?.type ?: type,
        rowItems = items,
        canLoadMore = canLoadMore,
        isContinueWatching = isContinueWatching,
        isLibrary = isLibrary,
        showTopTenRanking = id in topTenFeedKeys,
        cardLayout = resolveHomeCardLayout(this, activeProfile),
        artworkPreference = resolveContinueWatchingArtworkPreference(this, activeProfile)
    )
}

@Composable
private fun MobileHomeHeroItem(
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

    // Auto-rotation: billboard index changes → animate pager
    LaunchedEffect(billboardState.billboardIndex, poolSize) {
        if (poolSize <= 0) return@LaunchedEffect
        val currentIndex = pagerState.currentPage.floorMod(poolSize)
        val delta = shortestPageDelta(currentIndex, billboardState.billboardIndex, poolSize)
        if (delta != 0 && !pagerState.isScrollInProgress) {
            pagerState.animateScrollToPage(pagerState.currentPage + delta)
        }
    }

    // User swipe: pager page changes → sync billboard index
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
private fun HomeUserCollectionRow(
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
private fun HomeCategoryRow(
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

private fun collectionFolderCardImageHeight(
    fallbackLayout: String,
    widthPreset: String
): androidx.compose.ui.unit.Dp {
    return when (fallbackLayout) {
        "horizontal" -> horizontalCardHeight(widthPreset, DeviceType.Mobile)
        "square" -> posterCardWidth(widthPreset)
        else -> posterCardHeight(widthPreset)
    }
}

private fun collectionFolderTitleSlotHeight(hidePosterTitles: Boolean): androidx.compose.ui.unit.Dp {
    return if (hidePosterTitles) 0.dp else 30.dp
}

// Pure drawing card — no Meta, no UserProfile, no logic. All data pre-computed by caller.
@Composable
private fun HomeCatalogCard(
    artwork: String?,
    width: androidx.compose.ui.unit.Dp,
    imageHeight: androidx.compose.ui.unit.Dp,
    showTitle: Boolean,
    title: String,
    secondary: String,
    showProgressBar: Boolean,
    progress: Float,
    isUpNext: Boolean,
    upNextLabel: String,
    topTenRank: Int?,
    coverEmoji: String?,
    name: String,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)?
) {
    val context = LocalContext.current
    val request = remember(context, artwork) {
        ImageRequest.Builder(context)
            .data(artwork)
            .crossfade(false)
            .memoryCacheKey(artwork?.let { "home-catalog:$it" })
            .diskCacheKey(artwork)
            .build()
    }
    val clickModifier = if (onLongClick != null) {
        Modifier.combinedClickable(interactionSource = null, indication = null, onClick = onClick, onLongClick = onLongClick)
    } else {
        Modifier.clickable(interactionSource = null, indication = null, onClick = onClick)
    }
    Column(
        modifier = Modifier
            .width(width)
            .height(imageHeight + if (showTitle) 42.dp else 0.dp)
            .then(clickModifier)
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(imageHeight)
                .background(Color.White.copy(alpha = 0.05f)),
            contentAlignment = Alignment.Center
        ) {
            if (!artwork.isNullOrBlank()) {
                AsyncImage(
                    model = request,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
            } else {
                androidx.compose.material3.Text(
                    text = coverEmoji?.takeIf { it.isNotBlank() } ?: name.take(1).uppercase(),
                    color = Color.White.copy(alpha = if (coverEmoji.isNullOrBlank()) 0.2f else 0.82f),
                    fontSize = if (coverEmoji.isNullOrBlank()) 42.sp else 36.sp,
                    fontWeight = FontWeight.Black
                )
            }
            if (showProgressBar) {
                Box(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .padding(horizontal = 8.dp, vertical = 6.dp)
                        .height(4.dp)
                        .clip(RoundedCornerShape(999.dp))
                        .background(Color.White.copy(alpha = 0.2f))
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
                androidx.compose.material3.Text(
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
            if (topTenRank != null) {
                androidx.compose.material3.Text(
                    text = topTenRank.toString(),
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Black,
                    modifier = Modifier
                        .align(Alignment.TopStart)
                        .padding(7.dp)
                        .background(Color.Black.copy(alpha = 0.66f), RoundedCornerShape(6.dp))
                        .padding(horizontal = 7.dp, vertical = 3.dp)
                )
            }
        }
        if (showTitle) {
            androidx.compose.material3.Text(
                text = title,
                color = Color.White,
                fontSize = 12.sp,
                lineHeight = 13.sp,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(top = 4.dp)
            )
            if (secondary.isNotBlank()) {
                androidx.compose.material3.Text(
                    text = secondary,
                    color = Color.White.copy(alpha = 0.58f),
                    fontSize = 10.sp,
                    lineHeight = 11.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 0.dp)
                )
            }
        }
    }
}

@Composable
private fun MobileCollectionFolderCard(
    title: String,
    poster: String?,
    coverEmoji: String?,
    hideTitle: Boolean,
    reason: String?,
    hideTitlesByProfile: Boolean,
    fallbackLayout: String,
    widthPreset: String,
    titleSlotHeight: androidx.compose.ui.unit.Dp,
    onClick: () -> Unit
) {
    val context = LocalContext.current
    val layout = collectionFolderLayout(reason, fallbackLayout)
    val width = when (layout) {
        "horizontal" -> horizontalCardWidth(widthPreset, DeviceType.Mobile)
        else -> posterCardWidth(widthPreset)
    }
    val imageHeight = when (layout) {
        "horizontal" -> horizontalCardHeight(widthPreset, DeviceType.Mobile)
        "square" -> posterCardWidth(widthPreset)
        else -> posterCardHeight(widthPreset)
    }
    val displayPoster = poster?.takeIf { it.isNotBlank() }
    val request = remember(context, displayPoster) {
        ImageRequest.Builder(context)
            .data(displayPoster)
            .crossfade(false)
            .memoryCacheKey(displayPoster?.let { "home-collection:$it" })
            .diskCacheKey(displayPoster)
            .build()
    }
    val showTitle = !(hideTitlesByProfile || hideTitle)
    val totalHeight = imageHeight + titleSlotHeight

    Column(
        modifier = Modifier
            .width(width)
            .height(totalHeight)
            .clickable(
                interactionSource = null,
                indication = null,
                onClick = onClick
            ),
        verticalArrangement = Arrangement.spacedBy(4.dp)
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(imageHeight)
                .background(Color.White.copy(alpha = 0.05f)),
            contentAlignment = Alignment.Center
        ) {
            if (displayPoster != null) {
                AsyncImage(
                    model = request,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
            } else {
                androidx.compose.material3.Text(
                    text = coverEmoji?.takeIf { it.isNotBlank() } ?: title.take(1).uppercase(),
                    color = Color.White.copy(alpha = if (coverEmoji.isNullOrBlank()) 0.2f else 0.82f),
                    fontSize = if (coverEmoji.isNullOrBlank()) 42.sp else 36.sp,
                    fontWeight = FontWeight.Black
                )
            }
        }
        if (showTitle) {
            androidx.compose.material3.Text(
                text = title,
                color = Color.White,
                fontSize = 12.sp,
                lineHeight = 15.sp,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.height(titleSlotHeight - 4.dp)
            )
        }
    }
}

private fun collectionFolderLayout(reason: String?, fallbackLayout: String): String {
    return when (reason) {
        "wide" -> "horizontal"
        "square" -> "square"
        "poster" -> "vertical"
        else -> fallbackLayout
    }
}

private const val HOME_CATALOG_LOAD_MORE_THRESHOLD = 5

@Composable
private fun ContinueWatchingActionsSheet(
    meta: Meta,
    lang: String,
    onDismiss: () -> Unit,
    onDetails: () -> Unit,
    onForget: () -> Unit
) {
    val progress = ((meta.timeOffset ?: 0L).toFloat() / (meta.duration ?: 1L).toFloat()).coerceIn(0f, 1f)
    val episodeInfo = continueWatchingEpisodeLabel(meta)
    val typeLabel = if (meta.type == "movie") AppStrings.t(lang, "auto.movie") else AppStrings.t(lang, "auto.series")
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        containerColor = Color(0xFF10131A),
        contentColor = Color.White
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 8.dp)
                .padding(bottom = 28.dp)
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
                AsyncImage(
                    model = meta.continueWatchingPoster,
                    contentDescription = null,
                    modifier = Modifier
                        .width(78.dp)
                        .height(112.dp)
                        .clip(RoundedCornerShape(8.dp))
                        .background(Color.White.copy(alpha = 0.06f)),
                    contentScale = ContentScale.Crop
                )
                Column(modifier = Modifier.weight(1f)) {
                    androidx.compose.material3.Text(
                        text = meta.name,
                        color = Color.White,
                        fontSize = 20.sp,
                        fontWeight = FontWeight.Black,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                    Spacer(Modifier.height(6.dp))
                    androidx.compose.material3.Text(
                        text = listOfNotNull(typeLabel, meta.releaseInfo?.takeIf { it.isNotBlank() }, episodeInfo).joinToString("  "),
                        color = Color.White.copy(alpha = 0.62f),
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Bold,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                    Spacer(Modifier.height(12.dp))
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(4.dp)
                            .clip(RoundedCornerShape(999.dp))
                            .background(Color.White.copy(alpha = 0.14f))
                    ) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(progress)
                                .fillMaxHeight()
                                .background(Color.White)
                        )
                    }
                }
            }

            Spacer(Modifier.height(20.dp))
            ContinueWatchingSheetButton(FluxaIcons.Info, AppStrings.t(lang, "home.view_details"), onDetails)
            Spacer(Modifier.height(10.dp))
            ContinueWatchingSheetButton(FluxaIcons.DeleteOutline, AppStrings.t(lang, "home.forget_progress"), onForget)
        }
    }
}

@Composable
private fun ContinueWatchingSheetButton(icon: ImageVector, label: String, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(52.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(Color.White.copy(alpha = 0.08f))
            .clickable { onClick() }
            .padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        androidx.compose.material3.Icon(icon, null, tint = Color.White, modifier = Modifier.height(22.dp))
        androidx.compose.material3.Text(label, color = Color.White, fontSize = 15.sp, fontWeight = FontWeight.Bold)
    }
}
