package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogResumeUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine

class AndroidCatalogHomeDataSource(
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?
) : CatalogHomeDataSource {
    override fun observeHome(): Flow<CatalogHomeUiState> = combine(
        homeViewModel.categories,
        homeViewModel.isLoading
    ) { categories, isLoading ->
        val profile = activeProfile()
        CatalogHomeUiState(
            rows = orderHomeCategories(categories).map { category ->
                CatalogRowUiModel(
                    id = category.id,
                    title = displayHomeCategoryTitle(category, profile?.language),
                    canLoadMore = category.canLoadMore,
                    items = category.items.map { meta ->
                        CatalogItemUiModel(
                            id = meta.id,
                            type = meta.type,
                            card = meta.toCatalogCardUiModel(
                                cardLayout = resolveHomeCardLayout(category, profile),
                                artworkPreference = resolveContinueWatchingArtworkPreference(category, profile),
                                profile = profile,
                                cardScale = 1f,
                                showHorizontalLogo = true,
                                topTenRank = null,
                                isContinueWatchingCard = category.isContinueWatchingCategory(),
                                loadArtwork = true
                            ),
                            source = CatalogSourceUiModel(
                                addonTransportUrl = category.addonTransportUrl
                                    ?: category.catalogSources?.firstOrNull()?.transportUrl,
                                catalogType = category.catalogSources?.firstOrNull()?.type ?: category.type
                            ),
                            resume = meta.toCatalogResumeUiModel(),
                            backdropUrl = meta.homeHeroBackdrop()
                        )
                    }
                )
            },
            isLoading = isLoading
        )
    }

    override suspend fun refresh() {
        if (homeViewModel.categories.value.isNotEmpty() || homeViewModel.isLoading.value) return
        homeViewModel.loadInitialData(activeProfile())
    }

    override suspend fun loadMore(rowId: String) {
        homeViewModel.loadMore(rowId)
    }
}

private fun com.fluxa.app.data.remote.Meta.toCatalogResumeUiModel(): CatalogResumeUiModel? {
    val positionMs = timeOffset ?: 0L
    if (lastVideoId == null && positionMs <= 0L) return null
    return CatalogResumeUiModel(
        positionMs = positionMs,
        durationMs = duration,
        videoId = lastVideoId,
        streamUrl = lastStreamUrl,
        streamTitle = lastStreamTitle
    )
}
