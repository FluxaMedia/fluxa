package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.SearchResultRow
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.search.SearchDataSource
import com.fluxa.app.shared.feature.search.SearchUiState
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.withContext
import com.fluxa.app.ui.catalog.formatRuntimeLabel

class AndroidSearchDataSource(
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?,
    private val deviceType: DeviceType = DeviceType.Mobile,
) : SearchDataSource {
    private val query = MutableStateFlow("")

    override fun observeSearch(): Flow<SearchUiState> = combine(
        query,
        homeViewModel.searchResults,
        homeViewModel.searchRows,
        homeViewModel.searchHistory,
        homeViewModel.isSearchLoading
    ) { value, results, rows, history, isLoading ->
        val profile = activeProfile()
        withContext(Dispatchers.Default) {
            val sources = rows.toCatalogSourceMap()
            SearchUiState(
                query = value,
                results = results.toCatalogItems(profile, sources, deviceType),
                resultRows = rows.map { row ->
                    val source = CatalogSourceUiModel(
                        addonTransportUrl = row.sourceAddonTransportUrl,
                        catalogType = row.sourceAddonCatalogType,
                    )
                    CatalogRowUiModel(
                        id = row.id,
                        title = row.title,
                        items = row.items.toCatalogItems(
                            profile = profile,
                            sources = row.items.associate { meta ->
                                "${meta.type}:${meta.id}" to source
                            },
                            deviceType = deviceType,
                        )
                    )
                },
                recentItems = history.toCatalogItems(profile, deviceType = deviceType),
                isLoading = isLoading
            )
        }
    }

    override suspend fun search(query: String) {
        this.query.value = query
        homeViewModel.search(query)
    }

    override suspend fun recordSelection(item: CatalogItemUiModel) {
        homeViewModel.recordSearchSelection(item.id, item.type)
    }

    override suspend fun clearHistory() {
        homeViewModel.clearSearchHistory()
    }
}

internal fun List<Meta>.toCatalogItems(
    profile: UserProfile?,
    sources: Map<String, CatalogSourceUiModel> = emptyMap(),
    deviceType: DeviceType = DeviceType.Mobile,
): List<CatalogItemUiModel> {
    val cardLayout = profile?.safeCardLayout ?: "vertical"
    return map { meta ->
        val source = sources["${meta.type}:${meta.id}"] ?: sources[meta.id] ?: CatalogSourceUiModel()
        val card = meta.toCatalogCardUiModel(
            cardLayout = cardLayout,
            artworkPreference = null,
            profile = profile,
            cardScale = 1f,
            showHorizontalLogo = true,
            topTenRank = null,
            isContinueWatchingCard = false,
            loadArtwork = true,
            deviceType = deviceType,
        )
        CatalogItemUiModel(
            id = meta.id,
            type = meta.type,
            card = if (source.strictProviderData) card.copy(allowCoverFallback = false) else card,
            source = source,
            posterUrl = meta.poster,
            backdropUrl = meta.background,
            description = meta.description,
            releaseLabel = meta.releaseInfo,
            ratingLabel = meta.imdbRating,
            ageRating = meta.ageRating,
            genres = meta.genres.orEmpty(),
            seasonsCount = meta.seasonsCount,
            runtimeLabel = formatRuntimeLabel(meta.runtime)
        )
    }
}

private fun List<SearchResultRow>.toCatalogSourceMap(): Map<String, CatalogSourceUiModel> {
    return flatMap { row ->
        val source = CatalogSourceUiModel(
            addonTransportUrl = row.sourceAddonTransportUrl,
            catalogType = row.sourceAddonCatalogType
        )
        row.items.map { meta ->
            "${meta.type}:${meta.id}" to source
        }
    }.toMap()
}