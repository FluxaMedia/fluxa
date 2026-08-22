package com.fluxa.app.shared.feature.streambadges

import kotlinx.coroutines.flow.Flow

data class StreamBadgeImportUiModel(
    val sourceUrl: String,
    val filterCount: Int,
    val isActive: Boolean
)

data class StreamBadgesUiState(
    val imports: List<StreamBadgeImportUiModel> = emptyList(),
    val importUrlDraft: String = "",
    val isImporting: Boolean = false,
    val error: String? = null,
    val badgePlacement: String = "bottom"
)

sealed interface StreamBadgesAction {
    data class ImportUrlChanged(val url: String) : StreamBadgesAction
    data object ImportRequested : StreamBadgesAction
    data class SourceActivated(val sourceUrl: String) : StreamBadgesAction
    data class SourceRemoved(val sourceUrl: String) : StreamBadgesAction
    data object ErrorDismissed : StreamBadgesAction
    data class PlacementChanged(val placement: String) : StreamBadgesAction
}

interface StreamBadgesDataSource {
    fun observeStreamBadges(): Flow<StreamBadgesUiState>
    suspend fun setImportUrlDraft(url: String)
    suspend fun importFromDraft()
    suspend fun setActiveSource(sourceUrl: String)
    suspend fun removeSource(sourceUrl: String)
    suspend fun dismissError()
    suspend fun setBadgePlacement(placement: String) = Unit
}
