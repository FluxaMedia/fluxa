package com.fluxa.app.shared

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.ui.focus.focusRestorer
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.chrisbanes.haze.HazeStyle
import dev.chrisbanes.haze.HazeTint
import dev.chrisbanes.haze.hazeEffect
import dev.chrisbanes.haze.hazeSource
import dev.chrisbanes.haze.rememberHazeState
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlin.math.abs
import androidx.compose.animation.core.animateFloatAsState
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.stableLazyKey
import com.fluxa.app.shared.feature.catalog.CatalogBillboardUiModel
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.withProminentContinueWatchingCard
import com.fluxa.app.shared.feature.catalog.CategoryResultsScreen
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.ui.catalog.CONTINUE_WATCHING_CATEGORY_ID
import com.fluxa.app.shared.feature.addonstore.AddonStoreAction
import com.fluxa.app.shared.feature.addonstore.AddonStoreScreen
import com.fluxa.app.shared.feature.addonstore.AddonStoreUiState
import com.fluxa.app.shared.feature.auth.AuthAction
import com.fluxa.app.shared.feature.auth.AuthScreen
import com.fluxa.app.shared.feature.auth.AuthUiState
import com.fluxa.app.shared.feature.calendar.CalendarAction
import com.fluxa.app.shared.feature.calendar.CalendarScreen
import com.fluxa.app.shared.feature.calendar.CalendarUiState
import com.fluxa.app.shared.feature.calendar.NotificationsScreen
import com.fluxa.app.shared.feature.detail.DetailAction
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.detail.DetailScreen
import com.fluxa.app.shared.feature.detail.DetailUiState
import com.fluxa.app.shared.feature.detail.SourceSelectionScreen
import com.fluxa.app.shared.feature.discover.DiscoverAction
import com.fluxa.app.shared.feature.discover.DiscoverScreen
import com.fluxa.app.shared.feature.discover.DiscoverUiState
import com.fluxa.app.shared.feature.library.LibraryAction
import com.fluxa.app.shared.feature.library.LibraryFolderDetailScreen
import com.fluxa.app.shared.feature.library.LibraryScreen
import com.fluxa.app.shared.feature.library.LibraryUiState
import com.fluxa.app.shared.feature.plugins.PluginsAction
import com.fluxa.app.shared.feature.plugins.PluginsScreen
import com.fluxa.app.shared.feature.plugins.PluginsUiState
import com.fluxa.app.shared.feature.profile.ProfileAction
import com.fluxa.app.shared.feature.profile.ProfileEditScreen
import com.fluxa.app.shared.feature.profile.ProfileEditTarget
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.ProfileListScreen
import com.fluxa.app.shared.feature.profile.ProfileUiState
import com.fluxa.app.shared.feature.settings.SettingsAction
import com.fluxa.app.shared.feature.settings.SettingsScreen
import com.fluxa.app.shared.feature.settings.SettingsUiState
import com.fluxa.app.shared.feature.search.SearchAction
import com.fluxa.app.shared.feature.search.SearchScreen
import com.fluxa.app.shared.feature.search.SearchUiState
import com.fluxa.app.shared.feature.player.PlayerControlsSurface
import com.fluxa.app.shared.feature.player.PlayerRenderAction
import com.fluxa.app.shared.feature.player.PlayerRenderState
import com.fluxa.app.shared.feature.player.TrailerCue
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.cardRowSpacing
import com.fluxa.app.ui.catalog.LocalDeviceType
import com.fluxa.app.ui.catalog.LocalWindowWidthClass
import com.fluxa.app.ui.catalog.PosterActionSheet
import com.fluxa.app.ui.catalog.WindowWidthClass

