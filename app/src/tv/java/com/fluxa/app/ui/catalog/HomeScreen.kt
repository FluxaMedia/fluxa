@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)

package com.fluxa.app.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.LazyListPrefetchStrategy
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.withFrameNanos
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.focusRestorer
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.unit.dp
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogHomeStore
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch

private suspend fun FocusRequester.requestFocusWithRetry(maxAttempts: Int = 15) {
    repeat(maxAttempts) {
        try {
            requestFocus()
            return
        } catch (e: Exception) {
            withFrameNanos {}
        }
    }
}

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
    val scope = rememberCoroutineScope()
    val catalogHomeDataSource = remember(viewModel) {
        AndroidCatalogHomeDataSource(viewModel) { activeProfile }
    }
    val catalogHomeStore = remember(catalogHomeDataSource) {
        CatalogHomeStore(catalogHomeDataSource, scope)
    }
    val catalogHome by catalogHomeStore.state.collectAsStateWithLifecycle()
    LaunchedEffect(catalogHomeStore) {
        catalogHomeStore.dispatch(CatalogAction.Refresh)
    }
    val focusedMovie by viewModel.focusedMovie.collectAsStateWithLifecycle()
    val lang = activeProfile?.safeLanguage ?: "en"
    val rotatingHeroMovie = remember(catalogHome.billboard) {
        catalogHome.billboard?.item?.let { catalogHomeDataSource.resolveMeta(it.id, it.type) }
    }
    val heroMovie = if (activeProfile?.safeHeroFollowsFocusedItem == true) {
        focusedMovie ?: rotatingHeroMovie
    } else {
        rotatingHeroMovie
    }
    val heroLogoUrl = if (activeProfile?.safeHeroFollowsFocusedItem == true && focusedMovie != null) {
        focusedMovie?.logo ?: catalogHome.billboard?.logoUrl
    } else {
        catalogHome.billboard?.logoUrl
    }
    val rowSpecs = remember(catalogHome.rows, lang) {
        catalogHome.rows.mapNotNull { row -> row.toTvHomeRowSpec(catalogHomeDataSource) }
    }
    val shelfFocusRequesters = remember(rowSpecs.size) { List(rowSpecs.size) { FocusRequester() } }
    val heroPlayFocusRequester = remember { FocusRequester() }
    val heroFocusAnchor = remember { FocusRequester() }
    val topBarFirstItemFocusRequester = remember { FocusRequester() }
    val contentFocusRequester = remember { FocusRequester() }
    val listState = rememberLazyListState(
        initialFirstVisibleItemIndex = viewModel.savedTvHomeScrollIndex,
        initialFirstVisibleItemScrollOffset = viewModel.savedTvHomeScrollOffset,
        prefetchStrategy = remember { LazyListPrefetchStrategy(nestedPrefetchItemCount = 4) }
    )
    var focusedRowIndex by remember { mutableIntStateOf(viewModel.savedTvFocusedRowIndex) }
    val stableOnMovieClickState = rememberUpdatedState(onMovieClick)
    val stableOnPlayDirectState = rememberUpdatedState(onPlayDirect)
    val stableLoadMoreCategory = remember<(String) -> Unit> {
        { categoryId -> scope.launch { catalogHomeStore.dispatch(CatalogAction.LoadMore(categoryId)) } }
    }
    val useTopBar = activeProfile?.safeTvNavLayout == "top"
    val railGutter = if (useTopBar) 42.dp else 126.dp
    val contentTopPadding = if (useTopBar) 108.dp else 84.dp
    val hasHeroItem = heroMovie != null && catalogHome.showHeroSection

    val heroActiveState = remember { mutableStateOf(false) }
    var heroActive by heroActiveState

    val onMovieFocus = remember<(Meta, Int) -> Unit> { { meta, rowIdx ->
        viewModel.onMovieFocused(meta)
        focusedRowIndex = rowIdx
        heroActiveState.value = false
    } }

    DisposableEffect(Unit) {
        onDispose {
            viewModel.savedTvHomeScrollIndex = listState.firstVisibleItemIndex
            viewModel.savedTvHomeScrollOffset = listState.firstVisibleItemScrollOffset
            viewModel.savedTvFocusedRowIndex = focusedRowIndex
        }
    }

    var pendingFocusJob by remember { mutableStateOf<Job?>(null) }

    val navigateToFirstShelf: () -> Unit = remember(hasHeroItem) {
        {
            pendingFocusJob?.cancel()
            heroActiveState.value = false
            pendingFocusJob = scope.launch {
                if (hasHeroItem) listState.scrollToItem(1)
                shelfFocusRequesters.firstOrNull()?.requestFocusWithRetry()
            }
        }
    }

    val navigateToHero: () -> Unit = remember {
        {
            pendingFocusJob?.cancel()
            try { heroFocusAnchor.requestFocus() } catch (e: Exception) { }
            pendingFocusJob = scope.launch {
                listState.scrollToItem(0)
                snapshotFlow { listState.layoutInfo.visibleItemsInfo }
                    .first { items -> items.any { it.index == 0 } }
                heroPlayFocusRequester.requestFocusWithRetry()
            }
        }
    }

    LaunchedEffect(hasHeroItem) {
        val savedRow = viewModel.savedTvFocusedRowIndex
        if (savedRow >= 0 && shelfFocusRequesters.isNotEmpty()) {
            pendingFocusJob?.cancel()
            pendingFocusJob = scope.launch {
                listState.scrollToItem(viewModel.savedTvHomeScrollIndex, viewModel.savedTvHomeScrollOffset)
                shelfFocusRequesters.getOrNull(savedRow.coerceIn(0, shelfFocusRequesters.lastIndex))?.requestFocusWithRetry()
            }
        } else if (hasHeroItem) {
            heroActiveState.value = true
            pendingFocusJob?.cancel()
            pendingFocusJob = scope.launch { heroPlayFocusRequester.requestFocusWithRetry() }
        }
    }

    val screenHeight = LocalConfiguration.current.screenHeightDp.dp
    val heroMinHeight = (screenHeight - contentTopPadding - 110.dp).coerceAtLeast(0.dp)

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
    ) {

        if (!catalogHome.isLoading && rowSpecs.isEmpty() && heroMovie == null) {
            HomeEmptyProviderState(
                lang,
                onOpenSettings = onProfileClick,
                modifier = Modifier.padding(start = railGutter)
            )
        } else {
            HomeHeroBackdrop(
                selectedContent = heroMovie,
                activePreviewUrl = activePreviewUrl ?: catalogHome.billboard?.trailerUrl,
                sharedPlayer = sharedPlayer,
                seasonPostersOnHero = activeProfile?.safeHomeSeasonPostersOnHero ?: true
            )

            LazyColumn(
                state = listState,
                modifier = Modifier
                    .fillMaxSize()
                    .focusRequester(contentFocusRequester)
                    .focusRestorer(fallback = heroPlayFocusRequester),
                contentPadding = PaddingValues(top = contentTopPadding, bottom = 80.dp)
            ) {
                if (heroMovie != null && catalogHome.showHeroSection) {
                    item(key = "tv-home-hero", contentType = "hero") {
                        Box(
                            modifier = Modifier.heightIn(min = heroMinHeight),
                            contentAlignment = Alignment.BottomStart
                        ) {
                            HomeHeroPanel(
                                movie = heroMovie,
                                logoUrl = heroLogoUrl,
                                lang = lang,
                                playButtonFocusRequester = heroPlayFocusRequester,
                                onNavigateDown = navigateToFirstShelf,
                                onHeroFocused = { heroActiveState.value = true },
                                showButtons = heroActive,
                                aboveHeroFocusRequester = if (useTopBar) topBarFirstItemFocusRequester else null,
                                modifier = Modifier.padding(start = railGutter, bottom = 32.dp),
                                onPlayClick = { stableOnMovieClickState.value(heroMovie, null, null) },
                                onInfoClick = { stableOnMovieClickState.value(heroMovie, null, null) }
                            )
                        }
                    }
                }

                if (catalogHome.isLoading && rowSpecs.isEmpty()) {
                    items(4, key = { "skeleton-$it" }, contentType = { "skeleton" }) {
                        TvHomeShelfSkeleton(titleStartPadding = railGutter)
                    }
                }
                itemsIndexed(
                    rowSpecs,
                    key = { _, row -> row.categoryId },
                    contentType = { _, row -> "tv-shelf:${row.categoryType}:${row.cardLayout}:${row.isActionRow}:${row.topTenEnabled}" }
                ) { index, row ->
                    val isFirstRow = index == 0 && catalogHome.showHeroSection && heroMovie != null
                    val onFocusForRow = remember(index) { { meta: Meta -> onMovieFocus(meta, index) } }
                    HomeShelfRow(
                        title = row.title,
                        titleStartPadding = railGutter,
                        items = row.items,
                        cardLayout = row.cardLayout,
                        artworkPreference = row.artworkPreference,
                        activeProfile = activeProfile,
                        topTenEnabled = row.topTenEnabled,
                        onItemClick = { meta ->
                            if (row.isActionRow) stableOnPlayDirectState.value(meta)
                            else stableOnMovieClickState.value(meta, row.addonTransportUrl, row.addonCatalogType)
                        },
                        onItemFocus = onFocusForRow,
                        firstItemFocusRequester = shelfFocusRequesters.getOrNull(index),
                        upFocusRequester = when {
                            isFirstRow -> null
                            index == 0 && useTopBar -> topBarFirstItemFocusRequester
                            else -> null
                        },
                        onNavigateUp = if (isFirstRow) navigateToHero else null,
                        onNeedMore = {
                            if (row.canLoadMore && !row.isActionRow) stableLoadMoreCategory(row.categoryId)
                        },
                        onResolveTrailer = { meta -> viewModel.resolveExpandedPosterTrailer(meta) }
                    )
                }
            }
        }

        Box(
            modifier = Modifier
                .size(1.dp)
                .focusRequester(heroFocusAnchor)
                .focusable()
                .onKeyEvent { event ->
                    if (event.type == KeyEventType.KeyDown && event.key == Key.DirectionDown) {
                        navigateToFirstShelf()
                        true
                    } else false
                }
        )

        if (useTopBar) {
            TvHomeTopBar(
                lang = lang,
                selected = TvNavDestination.Home,
                onHomeClick = {},
                onSearchClick = onSearchClick,
                onWatchlistClick = onWatchlistClick,
                onExploreClick = { onExploreClick("movie", null) },
                onProfileClick = onProfileClick,
                contentFocusRequester = contentFocusRequester,
                firstItemFocusRequester = topBarFirstItemFocusRequester
            )
        } else {
            TvHomeNavRail(
                lang = lang,
                selected = TvNavDestination.Home,
                onHomeClick = {},
                onSearchClick = onSearchClick,
                onWatchlistClick = onWatchlistClick,
                onExploreClick = { onExploreClick("movie", null) },
                onProfileClick = onProfileClick,
                contentFocusRequester = contentFocusRequester
            )
        }
    }
}

@Immutable
private data class TvHomeRowSpec(
    val categoryId: String,
    val categoryType: String,
    val title: String,
    val items: List<Meta>,
    val canLoadMore: Boolean,
    val isActionRow: Boolean,
    val topTenEnabled: Boolean,
    val cardLayout: String,
    val artworkPreference: String?,
    val addonTransportUrl: String?,
    val addonCatalogType: String?
)

private fun CatalogRowUiModel.toTvHomeRowSpec(
    dataSource: AndroidCatalogHomeDataSource
): TvHomeRowSpec? {
    val resolvedItems = items.mapNotNull { item -> dataSource.resolveMeta(item.id, item.type) }
    if (resolvedItems.isEmpty()) return null
    val firstSource = items.firstOrNull()?.source
    return TvHomeRowSpec(
        categoryId = id,
        categoryType = categoryType,
        title = title,
        items = resolvedItems,
        canLoadMore = canLoadMore,
        isActionRow = isActionRow,
        topTenEnabled = topTenEnabled,
        cardLayout = cardLayout,
        artworkPreference = artworkPreference,
        addonTransportUrl = firstSource?.addonTransportUrl,
        addonCatalogType = firstSource?.catalogType
    )
}
