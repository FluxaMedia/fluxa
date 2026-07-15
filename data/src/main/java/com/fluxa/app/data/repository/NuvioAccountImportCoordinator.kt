package com.fluxa.app.data.repository

import android.util.Log
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder
import com.fluxa.app.data.local.LibraryCatalogSource
import com.fluxa.app.data.local.LibraryRemoteSource
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.NuvioCredentials
import com.fluxa.app.data.remote.NuvioService
import com.fluxa.app.data.remote.NuvioSession
import retrofit2.Response

enum class NuvioImportStep { PROFILE, ADDONS, LIBRARY, PROGRESS, HISTORY, COLLECTIONS }

class NuvioAccountImportCoordinator(
    private val nuvioService: NuvioService,
    private val profileManager: ProfileManager,
    private val watchlistManager: WatchlistManager,
    private val addonRepository: AddonRepository,
    private val supabaseUrl: String
) {
    suspend fun refreshProfileIfNeeded(profile: UserProfile): UserProfile {
        val refreshToken = profile.nuvioRefreshToken?.takeIf { it.isNotBlank() } ?: return profile
        val expiresAt = profile.nuvioTokenExpiresAt ?: 0L
        if (!profile.nuvioAccessToken.isNullOrBlank() && expiresAt > System.currentTimeMillis() + 60_000L) return profile
        val session = nuvioService.refreshToken(request = com.fluxa.app.data.remote.NuvioRefreshRequest(refreshToken)).requireBody()
        val refreshed = profile.copy(
            nuvioAccessToken = session.accessToken,
            nuvioRefreshToken = session.refreshToken.ifBlank { refreshToken },
            nuvioTokenExpiresAt = session.expiresIn?.let { System.currentTimeMillis() + it * 1000L },
            nuvioUserId = session.user?.id ?: profile.nuvioUserId,
            nuvioEmail = session.user?.email ?: profile.nuvioEmail
        )
        profileManager.saveProfile(refreshed)
        return refreshed
    }

    suspend fun signIn(email: String, password: String): Result<NuvioSession> = authenticate {
        nuvioService.signIn(credentials = NuvioCredentials(email, password))
    }

    suspend fun signUp(email: String, password: String): Result<NuvioSession> = authenticate {
        nuvioService.signUp(NuvioCredentials(email, password))
    }

    suspend fun sync(profile: UserProfile, onStep: (NuvioImportStep) -> Unit): UserProfile {
        val refreshedProfile = refreshProfileIfNeeded(profile)
        val accessToken = refreshedProfile.nuvioAccessToken?.takeIf { it.isNotBlank() }
            ?: throw IllegalStateException("Nuvio is not connected")
        val refreshToken = refreshedProfile.nuvioRefreshToken.orEmpty()
        val session = NuvioSession(
            accessToken = accessToken,
            refreshToken = refreshToken,
            expiresIn = refreshedProfile.nuvioTokenExpiresAt?.let { expiresAt ->
                ((expiresAt - System.currentTimeMillis()) / 1000L).coerceAtLeast(0L)
            },
            user = refreshedProfile.nuvioUserId?.let { userId ->
                com.fluxa.app.data.remote.NuvioUser(userId, refreshedProfile.nuvioEmail ?: refreshedProfile.email)
            }
        )
        return import(refreshedProfile, session, onStep)
    }

    private suspend inline fun authenticate(call: suspend () -> retrofit2.Response<NuvioSession>): Result<NuvioSession> {
        return try {
            val response = call()
            val session = response.body()
            if (response.isSuccessful && session != null) {
                Result.success(session)
            } else {
                Result.failure(Exception("Nuvio sign-in failed (${response.code()})"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun import(
        baseProfile: UserProfile,
        session: NuvioSession,
        onStep: (NuvioImportStep) -> Unit
    ): UserProfile {
        val connectedProfile = baseProfile.copy(
            email = session.user?.email ?: baseProfile.email,
            nuvioAccessToken = session.accessToken,
            nuvioRefreshToken = session.refreshToken,
            nuvioTokenExpiresAt = session.expiresIn?.let { System.currentTimeMillis() + it * 1000L },
            nuvioUserId = session.user?.id,
            nuvioEmail = session.user?.email ?: baseProfile.email
        )
        val token = "Bearer ${session.accessToken}"
        profileManager.saveProfileReplacingLocalAddons(connectedProfile)
        profileManager.setLastActiveProfile(connectedProfile)
        watchlistManager.setActiveProfile(baseProfile.id)

        var profile = connectedProfile
        val profiles = importOrDefault(NuvioImportStep.PROFILE, emptyList()) {
            nuvioService.pullProfiles(token).requireBody()
        }
        val avatars = importOrDefault(NuvioImportStep.PROFILE, emptyList()) {
            nuvioService.listAvatars().requireBody()
        }
        val primary = profiles.firstOrNull { it.profileIndex == connectedProfile.nuvioProfileIndex }
            ?: profiles.minByOrNull { it.profileIndex }
        val profileIndex = primary?.profileIndex ?: connectedProfile.nuvioProfileIndex ?: 1

        val existingProfiles = profileManager.getProfiles()
        val importedProfiles = profiles.map { remote ->
            val matchedProfile = existingProfiles.firstOrNull {
                it.nuvioUserId == connectedProfile.nuvioUserId && it.nuvioProfileIndex == remote.profileIndex
            }
            val localProfile = existingProfiles.firstOrNull {
                it.id != baseProfile.id &&
                    (it.email.equals(connectedProfile.email, ignoreCase = true) ||
                        it.nuvioEmail.equals(connectedProfile.nuvioEmail, ignoreCase = true)) &&
                    (it.nuvioProfileIndex == null || it.nuvioProfileIndex == remote.profileIndex) &&
                    (!it.profileName.isNullOrBlank() || !it.avatarUrl.isNullOrBlank())
            }
            val existing = matchedProfile ?: localProfile ?: baseProfile.takeIf {
                remote.profileIndex == profileIndex &&
                    (it.nuvioProfileIndex == null || it.nuvioProfileIndex == remote.profileIndex)
            }
            val avatarUrl = remote.avatarUrl ?: remote.avatarId
                ?.let { avatarId -> avatars.firstOrNull { it.id == avatarId }?.storagePath }
                ?.let { "$supabaseUrl/storage/v1/object/public/avatars/$it" }
            val remoteName = remote.name?.trim()?.takeIf { it.isNotBlank() }
            val preservedName = existing?.profileName?.trim()?.takeIf { it.isNotBlank() }
            (existing ?: connectedProfile.copy(id = stableNuvioProfileId(connectedProfile, remote.profileIndex))).copy(
                email = connectedProfile.email,
                profileName = when {
                    remoteName == null -> preservedName
                    remoteName.equals(connectedProfile.nuvioEmail, ignoreCase = true) && preservedName != null -> preservedName
                    else -> remoteName
                },
                avatarUrl = avatarUrl ?: existing?.avatarUrl,
                nuvioAccessToken = connectedProfile.nuvioAccessToken,
                nuvioRefreshToken = connectedProfile.nuvioRefreshToken,
                nuvioTokenExpiresAt = connectedProfile.nuvioTokenExpiresAt,
                nuvioUserId = connectedProfile.nuvioUserId,
                nuvioEmail = connectedProfile.nuvioEmail,
                nuvioProfileIndex = remote.profileIndex
            )
        }
        importedProfiles.forEach(profileManager::saveProfileReplacingLocalAddons)
        profile = importedProfiles.firstOrNull { it.nuvioProfileIndex == profileIndex }
            ?: connectedProfile.copy(nuvioProfileIndex = profileIndex)
        profileManager.setLastActiveProfile(profile)
        watchlistManager.setActiveProfile(profile.id)
        onStep(NuvioImportStep.PROFILE)

        val addons = try {
            nuvioService.pullAddons(token, profileId = "eq.$profileIndex").requireBody()
        } catch (error: Exception) {
            Log.w("NuvioImport", "Import step ${NuvioImportStep.ADDONS} failed; continuing without it", error)
            null
        }
        if (addons != null) {
            val orderedAddons = addons.sortedBy { it.sortOrder }
            profile = profile.copy(
                localAddons = orderedAddons.map { it.url }.distinct(),
                disabledLocalAddons = orderedAddons.filterNot { it.enabled }.map { it.url }.distinct()
            )
        }
        onStep(NuvioImportStep.ADDONS)

        val libraryItems = try {
            pullAllLibraryItems(token, profileIndex)
        } catch (error: Exception) {
            Log.w("NuvioImport", "Import step ${NuvioImportStep.LIBRARY} failed; continuing without it", error)
            null
        }
        val metaById = libraryItems.orEmpty().associate { item ->
            item.contentId to Meta(
                id = item.contentId,
                name = item.name,
                type = item.contentType,
                poster = item.poster,
                background = item.background,
                description = item.description,
                releaseInfo = item.releaseInfo,
                imdbRating = item.imdbRating?.toString(),
                genres = item.genres
            )
        }
        if (libraryItems != null) {
            watchlistManager.replaceWatchlist(metaById.values.toList())
        }
        onStep(NuvioImportStep.LIBRARY)

        val watchProgress = try {
            nuvioService.pullWatchProgress(token, mapOf("p_profile_id" to profileIndex)).requireBody()
        } catch (error: Exception) {
            Log.w("NuvioImport", "Import step ${NuvioImportStep.PROGRESS} failed; keeping existing playback progress", error)
            null
        }
        val importedProgressVideoIds = watchProgress.orEmpty().mapTo(mutableSetOf()) { entry ->
            if (entry.season != null && entry.episode != null) {
                "${entry.contentId}:${entry.season}:${entry.episode}"
            } else {
                entry.videoId.ifBlank { entry.contentId }
            }
        }
        if (watchProgress != null) {
            watchlistManager.clearAllPlaybackProgress()
        }
        val latestProgressByContent = watchProgress.orEmpty()
            .groupBy { it.contentId }
            .values
            .mapNotNull { entries -> entries.maxWithOrNull(compareBy({ it.lastWatched }, { it.position })) }
            .sortedByDescending { it.lastWatched }
        for (entry in latestProgressByContent) {
            val detail = if (
                entry.contentType == "series" ||
                entry.contentType == "tv" ||
                entry.contentType == "anime" ||
                entry.contentId !in metaById
            ) {
                runCatching {
                    addonRepository.getAddonMetaDetail(
                        type = entry.contentType,
                        id = entry.contentId,
                        authKey = "",
                        localAddons = profile.localAddons
                    )
                }.getOrNull()
            } else {
                null
            }
            val episode = detail?.videos?.firstOrNull {
                it.season == entry.season && it.number == entry.episode
            }
            val meta = metaById[entry.contentId]?.copy(
                continueWatchingPoster = episode?.thumbnail,
                continueWatchingBackground = episode?.thumbnail
            ) ?: run {
                Meta(
                    id = entry.contentId,
                    name = detail?.name?.takeIf { it.isNotBlank() } ?: entry.contentId,
                    type = entry.contentType,
                    poster = detail?.poster,
                    background = detail?.background,
                    continueWatchingPoster = episode?.thumbnail,
                    continueWatchingBackground = episode?.thumbnail
                )
            }
            val videoId = if (entry.season != null && entry.episode != null) {
                "${entry.contentId}:${entry.season}:${entry.episode}"
            } else {
                entry.videoId.ifBlank { entry.contentId }
            }
            val isUpNextPlaceholder = entry.position <= NUVIO_UP_NEXT_POSITION_MS &&
                entry.duration == NUVIO_UP_NEXT_DURATION_MS
            watchlistManager.savePlaybackProgress(
                meta = meta,
                timeOffset = if (isUpNextPlaceholder) 0L else entry.position,
                duration = if (isUpNextPlaceholder) 0L else entry.duration,
                lastVideoId = videoId,
                lastEpisodeName = episode?.name,
                updatedAt = entry.lastWatched
            )
            if (entry.contentType != "movie") {
                watchlistManager.markEpisodeWatched(entry.contentId, videoId, false)
            }
        }
        onStep(NuvioImportStep.PROGRESS)

        val watchedItems = try {
            pullAllWatchedItems(token, profileIndex)
        } catch (error: Exception) {
            Log.w("NuvioImport", "Import step ${NuvioImportStep.HISTORY} failed; keeping existing watched episodes", error)
            null
        }
        if (watchedItems != null) {
            watchlistManager.clearAllWatchedEpisodes()
        }
        val watchedBySeries = mutableMapOf<String, MutableSet<String>>()
        for (item in watchedItems.orEmpty()) {
            val videoId = if (item.season != null && item.episode != null) {
                "${item.contentId}:${item.season}:${item.episode}"
            } else {
                item.contentId
            }
            if (videoId !in importedProgressVideoIds) {
                watchedBySeries.getOrPut(item.contentId) { mutableSetOf() }.add(videoId)
            }
        }
        for ((seriesId, videoIds) in watchedBySeries) {
            watchlistManager.markEpisodesWatched(seriesId, videoIds)
        }
        onStep(NuvioImportStep.HISTORY)

        val collectionRows = importOrDefault(NuvioImportStep.COLLECTIONS, emptyList()) {
            nuvioService.pullCollections(token, mapOf("p_profile_id" to profileIndex)).requireBody()
        }
        val collections = collectionRows.flatMap { it.collectionsJson.orEmpty() }.map { collection ->
            LibraryUserCollection(
                id = collection.id ?: java.util.UUID.randomUUID().toString(),
                title = collection.title ?: "",
                imageUrl = collection.backdropImageUrl,
                showOnHome = collection.showOnHome ?: true,
                pinToTop = collection.pinToTop,
                viewMode = collection.viewMode,
                showAllTab = collection.showAllTab,
                focusGlowEnabled = collection.focusGlowEnabled ?: true,
                community = collection.community,
                folders = collection.folders?.map { folder ->
                    LibraryUserCollectionFolder(
                        id = folder.id ?: java.util.UUID.randomUUID().toString(),
                        title = folder.title ?: "",
                        coverImageUrl = folder.coverImageUrl,
                        coverEmoji = folder.coverEmoji,
                        focusGifUrl = folder.focusGifUrl,
                        focusGifEnabled = folder.focusGifEnabled ?: true,
                        titleLogoUrl = folder.titleLogoUrl,
                        heroBackdropUrl = folder.heroBackdropUrl,
                        heroVideoUrl = folder.heroVideoUrl,
                        shape = folder.tileShape.toNuvioFolderShape(),
                        hideTitle = folder.hideTitle,
                        catalogSources = folder.catalogSources?.map { source ->
                            LibraryCatalogSource(
                                addonId = source.addonId,
                                catalogId = source.catalogId ?: "",
                                type = source.type ?: "movie",
                                genre = source.genre
                            )
                        }?.filter { it.catalogId.isNotBlank() }
                        ,
                        sources = folder.catalogSources?.map { source ->
                            LibraryRemoteSource(
                                provider = source.provider ?: "addon",
                                title = source.title,
                                mediaType = source.mediaType,
                                traktListId = source.traktListId,
                                tmdbSourceType = source.tmdbSourceType,
                                tmdbId = source.tmdbId,
                                sortBy = source.sortBy,
                                sortHow = source.sortHow,
                                filters = source.filters,
                                addonId = source.addonId,
                                catalogId = source.catalogId,
                                type = source.type,
                                genre = source.genre
                            )
                        }
                    )
                }
            )
        }
        if (collectionRows.isNotEmpty()) {
            profile = profile.copy(libraryCollections = collections)
        }
        onStep(NuvioImportStep.COLLECTIONS)

        profile = profile.copy(nuvioLastSyncAt = System.currentTimeMillis())
        profileManager.saveProfileReplacingLocalAddons(profile)
        profileManager.setLastActiveProfile(profile)
        return profile
    }

    private suspend fun <T> importOrDefault(
        step: NuvioImportStep,
        fallback: T,
        call: suspend () -> T
    ): T = try {
        call()
    } catch (error: Exception) {
        Log.w("NuvioImport", "Import step $step failed; continuing without it", error)
        fallback
    }

    private suspend fun pullAllLibraryItems(token: String, profileIndex: Int): List<com.fluxa.app.data.remote.NuvioLibraryItem> {
        val items = mutableListOf<com.fluxa.app.data.remote.NuvioLibraryItem>()
        var offset = 0
        do {
            val page = nuvioService.pullLibrary(
                token,
                mapOf("p_profile_id" to profileIndex, "p_limit" to NUVIO_PAGE_SIZE, "p_offset" to offset)
            ).requireBody()
            items += page
            offset += page.size
        } while (page.size == NUVIO_PAGE_SIZE)
        return items
    }

    private suspend fun pullAllWatchedItems(token: String, profileIndex: Int): List<com.fluxa.app.data.remote.NuvioWatchedItem> {
        val items = mutableListOf<com.fluxa.app.data.remote.NuvioWatchedItem>()
        var pageNumber = 1
        do {
            val page = nuvioService.pullWatchedItems(
                token,
                mapOf("p_profile_id" to profileIndex, "p_page" to pageNumber, "p_page_size" to NUVIO_PAGE_SIZE)
            ).requireBody()
            items += page
            pageNumber += 1
        } while (page.size == NUVIO_PAGE_SIZE)
        return items
    }

    private companion object {
        const val NUVIO_PAGE_SIZE = 500
        const val NUVIO_UP_NEXT_POSITION_MS = 1_000L
        const val NUVIO_UP_NEXT_DURATION_MS = 99_999_000L
    }
}

private fun String?.toNuvioFolderShape(): String = when (this?.trim()?.lowercase()) {
    "landscape", "wide" -> "wide"
    "square" -> "square"
    else -> "poster"
}

private fun stableNuvioProfileId(profile: UserProfile, profileIndex: Int): String {
    val identity = profile.nuvioUserId ?: profile.nuvioEmail ?: profile.email
    return java.util.UUID.nameUUIDFromBytes("nuvio:$identity:$profileIndex".toByteArray()).toString()
}

private fun <T> Response<T>.requireBody(): T {
    if (!isSuccessful) throw IllegalStateException("Nuvio request failed (${code()})")
    return body() ?: throw IllegalStateException("Nuvio returned an empty response")
}
