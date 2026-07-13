package com.fluxa.app.shared.feature.library

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import kotlinx.coroutines.flow.Flow

enum class LibrarySection { Planned, Completed, Favorites, Downloads, Collections }

enum class LibraryTypeFilter { All, Movie, Series, Anime }

data class LibraryCollectionUiModel(
    val id: String? = null,
    val title: String,
    val subtitle: String,
    val items: List<CatalogItemUiModel>,
    val locked: Boolean = false
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
    val downloadGroups: List<LibraryDownloadGroupUiModel> = emptyList()
)

sealed interface LibraryAction {
    data object Refresh : LibraryAction
    data class ItemSelected(val item: CatalogItemUiModel) : LibraryAction
    data class CollectionCreated(val title: String) : LibraryAction
    data class CollectionRenamed(val id: String, val title: String) : LibraryAction
    data class CollectionDeleted(val id: String) : LibraryAction
    data class DownloadOpened(val id: String) : LibraryAction
    data class DownloadCancelled(val id: String) : LibraryAction
}

interface LibraryDataSource {
    fun observeLibrary(): Flow<LibraryUiState>
    suspend fun refresh()
    suspend fun createCollection(title: String)
    suspend fun renameCollection(id: String, title: String)
    suspend fun deleteCollection(id: String)
    suspend fun cancelDownload(id: String)
}
