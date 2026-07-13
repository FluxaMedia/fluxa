package com.fluxa.app.shared.feature.library

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class LibraryStore(
    private val dataSource: LibraryDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<LibraryUiState> = dataSource.observeLibrary()
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), LibraryUiState(isLoading = true))

    suspend fun dispatch(action: LibraryAction) {
        when (action) {
            LibraryAction.Refresh -> dataSource.refresh()
            is LibraryAction.ItemSelected -> Unit
            is LibraryAction.CollectionCreated -> dataSource.createCollection(action.title)
            is LibraryAction.CollectionRenamed -> dataSource.renameCollection(action.id, action.title)
            is LibraryAction.CollectionDeleted -> dataSource.deleteCollection(action.id)
            is LibraryAction.DownloadOpened -> Unit
            is LibraryAction.DownloadCancelled -> dataSource.cancelDownload(action.id)
        }
    }
}
