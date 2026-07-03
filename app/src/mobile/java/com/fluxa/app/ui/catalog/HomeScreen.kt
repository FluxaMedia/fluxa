@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(
    androidx.compose.foundation.ExperimentalFoundationApi::class,
    androidx.compose.material3.ExperimentalMaterial3Api::class
)

package com.fluxa.app.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyListPrefetchStrategy
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.fluxa.app.data.local.LibraryUserCollection
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
        initialFirstVisibleItemScrollOffset = viewModel.savedHomeScrollOffset,
        prefetchStrategy = remember { LazyListPrefetchStrategy(nestedPrefetchItemCount = 4) }
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
