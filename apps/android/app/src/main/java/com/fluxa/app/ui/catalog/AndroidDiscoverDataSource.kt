package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.discover.DiscoverDataSource
import com.fluxa.app.shared.feature.discover.DiscoverFilterOptionUiModel
import com.fluxa.app.shared.feature.discover.DiscoverFiltersUiModel
import com.fluxa.app.shared.feature.discover.DiscoverUiState
import com.fluxa.app.domain.discovery.DiscoverCatalogOption
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.withContext

class AndroidDiscoverDataSource(
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?,
    private val deviceType: DeviceType = DeviceType.Mobile,
) : DiscoverDataSource {
    private val filters = MutableStateFlow(DiscoverFiltersUiModel())
    private val catalogOptions = MutableStateFlow<List<DiscoverCatalogOption>>(emptyList())
    private val contentTypes = MutableStateFlow<List<String>>(emptyList())
    private var catalogProfileId: String? = null

    override fun observeDiscover(): Flow<DiscoverUiState> = combine(
        filters,
        homeViewModel.discoverUiState,
        catalogOptions,
        contentTypes
    ) { selectedFilters, state, localCatalogOptions, localContentTypes ->
        val profile = activeProfile()
        withContext(Dispatchers.Default) {
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
                            cardLayout = profile?.safeCardLayout ?: "vertical",
                            artworkPreference = null,
                            profile = profile,
                            cardScale = if (deviceType == DeviceType.TV) 1.45f else 1f,
                            showHorizontalLogo = true,
                            topTenRank = null,
                            isContinueWatchingCard = false,
                            loadArtwork = true,
                            deviceType = deviceType,
                        ),
                        source = CatalogSourceUiModel(source?.transportUrl, source?.type)
                    )
                },
                isLoading = state.isLoading
            )
        }
    }

    override suspend fun updateFilters(filters: DiscoverFiltersUiModel) {
        val previousFilters = this.filters.value
        if (filters != previousFilters) homeViewModel.clearDiscoverResults()
        this.filters.value = filters
        homeViewModel.setDiscoverLoading(true)
        val profileId = activeProfile()?.id
        val canUseCachedCatalogs = filters.contentType == previousFilters.contentType &&
            catalogOptions.value.isNotEmpty() && catalogProfileId == profileId
        if (canUseCachedCatalogs) {
            resolveAndDiscover(filters, catalogOptions.value)
        } else {
            homeViewModel.loadDiscoverCatalogFilters(filters.contentType, filters.catalogKey) { catalogs, types ->
                if (this.filters.value != filters) return@loadDiscoverCatalogFilters
                catalogOptions.value = catalogs
                contentTypes.value = types
                catalogProfileId = profileId
                // Core's single discoverRequested action has already loaded
                // catalogs and run the selected/default catalog. Only project
                // its resolved selection into the legacy Compose state here;
                // do not dispatch a second discover request.
                val plan = FluxaCoreNative.discoverSelectionPlan(
                    contentType = filters.contentType,
                    catalogs = catalogs,
                    selectedCatalogKey = filters.catalogKey,
                    extraValue = filters.genre
                )
                this.filters.value = filters.copy(
                    catalogKey = plan.selectedCatalogKey,
                    genre = plan.extraValue
                )
            }
        }
    }

    private fun resolveAndDiscover(filters: DiscoverFiltersUiModel, catalogs: List<DiscoverCatalogOption>) {
        val plan = FluxaCoreNative.discoverSelectionPlan(
            contentType = filters.contentType,
            catalogs = catalogs,
            selectedCatalogKey = filters.catalogKey,
            extraValue = filters.genre
        )
        val selectedCatalogKey = plan.selectedCatalogKey
        if (selectedCatalogKey == null) {
            homeViewModel.setDiscoverLoading(false)
            return
        }
        val selectedFilters = filters.copy(catalogKey = selectedCatalogKey, genre = plan.extraValue)
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
