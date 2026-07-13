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
import kotlinx.coroutines.launch

@Composable
fun FluxaAppHost(
    catalogHomeDataSource: CatalogHomeDataSource,
    language: String? = null,
    onCatalogAction: (CatalogAction) -> Unit = {},
    modifier: Modifier = Modifier
) {
    val scope = rememberCoroutineScope()
    val catalogHomeStore = remember(catalogHomeDataSource) {
        CatalogHomeStore(catalogHomeDataSource, scope)
    }
    val catalogHome by catalogHomeStore.state.collectAsState()
    val appState = rememberFluxaAppState()

    LaunchedEffect(catalogHome) {
        appState.updateCatalogHome(catalogHome)
    }
    LaunchedEffect(language) {
        appState.updateLanguage(language)
    }
    LaunchedEffect(catalogHomeStore) {
        catalogHomeStore.dispatch(CatalogAction.Refresh)
    }

    FluxaApp(
        state = appState.uiState,
        onDestinationSelected = appState::selectDestination,
        onCatalogAction = { action ->
            scope.launch {
                catalogHomeStore.dispatch(action)
            }
            onCatalogAction(action)
        },
        modifier = modifier
    )
}
