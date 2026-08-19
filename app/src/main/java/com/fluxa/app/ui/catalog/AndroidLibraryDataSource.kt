package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.LibraryRemoteSource
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder
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
import com.fluxa.app.shared.feature.library.LibraryFolderEditorUiModel
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
    val remoteLibrary: com.fluxa.app.ui.catalog.LibraryUiState,
    val userAddons: List<com.fluxa.app.data.remote.AddonDescriptor>
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
        homeViewModel.libraryUiState,
        homeViewModel.userAddons
    ) { watchlist, likedItems, remoteLibrary, userAddons ->
        AndroidLibrarySources(watchlist, likedItems, remoteLibrary, userAddons)
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

            val catalogOptions = sources.userAddons.flatMap { addon ->
                addon.manifest.catalogs.orEmpty()
                    .filter { catalog ->
                        catalog.extra.orEmpty().none { extra -> extra.isRequired == true && !extra.name.equals("genre", ignoreCase = true) }
                    }
                    .mapNotNull { catalog ->
                        val catalogId = catalog.id ?: return@mapNotNull null
                        val catalogType = catalog.type ?: return@mapNotNull null
                        com.fluxa.app.shared.feature.library.LibraryCatalogOptionUiModel(
                            addonId = addon.manifest.id,
                            addonName = addon.manifest.name,
                            catalogId = catalogId,
                            catalogType = catalogType,
                            catalogName = catalog.name ?: catalogId,
                            genreOptions = catalog.extra.orEmpty()
                                .firstOrNull { it.name.equals("genre", ignoreCase = true) }
                                ?.options.orEmpty()
                        )
                    }
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
                availableLibrarySources = availableSources,
                catalogOptions = catalogOptions
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

    override suspend fun folderForEditing(collectionId: String, folderId: String): LibraryFolderEditorUiModel? {
        val profile = activeProfile() ?: return null
        val folder = profile.safeLibraryCollections
            .firstOrNull { it.id == collectionId }
            ?.folders.orEmpty()
            .firstOrNull { it.id == folderId }
            ?: return null
        val tmdbSource = folder.sources.orEmpty().firstOrNull { it.provider == "tmdb" }
        val traktSource = folder.sources.orEmpty().firstOrNull { it.provider == "trakt" }
        val catalogSource = folder.catalogSources.orEmpty().firstOrNull()
        val sourceKind = when {
            traktSource != null -> com.fluxa.app.shared.feature.library.LibraryFolderSourceKind.Trakt
            catalogSource != null -> com.fluxa.app.shared.feature.library.LibraryFolderSourceKind.AddonCatalog
            else -> com.fluxa.app.shared.feature.library.LibraryFolderSourceKind.Tmdb
        }
        return LibraryFolderEditorUiModel(
            id = folder.id,
            title = folder.title,
            coverEmoji = folder.coverEmoji,
            sourceKind = sourceKind,
            tmdbSourceType = tmdbSource?.tmdbSourceType ?: "LIST",
            tmdbId = tmdbSource?.tmdbId?.toString().orEmpty(),
            traktInput = traktSource?.traktListId?.toString().orEmpty(),
            catalogAddonId = catalogSource?.addonId,
            catalogId = catalogSource?.catalogId,
            catalogGenre = catalogSource?.genre
        )
    }

    override suspend fun saveFolder(collectionId: String, folder: LibraryFolderEditorUiModel): Boolean {
        val profile = activeProfile() ?: return false
        val title = folder.title.trim()
        if (title.isEmpty()) return false

        var sources: List<LibraryRemoteSource>? = null
        var catalogSources: List<com.fluxa.app.data.local.LibraryCatalogSource>? = null
        when (folder.sourceKind) {
            com.fluxa.app.shared.feature.library.LibraryFolderSourceKind.Tmdb -> {
                val tmdbId = folder.tmdbId.trim().toLongOrNull() ?: return false
                sources = listOf(LibraryRemoteSource(provider = "tmdb", tmdbSourceType = folder.tmdbSourceType, tmdbId = tmdbId))
            }
            com.fluxa.app.shared.feature.library.LibraryFolderSourceKind.Trakt -> {
                val listId = com.fluxa.app.data.repository.TraktIntegration.resolveTraktListId(
                    folder.traktInput.trim(),
                    com.fluxa.app.BuildConfig.TRAKT_CLIENT_ID
                ) ?: return false
                sources = listOf(LibraryRemoteSource(provider = "trakt", traktListId = listId))
            }
            com.fluxa.app.shared.feature.library.LibraryFolderSourceKind.AddonCatalog -> {
                val addonId = folder.catalogAddonId?.trim().orEmpty()
                val catalogId = folder.catalogId?.trim().orEmpty()
                if (addonId.isEmpty() || catalogId.isEmpty()) return false
                val catalogType = homeViewModel.userAddons.value
                    .firstOrNull { it.manifest.id == addonId }
                    ?.manifest?.catalogs.orEmpty()
                    .firstOrNull { it.id == catalogId }
                    ?.type ?: "movie"
                catalogSources = listOf(
                    com.fluxa.app.data.local.LibraryCatalogSource(
                        addonId = addonId,
                        catalogId = catalogId,
                        type = catalogType,
                        genre = folder.catalogGenre?.trim()?.takeIf { it.isNotEmpty() }
                    )
                )
            }
        }

        val folderId = folder.id ?: "folder_${System.currentTimeMillis()}"
        val updated = profileManager.updateProfile(profile.id) { current ->
            current.copy(libraryCollections = current.safeLibraryCollections.map { collection ->
                if (collection.id != collectionId) return@map collection
                val existingFolders = collection.folders.orEmpty()
                val nextFolder = LibraryUserCollectionFolder(
                    id = folderId,
                    title = title,
                    coverEmoji = folder.coverEmoji?.trim()?.takeIf { it.isNotEmpty() },
                    sources = sources,
                    catalogSources = catalogSources
                )
                val nextFolders = if (existingFolders.any { it.id == folderId }) {
                    existingFolders.map { if (it.id == folderId) nextFolder else it }
                } else {
                    existingFolders + nextFolder
                }
                collection.copy(folders = nextFolders)
            })
        } ?: return false
        onProfileChanged(updated)
        return true
    }

    override suspend fun deleteFolder(collectionId: String, folderId: String): Boolean {
        val profile = activeProfile() ?: return false
        val updated = profileManager.updateProfile(profile.id) { current ->
            current.copy(libraryCollections = current.safeLibraryCollections.map { collection ->
                if (collection.id != collectionId) return@map collection
                collection.copy(folders = collection.folders.orEmpty().filterNot { it.id == folderId })
            })
        } ?: return false
        onProfileChanged(updated)
        return true
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
