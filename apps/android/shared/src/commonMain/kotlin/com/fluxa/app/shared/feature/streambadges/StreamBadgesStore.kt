package com.fluxa.app.shared.feature.streambadges

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class StreamBadgesStore(
    private val dataSource: StreamBadgesDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<StreamBadgesUiState> = dataSource.observeStreamBadges()
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), StreamBadgesUiState())

    suspend fun dispatch(action: StreamBadgesAction) {
        when (action) {
            is StreamBadgesAction.ImportUrlChanged -> dataSource.setImportUrlDraft(action.url)
            StreamBadgesAction.ImportRequested -> dataSource.importFromDraft()
            is StreamBadgesAction.SourceActivated -> dataSource.setActiveSource(action.sourceUrl)
            is StreamBadgesAction.SourceRemoved -> dataSource.removeSource(action.sourceUrl)
            StreamBadgesAction.ErrorDismissed -> dataSource.dismissError()
            is StreamBadgesAction.PlacementChanged -> dataSource.setBadgePlacement(action.placement)
        }
    }
}
