package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogHomeStore
import com.fluxa.app.shared.feature.catalog.TvCatalogHomeScreen
import com.fluxa.app.shared.platform.FluxaPlatformServices
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import kotlinx.coroutines.launch

@Composable
internal fun SharedTvHomeRoute(
    platformServices: FluxaPlatformServices,
    navigator: AppNavigator,
    language: String?
) {
    val scope = rememberCoroutineScope()
    val store = remember(platformServices.catalogHomeDataSource) {
        CatalogHomeStore(platformServices.catalogHomeDataSource, scope)
    }
    val state by store.state.collectAsState()

    LaunchedEffect(store) {
        store.dispatch(CatalogAction.Refresh)
    }

    TvCatalogHomeScreen(
        state = state,
        language = language,
        onAction = { action ->
            when (action) {
                is CatalogAction.ItemSelected -> {
                    val item = action.item
                    if (item.type == "catalog_folder") {
                        navigator.navigateTo(Screen.CategoryResults(item.id, item.card.title))
                    } else {
                        navigator.navigateTo(
                            Screen.Detail(
                                type = item.type,
                                id = item.id,
                                sourceAddonTransportUrl = item.source.addonTransportUrl,
                                sourceAddonCatalogType = item.source.catalogType
                            )
                        )
                    }
                }
                else -> scope.launch { store.dispatch(action) }
            }
        },
        onSearchRequested = { navigator.navigateTo(Screen.Search) },
        onLibraryRequested = { navigator.navigateTo(Screen.Watchlist) },
        onDiscoverRequested = { navigator.navigateTo(Screen.Explore()) },
        onSettingsRequested = { navigator.navigateTo(Screen.Settings()) }
    )
}