@Composable
internal fun FluxaHomeContent(
    state: FluxaAppUiState,
    catalogHome: CatalogHomeUiState,
    onCatalogAction: (CatalogAction) -> Unit,
    onCategorySelected: (id: String, title: String) -> Unit,
    hideContinueWatchingLabels: Boolean = false,
    continueWatchingWidthPreset: String = "medium",
    continueWatchingCornerPreset: String = "medium",
    continueWatchingDensity: String = "medium",
    continueWatchingLandscapeMode: Boolean = true,
    bottomContentInset: androidx.compose.ui.unit.Dp = 24.dp,
    modifier: Modifier
) {
    if (catalogHome.isLoading && catalogHome.rows.isEmpty()) {
        FluxaHomeSkeleton(modifier = modifier)
        return
    }

    if (catalogHome.rows.isEmpty()) {
        FluxaDestinationPlaceholder(
            language = state.language,
            destination = FluxaDestination.Home,
            modifier = modifier
        )
        return
    }

    val heroItems = catalogHome.heroItems
    val showHero = catalogHome.showHeroSection && heroItems.isNotEmpty()
    val orderedRows = remember(catalogHome.rows) {
        val contentRows = catalogHome.rows.filterNot { it.categoryType == "collection_folder" }
        contentRows.filter { it.id == CONTINUE_WATCHING_CATEGORY_ID } +
            contentRows.filterNot { it.id == CONTINUE_WATCHING_CATEGORY_ID }
    }
    val isDesktop = LocalDeviceType.current == DeviceType.Desktop
    val listState = rememberLazyListState()
    var initialHeroResolved by remember { mutableStateOf(false) }
    var posterActionItem by remember { mutableStateOf<CatalogItemUiModel?>(null) }

    LaunchedEffect(showHero) {
        if (showHero && !initialHeroResolved) {
            listState.scrollToItem(0)
            initialHeroResolved = true
        }
    }

    Box(modifier = modifier) {
        LazyColumn(
            state = listState,
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = bottomContentInset),
            verticalArrangement = Arrangement.spacedBy(if (isDesktop && showHero) 10.dp else if (isDesktop) 18.dp else 24.dp)
        ) {
            if (showHero) {
                item(key = "hero") {
                    FluxaHomeHero(
                        items = heroItems,
                        billboard = catalogHome.billboard,
                        language = state.language,
                        onCatalogAction = onCatalogAction,
                        modifier = Modifier
                    )
                }
            }
            items(
                items = orderedRows,
                key = { it.id },
                contentType = { "catalog-row" },
            ) { row ->
                val rowState = rememberLazyListState()
                val shouldLoadMore by remember(row.id, row.canLoadMore) {
                    derivedStateOf {
                        val lastVisible = rowState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0
                        row.canLoadMore && row.items.isNotEmpty() && lastVisible >= row.items.lastIndex - 4
                    }
                }
                LaunchedEffect(shouldLoadMore, row.items.size) {
                    if (shouldLoadMore) {
                        onCatalogAction(CatalogAction.LoadMore(row.id))
                    }
                }
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .then(
                                if (row.canLoadMore) {
                                    Modifier.clickable { onCategorySelected(row.id, row.title) }
                                } else {
                                    Modifier
                                }
                            )
                            .padding(horizontal = if (isDesktop) 24.dp else 20.dp, vertical = 4.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = row.title,
                            color = Color.White,
                            fontWeight = FontWeight.SemiBold,
                            fontSize = if (isDesktop) 18.sp else 16.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f)
                        )
                        if (row.canLoadMore) {
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(6.dp),
                                modifier = Modifier.padding(start = 12.dp)
                            ) {
                                if (isDesktop) {
                                    Text(
                                        text = AppStrings.t(state.language, "common.view_all"),
                                        color = Color.White.copy(alpha = 0.62f),
                                        fontSize = 12.sp,
                                        fontWeight = FontWeight.Medium
                                    )
                                }
                                Icon(
                                    imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                                    contentDescription = AppStrings.t(state.language, "common.view_all"),
                                    tint = Color.White.copy(alpha = 0.7f),
                                    modifier = Modifier.size(if (isDesktop) 18.dp else 20.dp)
                                )
                            }
                        }
                    }
                    val rowModifier = if (isDesktop) {
                        Modifier.draggable(
                            orientation = Orientation.Horizontal,
                            state = rememberDraggableState { delta ->
                                // Desktop drag events can arrive much faster than frames. Dispatch the
                                // delta directly instead of launching one coroutine per pointer event.
                                rowState.dispatchRawDelta(-delta)
                            }
                        )
                    } else {
                        Modifier
                    }
                    val isContinueWatchingRow = row.id == CONTINUE_WATCHING_CATEGORY_ID
                    val deviceType = LocalDeviceType.current
                    LazyRow(
                        state = rowState,
                        modifier = rowModifier,
                        contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = if (isDesktop) 24.dp else 20.dp),
                        horizontalArrangement = Arrangement.spacedBy(
                            if (isContinueWatchingRow) cardRowSpacing(continueWatchingDensity) else if (isDesktop) 10.dp else 12.dp
                        )
                    ) {
                        items(row.items, key = { it.stableLazyKey() }, contentType = { "catalog-card" }) { item ->
                            val cardItem = remember(item, row.id, isDesktop, hideContinueWatchingLabels, continueWatchingWidthPreset, continueWatchingCornerPreset, continueWatchingLandscapeMode) {
                                if (isContinueWatchingRow) {
                                    item.withProminentContinueWatchingCard(
                                        deviceType = deviceType,
                                        isDesktop = isDesktop,
                                        hideLabels = hideContinueWatchingLabels,
                                        widthPreset = continueWatchingWidthPreset,
                                        cornerPreset = continueWatchingCornerPreset,
                                        landscapeMode = continueWatchingLandscapeMode
                                    )
                                } else {
                                    item
                                }
                            }
                            CatalogCard(
                                model = cardItem.card,
                                onClick = { onCatalogAction(CatalogAction.ItemSelected(cardItem)) },
                                onLongClick = { posterActionItem = cardItem }
                            )
                        }
                    }
                }
            }
        }
    }

    posterActionItem?.let { item ->
        PosterActionSheet(
            item = item,
            language = state.language,
            onDismiss = { posterActionItem = null },
            onAddToLibrary = {
                posterActionItem = null
                onCatalogAction(CatalogAction.AddToLibraryRequested(item))
            }
        )
    }
}

