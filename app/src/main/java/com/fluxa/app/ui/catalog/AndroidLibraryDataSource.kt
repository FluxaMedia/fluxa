package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.isPlayable
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.providerAccountId
import com.fluxa.app.data.local.WatchlistStore
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.toLibraryCatalogItem
import com.fluxa.app.shared.feature.catalog.toLibraryFolderUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.library.LibraryCollectionUiModel
import com.fluxa.app.shared.feature.library.LibraryDataSource
import com.fluxa.app.shared.feature.library.LibraryDownloadEpisodeUiModel
import com.fluxa.app.shared.feature.library.LibraryDownloadGroupUiModel
import com.fluxa.app.shared.feature.library.LibraryFolderSectionUiModel
import com.fluxa.app.shared.feature.library.LibraryFolderUiModel
import com.fluxa.app.shared.feature.library.LibraryUiState
import com.fluxa.app.shared.feature.localmedia.LocalMediaLibraryService
import com.fluxa.app.shared.feature.localmedia.LocalMediaSourceInput
import com.fluxa.app.shared.feature.localmedia.toCatalogRows
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.withContext

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
    private val language: () -> String,
    private val localMediaLibrary: LocalMediaLibraryService,
    private val deviceType: DeviceType = DeviceType.Mobile,
) : LibraryDataSource {

    private fun profileFlow(): Flow<UserProfile?> = callbackFlow {
        val listener: () -> Unit = { trySend(activeProfile()) }
        trySend(activeProfile())
        profileManager.addChangeListener(listener)
        awaitClose { profileManager.removeChangeListener(listener) }
    }.distinctUntilChanged()

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
        profileFlow(),
        localMediaLibrary.state,
    ) { sources, isLoading, downloads, profile, localMedia ->
        val lang = language()
        withContext(Dispatchers.Default) {
            val watchlist = sources.watchlist
            val likedItems = sources.likedItems
            val libraryUiState = sources.remoteLibrary

            val source = profile?.integrationLibrarySource
                ?.trim()
                ?.lowercase()
                ?.takeIf { it == "local" || ThirdPartyProviderId.from(it) != null }
                ?: "local"
            val selectedProvider = ThirdPartyProviderId.from(source)

            val (planned, completed) = when (source) {
                "trakt" -> libraryUiState.traktPlanned.distinctBy { it.id } to libraryUiState.traktWatched.distinctBy { it.id }
                "simkl" -> libraryUiState.simklPlanned.distinctBy { it.id } to libraryUiState.simklCompleted.distinctBy { it.id }
                "anilist" -> (libraryUiState.anilistPlanned + libraryUiState.anilistWatching).distinctBy { it.id } to libraryUiState.anilistCompleted.distinctBy { it.id }
                "stremio" -> libraryUiState.stremioPlanned.distinctBy { it.id } to emptyList()
                "nuvio" -> libraryUiState.nuvioPlanned.distinctBy { it.id } to emptyList()
                else -> watchlist.distinctBy { it.id } to emptyList()
            }
            val favorites = when (source) {
                "trakt" -> libraryUiState.traktFavorites.distinctBy { it.id }
                "local" -> likedItems.distinctBy { it.id }
                else -> emptyList()
            }
            val (plannedLabelKey, completedLabelKey, completedSectionEnabled) = when (source) {
                "trakt" -> Triple("auto.watchlist", "auto.history", true)
                "simkl" -> Triple("auto.plan_to_watch", "auto.completed", true)
                "anilist" -> Triple("auto.planning", "auto.completed", true)
                "stremio", "nuvio" -> Triple("auto.watchlist", "auto.completed", false)
                else -> Triple("auto.planned", "auto.completed", false)
            }

            // My Collections belongs to the Fluxa/Nuvio profile, not to the selected
            // third-party library provider. Providers only control planned/completed/
            // favorites. Keeping this independent prevents providers without collection
            // support (for example Simkl) from hiding the user's collections.
            val collections = profile?.safeLibraryCollections.orEmpty().map { collection ->
                val folders = collection.folders.orEmpty()
                LibraryCollectionUiModel(
                    id = collection.id,
                    title = collection.title,
                    subtitle = "${if (folders.isNotEmpty()) folders.size else collection.itemIds.orEmpty().size}",
                    items = folders.map { folder -> folder.toLibraryCatalogItem(deviceType) },
                    folders = folders.map { folder -> folder.toLibraryFolderUiModel() },
                    locked = false
                )
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
                if (!profile?.authKey.isNullOrBlank()) add("stremio")
                if (!profile?.nuvioAccessToken.isNullOrBlank()) add("nuvio")
                if (!profile?.traktAccessToken.isNullOrBlank()) add("trakt")
                if (!profile?.simklAccessToken.isNullOrBlank()) add("simkl")
                if (!profile?.anilistAccessToken.isNullOrBlank()) add("anilist")
            }
            LibraryUiState(
                isLoading = (isLoading || libraryUiState.isLoading) &&
                    planned.isEmpty() &&
                    completed.isEmpty() &&
                    favorites.isEmpty() &&
                    collections.isEmpty() &&
                    downloadGroups.isEmpty(),
                planned = planned.toLibraryCatalogItems(profile, selectedProvider, deviceType),
                plannedLabelKey = plannedLabelKey,
                completed = completed.toLibraryCatalogItems(profile, selectedProvider, deviceType),
                completedLabelKey = completedLabelKey,
                completedSectionEnabled = completedSectionEnabled,
                favorites = favorites.toLibraryCatalogItems(profile, selectedProvider, deviceType),
                collections = collections,
                downloadGroups = downloadGroups,
                localMediaSupported = true,
                localMediaRows = localMedia.toCatalogRows(deviceType, lang),
                localMediaSources = localMedia.sources,
                localMediaIndexedFileCount = localMedia.indexedFileCount,
                localMediaUnmatchedFileCount = localMedia.unmatchedFileCount,
                localMediaIsScanning = localMedia.isScanning,
                localMediaError = localMedia.error,
                librarySource = source,
                availableLibrarySources = availableSources
            )
        }
    }

    override suspend fun refresh() {
        val profile = activeProfile()
        homeViewModel.loadLibraryData(profile)
        homeViewModel.loadLibraryItems(profile)
        val localState = localMediaLibrary.state.value
        if (localState.sources.isNotEmpty() && localState.lastScanAtMs == 0L) {
            localMediaLibrary.scan()
        }
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

    override suspend fun addLocalMediaSource(source: LocalMediaSourceInput) = localMediaLibrary.addSource(source)

    override suspend fun removeLocalMediaSource(sourceId: String) {
        localMediaLibrary.removeSource(sourceId)
    }

    override suspend fun scanLocalMedia(forceMetadata: Boolean) = localMediaLibrary.scan(forceMetadata)

    override suspend fun loadFolder(folder: LibraryFolderUiModel): List<LibraryFolderSectionUiModel> {
        val profile = activeProfile()
        val domainFolder = profile?.safeLibraryCollections.orEmpty()
            .asSequence()
            .flatMap { it.folders.orEmpty().asSequence() }
            .firstOrNull { it.id == folder.id }
            ?: return emptyList()
        return homeViewModel.loadFolderSections(domainFolder).map { (title, metas) ->
            LibraryFolderSectionUiModel(title = title, items = metas.toCatalogItems(profile, deviceType = deviceType))
        }
    }
}

private fun List<com.fluxa.app.data.remote.Meta>.toLibraryCatalogItems(
    profile: UserProfile?,
    providerId: ThirdPartyProviderId?,
    deviceType: DeviceType = DeviceType.Mobile,
): List<CatalogItemUiModel> {
    if (providerId == null) return toCatalogItems(profile, deviceType = deviceType)
    val accountId = profile?.providerAccountId(providerId)
    val source = CatalogSourceUiModel(
        catalogType = null,
        providerId = providerId.key,
        providerAccountId = accountId,
        strictProviderData = true
    )
    return toCatalogItems(
        profile = profile,
        sources = flatMap { meta ->
            listOf(
                "${meta.type}:${meta.id}" to source.copy(catalogType = meta.type),
                meta.id to source.copy(catalogType = meta.type)
            )
        }.toMap(),
        deviceType = deviceType,
    )
}
