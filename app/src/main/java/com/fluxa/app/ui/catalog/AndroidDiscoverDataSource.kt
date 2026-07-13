package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.discover.DiscoverDataSource
import com.fluxa.app.shared.feature.discover.DiscoverFilterOptionUiModel
import com.fluxa.app.shared.feature.discover.DiscoverFiltersUiModel
import com.fluxa.app.shared.feature.discover.DiscoverUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine

class AndroidDiscoverDataSource(
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?
) : DiscoverDataSource {
    private val filters = MutableStateFlow(DiscoverFiltersUiModel())

    override fun observeDiscover(): Flow<DiscoverUiState> = combine(
        filters,
        homeViewModel.discoverUiState
    ) { selectedFilters, state ->
        val profile = activeProfile()
        val language = profile?.language
        DiscoverUiState(
            filters = selectedFilters,
            typeOptions = state.contentTypes.map { type ->
                DiscoverFilterOptionUiModel(type, discoverContentTypeLabel(type, language))
            },
            catalogOptions = state.catalogs.map { DiscoverFilterOptionUiModel(it.key, it.label) },
            genreOptions = state.genres.map { DiscoverFilterOptionUiModel(it.id, it.label) },
            results = state.results.map { meta ->
                val source = state.resultSources["${meta.type}:${meta.id}"]
                    ?: state.resultSources[meta.id]
                CatalogItemUiModel(
                    id = meta.id,
                    type = meta.type,
                    card = meta.toCatalogCardUiModel(
                        cardLayout = if (profile?.safePosterLandscapeMode == true) "horizontal" else profile?.safeCardLayout ?: "vertical",
                        artworkPreference = null,
                        profile = profile,
                        cardScale = 1f,
                        showHorizontalLogo = true,
                        topTenRank = null,
                        isContinueWatchingCard = false,
                        loadArtwork = true
                    ),
                    source = CatalogSourceUiModel(source?.transportUrl, source?.type)
                )
            },
            isLoading = state.isLoading
        )
    }

    override suspend fun updateFilters(filters: DiscoverFiltersUiModel) {
        this.filters.value = filters
        homeViewModel.loadDiscoverCatalogFilters(filters.contentType, filters.catalogKey)
        homeViewModel.discover(
            type = filters.contentType,
            catalogKey = filters.catalogKey,
            genre = filters.genre,
            year = null,
            rating = null,
            provider = null,
            region = null
        )
    }

    override suspend fun loadMore() {
        val currentFilters = filters.value
        val catalog = homeViewModel.discoverUiState.value.catalogs
            .firstOrNull { it.key == currentFilters.catalogKey } ?: return
        homeViewModel.loadMoreDiscoverResults(
            transportUrl = catalog.transportUrl,
            contentType = catalog.type,
            catalogId = catalog.id,
            genre = currentFilters.genre
        )
    }
}

private fun discoverContentTypeLabel(type: String, language: String?): String {
    val key = when (type) {
        "movie" -> "auto.movie"
        "series" -> "auto.series"
        "anime" -> "auto.anime"
        else -> null
    }
    return key?.let { AppStrings.t(language, it) } ?: type.replaceFirstChar { it.uppercase() }
}
