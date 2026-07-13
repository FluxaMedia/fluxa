package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.SearchResultRow
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.search.SearchDataSource
import com.fluxa.app.shared.feature.search.SearchUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine

class AndroidSearchDataSource(
    private val homeViewModel: HomeViewModel,
    private val activeProfile: () -> UserProfile?
) : SearchDataSource {
    private val query = MutableStateFlow("")

    override fun observeSearch(): Flow<SearchUiState> = combine(
        query,
        homeViewModel.searchResults,
        homeViewModel.searchRows,
        homeViewModel.searchHistory,
        homeViewModel.isSearchLoading
    ) { value, results, rows, history, isLoading ->
        val sources = rows.toCatalogSourceMap()
        val profile = activeProfile()
        SearchUiState(
            query = value,
            results = results.toCatalogItems(profile, sources),
            resultRows = rows.map { row ->
                CatalogRowUiModel(
                    id = row.id,
                    title = row.title,
                    items = row.items.toCatalogItems(
                        profile,
                        mapOf(
                            *row.items.map { meta ->
                                "${meta.type}:${meta.id}" to CatalogSourceUiModel(row.sourceAddonTransportUrl, row.sourceAddonCatalogType)
                            }.toTypedArray()
                        )
                    )
                )
            },
            recentItems = history.toCatalogItems(profile),
            isLoading = isLoading
        )
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
    sources: Map<String, CatalogSourceUiModel> = emptyMap()
): List<CatalogItemUiModel> {
    val cardLayout = if (profile?.safePosterLandscapeMode == true) "horizontal" else profile?.safeCardLayout ?: "vertical"
    return map { meta ->
        CatalogItemUiModel(
            id = meta.id,
            type = meta.type,
            card = meta.toCatalogCardUiModel(
                cardLayout = cardLayout,
                artworkPreference = null,
                profile = profile,
                cardScale = 1f,
                showHorizontalLogo = true,
                topTenRank = null,
                isContinueWatchingCard = false,
                loadArtwork = true
            ),
            source = sources["${meta.type}:${meta.id}"] ?: sources[meta.id] ?: CatalogSourceUiModel()
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
