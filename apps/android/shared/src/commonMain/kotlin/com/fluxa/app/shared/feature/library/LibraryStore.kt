package com.fluxa.app.shared.feature.library

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn

class LibraryStore(
    private val dataSource: LibraryDataSource,
    private val scope: CoroutineScope
) {
    private val folderDetail = MutableStateFlow(LibraryFolderDetailUiState())
    private val folderEditor = MutableStateFlow(LibraryFolderEditorUiState())
    private var folderLoadJob: Job? = null

    val state: StateFlow<LibraryUiState> = combine(
        dataSource.observeLibrary(),
        folderDetail,
        folderEditor
    ) { base, folder, editor -> base.copy(folderDetail = folder, folderEditor = editor) }
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
                folderLoadJob?.cancel()
                folderDetail.value = LibraryFolderDetailUiState(folder = action.folder, isLoading = true)
                folderLoadJob = scope.launch {
                    val sections = dataSource.loadFolder(action.folder)
                    if (isActive && folderDetail.value.folder?.id == action.folder.id) {
                        folderDetail.value = LibraryFolderDetailUiState(
                            folder = action.folder,
                            sections = sections,
                            isLoading = false
                        )
                    }
                }
            }
            LibraryAction.FolderClosed -> {
                folderLoadJob?.cancel()
                folderLoadJob = null
                folderDetail.value = LibraryFolderDetailUiState()
            }
            is LibraryAction.FolderEditorOpened -> {
                val draft = action.folderId?.let { dataSource.folderForEditing(action.collectionId, it) }
                    ?: LibraryFolderEditorUiModel()
                folderEditor.value = LibraryFolderEditorUiState(isOpen = true, collectionId = action.collectionId, draft = draft)
            }
            LibraryAction.FolderEditorClosed -> {
                folderEditor.value = LibraryFolderEditorUiState()
            }
            is LibraryAction.FolderEditorDraftChanged -> {
                folderEditor.value = folderEditor.value.copy(draft = action.draft)
            }
            LibraryAction.FolderEditorSaveRequested -> {
                val current = folderEditor.value
                val collectionId = current.collectionId
                if (collectionId != null) {
                    folderEditor.value = current.copy(isSaving = true, error = null)
                    val saved = dataSource.saveFolder(collectionId, current.draft)
                    folderEditor.value = if (saved) {
                        LibraryFolderEditorUiState()
                    } else {
                        current.copy(isSaving = false, error = "save_failed")
                    }
                    if (saved) dataSource.refresh()
                }
            }
            is LibraryAction.FolderDeleteRequested -> {
                dataSource.deleteFolder(action.collectionId, action.folderId)
                folderEditor.value = LibraryFolderEditorUiState()
                dataSource.refresh()
            }
            is LibraryAction.SourceChanged -> dataSource.setLibrarySource(action.source)
            is LibraryAction.LocalMediaFolderPickerRequested -> Unit
            is LibraryAction.LocalMediaSourceAdded -> dataSource.addLocalMediaSource(action.source)
            is LibraryAction.LocalMediaSourceRemoved -> dataSource.removeLocalMediaSource(action.sourceId)
            is LibraryAction.LocalMediaScanRequested -> dataSource.scanLocalMedia(action.forceMetadata)
        }
    }
}
