package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.discover.DiscoverDataSource
import com.fluxa.app.shared.feature.discover.DiscoverFilterOptionUiModel
import com.fluxa.app.shared.feature.discover.DiscoverFiltersUiModel
import com.fluxa.app.shared.feature.discover.DiscoverUiState
import com.fluxa.app.domain.discovery.DiscoverCatalogOption
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine

class AndroidDiscoverDataSource(
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?
) : DiscoverDataSource {
    private val filters = MutableStateFlow(DiscoverFiltersUiModel())
    private val catalogOptions = MutableStateFlow<List<DiscoverCatalogOption>>(emptyList())
    private val contentTypes = MutableStateFlow<List<String>>(emptyList())

    override fun observeDiscover(): Flow<DiscoverUiState> = combine(
        filters,
        homeViewModel.discoverUiState,
        catalogOptions,
        contentTypes
    ) { selectedFilters, state, localCatalogOptions, localContentTypes ->
        val profile = activeProfile()
        val language = profile?.language
        val visibleCatalogOptions = localCatalogOptions.ifEmpty { state.catalogs }
        val selectedCatalog = visibleCatalogOptions.firstOrNull { it.key == selectedFilters.catalogKey }
        DiscoverUiState(
            filters = selectedFilters,
            typeOptions = localContentTypes.ifEmpty { state.contentTypes }.map { type ->
                DiscoverFilterOptionUiModel(type, discoverContentTypeLabel(type, language))
            },
            catalogOptions = visibleCatalogOptions.map { DiscoverFilterOptionUiModel(it.key, it.label) },
            genreOptions = (selectedCatalog?.genres ?: state.genres.mapNotNull { it.id })
                .map { DiscoverFilterOptionUiModel(it, it) },
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
        val previousFilters = this.filters.value
        if (filters != previousFilters) homeViewModel.clearDiscoverResults()
        homeViewModel.setDiscoverLoading(true)
        val availableCatalogs = homeViewModel.discoverCatalogOptions(filters.contentType)
        catalogOptions.value = availableCatalogs
        contentTypes.value = homeViewModel.discoverContentTypes()
        val selectedCatalogKey = filters.catalogKey
            ?.takeIf { key -> availableCatalogs.any { it.key == key } }
            ?: availableCatalogs.firstOrNull()?.key
        if (selectedCatalogKey != null) {
            val selectedFilters = filters.copy(catalogKey = selectedCatalogKey)
            this.filters.value = selectedFilters
            homeViewModel.discover(
                type = selectedFilters.contentType,
                catalogKey = selectedFilters.catalogKey,
                genre = selectedFilters.genre,
                year = null,
                rating = null,
                provider = null,
                region = null
            )
            return
        }
        if (filters.catalogKey == null || homeViewModel.discoverUiState.value.catalogs.none { it.key == filters.catalogKey }) {
            homeViewModel.loadDiscoverCatalogFilters(filters.contentType, filters.catalogKey) { catalogs ->
                val selectedCatalogKey = catalogs.firstOrNull()?.key
                if (selectedCatalogKey == null) {
                    homeViewModel.setDiscoverLoading(false)
                    return@loadDiscoverCatalogFilters
                }
                if (this.filters.value == filters) {
                    catalogOptions.value = catalogs
                    contentTypes.value = catalogs.map { it.type }.distinct()
                    val selectedFilters = filters.copy(catalogKey = selectedCatalogKey)
                    this.filters.value = selectedFilters
                    homeViewModel.discover(
                        type = selectedFilters.contentType,
                        catalogKey = selectedFilters.catalogKey,
                        genre = selectedFilters.genre,
                        year = null,
                        rating = null,
                        provider = null,
                        region = null
                    )
                }
            }
            return
        }
        this.filters.value = filters
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
        val catalog = catalogOptions.value
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
