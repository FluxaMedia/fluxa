package com.fluxa.app.shared.platform

import com.fluxa.app.data.local.WatchlistStore
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.library.LibraryDataSource
import com.fluxa.app.shared.feature.library.LibraryFolderSectionUiModel
import com.fluxa.app.shared.feature.library.LibraryFolderUiModel
import com.fluxa.app.shared.feature.library.LibraryUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine

data class AppleLibraryItemSnapshot(
    val id: String,
    val type: String,
    val title: String,
    val subtitle: String = "",
    val posterUrl: String? = null,
    val logoUrl: String? = null,
    val addonTransportUrl: String? = null,
    val catalogType: String? = null
)

data class AppleLibrarySnapshot(
    val planned: List<AppleLibraryItemSnapshot> = emptyList(),
    val completed: List<AppleLibraryItemSnapshot> = emptyList(),
    val favorites: List<AppleLibraryItemSnapshot> = emptyList(),
    val isLoading: Boolean = false
)

class AppleLibraryDataSource(
    private val watchlistStore: WatchlistStore
) : LibraryDataSource {
    private val state = MutableStateFlow(LibraryUiState())
    private var onRefreshRequested: () -> Unit = {}

    override fun observeLibrary(): Flow<LibraryUiState> = combine(
        state,
        watchlistStore.observeWatchlist(),
        watchlistStore.observeLiked()
    ) { snapshotState, watchlist, liked ->
        snapshotState.copy(
            planned = (watchlist.map { it.toLibraryItem() } + snapshotState.planned).distinctBy { it.id },
            favorites = liked.map { it.toLibraryItem() }
        )
    }

    override suspend fun refresh() {
        state.value = state.value.copy(isLoading = true)
        onRefreshRequested()
    }

    override suspend fun createCollection(title: String) {
        Unit
    }

    override suspend fun renameCollection(id: String, title: String) {
        Unit
    }

    override suspend fun deleteCollection(id: String) {
        Unit
    }

    override suspend fun cancelDownload(id: String) {
        Unit
    }

    override suspend fun loadFolder(folder: LibraryFolderUiModel): List<LibraryFolderSectionUiModel> = emptyList()

    override suspend fun setLibrarySource(source: String) {
        Unit
    }

    fun setOnRefreshRequested(handler: () -> Unit) {
        onRefreshRequested = handler
    }

    fun update(snapshot: AppleLibrarySnapshot) {
        state.value = LibraryUiState(
            isLoading = snapshot.isLoading,
            planned = snapshot.planned.map { it.toLibraryItem() },
            completed = snapshot.completed.map { it.toLibraryItem() },
            favorites = snapshot.favorites.map { it.toLibraryItem() }
        )
    }
}

private fun AppleLibraryItemSnapshot.toLibraryItem(): CatalogItemUiModel = appleCatalogItem(
    id = id,
    type = type,
    title = title,
    subtitle = subtitle,
    artworkUrl = posterUrl,
    logoUrl = logoUrl,
    addonTransportUrl = addonTransportUrl,
    catalogType = catalogType,
    cacheNamespace = "apple-library",
)

private fun Meta.toLibraryItem(): CatalogItemUiModel = AppleLibraryItemSnapshot(
    id = id,
    type = type,
    title = name,
    posterUrl = poster,
    logoUrl = logo
).toLibraryItem()
