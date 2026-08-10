package com.fluxa.app.data.repository

import com.fluxa.app.common.PlatformLog
import com.fluxa.app.data.local.LibraryCatalogSource
import com.fluxa.app.data.local.LibraryRemoteSource
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ProviderDataOwner
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.providerDataOwner
import com.fluxa.app.data.local.withProviderLastSyncAt
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.repository.library.ProviderDataStore
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.NuvioAvatarDto
import com.fluxa.app.data.remote.NuvioCollection
import com.fluxa.app.data.remote.NuvioCollectionFolder
import com.fluxa.app.data.remote.NuvioCollectionFolderSource
import com.fluxa.app.data.remote.NuvioCredentials
import com.fluxa.app.data.remote.NuvioProfileDto
import com.fluxa.app.data.remote.NuvioPluginDto
import com.fluxa.app.data.remote.NuvioService
import com.fluxa.app.data.remote.NuvioSession
import com.fluxa.app.data.remote.NuvioSessionDto
import com.fluxa.app.data.remote.toDto
import com.fluxa.app.core.rust.FluxaCoreNative
import com.google.gson.Gson
import com.google.gson.JsonArray
import com.google.gson.JsonObject
import retrofit2.Response

enum class NuvioImportStep { PROFILE, ADDONS, PLUGINS, LIBRARY, PROGRESS, HISTORY, COLLECTIONS }

