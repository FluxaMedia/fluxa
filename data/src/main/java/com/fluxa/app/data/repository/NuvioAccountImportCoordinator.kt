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
            val existing = existingProfiles.firstOrNull {
                it.nuvioUserId == connectedProfile.nuvioUserId && it.nuvioProfileIndex == remote.profileIndex
            }
            val avatarUrl = remote.avatarUrl ?: remote.avatarId
                ?.let { avatarId -> avatars.firstOrNull { it.id == avatarId }?.storagePath }
                ?.let { "$supabaseUrl/storage/v1/object/public/avatars/$it" }
            (existing ?: connectedProfile.copy(id = stableNuvioProfileId(connectedProfile, remote.profileIndex))).copy(
                email = connectedProfile.email,
                profileName = remote.name?.takeIf { it.isNotBlank() } ?: existing?.profileName,
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
            nuvioService.pullLibrary(token, mapOf("p_profile_id" to profileIndex, "p_limit" to 500, "p_offset" to 0)).requireBody()
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

        val watchProgress = importOrDefault(NuvioImportStep.PROGRESS, emptyList()) {
            nuvioService.pullWatchProgress(token, mapOf("p_profile_id" to profileIndex, "p_limit" to 200)).requireBody()
        }
        val importedProgressVideoIds = mutableMapOf<String, String>()
        for (entry in watchProgress) {
            val meta = metaById[entry.contentId] ?: run {
                val detail = runCatching {
                    addonRepository.getAddonMetaDetail(
                        type = entry.contentType,
                        id = entry.contentId,
                        authKey = "",
                        localAddons = profile.localAddons
                    )
                }.getOrNull()
                Meta(
                    id = entry.contentId,
                    name = detail?.name?.takeIf { it.isNotBlank() } ?: entry.contentId,
                    type = entry.contentType,
                    poster = detail?.poster,
                    background = detail?.background
                )
            }
            val videoId = if (entry.season != null && entry.episode != null) {
                "${entry.contentId}:${entry.season}:${entry.episode}"
            } else {
                entry.videoId.ifBlank { entry.contentId }
            }
            watchlistManager.savePlaybackProgress(
                meta = meta,
                timeOffset = entry.position,
                duration = entry.duration,
                lastVideoId = videoId
            )
            importedProgressVideoIds[entry.contentId] = videoId
        }
        onStep(NuvioImportStep.PROGRESS)

        val watchedItems = importOrDefault(NuvioImportStep.HISTORY, emptyList()) {
            nuvioService.pullWatchedItems(token, mapOf("p_profile_id" to profileIndex, "p_page" to 1, "p_page_size" to 500)).requireBody()
        }
        val watchedBySeries = mutableMapOf<String, MutableSet<String>>()
        for (item in watchedItems) {
            val videoId = if (item.season != null && item.episode != null) {
                "${item.contentId}:${item.season}:${item.episode}"
            } else {
                item.contentId
            }
            watchedBySeries.getOrPut(item.contentId) { mutableSetOf() }.add(videoId)
        }
        for ((seriesId, videoIds) in watchedBySeries) {
            watchlistManager.markEpisodesWatched(seriesId, videoIds)
            if (importedProgressVideoIds[seriesId]?.let(videoIds::contains) == true) {
                watchlistManager.clearPlaybackProgress(seriesId)
            }
        }
        onStep(NuvioImportStep.HISTORY)

        val collectionRows = importOrDefault(NuvioImportStep.COLLECTIONS, emptyList()) {
            nuvioService.pullCollections(token, mapOf("p_profile_id" to profileIndex)).requireBody()
        }
        val collections = collectionRows.firstOrNull()?.collectionsJson.orEmpty().map { collection ->
            LibraryUserCollection(
                id = collection.id ?: java.util.UUID.randomUUID().toString(),
                title = collection.title ?: "",
                imageUrl = collection.backdropImageUrl,
                showOnHome = collection.showOnHome ?: true,
                pinToTop = collection.pinToTop,
                viewMode = collection.viewMode,
                showAllTab = collection.showAllTab,
                focusGlowEnabled = collection.focusGlowEnabled ?: true,
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
