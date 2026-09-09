package com.fluxa.app.shared

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
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
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.RoundedCornerShape
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
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.stableLazyKey
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.withProminentContinueWatchingCard
import com.fluxa.app.shared.feature.catalog.CategoryResultsScreen
import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.ui.catalog.CONTINUE_WATCHING_CATEGORY_ID
import com.fluxa.app.ui.catalog.FluxaDimensions
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
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.cardRowSpacing
import com.fluxa.app.ui.catalog.LocalDeviceType
import com.fluxa.app.ui.catalog.LocalFluxaThemePack
import com.fluxa.app.ui.catalog.PosterActionSheet
import com.fluxa.app.ui.catalog.FluxaUiLayoutTokens
import com.fluxa.app.ui.catalog.LocalWindowWidthClass

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
    val widthClass = LocalWindowWidthClass.current
    val homeLayout = LocalFluxaThemePack.current.layouts.home
    val compactLayout = homeLayout == "compact"
    val catalogRowSpacing = when {
        compactLayout || widthClass == com.fluxa.app.ui.catalog.WindowWidthClass.Compact -> FluxaUiLayoutTokens.Compact.Dp.verticalSpacing
        isDesktop && showHero -> 10.dp
        isDesktop -> 18.dp
        else -> 24.dp
    }
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
            verticalArrangement = Arrangement.spacedBy(0.dp)
        ) {
            if (showHero && !compactLayout) {
                item(key = "hero") {
                    FluxaHomeHero(
                        items = heroItems,
                        billboard = catalogHome.billboard,
                        language = state.language,
                        onCatalogAction = onCatalogAction,
                        modifier = Modifier.padding(bottom = catalogRowSpacing)
                    )
                }
            }
            itemsIndexed(
                items = orderedRows,
                key = { _, row -> row.id },
                contentType = { _, _ -> "catalog-row" },
            ) { rowIndex, row ->
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
                val isContinueWatchingRow = row.id == CONTINUE_WATCHING_CATEGORY_ID
                val hasEpisodeLabels = row.items.any { it.card.subtitle.isNotBlank() }
                val continueWatchingMetadataExtraHeight =
                    FluxaDimensions.cardMetaBarWithEpisodeLabelHeight - FluxaDimensions.cardMetaBarHeight
                val rowSpacingAfter = when {
                    rowIndex == orderedRows.lastIndex -> 0.dp
                    isContinueWatchingRow && !hideContinueWatchingLabels && hasEpisodeLabels ->
                        (catalogRowSpacing - continueWatchingMetadataExtraHeight).coerceAtLeast(0.dp)
                    else -> catalogRowSpacing
                }
                Column(
                    modifier = Modifier.padding(bottom = rowSpacingAfter),
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
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
                                    imageVector = FluxaIcons.AutoMirrored.Filled.ArrowForward,
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
                    val deviceType = LocalDeviceType.current
                    val widthClass = com.fluxa.app.ui.catalog.LocalWindowWidthClass.current
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
                                        widthClass = widthClass,
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
        verticalArrangement = Arrangement.spacedBy(
            if (isDesktop) 18.dp else FluxaUiLayoutTokens.Compact.Dp.verticalSpacing
        )
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
