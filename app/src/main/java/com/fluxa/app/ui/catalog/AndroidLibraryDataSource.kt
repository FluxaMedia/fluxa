package com.fluxa.app.ui.catalog

import android.content.SharedPreferences
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
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.launch

private data class AndroidLibrarySources(
    val watchlist: List<com.fluxa.app.data.remote.Meta>,
    val likedItems: List<com.fluxa.app.data.remote.Meta>,
    val remoteLibrary: com.fluxa.app.ui.catalog.LibraryUiState
)

class AndroidLibraryDataSource(
    private val homeViewModel: HomeViewModel,
    private val profileManager: ProfileManager,
    private val activeProfile: () -> UserProfile?,
    private val onProfileChanged: (UserProfile) -> Unit,
    private val offlineDownloadManager: OfflineDownloadManager,
    private val watchlistStore: WatchlistStore,
    private val language: () -> String
) : LibraryDataSource {

    private val profileState = MutableStateFlow(activeProfile())

    private fun profileFlow(): Flow<UserProfile?> = callbackFlow {
        val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, _ -> launch { profileState.value = activeProfile() } }
        profileManager.registerOnChangeListener(listener)
        val forwardingJob = launch { profileState.collect { trySend(it) } }
        awaitClose {
            forwardingJob.cancel()
            profileManager.unregisterOnChangeListener(listener)
        }
    }

    private val librarySources = combine(
        watchlistStore.observeWatchlist(),
        watchlistStore.observeLiked(),
        homeViewModel.libraryUiState
    ) { watchlist, likedItems, remoteLibrary ->
        AndroidLibrarySources(watchlist, likedItems, remoteLibrary)
    }

    override fun observeLibrary(): Flow<LibraryUiState> = combine(
        librarySources,
        homeViewModel.isLoading,
        offlineDownloadManager.items,
        profileFlow()
    ) { sources, isLoading, downloads, profile ->
        val watchlist = sources.watchlist
        val likedItems = sources.likedItems
        val libraryUiState = sources.remoteLibrary
        val lang = language()

        val (planned, completed) = when (profile?.integrationLibrarySource) {
            "trakt" -> libraryUiState.traktPlanned.distinctBy { it.id } to libraryUiState.traktWatched.distinctBy { it.id }
            "simkl" -> libraryUiState.simklPlanned.distinctBy { it.id } to libraryUiState.simklCompleted.distinctBy { it.id }
            "anilist" -> (libraryUiState.anilistPlanned + libraryUiState.anilistWatching).distinctBy { it.id } to libraryUiState.anilistCompleted.distinctBy { it.id }
            "stremio" -> libraryUiState.stremioPlanned.distinctBy { it.id } to emptyList()
            "nuvio" -> libraryUiState.nuvioPlanned.distinctBy { it.id } to emptyList()
            else -> watchlist.distinctBy { it.id } to emptyList()
        }
        val favorites = when (profile?.integrationLibrarySource) {
            "trakt" -> libraryUiState.traktFavorites.distinctBy { it.id }
            else -> likedItems.distinctBy { it.id }
        }
        val (plannedLabelKey, completedLabelKey, completedSectionEnabled) = when (profile?.integrationLibrarySource) {
            "trakt" -> Triple("auto.watchlist", "auto.history", true)
            "simkl" -> Triple("auto.plan_to_watch", "auto.completed", true)
            "anilist" -> Triple("auto.planning", "auto.completed", true)
            "stremio", "nuvio" -> Triple("auto.watchlist", "auto.completed", false)
            else -> Triple("auto.planned", "auto.completed", false)
        }

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

        val profileDownloads = downloads.filter { it.profileId == profile?.id }
        val downloadGroups = OfflineDownloadGrouping.group(profileDownloads, String::toFileImageModel).map { group ->
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
                            "downloaded" -> AppStrings.t(lang, "downloads.status_downloaded")
                            "failed" -> AppStrings.t(lang, "downloads.status_failed")
                            "paused" -> AppStrings.t(lang, "downloads.status_paused")
                            "downloading" -> AppStrings.format(lang, "downloads.status_downloading", item.progress)
                            else -> AppStrings.t(lang, "downloads.status_queued")
                        },
                        sizeLabel = OfflineDownloadGrouping.sizeLabel(item).orEmpty(),
                        progressPercent = item.progress.coerceIn(0, 100),
                        isDownloaded = item.status == "downloaded",
                        isPlayable = item.isPlayable
                    )
                }
            )
        }

        val availableSources = buildList {
            add("local")
            if (!profile?.traktAccessToken.isNullOrBlank()) add("trakt")
            if (!profile?.simklAccessToken.isNullOrBlank()) add("simkl")
            if (!profile?.anilistAccessToken.isNullOrBlank()) add("anilist")
            if (!profile?.nuvioAccessToken.isNullOrBlank()) add("nuvio")
        }

        LibraryUiState(
            isLoading = (isLoading || libraryUiState.isLoading) &&
                planned.isEmpty() &&
                completed.isEmpty() &&
                favorites.isEmpty() &&
                collections.isEmpty() &&
                downloadGroups.isEmpty(),
            planned = planned.toCatalogItems(profile),
            plannedLabelKey = plannedLabelKey,
            completed = completed.toCatalogItems(profile),
            completedLabelKey = completedLabelKey,
            completedSectionEnabled = completedSectionEnabled,
            favorites = favorites.toCatalogItems(profile),
            collections = collections,
            downloadGroups = downloadGroups,
            librarySource = profile?.integrationLibrarySource ?: "local",
            availableLibrarySources = availableSources
        )
    }

    override suspend fun refresh() {
        val profile = activeProfile()
        homeViewModel.loadLibraryData(profile)
        homeViewModel.loadLibraryItems(profile)
    }

    override suspend fun createCollection(title: String) {
        val profile = activeProfile() ?: return
        val updated = profileManager.updateProfile(profile.id) {
            it.copy(libraryCollections = it.safeLibraryCollections + LibraryUserCollection(
                id = "local_${System.currentTimeMillis()}",
                title = title
            ))
        } ?: return
        onProfileChanged(updated)
    }

    override suspend fun renameCollection(id: String, title: String) {
        val profile = activeProfile() ?: return
        val updated = profileManager.updateProfile(profile.id) {
            it.copy(libraryCollections = it.safeLibraryCollections.map { collection ->
                if (collection.id == id) collection.copy(title = title) else collection
            })
        } ?: return
        onProfileChanged(updated)
    }

    override suspend fun deleteCollection(id: String) {
        val profile = activeProfile() ?: return
        val updated = profileManager.updateProfile(profile.id) {
            it.copy(libraryCollections = it.safeLibraryCollections.filterNot { collection -> collection.id == id })
        } ?: return
        onProfileChanged(updated)
    }

    override suspend fun cancelDownload(id: String) {
        offlineDownloadManager.cancel(id)
    }

    override suspend fun setLibrarySource(source: String) {
        val profile = activeProfile() ?: return
        val updated = profileManager.updateProfile(profile.id) {
            it.copy(integrationLibrarySource = source)
        } ?: return
        onProfileChanged(updated)
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