@Composable
private fun FluxaHomeSkeleton(modifier: Modifier = Modifier) {
    val isDesktop = LocalDeviceType.current == DeviceType.Desktop
    LazyColumn(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(if (isDesktop) 18.dp else 24.dp)
    ) {
        item(key = "hero-skeleton") {
            Box(
                modifier = (if (isDesktop) {
                    Modifier.fillMaxWidth().height(760.dp)
                } else {
                    Modifier.fillMaxWidth().aspectRatio(3f / 4f)
                }).skeletonShimmer(shape = androidx.compose.ui.graphics.RectangleShape)
            )
        }
        items(3, key = { "row-skeleton-$it" }) {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Box(
                    modifier = Modifier
                        .padding(horizontal = 20.dp)
                        .width(120.dp)
                        .height(16.dp)
                        .skeletonShimmer()
                )
                LazyRow(
                    contentPadding = PaddingValues(horizontal = 20.dp),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    items(6, key = { skeletonIndex -> skeletonIndex }) {
                        Box(
                            modifier = Modifier
                                .width(110.dp)
                                .aspectRatio(2f / 3f)
                                .skeletonShimmer()
                        )
                    }
                }
            }
        }
    }
}

private const val HERO_AUTO_SWIPE_DELAY_MS = 5000L
private const val DESKTOP_HERO_AUTO_SWIPE_DELAY_MS = 18000L

