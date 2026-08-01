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
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
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
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
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
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.ui.draw.alpha
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogAction
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
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.PosterActionSheet

@Composable
internal fun FluxaHomeContent(
    state: FluxaAppUiState,
    onCatalogAction: (CatalogAction) -> Unit,
    onCategorySelected: (id: String, title: String) -> Unit,
    bottomContentInset: androidx.compose.ui.unit.Dp = 24.dp,
    modifier: Modifier
) {
    if (state.catalogHome.isLoading && state.catalogHome.rows.isEmpty()) {
        FluxaHomeSkeleton(modifier = modifier)
        return
    }

    if (state.catalogHome.rows.isEmpty()) {
        FluxaDestinationPlaceholder(
            language = state.language,
            destination = FluxaDestination.Home,
            modifier = modifier
        )
        return
    }

    val heroItems = state.catalogHome.heroItems
    val showHero = state.catalogHome.showHeroSection && heroItems.isNotEmpty()
    val contentRows = state.catalogHome.rows.filterNot { it.categoryType == "collection_folder" }
    val orderedRows = contentRows.filter { it.id == CONTINUE_WATCHING_CATEGORY_ID } +
        contentRows.filterNot { it.id == CONTINUE_WATCHING_CATEGORY_ID }
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
            verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            if (showHero) {
                item(key = "hero") {
                    FluxaHomeHero(
                        items = heroItems,
                        billboard = state.catalogHome.billboard,
                        language = state.language,
                        onCatalogAction = onCatalogAction,
                        modifier = Modifier
                    )
                }
            }
            items(
                orderedRows,
                key = { it.id }
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
                            .padding(horizontal = 20.dp, vertical = 4.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = row.title,
                            color = Color.White,
                            fontWeight = FontWeight.SemiBold,
                            fontSize = 16.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f)
                        )
                        if (row.canLoadMore) {
                            Icon(
                                imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                                contentDescription = AppStrings.t(state.language, "common.view_all"),
                                tint = Color.White.copy(alpha = 0.7f),
                                modifier = Modifier
                                    .padding(start = 12.dp)
                                    .size(20.dp)
                            )
                        }
                    }
                    LazyRow(
                        state = rowState,
                        contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 20.dp),
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        items(row.items, key = { it.id }) { item ->
                            val cardItem = if (row.id == CONTINUE_WATCHING_CATEGORY_ID) {
                                item.withProminentContinueWatchingCard()
                            } else {
                                item
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
    LazyColumn(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        item(key = "hero-skeleton") {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(3f / 4f)
                    .skeletonShimmer(shape = androidx.compose.ui.graphics.RectangleShape)
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

    LaunchedEffect(pagerState.currentPage, pagerState.isScrollInProgress) {
        if (!pagerState.isScrollInProgress) {
            onCatalogAction(CatalogAction.HeroPageChanged(items[pagerState.currentPage % items.size]))
            if (loop) {
                delay(HERO_AUTO_SWIPE_DELAY_MS)
                if (!pagerState.isScrollInProgress) {
                    pagerState.animateScrollToPage(pagerState.currentPage + 1)
                }
            }
        }
    }

    Box(modifier = modifier.fillMaxWidth()) {
        HorizontalPager(
            state = pagerState,
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(3f / 4f)
                .heightIn(max = 560.dp)
        ) { page ->
            val item = items[page % items.size]
            val activeBillboard = billboard?.takeIf { it.item.id == item.id && it.item.type == item.type }
            FluxaHomeHeroSlide(
                item = item,
                trailerUrl = activeBillboard?.trailerUrl,
                trailerSubtitleCues = activeBillboard?.trailerSubtitleCues.orEmpty(),
                language = language,
                onCatalogAction = onCatalogAction
            )
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
                            .size(if (selected) 8.dp else 6.dp)
                            .background(
                                color = Color.White.copy(alpha = if (selected) 0.95f else 0.4f),
                                shape = CircleShape
                            )
                    )
                }
            }
        }
    }
}

@Composable
private fun FluxaHomeHeroSlide(
    item: CatalogItemUiModel,
    trailerUrl: String?,
    trailerSubtitleCues: List<TrailerCue>,
    language: String?,
    onCatalogAction: (CatalogAction) -> Unit
) {
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
            contentScale = ContentScale.Crop
        )
        val trailerSurface = LocalHeroTrailerSurface.current
        val trailerActive = trailerUrl != null && trailerSurface != null
        var activeSubtitle by remember(item.id, item.type) { mutableStateOf("") }
        if (trailerUrl != null && trailerSurface != null) {
            val trailerAlpha by animateFloatAsState(targetValue = 1f, label = "hero-trailer-fade")
            trailerSurface(trailerUrl, trailerSubtitleCues, { activeSubtitle = it }, Modifier.fillMaxSize().alpha(trailerAlpha))
        }
        val gradientStartY by animateFloatAsState(targetValue = if (trailerActive) 0.55f else 0.35f, label = "hero-gradient-start")
        val gradientMaxAlpha by animateFloatAsState(targetValue = if (trailerActive) 0.75f else 1f, label = "hero-gradient-alpha")
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
        val logoMaxHeight by animateDpAsState(targetValue = if (trailerActive) 34.dp else 64.dp, label = "hero-logo-height")
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .padding(start = 20.dp, end = 20.dp, top = 20.dp, bottom = if (trailerActive) 24.dp else 44.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            if (item.card.logoUrl != null) {
                FluxaRemoteImage(
                    imageUrl = item.card.logoUrl,
                    cacheKey = "home-hero-logo:${item.card.logoUrl}",
                    contentDescription = item.card.title,
                    modifier = Modifier.heightIn(max = logoMaxHeight).fillMaxWidth(0.65f),
                    contentScale = ContentScale.Fit
                )
            } else if (!trailerActive) {
                Text(
                    text = item.card.title,
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                    style = MaterialTheme.typography.headlineSmall
                )
            }
            val badgeParts = buildList {
                item.ageRating?.takeIf { it.isNotBlank() }?.let { add(it) }
                if (item.type == "series" && (item.seasonsCount ?: 0) > 0) {
                    add("${item.seasonsCount} ${AppStrings.t(language, "auto.seasons")}")
                } else {
                    item.runtimeLabel?.takeIf { it.isNotBlank() }?.let { add(it) }
                }
            }
            if (!trailerActive) {
                if (badgeParts.isNotEmpty()) {
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                        badgeParts.forEach { part ->
                            Text(text = part, color = Color.White.copy(alpha = 0.8f), fontSize = 13.sp)
                        }
                    }
                }
                item.description?.takeIf { it.isNotBlank() }?.let {
                    Text(
                        text = it,
                        color = Color.White.copy(alpha = 0.85f),
                        fontSize = 13.sp,
                        lineHeight = 15.sp,
                        maxLines = 3,
                        overflow = TextOverflow.Ellipsis,
                        textAlign = androidx.compose.ui.text.style.TextAlign.Center
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
        if (trailerActive && activeSubtitle.isNotBlank()) {
            Text(
                text = activeSubtitle,
                color = Color.White,
                fontWeight = FontWeight.Bold,
                fontSize = 16.sp,
                textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(bottom = 82.dp, start = 32.dp, end = 32.dp)
                    .clip(RoundedCornerShape(4.dp))
                    .background(Color.Black.copy(alpha = 0.58f))
                    .padding(horizontal = 10.dp, vertical = 4.dp)
            )
        }
    }
}
