@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)

package com.fluxa.app.shared.feature.catalog

import com.fluxa.app.ui.catalog.FluxaIcons

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Text
import androidx.compose.foundation.focusGroup
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.withFrameNanos
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.focusRestorer
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.cardRowSpacing
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.CONTINUE_WATCHING_CATEGORY_ID
import com.fluxa.app.ui.catalog.LocalFluxaThemePack
import com.fluxa.app.ui.catalog.LocalWindowWidthClass
import com.fluxa.app.ui.catalog.WindowWidthClass
import com.fluxa.app.shared.image.FluxaRemoteImage
import kotlinx.coroutines.launch

@Composable
fun TvCatalogHomeScreen(
    state: CatalogHomeUiState,
    onAction: (CatalogAction) -> Unit,
    language: String? = null,
    hideContinueWatchingLabels: Boolean = false,
    continueWatchingWidthPreset: String = "medium",
    continueWatchingCornerPreset: String = "medium",
    continueWatchingDensity: String = "medium",
    continueWatchingLandscapeMode: Boolean = true,
    heroFollowsFocusedItem: Boolean = true,
    sidebarCatalogFocusRequest: Int = 0,
    externalFirstCatalogFocus: FocusRequester? = null,
    externalHeroFocus: FocusRequester? = null,
    leftFocusRequester: FocusRequester? = null,
    onPosterFocusChanged: (FocusRequester) -> Unit = {},
    onHeroFocusChanged: (Boolean) -> Unit = {},
    modifier: Modifier = Modifier
) {
    val columnFocus = remember { FocusRequester() }
    val firstCatalogFocus = externalFirstCatalogFocus ?: remember { FocusRequester() }
    val heroFocus = externalHeroFocus ?: remember { FocusRequester() }
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()
    val compactLayout = LocalFluxaThemePack.current.layouts.home == "compact"
    val isExpanded = LocalWindowWidthClass.current == WindowWidthClass.Expanded
    var heroFocused by remember { mutableStateOf(false) }
    var focusedPoster by remember { mutableStateOf<CatalogItemUiModel?>(null) }
    var lastPosterFocusRequester by remember { mutableStateOf<FocusRequester?>(null) }
    var activeHeroItem by remember { mutableStateOf<CatalogItemUiModel?>(null) }
    val focusedHeroItem = focusedPoster.takeIf { heroFollowsFocusedItem && !heroFocused }
    val posterFocusTarget = lastPosterFocusRequester ?: firstCatalogFocus
    Box(modifier = modifier.fillMaxSize()) {
        if (state.rows.isEmpty()) {
            TvCatalogHomeLoading(Modifier.fillMaxSize())
        } else {
            val contentRows = state.rows.filterNot { it.categoryType == "collection_folder" }
            val orderedRows = contentRows.filter { it.id == CONTINUE_WATCHING_CATEGORY_ID } +
                contentRows.filterNot { it.id == CONTINUE_WATCHING_CATEGORY_ID }
            val heroItems = state.heroItems.ifEmpty {
                orderedRows.firstOrNull { it.id != CONTINUE_WATCHING_CATEGORY_ID }?.items.orEmpty()
            }
            LaunchedEffect(heroItems.firstOrNull()?.id) {
                if (activeHeroItem == null || heroItems.none { it.id == activeHeroItem?.id }) {
                    activeHeroItem = heroItems.firstOrNull()
                }
            }
            val showStickyHero = state.showHeroSection && !compactLayout && heroItems.isNotEmpty()
            val heroHeight = if (isExpanded) 360.dp else 432.dp
            val heroRailFocusScrollOffset = with(LocalDensity.current) { 120.dp.roundToPx() }
            val handleHeroFocus: (Boolean) -> Unit = { focused ->
                heroFocused = focused
                if (focused) focusedPoster = null
                onHeroFocusChanged(focused)
            }
            LaunchedEffect(showStickyHero, heroItems.firstOrNull()?.id) {
                withFrameNanos { }
                runCatching {
                    if (showStickyHero) heroFocus.requestFocus()
                    else firstCatalogFocus.requestFocus()
                }
            }
            LaunchedEffect(sidebarCatalogFocusRequest) {
                if (sidebarCatalogFocusRequest > 0) {
                    listState.scrollToItem(0, heroRailFocusScrollOffset)
                    withFrameNanos { }
                    posterFocusTarget.requestFocus()
                    withFrameNanos { }
                    posterFocusTarget.requestFocus()
                }
            }
            if (heroFocused && activeHeroItem != null) {
                FluxaRemoteImage(
                    imageUrl = activeHeroItem?.backdropUrl ?: activeHeroItem?.card?.artworkUrl,
                    cacheKey = "tv-page-hero:${activeHeroItem?.id}",
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
            } else if (focusedPoster != null) {
                Box(Modifier.fillMaxSize().background(Color.Black))
            }
            LazyColumn(
                modifier = Modifier
                    .fillMaxSize()
                    .focusRequester(columnFocus),
                state = listState,
                contentPadding = PaddingValues(top = 0.dp, bottom = 64.dp),
                verticalArrangement = Arrangement.spacedBy(if (compactLayout) 14.dp else 12.dp)
            ) {
                if (showStickyHero) {
                    item(key = "tv-hero") {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(heroHeight),
                            contentAlignment = Alignment.TopCenter
                        ) {
                            TvHeroRow(
                                items = heroItems,
                                language = language,
                                onItemClick = { onAction(CatalogAction.ItemSelected(it)) },
                                onPlayClick = { onAction(CatalogAction.PlayRequested(it)) },
                                modifier = Modifier.fillMaxSize(),
                                trailerItemId = state.billboard?.item?.id,
                                trailerUrl = state.billboard?.trailerUrl,
                                trailerSubtitleCues = state.billboard?.trailerSubtitleCues.orEmpty(),
                                focusedItemOverride = focusedHeroItem,
                                posterFocused = focusedPoster != null,
                                onActiveItemChanged = { activeHeroItem = it },
                                onFocusChanged = handleHeroFocus,
                                focusRequester = heroFocus,
                                downFocusRequester = posterFocusTarget,
                                onDownPressed = {
                                    scope.launch {
                                        listState.scrollToItem(0, heroRailFocusScrollOffset)
                                        withFrameNanos { }
                                        posterFocusTarget.requestFocus()
                                        withFrameNanos { }
                                        posterFocusTarget.requestFocus()
                                    }
                                }
                            )
                        }
                    }
                }
                items(orderedRows, key = { it.id }, contentType = { "catalog-row" }) { row ->
                    Column(
                        modifier = Modifier,
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Text(
                            text = row.title,
                            color = Color.White,
                            fontSize = 24.sp,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.padding(start = 76.dp, end = 24.dp)
                        )
                        val isContinueWatchingRow = row.id == CONTINUE_WATCHING_CATEGORY_ID
                        LazyRow(
                            modifier = Modifier.focusGroup(),
                            contentPadding = PaddingValues(start = 72.dp, end = 24.dp),
                            horizontalArrangement = Arrangement.spacedBy(
                                if (isContinueWatchingRow) cardRowSpacing(continueWatchingDensity) else 4.dp
                            )
                        ) {
                        itemsIndexed(row.items, key = { _, item -> item.stableLazyKey() }, contentType = { _, _ -> "catalog-card" }) { index, item ->
                                val cardItem = if (isContinueWatchingRow) {
                                    item.withProminentContinueWatchingCard(
                                        widthClass = com.fluxa.app.ui.catalog.LocalWindowWidthClass.current,
                                        hideLabels = hideContinueWatchingLabels,
                                        widthPreset = continueWatchingWidthPreset,
                                        cornerPreset = continueWatchingCornerPreset,
                                        landscapeMode = continueWatchingLandscapeMode
                                    )
                                } else {
                                    item
                                }
                                val itemFocus = if (
                                    row == orderedRows.firstOrNull() && item == row.items.firstOrNull()
                                ) firstCatalogFocus else remember(item.stableLazyKey()) { FocusRequester() }
                                CatalogCard(
                                    model = cardItem.card,
                                    onClick = { onAction(CatalogAction.ItemSelected(cardItem)) },
                                    modifier = Modifier
                                        .padding(4.dp)
                                        .focusRequester(itemFocus)
                                        .focusProperties {
                                            if (row == orderedRows.firstOrNull()) {
                                                up = heroFocus
                                            }
                                            if (index == 0) {
                                                leftFocusRequester?.let { left = it }
                                            }
                                        }
                                        .onFocusChanged {
                                            if (it.isFocused) {
                                                lastPosterFocusRequester = itemFocus
                                                onPosterFocusChanged(itemFocus)
                                                focusedPoster = cardItem
                                            } else if (focusedPoster?.stableLazyKey() == cardItem.stableLazyKey()) {
                                                scope.launch {
                                                    withFrameNanos { }
                                                    if (focusedPoster?.stableLazyKey() == cardItem.stableLazyKey()) {
                                                        focusedPoster = null
                                                    }
                                                }
                                            }
                                            if (it.isFocused && row.canLoadMore && index == row.items.lastIndex) {
                                                onAction(CatalogAction.LoadMore(row.id))
                                            }
                                        }
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun TvCatalogHomeLoading(modifier: Modifier) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 58.dp, vertical = 44.dp),
        verticalArrangement = Arrangement.spacedBy(28.dp)
    ) {
        repeat(3) {
            Box(
                modifier = Modifier
                    .fillMaxWidth(0.28f)
                    .height(28.dp)
                    .background(FluxaColors.surfaceRaised)
            )
            Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                repeat(6) {
                    Box(
                        modifier = Modifier
                            .width(132.dp)
                            .height(198.dp)
                            .background(FluxaColors.surfaceRaised)
                    )
                }
            }
        }
    }
}
