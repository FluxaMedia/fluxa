package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.shared.feature.catalog.CatalogBillboardUiModel
import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogResumeUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.catalog.toHomeCollectionRows
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.withContext
import com.fluxa.app.ui.catalog.formatRuntimeLabel

class AndroidCatalogHomeDataSource(
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?,
    private val profileManager: ProfileManager,
    private val deviceType: DeviceType = DeviceType.Mobile,
) : CatalogHomeDataSource {

    /**
     * Row/card mapping is intentionally isolated from billboard state. Billboard rotation,
     * trailer resolution and subtitle updates must not rebuild every row on the Home screen.
     */
    @OptIn(ExperimentalCoroutinesApi::class)
    override fun observeHome(): Flow<CatalogHomeUiState> {
        val rows = combine(
            homeViewModel.categories,
            homeViewModel.currentFilter,
            profileFlow(),
        ) { categories, filter, profile ->
            RowsInput(categories = categories, filter = filter, profile = profile)
        }.mapLatest { input ->
            withContext(Dispatchers.Default) {
                val profile = input.profile
                val orderedCategories = orderHomeCategories(input.categories, input.filter)
                RowsResolution(
                    rows = profile.toHomeCollectionRows(deviceType = deviceType) +
                        orderedCategories.map { category -> category.toRowUiModel(profile) },
                    categoriesByItem = input.categories.buildCategoryLookup(),
                    profile = profile,
                    filter = input.filter,
                )
            }
        }.distinctUntilChanged()

        return combine(
            rows,
            homeViewModel.isLoading,
            billboardResolution(),
        ) { rowsResolution, isLoading, billboard ->
            HomeInput(
                rows = rowsResolution,
                isLoading = isLoading,
                billboard = billboard,
            )
        }.mapLatest { input ->
            withContext(Dispatchers.Default) {
                val effectiveMovie = input.billboard.movie
                CatalogHomeUiState(
                    // Preserve this exact list reference when only billboard state changes.
                    rows = input.rows.rows,
                    isLoading = input.isLoading,
                    billboard = effectiveMovie?.let { movie ->
                        CatalogBillboardUiModel(
                            item = movie.toCatalogItemUiModel(
                                category = input.rows.categoriesByItem[movie.catalogLookupKey()],
                                profile = input.rows.profile,
                            ),
                            logoUrl = input.billboard.logoUrl,
                            trailerUrl = input.billboard.trailerUrl,
                            trailerSubtitleCues = input.billboard.trailerSubtitleCues,
                        )
                    },
                    heroItems = input.billboard.items.map { movie ->
                        movie.toCatalogItemUiModel(
                            category = input.rows.categoriesByItem[movie.catalogLookupKey()],
                            profile = input.rows.profile,
                        )
                    },
                    showHeroSection = input.rows.profile?.safeShowHeroSection != false,
                    activeFilter = input.rows.filter,
                )
            }
        }.distinctUntilChanged()
    }

    /** Avoid doing a full Home mapping synchronously during composition. */
    override fun initialHomeState(): CatalogHomeUiState = CatalogHomeUiState(
        isLoading = true,
        activeFilter = homeViewModel.currentFilter.value,
    )

    private data class RowsInput(
        val categories: List<HomeCategory>,
        val filter: String,
        val profile: UserProfile?,
    )

    private data class RowsResolution(
        val rows: List<CatalogRowUiModel>,
        val categoriesByItem: Map<String, HomeCategory>,
        val profile: UserProfile?,
        val filter: String,
    )

    private data class HomeInput(
        val rows: RowsResolution,
        val isLoading: Boolean,
        val billboard: BillboardResolution,
    )

    private fun profileFlow(): Flow<UserProfile?> = callbackFlow {
        val listener: () -> Unit = { trySend(activeProfile()) }
        trySend(activeProfile())
        profileManager.addChangeListener(listener)
        awaitClose { profileManager.removeChangeListener(listener) }
    }.distinctUntilChanged()

    private data class BillboardResolution(
        val movie: Meta?,
        val items: List<Meta>,
        val logoUrl: String?,
        val trailerUrl: String?,
        val trailerSubtitleCues: List<com.fluxa.app.shared.feature.player.TrailerCue>,
    )

    @OptIn(FlowPreview::class)
    private fun billboardResolution(): Flow<BillboardResolution> = combine(
        homeViewModel.billboardMovie,
        homeViewModel.billboardPool,
        homeViewModel.billboardLogo,
        homeViewModel.billboardTrailerUrl,
        homeViewModel.currentFilter,
    ) { billboardMovie, billboardPool, billboardLogo, billboardTrailerUrl, filter ->
        BillboardBase(billboardMovie, billboardPool, billboardLogo, billboardTrailerUrl, filter)
    }.combine(homeViewModel.billboardTrailerSubtitleCues) { base, cues ->
        resolveBillboardResolution(base.movie, base.pool, base.logo, base.trailerUrl, cues, base.filter)
    }
        // Movie/logo/trailer/cues are published through separate StateFlows. Coalesce the
        // back-to-back updates into a single UI emission instead of recomposing 3-4 times.
        .debounce(16L)
        .distinctUntilChanged()

    private data class BillboardBase(
        val movie: Meta?,
        val pool: List<Meta>,
        val logo: String?,
        val trailerUrl: String?,
        val filter: String,
    )

    private fun resolveBillboardResolution(
        billboardMovie: Meta?,
        billboardPool: List<Meta>,
        billboardLogo: String?,
        billboardTrailerUrl: String?,
        billboardTrailerSubtitleCues: List<com.fluxa.app.shared.feature.player.TrailerCue>,
        filter: String,
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
        return BillboardResolution(
            movie = effectiveMovie,
            items = heroItems.take(10),
            logoUrl = billboardLogo,
            trailerUrl = billboardTrailerUrl,
            trailerSubtitleCues = billboardTrailerSubtitleCues,
        )
    }

    private fun List<HomeCategory>.buildCategoryLookup(): Map<String, HomeCategory> = buildMap {
        for (category in this@buildCategoryLookup) {
            for (item in category.items) {
                putIfAbsent(item.catalogLookupKey(), category)
            }
        }
    }

    private fun Meta.catalogLookupKey(): String = "$type:$id"

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
        items = items.map { meta -> meta.toCatalogItemUiModel(category = this, profile = profile) },
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
            loadArtwork = true,
            deviceType = deviceType,
        )
        val providerSource = category?.isContinueWatchingCategory() == true
        val selectedProvider = profile?.safeContinueWatchingSource
        val providerId = if (providerSource) ThirdPartyProviderId.from(selectedProvider) else null
        return CatalogItemUiModel(
            id = id,
            type = type,
            card = if (providerId != null) card.copy(allowCoverFallback = false) else card,
            source = if (providerSource && profile != null) {
                CatalogSourceUiModel(
                    catalogType = type,
                    providerId = providerId?.key,
                    providerAccountId = providerId?.let(profile::providerAccountId),
                    strictProviderData = providerId != null,
                )
            } else {
                CatalogSourceUiModel(
                    addonTransportUrl = category?.addonTransportUrl
                        ?: category?.catalogSources?.firstOrNull()?.transportUrl,
                    catalogType = category?.catalogSources?.firstOrNull()?.type ?: category?.type,
                )
            },
            resume = toCatalogResumeUiModel(),
            posterUrl = poster,
            backdropUrl = if (providerSource) background else homeHeroBackdrop(),
            description = description,
            releaseLabel = releaseInfo,
            ratingLabel = imdbRating,
            ageRating = ageRating,
            genres = genres.orEmpty(),
            seasonsCount = seasonsCount,
            runtimeLabel = formatRuntimeLabel(runtime),
        )
    }
}

private fun Meta.resolveResumeProgressPercent(): Float? {
    resumeProgressPercent?.let { return it }
    val offset = timeOffset
    val total = duration
    if (offset != null && total != null && total > 0L) {
        return (offset.toFloat() / total.toFloat()) * 100f
    }
    return null
}

private fun Meta.toCatalogResumeUiModel(): CatalogResumeUiModel? {
    val progressPercent = resolveResumeProgressPercent()
    val positionMs = if (progressPercent == null) timeOffset ?: 0L else 0L
    if (lastVideoId == null && positionMs <= 0L && progressPercent == null) return null
    return CatalogResumeUiModel(
        positionMs = positionMs,
        durationMs = duration,
        videoId = lastVideoId,
        streamUrl = lastStreamUrl,
        streamTitle = lastStreamTitle,
        progressPercent = progressPercent,
    )
}
