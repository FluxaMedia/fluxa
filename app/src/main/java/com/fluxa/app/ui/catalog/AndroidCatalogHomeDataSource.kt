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
        homeViewModel.currentFilter,
        billboardResolution(),
    ) { categories, isLoading, filter, billboardResolution ->
        val profile = activeProfile()
        val orderedCategories = orderHomeCategories(categories, filter)
        val effectiveBillboardMeta = billboardResolution.movie ?: orderedCategories.firstOrNull()?.items?.firstOrNull()
        CatalogHomeUiState(
            rows = orderedCategories.map { category -> category.toRowUiModel(profile) },
            isLoading = isLoading,
            billboard = effectiveBillboardMeta?.let { movie ->
                CatalogBillboardUiModel(
                    item = movie.toCatalogItemUiModel(category = null, profile = profile),
                    logoUrl = billboardResolution.logoUrl,
                    trailerUrl = billboardResolution.trailerUrl
                )
            },
            showHeroSection = profile?.safeShowHeroSection != false,
            activeFilter = filter
        )
    }

    private data class BillboardResolution(
        val movie: com.fluxa.app.data.remote.Meta?,
        val logoUrl: String?,
        val trailerUrl: String?
    )

    private fun billboardResolution(): Flow<BillboardResolution> = combine(
        homeViewModel.billboardMovie,
        homeViewModel.billboardPool,
        homeViewModel.billboardLogo,
        homeViewModel.billboardTrailerUrl,
        homeViewModel.currentFilter
    ) { billboardMovie, billboardPool, billboardLogo, billboardTrailerUrl, filter ->
        val filteredPool = billboardPool.filter { it.matchesFilter(filter) }
        val effectiveMovie = billboardMovie?.takeIf { it.matchesFilter(filter) }
            ?: filteredPool.firstOrNull()
            ?: billboardMovie
        BillboardResolution(effectiveMovie, billboardLogo, billboardTrailerUrl)
    }

    override suspend fun refresh() {
        if (homeViewModel.categories.value.isNotEmpty() || homeViewModel.isLoading.value) return
        homeViewModel.loadInitialData(activeProfile())
    }

    override suspend fun loadMore(rowId: String) {
        homeViewModel.loadMore(rowId)
    }

    override suspend fun setFilter(filter: String) {
        homeViewModel.setFilter(filter)
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
            backdropUrl = homeHeroBackdrop(),
            description = description,
            ageRating = ageRating,
            seasonsCount = seasonsCount
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
