package com.fluxa.app.shared.feature.library

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import kotlinx.coroutines.flow.Flow

enum class LibrarySection { Planned, Completed, Favorites, Downloads, Collections }

enum class LibraryTypeFilter { All, Movie, Series, Anime }

data class LibraryFolderUiModel(
    val id: String,
    val title: String,
    val imageUrl: String? = null,
    val shape: String? = "poster",
    val catalogTitle: String? = null,
    val hideTitle: Boolean = false,
    val focusGifEnabled: Boolean = true,
    val coverEmoji: String? = null,
    val coverImageUrl: String? = null,
    val focusGifUrl: String? = null,
    val heroBackdropUrl: String? = null
)

data class LibraryCollectionUiModel(
    val id: String? = null,
    val title: String,
    val subtitle: String,
    val items: List<CatalogItemUiModel>,
    val folders: List<LibraryFolderUiModel> = emptyList(),
    val locked: Boolean = false
)

data class LibraryFolderSectionUiModel(
    val title: String,
    val items: List<CatalogItemUiModel>
)

data class LibraryFolderDetailUiState(
    val folder: LibraryFolderUiModel? = null,
    val sections: List<LibraryFolderSectionUiModel> = emptyList(),
    val isLoading: Boolean = false
)

data class LibraryDownloadEpisodeUiModel(
    val id: String,
    val title: String,
    val statusLabel: String,
    val sizeLabel: String,
    val progressPercent: Int,
    val isDownloaded: Boolean,
    val isPlayable: Boolean
)

data class LibraryDownloadGroupUiModel(
    val key: String,
    val title: String,
    val posterUrl: String?,
    val episodes: List<LibraryDownloadEpisodeUiModel>,
    val totalSizeLabel: String
)

data class LibraryUiState(
    val isLoading: Boolean = false,
    val planned: List<CatalogItemUiModel> = emptyList(),
    val completed: List<CatalogItemUiModel> = emptyList(),
    val favorites: List<CatalogItemUiModel> = emptyList(),
    val collections: List<LibraryCollectionUiModel> = emptyList(),
    val downloadGroups: List<LibraryDownloadGroupUiModel> = emptyList(),
    val folderDetail: LibraryFolderDetailUiState = LibraryFolderDetailUiState()
)

sealed interface LibraryAction {
    data object Refresh : LibraryAction
    data class ItemSelected(val item: CatalogItemUiModel) : LibraryAction
    data class CollectionCreated(val title: String) : LibraryAction
    data class CollectionRenamed(val id: String, val title: String) : LibraryAction
    data class CollectionDeleted(val id: String) : LibraryAction
    data class DownloadOpened(val id: String) : LibraryAction
    data class DownloadCancelled(val id: String) : LibraryAction
    data class FolderSelected(val folder: LibraryFolderUiModel) : LibraryAction
    data object FolderClosed : LibraryAction
}

interface LibraryDataSource {
    fun observeLibrary(): Flow<LibraryUiState>
    suspend fun refresh()
    suspend fun createCollection(title: String)
    suspend fun renameCollection(id: String, title: String)
    suspend fun deleteCollection(id: String)
    suspend fun cancelDownload(id: String)
    suspend fun loadFolder(folder: LibraryFolderUiModel): List<LibraryFolderSectionUiModel>
}
