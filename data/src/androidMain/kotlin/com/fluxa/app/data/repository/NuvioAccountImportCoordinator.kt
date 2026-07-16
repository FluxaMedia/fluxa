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
import com.fluxa.app.data.remote.NuvioSessionDto
import com.fluxa.app.data.remote.toDto
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
        val session = nuvioService.refreshToken(request = com.fluxa.app.data.remote.NuvioRefreshRequest(refreshToken).toDto()).requireBody().toDomain()
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

    private suspend inline fun authenticate(call: suspend () -> retrofit2.Response<NuvioSessionDto>): Result<NuvioSession> {
        return try {
            val response = call()
            val session = response.body()?.toDomain()
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
            nuvioService.pullProfiles(token).requireBody().map { it.toDomain() }
        }
        val avatars = importOrDefault(NuvioImportStep.PROFILE, emptyList()) {
            nuvioService.listAvatars().requireBody().map { it.toDomain() }
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
            nuvioService.pullAddons(token, profileId = "eq.$profileIndex").requireBody().map { it.toDomain() }
        } catch (error: Exception) {
            Log.w("NuvioImport", "Import step ${NuvioImportStep.ADDONS} failed; continuing without it", error)
            null
        }
        if (addons != null) {
            val addonState = NuvioImportPolicy.addonState(addons)
            profile = profile.copy(
                localAddons = addonState.installedUrls,
                disabledLocalAddons = addonState.disabledUrls
            )
        }
        onStep(NuvioImportStep.ADDONS)

        val libraryItems = try {
            pullAllLibraryItems(token, profileIndex)
        } catch (error: Exception) {
            Log.w("NuvioImport", "Import step ${NuvioImportStep.LIBRARY} failed; continuing without it", error)
            null
        }
        val metaById = NuvioImportPolicy.libraryMetas(libraryItems.orEmpty())
        if (libraryItems != null) {
            watchlistManager.replaceWatchlist(metaById.values.toList())
        }
        onStep(NuvioImportStep.LIBRARY)

        val watchProgress = try {
            nuvioService.pullWatchProgress(token, mapOf("p_profile_id" to profileIndex)).requireBody().map { it.toDomain() }
        } catch (error: Exception) {
            Log.w("NuvioImport", "Import step ${NuvioImportStep.PROGRESS} failed; keeping existing playback progress", error)
            null
        }
        val importedProgressVideoIds = watchProgress.orEmpty().mapTo(mutableSetOf(), NuvioImportPolicy::progressVideoId)
        if (watchProgress != null) {
            watchlistManager.clearAllPlaybackProgress()
        }
        val latestProgressByContent = NuvioImportPolicy.latestProgressByContent(watchProgress.orEmpty())
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
            val videoId = NuvioImportPolicy.progressVideoId(entry)
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
        val watchedBySeries = NuvioImportPolicy.watchedVideoIds(watchedItems.orEmpty(), importedProgressVideoIds)
        for ((seriesId, videoIds) in watchedBySeries) {
            watchlistManager.markEpisodesWatched(seriesId, videoIds)
        }
        onStep(NuvioImportStep.HISTORY)

        val collectionRows = importOrDefault(NuvioImportStep.COLLECTIONS, emptyList()) {
            nuvioService.pullCollections(token, mapOf("p_profile_id" to profileIndex)).requireBody().map { it.toDomain() }
        }
        val collections = NuvioImportPolicy.collections(collectionRows) { java.util.UUID.randomUUID().toString() }
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
            ).requireBody().map { it.toDomain() }
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
            ).requireBody().map { it.toDomain() }
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

private fun stableNuvioProfileId(profile: UserProfile, profileIndex: Int): String {
    val identity = profile.nuvioUserId ?: profile.nuvioEmail ?: profile.email
    return java.util.UUID.nameUUIDFromBytes("nuvio:$identity:$profileIndex".toByteArray()).toString()
}

private fun <T> Response<T>.requireBody(): T {
    if (!isSuccessful) throw IllegalStateException("Nuvio request failed (${code()})")
    return body() ?: throw IllegalStateException("Nuvio returned an empty response")
}
