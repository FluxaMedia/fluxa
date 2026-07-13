package com.fluxa.app.shared.feature.search

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class SearchStore(
    private val dataSource: SearchDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<SearchUiState> = dataSource.observeSearch()
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), SearchUiState())

    suspend fun dispatch(action: SearchAction) {
        when (action) {
            is SearchAction.QueryChanged -> dataSource.search(action.value)
            SearchAction.ClearHistory -> dataSource.clearHistory()
            is SearchAction.ItemSelected -> Unit
        }
    }
}
