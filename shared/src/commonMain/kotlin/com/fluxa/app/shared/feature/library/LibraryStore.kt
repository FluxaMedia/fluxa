package com.fluxa.app.shared.feature.library

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn

class LibraryStore(
    private val dataSource: LibraryDataSource,
    scope: CoroutineScope
) {
    private val folderDetail = MutableStateFlow(LibraryFolderDetailUiState())

    val state: StateFlow<LibraryUiState> = combine(
        dataSource.observeLibrary(),
        folderDetail
    ) { base, folder -> base.copy(folderDetail = folder) }
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
            is LibraryAction.FolderSelected -> {
                folderDetail.value = LibraryFolderDetailUiState(folder = action.folder, isLoading = true)
                val sections = dataSource.loadFolder(action.folder)
                folderDetail.value = LibraryFolderDetailUiState(folder = action.folder, sections = sections, isLoading = false)
            }
            LibraryAction.FolderClosed -> {
                folderDetail.value = LibraryFolderDetailUiState()
            }
        }
    }
}
