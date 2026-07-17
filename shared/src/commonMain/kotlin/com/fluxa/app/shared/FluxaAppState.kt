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
import com.fluxa.app.shared.feature.detail.DetailUiModel
import com.fluxa.app.shared.feature.profile.ProfileEditTarget

@Stable
class FluxaAppState internal constructor(initialState: FluxaAppUiState) {
    var uiState by mutableStateOf(initialState)
        private set

    fun selectDestination(destination: FluxaDestination) {
        uiState = uiState.copy(
            destination = destination,
            selectedDetail = null,
            selectedCategoryId = null,
            selectedCategoryTitle = null,
            editingProfile = null,
            settingsBackStack = emptyList(),
            initialLibrarySection = null
        )
    }

    fun selectDetail(item: CatalogItemUiModel) {
        uiState = uiState.copy(
            selectedDetail = DetailRequestUiModel(
                id = item.id,
                type = item.type,
                source = item.source,
                initialProgress = item.resume?.positionMs,
                lastVideoId = item.resume?.videoId,
                lastStreamUrl = item.resume?.streamUrl,
                lastStreamTitle = item.resume?.streamTitle,
                initialContent = DetailUiModel(
                    id = item.id,
                    type = item.type,
                    title = item.card.title,
                    description = item.description.orEmpty(),
                    posterUrl = item.card.artworkUrl,
                    backgroundUrl = item.backdropUrl ?: item.card.artworkUrl,
                    logoUrl = item.card.logoUrl,
                    releaseLabel = item.card.subtitle,
                    ratingLabel = "",
                    runtimeLabel = item.runtimeLabel,
                    ageRating = item.ageRating,
                    isInWatchlist = false,
                    relatedItems = emptyList(),
                    availableSeasons = if (item.type == "series" && (item.seasonsCount ?: 0) > 0) {
                        (1..item.seasonsCount!!).toList()
                    } else {
                        emptyList()
                    },
                    resumeVideoId = item.resume?.videoId,
                    resumeProgress = item.resume?.positionMs ?: 0L
                )
            )
        )
    }

    fun selectDetail(request: DetailRequestUiModel) {
        uiState = uiState.copy(selectedDetail = request)
    }

    fun clearDetail() {
        uiState = uiState.copy(selectedDetail = null, showSourceSelection = false)
    }

    fun openSourceSelection() {
        uiState = uiState.copy(showSourceSelection = true)
    }

    fun closeSourceSelection() {
        uiState = uiState.copy(showSourceSelection = false)
    }

    fun selectCategory(id: String, title: String) {
        uiState = uiState.copy(selectedCategoryId = id, selectedCategoryTitle = title, selectedDetail = null)
    }

    fun clearCategory() {
        uiState = uiState.copy(selectedCategoryId = null, selectedCategoryTitle = null)
    }

    fun openNotifications() {
        uiState = uiState.copy(showNotifications = true)
    }

    fun closeNotifications() {
        uiState = uiState.copy(showNotifications = false)
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

    fun pushSettingsCategory(category: com.fluxa.app.shared.feature.settings.SettingsCategory) {
        uiState = uiState.copy(settingsBackStack = uiState.settingsBackStack + category)
    }

    fun selectSettingsCategory(category: com.fluxa.app.shared.feature.settings.SettingsCategory) {
        uiState = uiState.copy(settingsBackStack = listOf(category))
    }

    fun popSettingsCategory() {
        uiState = uiState.copy(settingsBackStack = uiState.settingsBackStack.dropLast(1))
    }

    fun openLibraryDownloads() {
        uiState = uiState.copy(
            destination = FluxaDestination.Library,
            initialLibrarySection = com.fluxa.app.shared.feature.library.LibrarySection.Downloads,
            selectedDetail = null,
            selectedCategoryId = null,
            selectedCategoryTitle = null,
            editingProfile = null,
            settingsBackStack = emptyList()
        )
    }
}

@Composable
fun rememberFluxaAppState(initialState: FluxaAppUiState = FluxaAppUiState()): FluxaAppState {
    return remember { FluxaAppState(initialState) }
}
