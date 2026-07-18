package com.fluxa.app.data.repository

import android.util.Log
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.NuvioAvatarDto
import com.fluxa.app.data.remote.NuvioCredentials
import com.fluxa.app.data.remote.NuvioLibraryItemDto
import com.fluxa.app.data.remote.NuvioProfileDto
import com.fluxa.app.data.remote.NuvioService
import com.fluxa.app.data.remote.NuvioSession
import com.fluxa.app.data.remote.NuvioSessionDto
import com.fluxa.app.data.remote.NuvioWatchProgressDto
import com.fluxa.app.data.remote.NuvioWatchedItemDto
import com.fluxa.app.data.remote.toDto
import com.fluxa.app.core.rust.FluxaCoreNative
import com.google.gson.Gson
import com.google.gson.JsonArray
import com.google.gson.JsonObject
import retrofit2.Response

enum class NuvioImportStep { PROFILE, ADDONS, LIBRARY, PROGRESS, HISTORY, COLLECTIONS }

class NuvioAccountImportCoordinator(
    private val nuvioService: NuvioService,
    private val profileManager: ProfileManager,
    private val watchlistManager: WatchlistManager,
    private val addonRepository: AddonRepository,
    private val supabaseUrl: String,
    private val gson: Gson
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

    suspend fun sync(profile: UserProfile, onStep: (NuvioImportStep) -> Unit): NuvioImportResult {
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
    ): NuvioImportResult {
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

        val profileDtos = importOrDefault(NuvioImportStep.PROFILE, emptyList()) {
            nuvioService.pullProfiles(token).requireBody()
        }
        val avatarDtos = importOrDefault(NuvioImportStep.PROFILE, emptyList()) {
            nuvioService.listAvatars().requireBody()
        }
        val primaryIndex = profileDtos.firstOrNull { it.profileIndex == connectedProfile.nuvioProfileIndex }?.profileIndex
            ?: profileDtos.minByOrNull { it.profileIndex }?.profileIndex ?: 1

        var profile = mergeProfiles(baseProfile, connectedProfile, profileDtos, avatarDtos, primaryIndex)
        profileManager.setLastActiveProfile(profile)
        watchlistManager.setActiveProfile(profile.id)
        onStep(NuvioImportStep.PROFILE)

        val addonDtos = try {
            nuvioService.pullAddons(token, profileId = "eq.$primaryIndex").requireBody()
        } catch (error: Exception) {
            Log.w("NuvioImport", "Import step ${NuvioImportStep.ADDONS} failed; continuing without it", error)
            null
        }
        if (addonDtos != null) {
            val addonState = NuvioImportPolicy.addonState(addonDtos.map { it.toDomain() })
            profile = profile.copy(
                localAddons = addonState.installedUrls,
                disabledLocalAddons = addonState.disabledUrls
            )
        }
        onStep(NuvioImportStep.ADDONS)

        val libraryItems = try {
            pullAllLibraryItems(token, primaryIndex)
        } catch (error: Exception) {
            Log.w("NuvioImport", "Import step ${NuvioImportStep.LIBRARY} failed; continuing without it", error)
            null
        }
        val libraryJson = gson.toJsonTree(libraryItems.orEmpty()).asJsonArray
        if (libraryItems != null) {
            val watchlistItems = gson.fromJson(NuvioCoreBridge.libraryToWatchlist(libraryJson), Array<Meta>::class.java).toList()
            watchlistManager.replaceWatchlist(watchlistItems)
        }
        onStep(NuvioImportStep.LIBRARY)

        val watchProgressDtos = try {
            nuvioService.pullWatchProgress(token, mapOf("p_profile_id" to primaryIndex)).requireBody()
        } catch (error: Exception) {
            Log.w("NuvioImport", "Import step ${NuvioImportStep.PROGRESS} failed; keeping existing playback progress", error)
            null
        }
        val watchedItemDtos = try {
            pullAllWatchedItems(token, primaryIndex)
        } catch (error: Exception) {
            Log.w("NuvioImport", "Import step ${NuvioImportStep.HISTORY} failed; keeping existing watched episodes", error)
            null
        }

        if (watchProgressDtos != null) {
            val watchProgressJson = gson.toJsonTree(watchProgressDtos).asJsonArray
            val addonMetas = fetchAddonMetaNeeds(watchProgressJson, libraryJson, profile.localAddons)
            val mergePlan = NuvioCoreBridge.importMergePlan(
                libraryJson,
                addonMetas,
                watchProgressJson,
                gson.toJsonTree(watchedItemDtos.orEmpty()).asJsonArray
            )
            val progressItems = mergePlan.getAsJsonObject("progress").entrySet().mapNotNull { (_, value) ->
                progressEntryToMeta(value.asJsonObject)
            }
            watchlistManager.replaceExternalContinueWatching(setOf("nuvio"), progressItems)

            val watchedBySeries = mutableMapOf<String, MutableSet<String>>()
            mergePlan.getAsJsonObject("watched").entrySet().forEach { (key, value) ->
                if (!value.asBoolean) return@forEach
                val seriesId = key.substringBefore(':')
                watchedBySeries.getOrPut(seriesId, ::mutableSetOf).add(key)
            }
            watchlistManager.replaceExternalWatchedEpisodes("nuvio", watchedBySeries)
            reconcileLocalProgressToNuvio(token, primaryIndex, watchProgressDtos)
        }
        onStep(NuvioImportStep.PROGRESS)
        onStep(NuvioImportStep.HISTORY)

        val collectionRows = importOrDefault(NuvioImportStep.COLLECTIONS, emptyList()) {
            nuvioService.pullCollections(token, mapOf("p_profile_id" to primaryIndex)).requireBody()
        }
        val flatCollections = collectionRows.flatMap { it.collectionsJson.orEmpty() }
        if (flatCollections.isNotEmpty()) {
            val mappedCollections = gson.toJsonTree(flatCollections).asJsonArray.let(NuvioCoreBridge::mapCollections)
            profile = profile.copy(
                libraryCollections = gson.fromJson(mappedCollections, Array<LibraryUserCollection>::class.java).toList()
            )
        }
        onStep(NuvioImportStep.COLLECTIONS)

        profile = profile.copy(nuvioLastSyncAt = System.currentTimeMillis())
        val latestProfile = profileManager.getProfiles().firstOrNull { it.id == profile.id }
        val finalProfile = (latestProfile ?: profile).copy(
            email = profile.email,
            nuvioAccessToken = profile.nuvioAccessToken,
            nuvioRefreshToken = profile.nuvioRefreshToken,
            nuvioTokenExpiresAt = profile.nuvioTokenExpiresAt,
            nuvioUserId = profile.nuvioUserId,
            nuvioEmail = profile.nuvioEmail,
            nuvioProfileIndex = profile.nuvioProfileIndex,
            localAddons = profile.localAddons,
            disabledLocalAddons = profile.disabledLocalAddons,
            libraryCollections = profile.libraryCollections,
            nuvioLastSyncAt = profile.nuvioLastSyncAt
        )
        profileManager.saveProfileReplacingLocalAddons(finalProfile)
        profileManager.setLastActiveProfile(finalProfile)
        profileManager.clearExternalSyncFailure(finalProfile.id, "nuvio")
        return NuvioImportResult(finalProfile, watchlistManager.getExternalContinueWatchingSnapshot())
    }

    private fun mergeProfiles(
        baseProfile: UserProfile,
        connectedProfile: UserProfile,
        remoteProfiles: List<NuvioProfileDto>,
        avatars: List<NuvioAvatarDto>,
        primaryIndex: Int
    ): UserProfile {
        val remoteProfilesJson = gson.toJsonTree(remoteProfiles.map { it.withResolvedAvatarUrl(avatars, supabaseUrl) }).asJsonArray
        val existingById = profileManager.getProfiles().associateBy { it.id }.toMutableMap()

        remoteProfiles.forEach { remote ->
            val alreadyLinked = existingById.values.any {
                it.nuvioUserId == connectedProfile.nuvioUserId && it.nuvioProfileIndex == remote.profileIndex
            }
            if (alreadyLinked) return@forEach
            val localMatch = existingById.values.firstOrNull {
                it.id != baseProfile.id &&
                    (it.email.equals(connectedProfile.email, ignoreCase = true) ||
                        it.nuvioEmail.equals(connectedProfile.nuvioEmail, ignoreCase = true)) &&
                    (it.nuvioProfileIndex == null || it.nuvioProfileIndex == remote.profileIndex) &&
                    (!it.profileName.isNullOrBlank() || !it.avatarUrl.isNullOrBlank())
            }
            val target = localMatch ?: existingById[baseProfile.id]?.takeIf {
                remote.profileIndex == primaryIndex && (it.nuvioProfileIndex == null || it.nuvioProfileIndex == remote.profileIndex)
            }
            if (target != null) {
                existingById[target.id] = target.copy(
                    nuvioUserId = connectedProfile.nuvioUserId,
                    nuvioProfileIndex = remote.profileIndex
                )
            }
        }

        val sessionProfileJson = JsonObject().apply {
            addProperty("nuvioUserId", connectedProfile.nuvioUserId)
            addProperty("nuvioEmail", connectedProfile.nuvioEmail)
            addProperty("email", connectedProfile.email)
            addProperty("nuvioAccessToken", connectedProfile.nuvioAccessToken)
            addProperty("nuvioRefreshToken", connectedProfile.nuvioRefreshToken)
            connectedProfile.nuvioTokenExpiresAt?.let { addProperty("nuvioTokenExpiresAt", it) }
        }
        val existingProfilesJson = gson.toJsonTree(existingById.values.toList()).asJsonArray

        val merged = NuvioCoreBridge.buildLocalProfiles(sessionProfileJson, remoteProfilesJson, gson.toJsonTree(avatars), existingProfilesJson)
        val mergedProfiles = merged.map { element ->
            val obj = element.asJsonObject
            val importedName = obj.remove("name")?.takeUnless { it.isJsonNull }?.asString
            if (importedName != null) obj.addProperty("profileName", importedName)
            gson.fromJson(obj, UserProfile::class.java)
        }
        mergedProfiles.forEach(profileManager::saveProfileReplacingLocalAddons)
        return mergedProfiles.firstOrNull { it.nuvioProfileIndex == primaryIndex }
            ?: connectedProfile.copy(nuvioProfileIndex = primaryIndex)
    }

    private suspend fun fetchAddonMetaNeeds(watchProgressJson: JsonArray, libraryJson: JsonArray, localAddons: List<String>?): JsonObject {
        val needs = NuvioCoreBridge.progressMetaNeeds(watchProgressJson, libraryJson)
        val addonMetas = JsonObject()
        needs.forEach { need ->
            val obj = need.asJsonObject
            val contentId = obj.get("contentId")?.takeUnless { it.isJsonNull }?.asString ?: return@forEach
            val contentType = obj.get("contentType")?.takeUnless { it.isJsonNull }?.asString ?: return@forEach
            val detail = runCatching {
                addonRepository.getAddonMetaDetail(type = contentType, id = contentId, authKey = "", localAddons = localAddons)
            }.getOrNull() ?: return@forEach
            val metaJson = JsonObject().apply {
                addProperty("name", detail.name)
                detail.poster?.let { addProperty("poster", it) }
                detail.background?.let { addProperty("background", it) }
                add(
                    "videos",
                    gson.toJsonTree(
                        detail.videos.orEmpty().map { video ->
                            mapOf("season" to video.season, "episode" to video.number, "title" to video.name, "thumbnail" to video.thumbnail)
                        }
                    )
                )
            }
            addonMetas.add(contentId, metaJson)
        }
        return addonMetas
    }

    private fun progressEntryToMeta(entry: JsonObject): Meta? {
        val meta = entry.getAsJsonObject("meta") ?: return null
        val id = meta.get("id")?.takeUnless { it.isJsonNull }?.asString ?: return null
        val thumbnail = entry.get("lastEpisodeThumbnail")?.takeUnless { it.isJsonNull }?.asString
        val savedAt = entry.get("savedAt")?.takeUnless { it.isJsonNull }?.asString
        val lastWatchedAt = savedAt?.let { runCatching { java.time.Instant.parse(it).toEpochMilli() }.getOrNull() }
        return Meta(
            id = id,
            name = meta.get("name")?.takeUnless { it.isJsonNull }?.asString ?: id,
            type = meta.get("type")?.takeUnless { it.isJsonNull }?.asString ?: "",
            poster = meta.get("poster")?.takeUnless { it.isJsonNull }?.asString,
            background = meta.get("background")?.takeUnless { it.isJsonNull }?.asString,
            timeOffset = entry.get("timeOffset")?.takeUnless { it.isJsonNull }?.asLong,
            duration = entry.get("duration")?.takeUnless { it.isJsonNull }?.asLong,
            lastVideoId = entry.get("lastVideoId")?.takeUnless { it.isJsonNull }?.asString,
            lastEpisodeName = entry.get("lastEpisodeName")?.takeUnless { it.isJsonNull }?.asString,
            lastWatchedAt = lastWatchedAt,
            reason = "Nuvio",
            continueWatchingPoster = thumbnail,
            continueWatchingBackground = thumbnail
        )
    }

    private suspend fun reconcileLocalProgressToNuvio(
        token: String,
        profileIndex: Int,
        remoteProgress: List<NuvioWatchProgressDto>
    ) {
        val localItems = watchlistManager.getContinueWatchingSnapshot()
            .filter { (it.timeOffset ?: 0L) > 0L && (it.duration ?: 0L) > 0L }
        if (localItems.isEmpty()) return

        val remoteByKey = remoteProgress.associateBy { dto ->
            val season = dto.season
            val episode = dto.episode
            if (season != null && episode != null) "${dto.contentId}:$season:$episode" else dto.contentId
        }

        val entries = localItems.mapNotNull { meta ->
            val localUpdatedAt = meta.lastWatchedAt ?: return@mapNotNull null
            val locator = meta.lastVideoId?.let { FluxaCoreNative.parseEpisodeLocator(it) }
            val key = if (locator != null) "${meta.id}:${locator.season}:${locator.episode}" else meta.id
            val remote = remoteByKey[key]
            if (remote != null && remote.lastWatched >= localUpdatedAt) return@mapNotNull null
            NuvioSyncRequests.playbackProgress(meta, meta.lastVideoId, meta.timeOffset ?: 0L, meta.duration ?: 0L, localUpdatedAt)
        }
        if (entries.isEmpty()) return

        runCatching {
            val response = nuvioService.pushWatchProgress(
                "Bearer $token",
                mapOf("p_profile_id" to profileIndex, "p_entries" to entries)
            )
            if (!response.isSuccessful) throw IllegalStateException("Nuvio pushWatchProgress failed (${response.code()})")
        }.onFailure { error ->
            Log.w("NuvioImport", "Failed to reconcile local progress to Nuvio", error)
        }
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

    private suspend fun pullAllLibraryItems(token: String, profileIndex: Int): List<NuvioLibraryItemDto> {
        val items = mutableListOf<NuvioLibraryItemDto>()
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

    private suspend fun pullAllWatchedItems(token: String, profileIndex: Int): List<NuvioWatchedItemDto> {
        val items = mutableListOf<NuvioWatchedItemDto>()
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
    }
}

data class NuvioImportResult(val profile: UserProfile, val externalContinueWatching: List<Meta>)

private fun NuvioProfileDto.withResolvedAvatarUrl(avatars: List<NuvioAvatarDto>, supabaseUrl: String): NuvioProfileDto {
    if (!avatarUrl.isNullOrBlank()) return this
    val storagePath = avatarId?.let { id -> avatars.firstOrNull { it.id == id }?.storagePath } ?: return this
    return copy(avatarUrl = "${supabaseUrl.trimEnd('/')}/storage/v1/object/public/avatars/$storagePath")
}

private fun <T> Response<T>.requireBody(): T {
    if (!isSuccessful) throw IllegalStateException("Nuvio request failed (${code()})")
    return body() ?: throw IllegalStateException("Nuvio returned an empty response")
}
