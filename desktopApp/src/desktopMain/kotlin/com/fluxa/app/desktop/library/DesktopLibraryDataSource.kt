package com.fluxa.app.desktop.library

import com.fluxa.app.data.local.WatchlistStore
import com.fluxa.app.desktop.home.toDesktopCatalogItemUiModel
import com.fluxa.app.shared.feature.library.LibraryDataSource
import com.fluxa.app.shared.feature.library.LibraryFolderSectionUiModel
import com.fluxa.app.shared.feature.library.LibraryFolderUiModel
import com.fluxa.app.shared.feature.library.LibraryUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine

class DesktopLibraryDataSource(
    private val watchlistStore: WatchlistStore
) : LibraryDataSource {
    override fun observeLibrary(): Flow<LibraryUiState> = combine(
        watchlistStore.observeWatchlist(),
        watchlistStore.observeLiked()
    ) { watchlist, favorites ->
        LibraryUiState(
            planned = watchlist.map { it.toDesktopCatalogItemUiModel(it.type) },
            favorites = favorites.map { it.toDesktopCatalogItemUiModel(it.type) }
        )
    }

    override suspend fun refresh() = Unit
    override suspend fun createCollection(title: String) = Unit
    override suspend fun renameCollection(id: String, title: String) = Unit
    override suspend fun deleteCollection(id: String) = Unit
    override suspend fun cancelDownload(id: String) = Unit
    override suspend fun loadFolder(folder: LibraryFolderUiModel): List<LibraryFolderSectionUiModel> = emptyList()
    override suspend fun setLibrarySource(source: String) = Unit
}
