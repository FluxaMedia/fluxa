package com.fluxa.app.data.repository

import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder
import com.fluxa.app.data.local.LibraryCatalogSource
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
        watchlistManager.setActiveProfile(baseProfile.id)

        var profile = connectedProfile
        val profiles = nuvioService.pullProfiles(token).requireBody()
        val primary = profiles.firstOrNull { it.profileIndex == connectedProfile.nuvioProfileIndex }
            ?: profiles.minByOrNull { it.profileIndex }
        val profileIndex = primary?.profileIndex ?: connectedProfile.nuvioProfileIndex ?: 1

        if (primary != null) {
            val avatarUrl = primary.avatarUrl ?: primary.avatarId?.let { avatarId ->
                nuvioService.listAvatars().requireBody()
                    ?.firstOrNull { it.id == avatarId }
                    ?.storagePath
                    ?.let { "${supabaseUrl}storage/v1/object/public/avatars/$it" }
            }
            profile = profile.copy(
                avatarUrl = avatarUrl ?: profile.avatarUrl,
                nuvioProfileIndex = profileIndex
            )
        } else {
            profile = profile.copy(nuvioProfileIndex = profileIndex)
        }
        onStep(NuvioImportStep.PROFILE)

        val addons = nuvioService.pullAddons(token, profileId = "eq.$profileIndex").requireBody()
        if (addons.isNotEmpty()) {
            val enabledUrls = addons.filter { it.enabled }.sortedBy { it.sortOrder }.map { it.url }
            profile = profile.copy(localAddons = (profile.localAddons.orEmpty() + enabledUrls).distinct())
        }
        onStep(NuvioImportStep.ADDONS)

        val libraryItems = nuvioService.pullLibrary(token, mapOf("p_profile_id" to profileIndex, "p_limit" to 500, "p_offset" to 0)).requireBody()
        val metaById = libraryItems.associate { item ->
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
        for (meta in metaById.values) {
            if (!watchlistManager.isInWatchlist(meta.id)) {
                watchlistManager.toggleWatchlist(meta)
            }
        }
        onStep(NuvioImportStep.LIBRARY)

        val watchProgress = nuvioService.pullWatchProgress(token, mapOf("p_profile_id" to profileIndex, "p_limit" to 200)).requireBody()
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
        }
        onStep(NuvioImportStep.PROGRESS)

        val watchedItems = nuvioService.pullWatchedItems(token, mapOf("p_profile_id" to profileIndex, "p_page" to 1, "p_page_size" to 500)).requireBody()
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
        }
        onStep(NuvioImportStep.HISTORY)

        val collectionRows = nuvioService.pullCollections(token, mapOf("p_profile_id" to profileIndex)).requireBody()
        val collections = collectionRows.firstOrNull()?.collectionsJson.orEmpty().map { collection ->
            LibraryUserCollection(
                id = collection.id ?: java.util.UUID.randomUUID().toString(),
                title = collection.title ?: "",
                imageUrl = collection.backdropImageUrl,
                showOnHome = collection.pinToTop,
                pinToTop = collection.pinToTop,
                viewMode = collection.viewMode,
                showAllTab = collection.showAllTab,
                folders = collection.folders?.map { folder ->
                    LibraryUserCollectionFolder(
                        id = folder.id ?: java.util.UUID.randomUUID().toString(),
                        title = folder.title ?: "",
                        coverImageUrl = folder.coverImageUrl,
                        coverEmoji = folder.coverEmoji,
                        focusGifUrl = folder.focusGifUrl,
                        titleLogoUrl = folder.titleLogoUrl,
                        shape = folder.tileShape,
                        hideTitle = folder.hideTitle,
                        catalogSources = folder.catalogSources?.map { source ->
                            LibraryCatalogSource(
                                addonId = source.addonId,
                                catalogId = source.catalogId ?: "",
                                type = source.type ?: "movie"
                            )
                        }
                    )
                }
            )
        }
        if (collections.isNotEmpty()) {
            profile = profile.copy(libraryCollections = collections)
        }
        onStep(NuvioImportStep.COLLECTIONS)

        profile = profile.copy(nuvioLastSyncAt = System.currentTimeMillis())
        profileManager.saveProfile(profile)
        profileManager.setLastActiveProfile(profile)
        return profile
    }
}

private fun <T> Response<T>.requireBody(): T {
    if (!isSuccessful) throw IllegalStateException("Nuvio request failed (${code()})")
    return body() ?: throw IllegalStateException("Nuvio returned an empty response")
}
