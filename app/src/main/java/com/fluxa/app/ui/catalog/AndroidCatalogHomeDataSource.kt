package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.shared.feature.catalog.CatalogBillboardUiModel
import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogResumeUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.library.toCatalogCardUiModel
import com.fluxa.app.shared.feature.library.LibraryFolderUiModel
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
        homeState(categories, isLoading, filter, billboardResolution)
    }

    override fun initialHomeState(): CatalogHomeUiState = homeState(
        categories = homeViewModel.categories.value,
        isLoading = homeViewModel.isLoading.value,
        filter = homeViewModel.currentFilter.value,
        billboardResolution = resolveBillboardResolution(
            billboardMovie = homeViewModel.billboardMovie.value,
            billboardPool = homeViewModel.billboardPool.value,
            billboardLogo = homeViewModel.billboardLogo.value,
            billboardTrailerUrl = homeViewModel.billboardTrailerUrl.value,
            billboardTrailerSubtitleCues = homeViewModel.billboardTrailerSubtitleCues.value,
            filter = homeViewModel.currentFilter.value
        )
    )

    private fun homeState(
        categories: List<HomeCategory>,
        isLoading: Boolean,
        filter: String,
        billboardResolution: BillboardResolution
    ): CatalogHomeUiState {
        val profile = activeProfile()
        val orderedCategories = orderHomeCategories(categories, filter)
        val effectiveBillboardMeta = billboardResolution.movie
        return CatalogHomeUiState(
            rows = profile.homeCollectionRows() + orderedCategories.map { category -> category.toRowUiModel(profile) },
            isLoading = isLoading,
            billboard = effectiveBillboardMeta?.let { movie ->
                CatalogBillboardUiModel(
                    item = movie.toCatalogItemUiModel(
                        category = categories.categoryFor(movie),
                        profile = profile
                    ),
                    logoUrl = billboardResolution.logoUrl,
                    trailerUrl = billboardResolution.trailerUrl,
                    trailerSubtitleCues = billboardResolution.trailerSubtitleCues
                )
            },
            heroItems = billboardResolution.items.map { movie ->
                movie.toCatalogItemUiModel(
                    category = categories.categoryFor(movie),
                    profile = profile
                )
            },
            showHeroSection = profile?.safeShowHeroSection != false,
            activeFilter = filter
        )
    }

    private data class BillboardResolution(
        val movie: com.fluxa.app.data.remote.Meta?,
        val items: List<com.fluxa.app.data.remote.Meta>,
        val logoUrl: String?,
        val trailerUrl: String?,
        val trailerSubtitleCues: List<com.fluxa.app.shared.feature.player.TrailerCue>
    )

    private fun billboardResolution(): Flow<BillboardResolution> = combine(
        homeViewModel.billboardMovie,
        homeViewModel.billboardPool,
        homeViewModel.billboardLogo,
        homeViewModel.billboardTrailerUrl,
        homeViewModel.currentFilter
    ) { billboardMovie, billboardPool, billboardLogo, billboardTrailerUrl, filter ->
        BillboardBase(billboardMovie, billboardPool, billboardLogo, billboardTrailerUrl, filter)
    }.combine(homeViewModel.billboardTrailerSubtitleCues) { base, cues ->
        resolveBillboardResolution(base.movie, base.pool, base.logo, base.trailerUrl, cues, base.filter)
    }

    private data class BillboardBase(
        val movie: com.fluxa.app.data.remote.Meta?,
        val pool: List<com.fluxa.app.data.remote.Meta>,
        val logo: String?,
        val trailerUrl: String?,
        val filter: String
    )

    private fun resolveBillboardResolution(
        billboardMovie: com.fluxa.app.data.remote.Meta?,
        billboardPool: List<com.fluxa.app.data.remote.Meta>,
        billboardLogo: String?,
        billboardTrailerUrl: String?,
        billboardTrailerSubtitleCues: List<com.fluxa.app.shared.feature.player.TrailerCue>,
        filter: String
    ): BillboardResolution {
        val filteredPool = billboardPool.filter { it.matchesFilter(filter) }
        val effectiveMovie = billboardMovie?.takeIf { it.matchesFilter(filter) }
            ?: filteredPool.firstOrNull()
            ?: billboardMovie
        val effectiveMovieInPool = effectiveMovie != null && filteredPool.any { candidate ->
            candidate.id == effectiveMovie.id && candidate.type == effectiveMovie.type
        }
        val heroItems = if (effectiveMovieInPool) {
            filteredPool
        } else {
            listOfNotNull(effectiveMovie) + filteredPool
        }
        return BillboardResolution(effectiveMovie, heroItems, billboardLogo, billboardTrailerUrl, billboardTrailerSubtitleCues)
    }

    private fun List<HomeCategory>.categoryFor(meta: Meta): HomeCategory? = firstOrNull { category ->
        category.items.any { item -> item.id == meta.id && item.type == meta.type }
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
        artworkPreference = null,
        isActionRow = isContinueWatchingOrUpcomingCategory() || id == "library",
        topTenEnabled = id in profile?.safeTopTenFeedToggles.orEmpty(),
        items = items.map { meta -> meta.toCatalogItemUiModel(category = this, profile = profile) }
    )

    private fun Meta.toCatalogItemUiModel(category: HomeCategory?, profile: UserProfile?): CatalogItemUiModel {
        val card = toCatalogCardUiModel(
            cardLayout = category?.let { resolveHomeCardLayout(it, profile) } ?: "poster",
            artworkPreference = null,
            profile = profile,
            cardScale = 1f,
            showHorizontalLogo = true,
            topTenRank = null,
            isContinueWatchingCard = category?.isContinueWatchingOrUpcomingCategory() == true,
            loadArtwork = true
        )
        return CatalogItemUiModel(
            id = id,
            type = type,
            card = card,
            source = CatalogSourceUiModel(
                addonTransportUrl = category?.addonTransportUrl
                    ?: category?.catalogSources?.firstOrNull()?.transportUrl,
                catalogType = category?.catalogSources?.firstOrNull()?.type ?: category?.type
            ),
            resume = toCatalogResumeUiModel(),
            backdropUrl = homeHeroBackdrop(),
            description = description,
            ageRating = ageRating,
            seasonsCount = seasonsCount,
            runtimeLabel = runtime
        )
    }

    private fun UserProfile?.homeCollectionRows(): List<CatalogRowUiModel> {
        val profile = this ?: return emptyList()
        return profile.safeLibraryCollections
            .asSequence()
            .filter { it.showOnHome == true && it.folders.orEmpty().isNotEmpty() }
            .map { collection ->
                CatalogRowUiModel(
                    id = "collection:${collection.id}",
                    title = collection.title,
                    categoryType = "catalog_folder",
                    cardLayout = profile.safeCardLayout,
                    items = collection.folders.orEmpty().map { folder ->
                        CatalogItemUiModel(
                            id = folder.id,
                            type = "catalog_folder",
                            card = folder.toSharedUiModel().toCatalogCardUiModel(profile.safePosterWidthPreset ?: "medium"),
                            backdropUrl = folder.heroBackdropUrl ?: folder.effectiveImageUrl()
                        )
                    }
                )
            }
            .toList()
    }
}

private fun LibraryUserCollectionFolder.toSharedUiModel(): LibraryFolderUiModel = LibraryFolderUiModel(
    id = id,
    title = title,
    imageUrl = imageUrl,
    shape = shape,
    catalogTitle = catalogTitle,
    hideTitle = hideTitle == true,
    focusGifEnabled = focusGifEnabled != false,
    coverEmoji = coverEmoji,
    coverImageUrl = coverImageUrl,
    focusGifUrl = focusGifUrl,
    heroBackdropUrl = heroBackdropUrl
)

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
