package com.fluxa.app.shared

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeStore
import com.fluxa.app.shared.feature.detail.DetailAction
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.detail.DetailStore
import com.fluxa.app.shared.feature.search.SearchAction
import com.fluxa.app.shared.feature.search.SearchDataSource
import com.fluxa.app.shared.feature.search.SearchStore
import com.fluxa.app.shared.platform.FluxaDetailServices
import com.fluxa.app.shared.platform.FluxaPlatformServices
import com.fluxa.app.shared.platform.FluxaSearchServices
import kotlinx.coroutines.launch

@Composable
fun FluxaAppHost(
    platformServices: FluxaPlatformServices,
    language: String? = null,
    onCatalogAction: (CatalogAction) -> Unit = {},
    modifier: Modifier = Modifier
) {
    FluxaAppHost(
        catalogHomeDataSource = platformServices.catalogHomeDataSource,
        detailDataSource = (platformServices as? FluxaDetailServices)?.detailDataSource,
        searchDataSource = (platformServices as? FluxaSearchServices)?.searchDataSource,
        language = language,
        onCatalogAction = onCatalogAction,
        modifier = modifier
    )
}

@Composable
fun FluxaAppHost(
    catalogHomeDataSource: CatalogHomeDataSource,
    detailDataSource: DetailDataSource? = null,
    searchDataSource: SearchDataSource? = null,
    language: String? = null,
    onCatalogAction: (CatalogAction) -> Unit = {},
    modifier: Modifier = Modifier
) {
    val scope = rememberCoroutineScope()
    val catalogHomeStore = remember(catalogHomeDataSource) {
        CatalogHomeStore(catalogHomeDataSource, scope)
    }
    val catalogHome by catalogHomeStore.state.collectAsState()
    val searchStore = searchDataSource?.let { source ->
        remember(source) { SearchStore(source, scope) }
    }
    val searchState = searchStore?.state?.collectAsState()?.value
    val appState = rememberFluxaAppState()
    val selectedDetail = appState.uiState.selectedDetail
    val detailStore = selectedDetail?.let { item ->
        detailDataSource?.let { source ->
            remember(item.id, item.type, source) {
                DetailStore(item.id, item.type, source, scope)
            }
        }
    }
    val detailState = detailStore?.state?.collectAsState()?.value

    LaunchedEffect(catalogHome) {
        appState.updateCatalogHome(catalogHome)
    }
    LaunchedEffect(language) {
        appState.updateLanguage(language)
    }
    LaunchedEffect(catalogHomeStore) {
        catalogHomeStore.dispatch(CatalogAction.Refresh)
    }
    LaunchedEffect(detailStore) {
        detailStore?.load()
    }

    FluxaApp(
        state = appState.uiState,
        onDestinationSelected = appState::selectDestination,
        onCatalogAction = { action ->
            if (action is CatalogAction.ItemSelected) {
                appState.selectDetail(action.item)
            }
            scope.launch {
                catalogHomeStore.dispatch(action)
            }
            onCatalogAction(action)
        },
        detailState = detailState,
        onDetailAction = { action ->
            if (action is DetailAction.RelatedItemSelected) {
                appState.selectDetail(action.item)
                onCatalogAction(CatalogAction.ItemSelected(action.item))
            }
            scope.launch {
                detailStore?.dispatch(action)
            }
        },
        searchState = searchState,
        onSearchAction = { action ->
            if (action is SearchAction.ItemSelected) {
                appState.selectDetail(action.item)
                onCatalogAction(CatalogAction.ItemSelected(action.item))
            }
            scope.launch {
                searchStore?.dispatch(action)
            }
        },
        modifier = modifier
    )
}
