package com.fluxa.app.shared.feature.catalog

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class CatalogHomeStore(
    private val dataSource: CatalogHomeDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<CatalogHomeUiState> = dataSource.observeHome()
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), CatalogHomeUiState(isLoading = true))

    suspend fun dispatch(action: CatalogAction) {
        when (action) {
            CatalogAction.Refresh -> dataSource.refresh()
            is CatalogAction.LoadMore -> dataSource.loadMore(action.rowId)
            is CatalogAction.ItemSelected,
            is CatalogAction.PlayRequested,
            is CatalogAction.ResumeRequested -> Unit
        }
    }
}
