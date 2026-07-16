package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.isPlayable
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistStore
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.library.LibraryCollectionUiModel
import com.fluxa.app.shared.feature.library.LibraryDataSource
import com.fluxa.app.shared.feature.library.LibraryDownloadEpisodeUiModel
import com.fluxa.app.shared.feature.library.LibraryDownloadGroupUiModel
import com.fluxa.app.shared.feature.library.LibraryFolderSectionUiModel
import com.fluxa.app.shared.feature.library.LibraryFolderUiModel
import com.fluxa.app.shared.feature.library.LibraryUiState
import com.fluxa.app.shared.feature.library.toCatalogCardUiModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine

class AndroidLibraryDataSource(
    private val homeViewModel: HomeViewModel,
    private val profileManager: ProfileManager,
    private val activeProfile: () -> UserProfile?,
    private val onProfileChanged: (UserProfile) -> Unit,
    private val offlineDownloadManager: OfflineDownloadManager,
    private val watchlistStore: WatchlistStore,
    private val language: () -> String
) : LibraryDataSource {

    override fun observeLibrary(): Flow<LibraryUiState> = combine(
        watchlistStore.observeWatchlist(),
        watchlistStore.observeLiked(),
        homeViewModel.libraryUiState,
        homeViewModel.isLoading,
        offlineDownloadManager.items
    ) { watchlist, likedItems, libraryUiState, isLoading, downloads ->
        val profile = activeProfile()
        val lang = language()

        val planned = (watchlist + libraryUiState.traktPlanned + libraryUiState.malPlanned + libraryUiState.simklPlanned)
            .distinctBy { it.id }
        val completed = (libraryUiState.traktWatched + libraryUiState.malCompleted + libraryUiState.simklCompleted)
            .distinctBy { it.id }
        val favorites = likedItems.distinctBy { it.id }

        val collections = buildList {
            if (libraryUiState.traktCollection.isNotEmpty()) {
                add(
                    LibraryCollectionUiModel(
                        id = null,
                        title = "Trakt Collection",
                        subtitle = "${libraryUiState.traktCollection.size}",
                        items = libraryUiState.traktCollection.toCatalogItems(profile),
                        locked = true
                    )
                )
            }
            profile?.safeLibraryCollections.orEmpty().forEach { collection ->
                val folders = collection.folders.orEmpty()
                add(
                    LibraryCollectionUiModel(
                        id = collection.id,
                        title = collection.title,
                        subtitle = "${if (folders.isNotEmpty()) folders.size else collection.itemIds.orEmpty().size}",
                        items = folders.map { folder -> folder.toLibraryCatalogItem() },
                        folders = folders.map { folder -> folder.toLibraryFolderUiModel() },
                        locked = false
                    )
                )
            }
        }

        val profileDownloads = downloads.filter { it.profileId == null || it.profileId == profile?.id }
        val downloadGroups = profileDownloads.toOfflineDownloadGroups(String::toFileImageModel).map { group ->
            LibraryDownloadGroupUiModel(
                key = group.key,
                title = group.title,
                posterUrl = group.poster,
                totalSizeLabel = group.totalBytes.formatDownloadBytes(),
                episodes = group.episodes.map { item ->
                    LibraryDownloadEpisodeUiModel(
                        id = item.id,
                        title = item.title,
                        statusLabel = when (item.status) {
                            "downloaded" -> AppStrings.t(lang, "downloads.status_downloaded")
                            "failed" -> AppStrings.t(lang, "downloads.status_failed")
                            "paused" -> AppStrings.t(lang, "downloads.status_paused")
                            "downloading" -> AppStrings.format(lang, "downloads.status_downloading", item.progress)
                            else -> AppStrings.t(lang, "downloads.status_queued")
                        },
                        sizeLabel = item.effectiveSizeLabel().orEmpty(),
                        progressPercent = item.progress.coerceIn(0, 100),
                        isDownloaded = item.status == "downloaded",
                        isPlayable = item.isPlayable
                    )
                }
            )
        }

        LibraryUiState(
            isLoading = isLoading,
            planned = planned.toCatalogItems(profile),
            completed = completed.toCatalogItems(profile),
            favorites = favorites.toCatalogItems(profile),
            collections = collections,
            downloadGroups = downloadGroups
        )
    }

    override suspend fun refresh() {
        val profile = activeProfile()
        homeViewModel.loadLibraryData(profile)
        homeViewModel.loadLibraryItems(profile)
    }

    override suspend fun createCollection(title: String) {
        val profile = activeProfile() ?: return
        val updated = profile.copy(
            libraryCollections = profile.safeLibraryCollections + LibraryUserCollection(
                id = "local_${System.currentTimeMillis()}",
                title = title
            )
        )
        profileManager.saveProfile(updated)
        onProfileChanged(updated)
    }

    override suspend fun renameCollection(id: String, title: String) {
        val profile = activeProfile() ?: return
        val updated = profile.copy(
            libraryCollections = profile.safeLibraryCollections.map {
                if (it.id == id) it.copy(title = title) else it
            }
        )
        profileManager.saveProfile(updated)
        onProfileChanged(updated)
    }

    override suspend fun deleteCollection(id: String) {
        val profile = activeProfile() ?: return
        val updated = profile.copy(
            libraryCollections = profile.safeLibraryCollections.filterNot { it.id == id }
        )
        profileManager.saveProfile(updated)
        onProfileChanged(updated)
    }

    override suspend fun cancelDownload(id: String) {
        offlineDownloadManager.cancel(id)
    }

    override suspend fun loadFolder(folder: LibraryFolderUiModel): List<LibraryFolderSectionUiModel> {
        val profile = activeProfile()
        val domainFolder = profile?.safeLibraryCollections.orEmpty()
            .asSequence()
            .flatMap { it.folders.orEmpty().asSequence() }
            .firstOrNull { it.id == folder.id }
            ?: return emptyList()
        return homeViewModel.loadFolderSections(domainFolder).map { (title, metas) ->
            LibraryFolderSectionUiModel(title = title, items = metas.toCatalogItems(profile))
        }
    }
}

private fun LibraryUserCollectionFolder.toLibraryCatalogItem(): CatalogItemUiModel {
    val uiModel = toLibraryFolderUiModel()
    return CatalogItemUiModel(
        id = id,
        type = "catalog_folder",
        card = uiModel.toCatalogCardUiModel(),
        backdropUrl = heroBackdropUrl ?: effectiveImageUrl()
    )
}

private fun LibraryUserCollectionFolder.toLibraryFolderUiModel(): LibraryFolderUiModel = LibraryFolderUiModel(
    id = id,
    title = title,
    imageUrl = imageUrl,
    shape = shape,
    catalogTitle = catalogTitle,
    hideTitle = hideTitle == true,
    focusGifEnabled = focusGifEnabled != false,
    coverEmoji = coverEmoji,
    coverImageUrl = coverImageUrl,
    focusGifUrl = focusGifUrl,
    heroBackdropUrl = heroBackdropUrl
)
