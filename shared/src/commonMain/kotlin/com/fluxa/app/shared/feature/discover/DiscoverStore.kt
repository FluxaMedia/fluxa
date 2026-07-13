package com.fluxa.app.shared.feature.discover

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class DiscoverStore(
    private val dataSource: DiscoverDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<DiscoverUiState> = dataSource.observeDiscover()
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), DiscoverUiState())

    suspend fun dispatch(action: DiscoverAction) {
        when (action) {
            is DiscoverAction.FiltersChanged -> dataSource.updateFilters(action.filters)
            is DiscoverAction.ItemSelected -> Unit
            DiscoverAction.LoadMore -> dataSource.loadMore()
        }
    }
}
