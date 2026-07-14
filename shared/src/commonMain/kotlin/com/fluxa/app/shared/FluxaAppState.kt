package com.fluxa.app.shared

import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.Composable
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.profile.ProfileEditTarget

@Stable
class FluxaAppState internal constructor(initialState: FluxaAppUiState) {
    var uiState by mutableStateOf(initialState)
        private set

    fun selectDestination(destination: FluxaDestination) {
        uiState = uiState.copy(destination = destination, selectedDetail = null, editingProfile = null)
    }

    fun selectDetail(item: CatalogItemUiModel) {
        uiState = uiState.copy(selectedDetail = DetailRequestUiModel(item.id, item.type, item.source))
    }

    fun selectDetail(request: DetailRequestUiModel) {
        uiState = uiState.copy(selectedDetail = request)
    }

    fun beginProfileEdit(target: ProfileEditTarget?) {
        uiState = uiState.copy(editingProfile = target)
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
