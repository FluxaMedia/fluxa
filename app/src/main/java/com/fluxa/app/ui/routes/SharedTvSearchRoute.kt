package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import com.fluxa.app.shared.feature.search.SearchAction
import com.fluxa.app.shared.feature.search.SearchScreen
import com.fluxa.app.shared.feature.search.SearchStore
import com.fluxa.app.shared.platform.FluxaSearchServices
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import kotlinx.coroutines.launch

@Composable
internal fun SharedTvSearchRoute(
    platformServices: FluxaSearchServices,
    navigator: AppNavigator,
    language: String?
) {
    val scope = rememberCoroutineScope()
    val store = remember(platformServices.searchDataSource) {
        SearchStore(platformServices.searchDataSource, scope)
    }
    val state by store.state.collectAsState()

    SearchScreen(
        state = state,
        language = language,
        onQueryChanged = { value ->
            scope.launch { store.dispatch(SearchAction.QueryChanged(value)) }
        },
        onItemSelected = { item ->
            scope.launch { store.dispatch(SearchAction.ItemSelected(item)) }
            navigator.navigateTo(
                Screen.Detail(
                    type = item.type,
                    id = item.id,
                    sourceAddonTransportUrl = item.source.addonTransportUrl,
                    sourceAddonCatalogType = item.source.catalogType
                )
            )
        },
        onClearHistory = {
            scope.launch { store.dispatch(SearchAction.ClearHistory) }
        }
    )
}