class NuvioAccountImportCoordinator(
    private val nuvioService: NuvioService,
    private val profileManager: ProfileManager,
    private val watchlistManager: WatchlistManager,
    private val providerDataStore: ProviderDataStore,
    private val deltaSyncEngine: NuvioDeltaSyncEngine,
    private val supabaseUrl: String,
    private val gson: Gson
) {
    suspend fun refreshProfileIfNeeded(profile: UserProfile): UserProfile {
        val refreshToken = profile.nuvioRefreshToken?.takeIf { it.isNotBlank() } ?: return profile
        val expiresAt = profile.nuvioTokenExpiresAt ?: 0L
        if (!profile.nuvioAccessToken.isNullOrBlank() && expiresAt > System.currentTimeMillis() + 60_000L) return profile
        val expectedOwner = profile.providerDataOwner(ThirdPartyProviderId.NUVIO)
        val session = nuvioService.refreshToken(request = com.fluxa.app.data.remote.NuvioRefreshRequest(refreshToken).toDto()).requireBody().toDomain()
        val currentProfile = profileManager.getProfiles().firstOrNull { it.id == profile.id }
        if (expectedOwner != null && currentProfile?.providerDataOwner(ThirdPartyProviderId.NUVIO) != expectedOwner) {
            throw IllegalStateException("Nuvio account changed or disconnected during token refresh")
        }
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

    /** Pulls the plugin snapshot for the active Nuvio profile, honoring uses_primary_plugins. */
    suspend fun pullPluginsForProfile(profile: UserProfile): List<NuvioPluginDto> {
        val refreshedProfile = refreshProfileIfNeeded(profile)
        val accessToken = refreshedProfile.nuvioAccessToken?.takeIf { it.isNotBlank() }
            ?: return emptyList()
        val authorization = "Bearer $accessToken"
        val profiles = runCatching {
            nuvioService.pullProfiles(authorization).requireBody()
        }.getOrDefault(emptyList())
        val requestedIndex = refreshedProfile.nuvioProfileIndex
        val profileIndex = when {
            requestedIndex != null && profiles.isEmpty() -> requestedIndex
            requestedIndex != null && profiles.any { it.profileIndex == requestedIndex } -> requestedIndex
            else -> profiles.minByOrNull { it.profileIndex }?.profileIndex ?: requestedIndex ?: 1
        }
        val effectiveScopes = nuvioService.resolveEffectiveProfileScopes(
            authorization = authorization,
            profileIndex = profileIndex,
            knownProfiles = profiles,
        )
        return nuvioService.pullPlugins(
            authorization = authorization,
            profileId = "eq.${effectiveScopes.plugins}",
        ).requireBody()
    }

    suspend fun signIn(email: String, password: String): Result<NuvioSession> = authenticate {
        nuvioService.signIn(credentials = NuvioCredentials(email, password))
    }

    suspend fun signUp(email: String, password: String): Result<NuvioSession> = authenticate {
        nuvioService.signUp(NuvioCredentials(email, password))
    }

    suspend fun sync(
        profile: UserProfile,
        onStep: (NuvioImportStep) -> Unit,
        onItemProgress: (index: Int, total: Int, title: String) -> Unit = { _, _, _ -> }
    ): NuvioImportResult {
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
        return import(refreshedProfile, session, onStep, onItemProgress)
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
        onStep: (NuvioImportStep) -> Unit,
        onItemProgress: (index: Int, total: Int, title: String) -> Unit = { _, _, _ -> }
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
        profileManager.saveProfile(connectedProfile)
        profileManager.setLastActiveProfile(connectedProfile)
        watchlistManager.setActiveProfile(baseProfile.id)

        val profileDtos = importOrDefault(NuvioImportStep.PROFILE, emptyList()) {
            nuvioService.pullProfiles(token).requireBody()
        }
        val avatarDtos = importOrDefault(NuvioImportStep.PROFILE, emptyList()) {
            nuvioService.listAvatars().requireBody()
        }
        val requestedProfileIndex = connectedProfile.nuvioProfileIndex
        val primaryIndex = when {
            requestedProfileIndex != null && profileDtos.isEmpty() -> requestedProfileIndex
            requestedProfileIndex != null && profileDtos.any { it.profileIndex == requestedProfileIndex } -> requestedProfileIndex
            else -> profileDtos.minByOrNull { it.profileIndex }?.profileIndex ?: requestedProfileIndex ?: 1
        }
        val effectiveScopes = nuvioService.resolveEffectiveProfileScopes(
            authorization = token,
            profileIndex = primaryIndex,
            knownProfiles = profileDtos,
        )

        var profile = mergeProfiles(baseProfile, connectedProfile, profileDtos, avatarDtos, primaryIndex)
        profileManager.setLastActiveProfile(profile)
        watchlistManager.setActiveProfile(profile.id)
        val providerOwner = profile.providerDataOwner(ThirdPartyProviderId.NUVIO)
            ?: throw IllegalStateException("Nuvio account identity is unavailable")
        val providerWriteLease = providerDataStore.lease(providerOwner)
        ensureNuvioOwnerStillConnected(profile.id, providerOwner)
        onStep(NuvioImportStep.PROFILE)

        val addonDtos = try {
            nuvioService.pullAddons(token, profileId = "eq.${effectiveScopes.addons}").requireBody()
        } catch (error: Exception) {
            PlatformLog.w("NuvioImport", "Import step ${NuvioImportStep.ADDONS} failed; continuing without it", error)
            null
        }
        if (addonDtos != null) {
            addonDtos.forEachIndexed { index, addon ->
                onItemProgress(index + 1, addonDtos.size, addon.name ?: addon.url)
            }
            // Provider add-ons remain a Nuvio snapshot. They are deliberately not
            // copied into Fluxa's localAddons/disabledLocalAddons lists.
        }
        onStep(NuvioImportStep.ADDONS)

        val plugins = try {
            nuvioService.pullPlugins(token, profileId = "eq.${effectiveScopes.plugins}").requireBody()
        } catch (error: Exception) {
            PlatformLog.w("NuvioImport", "Import step ${NuvioImportStep.PLUGINS} failed; continuing without it", error)
            emptyList<NuvioPluginDto>()
        }
        onStep(NuvioImportStep.PLUGINS)

        val syncScope = profile.nuvioSyncScope()
            ?: throw IllegalStateException("Nuvio sync scope is unavailable")
        val libraryItems = try {
            deltaSyncEngine.syncLibrary(token, syncScope, primaryIndex).also { items ->
                items.forEachIndexed { index, item ->
                    onItemProgress(index + 1, items.size, item.name)
                }
            }
        } catch (error: Exception) {
            PlatformLog.w("NuvioImport", "Import step ${NuvioImportStep.LIBRARY} failed; continuing without it", error)
            null
        }
        val libraryJson = gson.toJsonTree(libraryItems.orEmpty()).asJsonArray
        // The library is consumed through NuvioProviderAdapter. It is not
        // mirrored into the local profile or local watchlist.
        onStep(NuvioImportStep.LIBRARY)

        val progressSync = try {
            deltaSyncEngine.syncProgress(token, syncScope, primaryIndex)
        } catch (error: Exception) {
            PlatformLog.w("NuvioImport", "Import step ${NuvioImportStep.PROGRESS} failed; keeping existing playback progress", error)
            null
        }
        val watchedItemDtos = try {
            deltaSyncEngine.syncHistory(token, syncScope, primaryIndex)
        } catch (error: Exception) {
            PlatformLog.w("NuvioImport", "Import step ${NuvioImportStep.HISTORY} failed; keeping existing watched episodes", error)
            null
        }

        if (progressSync != null) {
            val watchProgressJson = gson.toJsonTree(progressSync.progress).asJsonArray
            // Nuvio is authoritative here. Never enrich its progress with metadata add-ons.
            val addonMetas = JsonObject()
            val mergePlan = NuvioCoreBridge.importMergePlan(
                libraryJson,
                addonMetas,
                watchProgressJson,
                gson.toJsonTree(watchedItemDtos.orEmpty()).asJsonArray
            )
            val libraryByContentId = libraryItems.orEmpty().associateBy { it.contentId }
            val progressItems = FluxaCoreNative.mergeContinueWatchingDuplicates(progressSync.continueWatching
                .mapNotNull { dto -> dto.toContinueWatchingMeta(libraryByContentId[dto.contentId]) }
            )
            ensureNuvioOwnerStillConnected(profile.id, providerOwner)
            check(providerDataStore.replaceContinueWatching(providerWriteLease, progressItems)) {
                "Nuvio account changed or disconnected while progress was syncing"
            }

            val watchedBySeries = mutableMapOf<String, MutableSet<String>>()
            mergePlan.getAsJsonObject("watched").entrySet().forEach { (key, value) ->
                if (!value.asBoolean) return@forEach
                val (seriesId, normalizedVideoId) = providerEpisodeSeriesId(key) ?: return@forEach
                watchedBySeries.getOrPut(seriesId, ::mutableSetOf).add(normalizedVideoId)
            }
            ensureNuvioOwnerStillConnected(profile.id, providerOwner)
            check(providerDataStore.replaceWatchedEpisodes(providerWriteLease, watchedBySeries)) {
                "Nuvio account changed or disconnected while history was syncing"
            }
        }
        onStep(NuvioImportStep.PROGRESS)
        onStep(NuvioImportStep.HISTORY)

        val collectionRows = importOrDefault(NuvioImportStep.COLLECTIONS, emptyList()) {
            nuvioService.pullCollections(token, mapOf("p_profile_id" to primaryIndex)).requireBody()
        }
        val flatCollections = collectionRows.flatMap { row ->
            row.collectionsJson.orEmpty().map { it.toDomain() }
        }
        val importedCollections = flatCollections.mapIndexed { index, collection ->
            collection.toLibraryUserCollection(primaryIndex, index)
        }
        onStep(NuvioImportStep.COLLECTIONS)

        val syncedAt = System.currentTimeMillis()
        profile = profile
            .copy(nuvioLastSyncAt = syncedAt)
            .withProviderLastSyncAt(ThirdPartyProviderId.NUVIO, syncedAt)
        val latestProfile = profileManager.getProfiles().firstOrNull { it.id == profile.id }
            ?: throw IllegalStateException("Fluxa profile was removed during Nuvio sync")
        ensureNuvioOwnerStillConnected(profile.id, providerOwner, latestProfile)
        // Collections are profile-owned data. Nuvio collections are imported into the
        // same profile collection store and merged with collections created locally in
        // Fluxa, so selecting Simkl/Trakt/etc. as the library source cannot hide them.
        val mergedCollections = mergeProfileCollections(
            existing = latestProfile.safeLibraryCollections,
            imported = importedCollections,
        )
        val finalProfile = latestProfile.copy(
            email = profile.email,
            nuvioAccessToken = profile.nuvioAccessToken,
            nuvioRefreshToken = profile.nuvioRefreshToken,
            nuvioTokenExpiresAt = profile.nuvioTokenExpiresAt,
            nuvioUserId = profile.nuvioUserId,
            nuvioEmail = profile.nuvioEmail,
            nuvioProfileIndex = profile.nuvioProfileIndex,
            nuvioLastSyncAt = profile.nuvioLastSyncAt,
            providerSyncTimestamps = profile.providerSyncTimestamps,
            libraryCollections = mergedCollections,
        )
        profileManager.saveProfile(finalProfile)
        profileManager.setLastActiveProfile(finalProfile)
        profileManager.clearExternalSyncFailure(finalProfile.id, "nuvio")
        return NuvioImportResult(
            profile = finalProfile,
            externalContinueWatching = finalProfile.providerDataOwner(ThirdPartyProviderId.NUVIO)
                ?.let { providerDataStore.getContinueWatching(it) }
                .orEmpty(),
            plugins = plugins
        )
    }

    private fun ensureNuvioOwnerStillConnected(
        profileId: String,
        expectedOwner: ProviderDataOwner,
        currentProfile: UserProfile? = profileManager.getProfiles().firstOrNull { it.id == profileId }
    ) {
        val currentOwner = currentProfile?.providerDataOwner(ThirdPartyProviderId.NUVIO)
        if (currentOwner != expectedOwner || currentProfile?.nuvioAccessToken.isNullOrBlank()) {
            throw IllegalStateException("Nuvio account changed or disconnected during sync")
        }
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

    private suspend fun <T> importOrDefault(
        step: NuvioImportStep,
        defaultValue: T,
        block: suspend () -> T,
    ): T = try {
        block()
    } catch (error: Exception) {
        PlatformLog.w(
            "NuvioImport",
            "Import step $step failed; continuing with the existing/default value",
            error,
        )
        defaultValue
    }

}


private fun mergeProfileCollections(
    existing: List<LibraryUserCollection>,
    imported: List<LibraryUserCollection>,
): List<LibraryUserCollection> {
    if (imported.isEmpty()) return existing
    val merged = LinkedHashMap<String, LibraryUserCollection>(existing.size + imported.size)
    existing.forEach { collection -> merged[collection.id] = collection }
    imported.forEach { collection -> merged[collection.id] = collection }
    return merged.values.toList()
}

private fun NuvioCollection.toLibraryUserCollection(profileIndex: Int, index: Int): LibraryUserCollection {
    val resolvedTitle = title?.trim().takeUnless { it.isNullOrBlank() } ?: "Collection ${index + 1}"
    val resolvedId = id?.trim().takeUnless { it.isNullOrBlank() }
        ?: "nuvio_${profileIndex}_${resolvedTitle.lowercase().replace(Regex("[^a-z0-9]+"), "_").trim('_').ifBlank { index.toString() }}"
    return LibraryUserCollection(
        id = resolvedId,
        title = resolvedTitle,
        imageUrl = backdropImageUrl,
        showOnHome = showOnHome ?: false,
        folders = folders.orEmpty().mapIndexed { folderIndex, folder ->
            folder.toLibraryUserCollectionFolder(resolvedId, folderIndex)
        },
        showAllTab = showAllTab ?: true,
        viewMode = viewMode ?: "FOLLOW_LAYOUT",
        pinToTop = pinToTop ?: false,
        focusGlowEnabled = focusGlowEnabled ?: true,
        community = community,
    )
}

private fun NuvioCollectionFolder.toLibraryUserCollectionFolder(
    collectionId: String,
    index: Int,
): LibraryUserCollectionFolder {
    val resolvedTitle = title?.trim().takeUnless { it.isNullOrBlank() } ?: "Folder ${index + 1}"
    val resolvedId = id?.trim().takeUnless { it.isNullOrBlank() }
        ?: "${collectionId}_folder_$index"
    val sourceRows = catalogSources.orEmpty()
    val addonSources = sourceRows.mapNotNull(NuvioCollectionFolderSource::toLibraryCatalogSourceOrNull)
    val remoteSources = sourceRows.mapNotNull(NuvioCollectionFolderSource::toLibraryRemoteSourceOrNull)
    return LibraryUserCollectionFolder(
        id = resolvedId,
        title = resolvedTitle,
        imageUrl = coverImageUrl,
        shape = tileShape ?: "poster",
        hideTitle = hideTitle ?: false,
        focusGifEnabled = focusGifEnabled ?: true,
        catalogSources = addonSources,
        sources = remoteSources,
        coverEmoji = coverEmoji,
        coverImageUrl = coverImageUrl,
        focusGifUrl = focusGifUrl,
        titleLogoUrl = titleLogoUrl,
        heroBackdropUrl = heroBackdropUrl,
        heroVideoUrl = heroVideoUrl,
    )
}

private fun NuvioCollectionFolderSource.toLibraryCatalogSourceOrNull(): LibraryCatalogSource? {
    val resolvedCatalogId = catalogId?.trim().takeUnless { it.isNullOrBlank() } ?: return null
    val resolvedType = type?.trim().takeUnless { it.isNullOrBlank() } ?: return null
    return LibraryCatalogSource(
        addonId = addonId?.takeIf(String::isNotBlank),
        catalogId = resolvedCatalogId,
        type = resolvedType,
        genre = genre?.takeIf(String::isNotBlank),
        displayName = title?.takeIf(String::isNotBlank),
    )
}

private fun NuvioCollectionFolderSource.toLibraryRemoteSourceOrNull(): LibraryRemoteSource? {
    val resolvedProvider = provider
        ?.trim()
        ?.lowercase()
        ?.takeIf { it == "trakt" || it == "tmdb" }
        ?: when {
            traktListId != null -> "trakt"
            tmdbId != null -> "tmdb"
            else -> return null
        }
    return LibraryRemoteSource(
        provider = resolvedProvider,
        title = title,
        mediaType = mediaType,
        traktListId = traktListId,
        tmdbSourceType = tmdbSourceType,
        tmdbId = tmdbId,
        sortBy = sortBy,
        sortHow = sortHow,
        filters = filters,
        addonId = addonId,
        catalogId = catalogId,
        type = type,
        genre = genre,
    )
}

data class NuvioImportResult(
    val profile: UserProfile,
    val externalContinueWatching: List<Meta>,
    val plugins: List<NuvioPluginDto> = emptyList()
)

private fun NuvioProfileDto.withResolvedAvatarUrl(avatars: List<NuvioAvatarDto>, supabaseUrl: String): NuvioProfileDto {
    if (!avatarUrl.isNullOrBlank()) return this
    val storagePath = avatarId?.let { id -> avatars.firstOrNull { it.id == id }?.storagePath } ?: return this
    return copy(avatarUrl = "${supabaseUrl.trimEnd('/')}/storage/v1/object/public/avatars/$storagePath")
}

private fun <T> Response<T>.requireBody(): T {
    if (!isSuccessful) throw IllegalStateException("Nuvio request failed (${code()})")
    return body() ?: throw IllegalStateException("Nuvio returned an empty response")
}


private fun providerEpisodeSeriesId(videoId: String): Pair<String, String>? {
    val parts = videoId.split(':')
    if (parts.size < 3) return null
    val season = parts[parts.lastIndex - 1].toIntOrNull() ?: return null
    val episode = parts.last().toIntOrNull() ?: return null
    val seriesId = parts.dropLast(2).joinToString(":").takeIf(String::isNotBlank) ?: return null
    return seriesId to "$seriesId:$season:$episode"
}
