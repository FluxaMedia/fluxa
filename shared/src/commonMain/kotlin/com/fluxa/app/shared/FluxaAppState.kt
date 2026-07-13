package com.fluxa.app.shared

import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.Composable
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState

@Stable
class FluxaAppState internal constructor(initialState: FluxaAppUiState) {
    var uiState by mutableStateOf(initialState)
        private set

    fun selectDestination(destination: FluxaDestination) {
        uiState = uiState.copy(destination = destination)
    }

    fun updateCatalogHome(catalogHome: CatalogHomeUiState) {
        uiState = uiState.copy(catalogHome = catalogHome)
    }

    fun updateLanguage(language: String?) {
        uiState = uiState.copy(language = language)
    }
}

@Composable
fun rememberFluxaAppState(initialState: FluxaAppUiState = FluxaAppUiState()): FluxaAppState {
    return remember { FluxaAppState(initialState) }
}
