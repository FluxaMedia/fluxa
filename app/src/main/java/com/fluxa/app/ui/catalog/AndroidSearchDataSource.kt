package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
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
        homeViewModel.searchHistory,
        homeViewModel.isSearchLoading
    ) { value, results, history, isLoading ->
        SearchUiState(
            query = value,
            results = results.toCatalogItems(activeProfile()),
            recentItems = history.toCatalogItems(activeProfile()),
            isLoading = isLoading
        )
    }

    override suspend fun search(query: String) {
        this.query.value = query
        homeViewModel.search(query)
    }

    override suspend fun clearHistory() {
        homeViewModel.clearSearchHistory()
    }
}

internal fun List<Meta>.toCatalogItems(profile: UserProfile?): List<CatalogItemUiModel> {
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
            )
        )
    }
}
