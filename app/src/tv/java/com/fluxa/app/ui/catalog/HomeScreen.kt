package com.fluxa.app.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta

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
    val billboardState = rememberHomeBillboardState(activeProfile, viewModel)
    val lang = activeProfile?.safeLanguage ?: "en"
    val heroMovie = billboardState.displayBillboardMovie
        ?: billboardState.filteredBillboardPool.firstOrNull()
        ?: billboardState.billboardPool.firstOrNull()
        ?: catalogState.orderedCategories.firstOrNull()?.items?.firstOrNull()
    val rowSpecs = remember(catalogState.orderedCategories, activeProfile, catalogState.topTenFeedKeys, lang) {
        catalogState.orderedCategories.map { category ->
            category.toTvHomeRowSpec(activeProfile, catalogState.topTenFeedKeys, lang)
        }
    }
    val firstShelfFocusRequester = remember { FocusRequester() }
    val heroPlayFocusRequester = remember { FocusRequester() }
    val stableOnMovieClickState = rememberUpdatedState(onMovieClick)
    val stableOnPlayDirectState = rememberUpdatedState(onPlayDirect)
    val stableLoadMoreCategory = remember<(String) -> Unit> { { categoryId -> viewModel.loadMore(categoryId) } }
    val onMovieFocus = remember<(Meta) -> Unit> { { meta -> viewModel.onMovieFocused(meta) } }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
    ) {
        if (!catalogState.isLoading && rowSpecs.isEmpty() && heroMovie == null) {
            HomeEmptyProviderState(lang)
            return@Box
        }

        HomeHeroBackdrop(
            selectedContent = heroMovie,
            activePreviewUrl = activePreviewUrl,
            sharedPlayer = sharedPlayer,
            seasonPostersOnHero = activeProfile?.safeHomeSeasonPostersOnHero ?: true
        )

        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(start = 84.dp),
            contentPadding = PaddingValues(top = 84.dp, bottom = 80.dp)
        ) {
            if (heroMovie != null && catalogState.showHeroSection) {
                item(key = "tv-home-hero", contentType = "hero") {
                    HomeHeroPanel(
                        movie = heroMovie,
                        logoUrl = billboardState.billboardLogo,
                        lang = lang,
                        playButtonFocusRequester = heroPlayFocusRequester,
                        belowHeroFocusRequester = firstShelfFocusRequester,
                        modifier = Modifier.padding(start = 42.dp, bottom = 32.dp),
                        onPlayClick = { heroMovie?.let { stableOnMovieClickState.value(it, null, null) } },
                        onInfoClick = { heroMovie?.let { stableOnMovieClickState.value(it, null, null) } }
                    )
                }
            }

            itemsIndexed(
                rowSpecs,
                key = { _, row -> row.categoryId },
                contentType = { _, row -> "tv-shelf:${row.categoryType}:${row.cardLayout}:${row.isActionRow}:${row.topTenEnabled}" }
            ) { index, row ->
                HomeShelfRow(
                    title = row.title,
                    items = row.items,
                    cardLayout = row.cardLayout,
                    artworkPreference = row.artworkPreference,
                    activeProfile = activeProfile,
                    topTenEnabled = row.topTenEnabled,
                    onItemClick = { meta ->
                        if (row.isActionRow) stableOnPlayDirectState.value(meta)
                        else stableOnMovieClickState.value(meta, row.addonTransportUrl, row.addonCatalogType)
                    },
                    onItemFocus = onMovieFocus,
                    firstItemFocusRequester = if (index == 0) firstShelfFocusRequester else null,
                    upFocusRequester = if (index == 0 && catalogState.showHeroSection && heroMovie != null) heroPlayFocusRequester else null,
                    onNeedMore = {
                        if (row.canLoadMore && !row.isActionRow) stableLoadMoreCategory(row.categoryId)
                    }
                )
            }
        }

        HomeNavRail(
            lang = lang,
            onSearchClick = onSearchClick,
            onWatchlistClick = onWatchlistClick,
            onExploreClick = { onExploreClick("movie", null) },
            onProfileClick = onProfileClick,
            contentFocusRequester = heroPlayFocusRequester
        )
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

private fun HomeCategory.toTvHomeRowSpec(
    activeProfile: UserProfile?,
    topTenFeedKeys: Set<String>,
    lang: String
): TvHomeRowSpec {
    val isActionRow = isContinueWatchingCategory() || id == "library"
    return TvHomeRowSpec(
        categoryId = id,
        categoryType = type,
        title = displayHomeCategoryTitle(this, lang),
        items = items,
        canLoadMore = canLoadMore,
        isActionRow = isActionRow,
        topTenEnabled = id in topTenFeedKeys,
        cardLayout = resolveHomeCardLayout(this, activeProfile),
        artworkPreference = resolveContinueWatchingArtworkPreference(this, activeProfile),
        addonTransportUrl = addonTransportUrl ?: catalogSources?.firstOrNull()?.transportUrl,
        addonCatalogType = catalogSources?.firstOrNull()?.type ?: type
    )
}
