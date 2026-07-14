package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.shared.feature.catalog.CatalogBillboardUiModel
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
        homeViewModel.isLoading,
        homeViewModel.billboardMovie,
        homeViewModel.billboardLogo,
        homeViewModel.billboardTrailerUrl
    ) { categories, isLoading, billboardMovie, billboardLogo, billboardTrailerUrl ->
        val profile = activeProfile()
        CatalogHomeUiState(
            rows = orderHomeCategories(categories).map { category -> category.toRowUiModel(profile) },
            isLoading = isLoading,
            billboard = billboardMovie?.let { movie ->
                CatalogBillboardUiModel(
                    item = movie.toCatalogItemUiModel(category = null, profile = profile),
                    logoUrl = billboardLogo,
                    trailerUrl = billboardTrailerUrl
                )
            },
            showHeroSection = profile?.safeShowHeroSection != false
        )
    }

    override suspend fun refresh() {
        if (homeViewModel.categories.value.isNotEmpty() || homeViewModel.isLoading.value) return
        homeViewModel.loadInitialData(activeProfile())
    }

    override suspend fun loadMore(rowId: String) {
        homeViewModel.loadMore(rowId)
    }

    fun resolveMeta(id: String, type: String): Meta? =
        homeViewModel.categories.value.firstNotNullOfOrNull { category ->
            category.items.firstOrNull { it.id == id && it.type == type }
        } ?: homeViewModel.billboardMovie.value?.takeIf { it.id == id && it.type == type }

    private fun HomeCategory.toRowUiModel(profile: UserProfile?): CatalogRowUiModel = CatalogRowUiModel(
        id = id,
        title = displayHomeCategoryTitle(this, profile?.language),
        canLoadMore = canLoadMore,
        categoryType = type,
        cardLayout = resolveHomeCardLayout(this, profile),
        artworkPreference = resolveContinueWatchingArtworkPreference(this, profile),
        isActionRow = isContinueWatchingCategory() || id == "library",
        topTenEnabled = id in profile?.safeTopTenFeedToggles.orEmpty(),
        items = items.map { meta -> meta.toCatalogItemUiModel(category = this, profile = profile) }
    )

    private fun Meta.toCatalogItemUiModel(category: HomeCategory?, profile: UserProfile?): CatalogItemUiModel =
        CatalogItemUiModel(
            id = id,
            type = type,
            card = toCatalogCardUiModel(
                cardLayout = category?.let { resolveHomeCardLayout(it, profile) } ?: "poster",
                artworkPreference = category?.let { resolveContinueWatchingArtworkPreference(it, profile) },
                profile = profile,
                cardScale = 1f,
                showHorizontalLogo = true,
                topTenRank = null,
                isContinueWatchingCard = category?.isContinueWatchingCategory() == true,
                loadArtwork = true
            ),
            source = CatalogSourceUiModel(
                addonTransportUrl = category?.addonTransportUrl
                    ?: category?.catalogSources?.firstOrNull()?.transportUrl,
                catalogType = category?.catalogSources?.firstOrNull()?.type ?: category?.type
            ),
            resume = toCatalogResumeUiModel(),
            backdropUrl = homeHeroBackdrop()
        )
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