@Composable
private fun FluxaHomeHero(
    items: List<CatalogItemUiModel>,
    billboard: CatalogBillboardUiModel?,
    language: String?,
    onCatalogAction: (CatalogAction) -> Unit,
    modifier: Modifier = Modifier
) {
    val loop = items.size > 1
    val virtualPageCount = if (loop) Int.MAX_VALUE else items.size
    val startPage = if (loop) (Int.MAX_VALUE / 2) - (Int.MAX_VALUE / 2) % items.size else 0
    val pagerState = rememberPagerState(initialPage = startPage, pageCount = { virtualPageCount })
    val coroutineScope = rememberCoroutineScope()
    var mouseDragDistance by remember { mutableFloatStateOf(0f) }
    val mouseDragState = rememberDraggableState { delta -> mouseDragDistance += delta }
    val isExpanded = LocalWindowWidthClass.current == WindowWidthClass.Expanded
    val isDesktop = LocalDeviceType.current == DeviceType.Desktop
    val heroAspectRatio = if (isExpanded) 12f / 5f else 3f / 4f
    val heroMaxHeight = if (isExpanded) 440.dp else 560.dp

    LaunchedEffect(pagerState.currentPage) {
        onCatalogAction(CatalogAction.HeroPageChanged(items[pagerState.currentPage % items.size]))
    }

    if (loop) {
        LaunchedEffect(items.size) {
            while (true) {
                delay(if (isDesktop) DESKTOP_HERO_AUTO_SWIPE_DELAY_MS else HERO_AUTO_SWIPE_DELAY_MS)
                if (!pagerState.isScrollInProgress) {
                    runCatching { pagerState.animateScrollToPage(pagerState.currentPage + 1) }
                }
            }
        }
    }

    Box(
        modifier = modifier
            .fillMaxWidth()
            .then(
                if (isDesktop && loop) {
                    Modifier.draggable(
                        state = mouseDragState,
                        orientation = Orientation.Horizontal,
                        onDragStarted = { mouseDragDistance = 0f },
                        onDragStopped = { velocity ->
                            val shouldPage = abs(mouseDragDistance) >= 72f || abs(velocity) >= 650f
                            if (shouldPage) {
                                val delta = if (mouseDragDistance < 0f || (mouseDragDistance == 0f && velocity < 0f)) 1 else -1
                                coroutineScope.launch {
                                    runCatching { pagerState.animateScrollToPage(pagerState.currentPage + delta) }
                                }
                            }
                            mouseDragDistance = 0f
                        }
                    )
                } else {
                    Modifier
                }
            )
    ) {
        HorizontalPager(
            state = pagerState,
            userScrollEnabled = !isDesktop && items.size > 1,
            modifier = if (isDesktop) {
                // Desktop uses a taller cinematic viewport: large enough for artwork and landing-page
                // typography while still leaving Continue Watching reachable on a 1080p window.
                Modifier.fillMaxWidth().height(760.dp)
            } else {
                Modifier
                    .fillMaxWidth()
                    .aspectRatio(heroAspectRatio)
                    .heightIn(max = heroMaxHeight)
            }
        ) { page ->
            val item = items[page % items.size]
            val activeBillboard = billboard?.takeIf { it.item.id == item.id && it.item.type == item.type }
            FluxaHomeHeroSlide(
                item = item,
                trailerUrl = activeBillboard?.trailerUrl,
                trailerSubtitleCues = activeBillboard?.trailerSubtitleCues.orEmpty(),
                language = language,
                isExpanded = isExpanded,
                onCatalogAction = onCatalogAction
            )
        }
        Box(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .height(if (isDesktop) 86.dp else 56.dp)
                .background(
                    Brush.verticalGradient(
                        colorStops = arrayOf(
                            0f to Color.Transparent,
                            0.58f to FluxaColors.background.copy(alpha = 0.28f),
                            1f to FluxaColors.background
                        )
                    )
                )
        )
        if (items.size > 1 && isDesktop) {
            Row(
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(end = 28.dp, bottom = 24.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                HeroPagerButton(
                    icon = Icons.AutoMirrored.Filled.ArrowBack,
                    contentDescription = "Previous hero item",
                    onClick = { coroutineScope.launch { pagerState.animateScrollToPage(pagerState.currentPage - 1) } }
                )
                HeroPagerButton(
                    icon = Icons.AutoMirrored.Filled.ArrowForward,
                    contentDescription = "Next hero item",
                    onClick = { coroutineScope.launch { pagerState.animateScrollToPage(pagerState.currentPage + 1) } }
                )
            }
        }
        if (items.size > 1) {
            val current = pagerState.currentPage % items.size
            Row(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(bottom = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                items.indices.forEach { index ->
                    val selected = index == current
                    Box(
                        modifier = Modifier
                            .size(width = if (selected) 20.dp else 6.dp, height = 6.dp)
                            .background(
                                color = Color.White,
                                shape = RoundedCornerShape(50)
                            )
                    )
                }
            }
        }
    }
}

@Composable
private fun HeroPagerButton(
    icon: ImageVector,
    contentDescription: String,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    Box(
        modifier = modifier
            .size(38.dp)
            .clip(CircleShape)
            .onFocusChanged { focused = it.isFocused }
            .background(if (focused) Color.White else Color.Black.copy(alpha = 0.34f))
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(
            imageVector = icon,
            contentDescription = contentDescription,
            tint = if (focused) Color.Black else Color.White,
            modifier = Modifier.size(19.dp)
        )
    }
}

@Composable
private fun FluxaHomeHeroSlide(
    item: CatalogItemUiModel,
    trailerUrl: String?,
    trailerSubtitleCues: List<TrailerCue>,
    language: String?,
    isExpanded: Boolean,
    onCatalogAction: (CatalogAction) -> Unit
) {
    val isDesktop = LocalDeviceType.current == DeviceType.Desktop
    Box(
        modifier = Modifier
            .fillMaxSize()
            .clip(RoundedCornerShape(0.dp))
            .clickable { onCatalogAction(CatalogAction.ItemSelected(item)) }
    ) {
        val heroArtworkUrl = item.backdropUrl ?: item.card.artworkUrl
        FluxaRemoteImage(
            imageUrl = heroArtworkUrl,
            cacheKey = heroArtworkUrl?.let { "home-hero:$it" },
            contentDescription = item.card.title,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop,
            alignment = if (isDesktop) Alignment.Center else if (isExpanded) Alignment.TopCenter else Alignment.Center
        )

        val trailerSurface = LocalHeroTrailerSurface.current
        val trailerActive = trailerUrl != null && trailerSurface != null
        val activeSubtitle = remember(item.id, item.type) { mutableStateOf("") }
        if (trailerUrl != null && trailerSurface != null) {
            trailerSurface(
                trailerUrl,
                trailerSubtitleCues,
                { activeSubtitle.value = it },
                Modifier.fillMaxSize()
            )
        }

        // The desktop hero deliberately mirrors the composition used by large streaming
        // landing pages: keep the artwork readable on the right while creating a stable,
        // high-contrast text canvas on the lower-left. The bottom fade also blends the
        // hero into the first Home row instead of ending as a hard image rectangle.
        if (isDesktop) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(
                        Brush.horizontalGradient(
                            colorStops = arrayOf(
                                0.00f to FluxaColors.background.copy(alpha = 0.90f),
                                0.22f to FluxaColors.background.copy(alpha = 0.70f),
                                0.48f to FluxaColors.background.copy(alpha = 0.30f),
                                0.68f to Color.Transparent,
                                1.00f to Color.Transparent,
                            )
                        )
                    )
            )
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(
                        Brush.verticalGradient(
                            colorStops = arrayOf(
                                0.00f to Color.Black.copy(alpha = 0.06f),
                                0.56f to Color.Transparent,
                                0.82f to FluxaColors.background.copy(alpha = if (trailerActive) 0.38f else 0.22f),
                                1.00f to FluxaColors.background.copy(alpha = if (trailerActive) 0.88f else 0.70f),
                            )
                        )
                    )
            )
        } else {
            val gradientStartY by animateFloatAsState(
                targetValue = if (trailerActive) 0.55f else 0.35f,
                label = "hero-gradient-start"
            )
            val gradientMaxAlpha by animateFloatAsState(
                targetValue = if (trailerActive) 0.75f else 1f,
                label = "hero-gradient-alpha"
            )
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(
                        Brush.verticalGradient(
                            colors = listOf(Color.Transparent, FluxaColors.background.copy(alpha = gradientMaxAlpha)),
                            startY = gradientStartY
                        )
                    )
            )
        }

        val logoMaxHeight by animateDpAsState(
            targetValue = when {
                trailerActive -> if (isDesktop) 50.dp else 34.dp
                isDesktop -> 92.dp
                else -> 64.dp
            },
            label = "hero-logo-height"
        )
        val contentAlignment = if (isDesktop || isExpanded) Alignment.Start else Alignment.CenterHorizontally
        val textAlign = if (isDesktop || isExpanded) {
            androidx.compose.ui.text.style.TextAlign.Start
        } else {
            androidx.compose.ui.text.style.TextAlign.Center
        }

        Column(
            horizontalAlignment = contentAlignment,
            modifier = Modifier
                .align(if (isExpanded || isDesktop) Alignment.BottomStart else Alignment.BottomCenter)
                .then(
                    when {
                        isDesktop -> Modifier.fillMaxWidth(0.56f).widthIn(max = 720.dp)
                        isExpanded -> Modifier.fillMaxWidth(0.45f)
                        else -> Modifier.fillMaxWidth()
                    }
                )
                .padding(
                    start = if (isDesktop) 44.dp else if (isExpanded) 40.dp else 20.dp,
                    end = if (isDesktop) 28.dp else 20.dp,
                    top = 20.dp,
                    bottom = if (trailerActive) 34.dp else if (isDesktop) 48.dp else 44.dp
                ),
            verticalArrangement = Arrangement.spacedBy(if (isDesktop) 12.dp else 8.dp)
        ) {
            if (item.card.logoUrl != null) {
                FluxaRemoteImage(
                    imageUrl = item.card.logoUrl,
                    cacheKey = "home-hero-logo:${item.card.logoUrl}",
                    contentDescription = item.card.title,
                    modifier = Modifier
                        .heightIn(max = logoMaxHeight)
                        .widthIn(max = if (isDesktop) 380.dp else if (isExpanded) 260.dp else 220.dp),
                    contentScale = ContentScale.Fit,
                    trimTransparentPadding = true
                )
            } else if (!trailerActive) {
                Text(
                    text = item.card.title,
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    textAlign = textAlign,
                    fontSize = if (isDesktop) 48.sp else MaterialTheme.typography.headlineSmall.fontSize,
                    lineHeight = if (isDesktop) 50.sp else MaterialTheme.typography.headlineSmall.lineHeight
                )
            }

            if (!trailerActive) {
                if (isDesktop) {
                    val metadataParts = remember(
                        item.type,
                        item.genres,
                        item.releaseLabel,
                        item.seasonsCount,
                        item.runtimeLabel,
                        item.ageRating,
                        language,
                    ) {
                        buildList {
                            add(
                                if (item.type == "series") {
                                    AppStrings.t(language, "auto.series")
                                } else {
                                    AppStrings.t(language, "auto.movie")
                                }
                            )
                            item.genres.firstOrNull { it.isNotBlank() }?.let { add(it) }
                            item.releaseLabel?.takeIf { it.isNotBlank() }?.let { add(it) }
                            if (item.type == "series" && (item.seasonsCount ?: 0) > 0) {
                                add("${item.seasonsCount} ${AppStrings.t(language, "auto.seasons")}")
                            } else {
                                item.runtimeLabel?.takeIf { it.isNotBlank() }?.let { add(it) }
                            }
                            item.ageRating?.takeIf { it.isNotBlank() }?.let { add(it) }
                        }
                    }

                    if (metadataParts.isNotEmpty()) {
                        Row(
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            metadataParts.forEachIndexed { index, part ->
                                if (index > 0) {
                                    Text(
                                        text = "•",
                                        color = Color.White.copy(alpha = 0.48f),
                                        fontSize = 14.sp,
                                        fontWeight = FontWeight.Bold
                                    )
                                }
                                Text(
                                    text = part,
                                    color = Color.White.copy(alpha = 0.92f),
                                    fontSize = 15.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                            }
                        }
                    }

                    item.description?.takeIf { it.isNotBlank() }?.let { description ->
                        val shortenedDescription = remember(description) { shortenHeroSynopsis(description) }
                        Text(
                            text = shortenedDescription,
                            color = Color.White.copy(alpha = 0.94f),
                            fontSize = 18.sp,
                            lineHeight = 23.sp,
                            fontWeight = FontWeight.Medium,
                            maxLines = 3,
                            overflow = TextOverflow.Ellipsis,
                            textAlign = textAlign,
                            modifier = Modifier.widthIn(max = 650.dp)
                        )
                    }

                    Row(
                        modifier = Modifier.padding(top = 10.dp),
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Row(
                            modifier = Modifier
                                .height(50.dp)
                                .clip(RoundedCornerShape(12.dp))
                                .background(Color.White)
                                .clickable { onCatalogAction(CatalogAction.PlayRequested(item)) }
                                .padding(horizontal = 22.dp),
                            horizontalArrangement = Arrangement.Center,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Icon(
                                imageVector = Icons.Rounded.PlayArrow,
                                contentDescription = null,
                                tint = Color.Black,
                                modifier = Modifier.size(28.dp)
                            )
                            Text(
                                text = AppStrings.t(language, "common.play"),
                                color = Color.Black,
                                fontWeight = FontWeight.ExtraBold,
                                fontSize = 16.sp,
                                modifier = Modifier.padding(start = 6.dp)
                            )
                        }

                        Row(
                            modifier = Modifier
                                .height(50.dp)
                                .clip(RoundedCornerShape(12.dp))
                                .background(Color.White.copy(alpha = 0.18f))
                                .clickable { onCatalogAction(CatalogAction.ItemSelected(item)) }
                                .padding(horizontal = 22.dp),
                            horizontalArrangement = Arrangement.Center,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(
                                text = AppStrings.t(language, "home.more_info"),
                                color = Color.White,
                                fontWeight = FontWeight.Bold,
                                fontSize = 16.sp
                            )
                        }
                    }
                } else {
                    // Keep the existing Mobile/TV visual hierarchy untouched.
                    val badgeParts = remember(
                        item.genres,
                        item.ageRating,
                        item.type,
                        item.seasonsCount,
                        item.runtimeLabel,
                        language
                    ) {
                        buildList {
                            item.genres.firstOrNull { it.isNotBlank() }?.let { add(it) }
                            item.ageRating?.takeIf { it.isNotBlank() }?.let { add(it) }
                            if (item.type == "series" && (item.seasonsCount ?: 0) > 0) {
                                add("${item.seasonsCount} ${AppStrings.t(language, "auto.seasons")}")
                            } else {
                                item.runtimeLabel?.takeIf { it.isNotBlank() }?.let { add(it) }
                            }
                        }
                    }
                    if (badgeParts.isNotEmpty()) {
                        Row(
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            badgeParts.forEachIndexed { index, part ->
                                if (index > 0) {
                                    Text(
                                        text = "•",
                                        color = Color.White.copy(alpha = 0.5f),
                                        fontSize = 12.sp,
                                        fontWeight = FontWeight.Bold
                                    )
                                }
                                Text(text = part, color = Color.White.copy(alpha = 0.8f), fontSize = 13.sp)
                            }
                        }
                    }
                    item.description?.takeIf { it.isNotBlank() }?.let { description ->
                        val shortenedDescription = remember(description) { shortenHeroSynopsis(description) }
                        Text(
                            text = shortenedDescription,
                            color = Color.White.copy(alpha = 0.85f),
                            fontSize = 13.sp,
                            lineHeight = 15.sp,
                            maxLines = 3,
                            overflow = TextOverflow.Ellipsis,
                            textAlign = textAlign
                        )
                    }
                    Row(
                        modifier = Modifier
                            .padding(top = 6.dp)
                            .height(48.dp)
                            .clip(RoundedCornerShape(14.dp))
                            .background(Color.White)
                            .clickable { onCatalogAction(CatalogAction.PlayRequested(item)) }
                            .padding(horizontal = 24.dp),
                        horizontalArrangement = Arrangement.Center,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.PlayArrow,
                            contentDescription = null,
                            tint = Color.Black,
                            modifier = Modifier.size(26.dp)
                        )
                        Text(
                            text = AppStrings.t(language, "common.play"),
                            color = Color.Black,
                            fontWeight = FontWeight.ExtraBold,
                            fontSize = 15.sp,
                            modifier = Modifier.padding(start = 7.dp)
                        )
                    }
                }
            }
        }

        if (trailerActive) {
            HeroTrailerSubtitle(
                subtitleState = activeSubtitle,
                modifier = Modifier.align(Alignment.BottomCenter)
            )
        }
    }
}

@Composable
private fun HeroTrailerSubtitle(
    subtitleState: androidx.compose.runtime.State<String>,
    modifier: Modifier = Modifier,
) {
    val subtitle by subtitleState
    if (subtitle.isNotBlank()) {
        Text(
            text = subtitle,
            color = Color.White,
            fontWeight = FontWeight.Bold,
            fontSize = 16.sp,
            textAlign = androidx.compose.ui.text.style.TextAlign.Center,
            modifier = modifier
                .padding(bottom = 82.dp, start = 32.dp, end = 32.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(Color.Black.copy(alpha = 0.58f))
                .padding(horizontal = 10.dp, vertical = 4.dp)
        )
    }
}
