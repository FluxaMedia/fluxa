package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import com.fluxa.app.shared.feature.discover.DiscoverAction
import com.fluxa.app.shared.feature.discover.DiscoverScreen
import com.fluxa.app.shared.feature.discover.DiscoverStore
import com.fluxa.app.shared.platform.FluxaDiscoverServices
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import kotlinx.coroutines.launch

@Composable
internal fun SharedTvDiscoverRoute(
    platformServices: FluxaDiscoverServices,
    navigator: AppNavigator,
    language: String?,
    initialType: String,
    initialGenre: String?
) {
    val scope = rememberCoroutineScope()
    val store = remember(platformServices.discoverDataSource) {
        DiscoverStore(platformServices.discoverDataSource, scope)
    }
    val state by store.state.collectAsState()

    LaunchedEffect(store, initialType, initialGenre) {
        store.dispatch(
            DiscoverAction.FiltersChanged(
                state.filters.copy(contentType = initialType, genre = initialGenre, catalogKey = null)
            )
        )
    }

    DiscoverScreen(
        state = state,
        language = language,
        onFiltersChanged = { filters ->
            scope.launch { store.dispatch(DiscoverAction.FiltersChanged(filters)) }
        },
        onItemSelected = { item ->
            navigator.navigateTo(
                Screen.Detail(
                    type = item.type,
                    id = item.id,
                    sourceAddonTransportUrl = item.source.addonTransportUrl,
                    sourceAddonCatalogType = item.source.catalogType
                )
            )
        },
        onLoadMore = {
            scope.launch { store.dispatch(DiscoverAction.LoadMore) }
        }
    )
}
