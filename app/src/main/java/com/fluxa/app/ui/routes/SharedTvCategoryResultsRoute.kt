package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.catalog.TvCatalogHomeScreen
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.toCatalogItems

@Composable
internal fun SharedTvCategoryResultsRoute(
    categoryId: String,
    title: String,
    homeViewModel: HomeViewModel,
    activeProfile: UserProfile?,
    navigator: AppNavigator
) {
    val categories by homeViewModel.categories.collectAsState()
    val category = categories.firstOrNull { it.id == categoryId }
    val source = category?.let {
        CatalogSourceUiModel(
            addonTransportUrl = it.addonTransportUrl ?: it.catalogSources?.firstOrNull()?.transportUrl,
            catalogType = it.catalogSources?.firstOrNull()?.type ?: it.type
        )
    } ?: CatalogSourceUiModel()
    val row = category?.let {
        CatalogRowUiModel(
            id = it.id,
            title = it.name,
            canLoadMore = it.canLoadMore,
            items = it.items.toCatalogItems(activeProfile).map { item -> item.copy(source = source) }
        )
    }

    TvCatalogHomeScreen(
        state = CatalogHomeUiState(rows = listOfNotNull(row), isLoading = category == null),
        language = activeProfile?.language,
        onAction = { action ->
            when (action) {
                is CatalogAction.ItemSelected -> {
                    val item = action.item
                    navigator.navigateTo(
                        Screen.Detail(
                            type = item.type,
                            id = item.id,
                            sourceAddonTransportUrl = item.source.addonTransportUrl,
                            sourceAddonCatalogType = item.source.catalogType
                        )
                    )
                }
                is CatalogAction.LoadMore -> homeViewModel.loadMore(action.rowId)
                else -> Unit
            }
        },
        onHomeRequested = { navigator.navigateTo(Screen.Home, clearStack = true) },
        onSearchRequested = { navigator.navigateTo(Screen.Search, clearStack = true) },
        onLibraryRequested = { navigator.navigateTo(Screen.Watchlist, clearStack = true) },
        onDiscoverRequested = { navigator.navigateTo(Screen.Explore(), clearStack = true) },
        onSettingsRequested = { navigator.navigateTo(Screen.Settings(), clearStack = true) }
    )
}
