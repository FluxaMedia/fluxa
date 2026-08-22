package com.fluxa.app.shared.platform

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.OfflineDownloadItem
import com.fluxa.app.data.local.WatchlistStore
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.library.LibraryDataSource
import com.fluxa.app.shared.feature.library.LibraryDownloadEpisodeUiModel
import com.fluxa.app.shared.feature.library.LibraryDownloadGroupUiModel
import com.fluxa.app.shared.feature.library.LibraryFolderSectionUiModel
import com.fluxa.app.shared.feature.library.LibraryFolderUiModel
import com.fluxa.app.shared.feature.library.LibraryUiState
import com.fluxa.app.ui.catalog.OfflineDownloadGrouping
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

data class AppleOfflineDownloadItemSnapshot(
    val id: String,
    val metaId: String,
    val metaType: String,
    val title: String,
    val episodeTitle: String? = null,
    val videoId: String? = null,
    val posterUrl: String? = null,
    val backgroundUrl: String? = null,
    val videoPath: String = "",
    val status: String = "queued",
    val progress: Int = 0,
    val downloadedBytes: Long = 0L,
    val totalBytes: Long = 0L,
    val errorMessage: String? = null
)

class AppleLibraryDataSource(
    private val watchlistStore: WatchlistStore
) : LibraryDataSource {
    private val state = MutableStateFlow(LibraryUiState())
    private var onRefreshRequested: () -> Unit = {}
    private var onCancelDownloadRequested: (String) -> Unit = {}
    private var offlineDownloads: List<OfflineDownloadItem> = emptyList()
    private var language: String = "en"

    override fun observeLibrary(): Flow<LibraryUiState> = combine(
        state,
        watchlistStore.observeWatchlist(),
        watchlistStore.observeLiked()
    ) { snapshotState, watchlist, liked ->
        snapshotState.copy(
            planned = (watchlist.map { it.toLibraryItem() } + snapshotState.planned).distinctBy { it.id },
            favorites = liked.map { it.toLibraryItem() },
            downloadGroups = buildDownloadGroups()
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
        onCancelDownloadRequested(id)
    }

    override suspend fun loadFolder(folder: LibraryFolderUiModel): List<LibraryFolderSectionUiModel> = emptyList()

    override suspend fun setLibrarySource(source: String) {
        Unit
    }

    fun setOnRefreshRequested(handler: () -> Unit) {
        onRefreshRequested = handler
    }

    fun setOnCancelDownloadRequested(handler: (String) -> Unit) {
        onCancelDownloadRequested = handler
    }

    fun update(snapshot: AppleLibrarySnapshot) {
        state.value = state.value.copy(
            isLoading = snapshot.isLoading,
            planned = snapshot.planned.map { it.toLibraryItem() },
            completed = snapshot.completed.map { it.toLibraryItem() },
            favorites = snapshot.favorites.map { it.toLibraryItem() },
            downloadGroups = buildDownloadGroups()
        )
    }

    fun updateDownloads(items: List<AppleOfflineDownloadItemSnapshot>, language: String) {
        this.language = language
        offlineDownloads = items.map { it.toOfflineDownloadItem() }
        state.value = state.value.copy(downloadGroups = buildDownloadGroups())
    }

    private fun buildDownloadGroups(): List<LibraryDownloadGroupUiModel> =
        OfflineDownloadGrouping.group(offlineDownloads).map { group ->
            LibraryDownloadGroupUiModel(
                key = group.key,
                title = group.title,
                posterUrl = group.poster,
                totalSizeLabel = OfflineDownloadGrouping.formatBytes(group.totalBytes),
                episodes = group.episodes.map { item ->
                    LibraryDownloadEpisodeUiModel(
                        id = item.id,
                        title = item.title,
                        statusLabel = when (item.status) {
                            "downloaded" -> AppStrings.t(language, "downloads.status_downloaded")
                            "failed" -> AppStrings.t(language, "downloads.status_failed")
                            "paused" -> AppStrings.t(language, "downloads.status_paused")
                            "downloading" -> AppStrings.format(language, "downloads.status_downloading", item.progress)
                            else -> AppStrings.t(language, "downloads.status_queued")
                        },
                        sizeLabel = OfflineDownloadGrouping.sizeLabel(item).orEmpty(),
                        progressPercent = item.progress.coerceIn(0, 100),
                        isDownloaded = item.status == "downloaded",
                        isPlayable = item.status == "downloaded" && item.videoPath.isNotBlank()
                    )
                }
            )
        }
}

private fun AppleOfflineDownloadItemSnapshot.toOfflineDownloadItem() = OfflineDownloadItem(
    id = id,
    profileId = null,
    metaId = metaId,
    metaType = metaType,
    title = title,
    episodeTitle = episodeTitle,
    videoId = videoId,
    poster = posterUrl,
    background = backgroundUrl,
    videoPath = videoPath,
    status = status,
    progress = progress,
    downloadedBytes = downloadedBytes,
    totalBytes = totalBytes,
    error = errorMessage
)

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
