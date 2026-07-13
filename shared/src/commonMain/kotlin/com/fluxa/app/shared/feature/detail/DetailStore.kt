package com.fluxa.app.shared.feature.detail

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class DetailStore(
    private val id: String,
    private val type: String,
    private val dataSource: DetailDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<DetailUiState> = dataSource.observeDetail(id, type)
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), DetailUiState(isLoading = true))

    suspend fun load() {
        dataSource.loadDetail(id, type)
    }

    suspend fun dispatch(action: DetailAction) {
        when (action) {
            DetailAction.ToggleWatchlist -> dataSource.toggleWatchlist(id, type)
            DetailAction.Play,
            is DetailAction.RelatedItemSelected -> Unit
        }
    }
}
