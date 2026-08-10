package com.fluxa.app.data.repository

import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.NuvioLibraryItemDto
import com.fluxa.app.data.remote.NuvioRefreshRequest
import com.fluxa.app.data.remote.NuvioService
import com.fluxa.app.data.remote.NuvioWatchProgressDto
import com.fluxa.app.data.remote.NuvioWatchedItemDto
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.remote.toDto
import com.fluxa.app.domain.discovery.supportsStremioResource
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import kotlinx.coroutines.withTimeoutOrNull
import retrofit2.Response
import javax.inject.Inject

class NuvioSyncCoordinator @Inject constructor(
    private val nuvioService: NuvioService,
    private val profileManager: ProfileManager,
    private val addonRepository: AddonRepository,
    private val deltaSyncEngine: NuvioDeltaSyncEngine,
    private val gson: com.google.gson.Gson,
) {

    suspend fun fetchAddonCount(profile: UserProfile): Int {
        val context = syncContext(profile) ?: return 0
        val effective = nuvioService.resolveEffectiveProfileScopes(
            context.authorization,
            context.profileIndex,
        )
        return nuvioService.pullAddons(
            authorization = context.authorization,
            profileId = "eq.${effective.addons}",
        ).bodyOrNull().orEmpty().size
    }

    suspend fun fetchLibrary(profile: UserProfile): List<Meta> {
        val context = syncContext(profile) ?: return emptyList()
        return deltaSyncEngine.syncLibrary(
            context.authorization,
            context.scope,
            context.profileIndex,
        ).map { dto -> dto.toLibraryMeta() }
    }

    /**
     * Nuvio is the source of truth for progress. Raw progress remains keyed by
     * `progress_key`; only the final Continue Watching projection collapses a
     * series to its newest episode.
     */
    suspend fun fetchContinueWatching(profile: UserProfile): List<Meta> = coroutineScope {
        val context = syncContext(profile) ?: return@coroutineScope emptyList()

        val libraryRequest = async {
            deltaSyncEngine.syncLibrary(
                context.authorization,
                context.scope,
                context.profileIndex,
            )
        }
        val progressRequest = async {
            deltaSyncEngine.syncProgress(
                context.authorization,
                context.scope,
                context.profileIndex,
            )
        }

        val library = libraryRequest.await().associateBy { it.nuvioLibraryIdentity() }
        val progress = progressRequest.await().continueWatching
        if (progress.isEmpty()) return@coroutineScope emptyList()

        val needsMetadata = NuvioCoreBridge.progressMetaNeeds(
            gson.toJsonTree(progress),
            gson.toJsonTree(library.values),
        ).mapNotNull { need ->
            need.asJsonObject.get("progressKey")?.asString
                ?.let { key -> progress.firstOrNull { it.canonicalProgressKey() == key } }
        }
        val metadataByProgressKey = if (needsMetadata.isEmpty()) {
            emptyMap()
        } else {
            resolveNuvioProgressMetadata(
                authorization = context.authorization,
                profileIndex = context.profileIndex,
                progress = needsMetadata,
                library = library,
            )
        }

        progress.mapNotNull { dto ->
            dto.toContinueWatchingMeta(
                libraryItem = library[dto.nuvioLibraryIdentity()],
                metadataDetail = metadataByProgressKey[dto.canonicalProgressKey()],
            )
        }
    }

    /** Recent Nuvio watch-history projection for provider library UI. */
    suspend fun fetchWatched(profile: UserProfile): List<Meta> {
        val context = syncContext(profile) ?: return emptyList()
        val history = deltaSyncEngine.syncHistory(
            context.authorization,
            context.scope,
            context.profileIndex,
        )
        val latestByContent = LinkedHashMap<String, NuvioWatchedItemDto>()
        history.sortedByDescending { it.watchedAt ?: Long.MIN_VALUE }.forEach { item ->
            val key = "${item.contentType.trim().lowercase()}:${item.contentId.trim()}"
            latestByContent.putIfAbsent(key, item)
        }
        return latestByContent.values.map { item ->
            val episodeCode = if (item.season != null && item.episode != null) {
                "S${item.season} E${item.episode}"
            } else null
            Meta(
                id = item.contentId,
                name = item.title?.takeIf(String::isNotBlank) ?: item.contentId,
                type = item.contentType,
                lastEpisodeName = episodeCode,
                lastWatchedAt = item.watchedAt,
                reason = ThirdPartyProviderId.NUVIO.reasonLabel,
            )
        }
    }

    suspend fun fetchWatchedEpisodeTimestamps(profile: UserProfile): Map<String, Long> {
        val context = syncContext(profile) ?: return emptyMap()
        return deltaSyncEngine.syncHistory(
            context.authorization,
            context.scope,
            context.profileIndex,
        ).mapNotNull { item ->
            val season = item.season ?: return@mapNotNull null
            val episode = item.episode ?: return@mapNotNull null
            val watchedAt = item.watchedAt ?: return@mapNotNull null
            "${item.contentId}:$season:$episode" to watchedAt
        }.toMap()
    }

    /**
     * Resolve metadata without probing unknown add-ons. A library item's
     * `addon_base_url` is a trusted metadata source and is tried directly. Any
     * fallback add-ons must already have a cached manifest that declares `meta`;
     * cache misses are skipped, so subtitle/stream-only add-ons receive no
     * manifest or /meta request from this Continue Watching path.
     */
    private suspend fun resolveNuvioProgressMetadata(
        authorization: String,
        profileIndex: Int,
        progress: List<NuvioWatchProgressDto>,
        library: Map<String, NuvioLibraryItemDto>,
    ): Map<String, MetaDetail> = coroutineScope {
        val effective = nuvioService.resolveEffectiveProfileScopes(authorization, profileIndex)
        val enabledAddons = nuvioService.pullAddons(
            authorization = authorization,
            profileId = "eq.${effective.addons}",
        ).bodyOrNull().orEmpty()
            .filter { it.enabled && it.url.isNotBlank() }
            .sortedBy { it.sortOrder }
        if (enabledAddons.isEmpty()) return@coroutineScope emptyMap()

        val cachedManifestSemaphore = Semaphore(4)
        val cachedMetadataAddons = enabledAddons.map { addon ->
            async {
                cachedManifestSemaphore.withPermit {
                    addonRepository.getCachedAddonManifest(addon.url)
                }
            }
        }.awaitAll()
            .filterNotNull()
            .filter { descriptor -> descriptor.supportsStremioResource("meta") }

        val semaphore = Semaphore(4)
        progress
            .distinctBy { it.canonicalProgressKey() }
            .map { entry ->
                async {
                    semaphore.withPermit {
                        val preferredUrl = library[entry.nuvioLibraryIdentity()]?.addonBaseUrl
                            ?.takeIf(String::isNotBlank)
                            ?.let { source ->
                                enabledAddons.firstOrNull { sameAddonTransport(it.url, source) }?.url
                            }
                        val preferredDetail = preferredUrl?.let { source ->
                            resolveFromKnownMetadataAddon(entry, source)
                        }
                        val detail = preferredDetail ?: resolveFromCachedMetadataAddons(
                            entry,
                            cachedMetadataAddons.filterNot { descriptor ->
                                preferredUrl != null && sameAddonTransport(descriptor.transportUrl, preferredUrl)
                            },
                        )
                        detail?.let { entry.canonicalProgressKey() to it }
                    }
                }
            }
            .awaitAll()
            .filterNotNull()
            .toMap()
    }

    private suspend fun resolveFromKnownMetadataAddon(
        progress: NuvioWatchProgressDto,
        transportUrl: String,
    ): MetaDetail? {
        val candidateTypes = progress.candidateMetaTypes()
        if (candidateTypes.isEmpty()) return null
        return withTimeoutOrNull(2_500L) {
            addonRepository.getMetaDetailFromSpecificAddon(
                transportUrl = transportUrl,
                type = candidateTypes.first(),
                id = progress.contentId,
                alternateTypes = candidateTypes.drop(1),
            )
        }
    }

    private suspend fun resolveFromCachedMetadataAddons(
        progress: NuvioWatchProgressDto,
        addons: List<AddonDescriptor>,
    ): MetaDetail? {
        val candidateTypes = progress.candidateMetaTypes()
        for (addon in addons) {
            val supportedTypes = candidateTypes.filter { type ->
                addon.supportsStremioResource("meta", type, progress.contentId)
            }
            if (supportedTypes.isEmpty()) continue
            val detail = withTimeoutOrNull(2_500L) {
                addonRepository.getMetaDetailFromSpecificAddon(
                    transportUrl = addon.transportUrl,
                    type = supportedTypes.first(),
                    id = progress.contentId,
                    alternateTypes = supportedTypes.drop(1),
                )
            }
            if (detail != null) return detail
        }
        return null
    }

    suspend fun isHealthy(): Boolean = runCatching {
        nuvioService.healthCheck().bodyOrNull()?.status?.lowercase() in setOf("healthy", "ok")
    }.getOrDefault(false)

    /** Uses the v1.2 incremental library mutation endpoints; never legacy full replace. */
    suspend fun pushWatchlist(profile: UserProfile, meta: Meta, isInWatchlist: Boolean): Boolean {
        val context = syncContext(profile) ?: return false
        if (isInWatchlist) {
            val item = NuvioSyncRequests.libraryItem(meta, System.currentTimeMillis())
                .toMutableMap()
                .apply {
                    this["content_id"] = meta.id.trim()
                    this["content_type"] = canonicalNuvioContentType(meta.type)
                }
            nuvioService.pushLibraryItems(
                context.authorization,
                mapOf(
                    "p_profile_id" to context.profileIndex,
                    "p_items" to listOf(item),
                ),
            ).requireSuccess()
        } else {
            nuvioService.deleteLibraryItems(
                context.authorization,
                mapOf(
                    "p_profile_id" to context.profileIndex,
                    "p_keys" to listOf(
                        mapOf(
                            "content_id" to meta.id,
                            "content_type" to canonicalNuvioContentType(meta.type),
                        ),
                    ),
                ),
            ).requireSuccess()
        }
        deltaSyncEngine.invalidate(context.scope, "library")
        stampSync(context.profile)
        return true
    }

    suspend fun pushWatched(
        profile: UserProfile,
        meta: Meta,
        episodes: List<Video>,
        watched: Boolean,
    ): Boolean {
        val context = syncContext(profile) ?: return false
        val items = NuvioSyncRequests.watchedItems(meta, episodes, System.currentTimeMillis())
            .map { item ->
                item.toMutableMap().apply {
                    this["content_id"] = meta.id.trim()
                    this["content_type"] = canonicalNuvioContentType(meta.type)
                }
            }
        if (items.isEmpty()) return false
        val keys = items.map { item ->
            mapOf(
                "content_id" to item["content_id"],
                "season" to item["season"],
                "episode" to item["episode"],
            )
        }
        if (watched) {
            nuvioService.pushWatchedItems(
                context.authorization,
                mapOf("p_profile_id" to context.profileIndex, "p_items" to items),
            ).requireSuccess()
        } else {
            nuvioService.deleteWatchedItems(
                context.authorization,
                mapOf("p_profile_id" to context.profileIndex, "p_keys" to keys),
            ).requireSuccess()
        }
        deltaSyncEngine.invalidate(context.scope, "history")
        stampSync(context.profile)
        return true
    }

    suspend fun clearPlaybackProgress(profile: UserProfile, meta: Meta): Boolean {
        val context = syncContext(profile) ?: return false
        val progressKey = meta.nuvioProgressKey()
        nuvioService.deleteWatchProgress(
            context.authorization,
            mapOf(
                "p_profile_id" to context.profileIndex,
                "p_progress_key" to progressKey,
            ),
        ).requireSuccess()
        deltaSyncEngine.invalidate(context.scope, "progress")
        stampSync(context.profile)
        return true
    }

    suspend fun pushPlaybackProgress(
        profile: UserProfile,
        meta: Meta,
        videoId: String?,
        position: Long,
        duration: Long,
    ) {
        if (duration <= 0L) return
        val context = syncContext(profile) ?: return
        val entry = NuvioSyncRequests.playbackProgress(
            meta,
            videoId,
            position,
            duration,
            System.currentTimeMillis(),
        ).toMutableMap()
        // Nuvio v1.2 computes progress_key server-side. Ensure the canonical
        // content identity plus S/E fields are present and do not send a stale key.
        entry.remove("progress_key")
        entry["content_id"] = meta.id.trim()
        entry["content_type"] = canonicalNuvioContentType(meta.type)
        val locator = meta.episodeLocator(videoId)
        locator?.let { (season, episode) ->
            entry["season"] = season
            entry["episode"] = episode
        }
        val resolvedVideoId = videoId?.trim()?.takeIf(String::isNotBlank)
            ?: if (canonicalNuvioContentType(meta.type) == "movie") {
                meta.id.trim()
            } else {
                locator?.let { (season, episode) -> "${meta.id.trim()}:$season:$episode" }
            }
            ?: return
        entry["video_id"] = resolvedVideoId
        nuvioService.pushWatchProgress(
            context.authorization,
            mapOf(
                "p_profile_id" to context.profileIndex,
                "p_entries" to listOf(entry),
            ),
        ).requireSuccess()
        deltaSyncEngine.invalidate(context.scope, "progress")
        // Nuvio may auto-upsert watched history when progress crosses completion.
        deltaSyncEngine.invalidate(context.scope, "history")
        stampSync(context.profile)
    }

    private suspend fun syncContext(profile: UserProfile): NuvioContext? {
        val current = freshProfile(profile) ?: return null
        val token = current.nuvioAccessToken?.takeIf(String::isNotBlank) ?: return null
        val index = current.nuvioProfileIndex ?: return null
        val scope = current.nuvioSyncScope() ?: return null
        return NuvioContext(
            profile = current,
            authorization = "Bearer $token",
            profileIndex = index,
            scope = scope,
        )
    }

    private fun stampSync(profile: UserProfile) {
        profileManager.updateProfile(profile.id) { it.copy(nuvioLastSyncAt = System.currentTimeMillis()) }
    }

    private suspend fun freshProfile(profile: UserProfile): UserProfile? {
        val accessToken = profile.nuvioAccessToken?.takeIf { it.isNotBlank() } ?: return null
        val expiresAt = profile.nuvioTokenExpiresAt ?: 0L
        if (expiresAt > System.currentTimeMillis() + 60_000L) return profile
        val refreshToken = profile.nuvioRefreshToken?.takeIf { it.isNotBlank() } ?: return profile
        val session = nuvioService.refreshToken(
            request = NuvioRefreshRequest(refreshToken).toDto(),
        ).bodyOrNull()?.toDomain() ?: throw IllegalStateException("Nuvio token refresh failed")
        return profile.copy(
            nuvioAccessToken = session.accessToken.ifBlank { accessToken },
            nuvioRefreshToken = session.refreshToken.ifBlank { refreshToken },
            nuvioTokenExpiresAt = session.expiresIn?.let { System.currentTimeMillis() + it * 1000L },
            nuvioUserId = session.user?.id ?: profile.nuvioUserId,
            nuvioEmail = session.user?.email ?: profile.nuvioEmail,
        ).also(profileManager::saveProfile)
    }
}

private data class NuvioContext(
    val profile: UserProfile,
    val authorization: String,
    val profileIndex: Int,
    val scope: String,
)

private fun NuvioLibraryItemDto.toLibraryMeta(): Meta {
    val item = toDomain()
    return Meta(
        id = item.contentId,
        name = item.name,
        type = item.contentType,
        poster = item.poster,
        background = item.background,
        description = item.description,
        releaseInfo = item.releaseInfo,
        imdbRating = item.imdbRating?.toString(),
        genres = item.genres,
        reason = ThirdPartyProviderId.NUVIO.reasonLabel,
    )
}

private fun NuvioWatchProgressDto.candidateMetaTypes(): List<String> {
    val alternateTypes = when (contentType.trim().lowercase()) {
        "series", "show", "tv", "anime" -> listOf("series", "show", "tv", "anime")
        "movie", "film" -> listOf("movie")
        else -> emptyList()
    }
    return (listOf(contentType) + alternateTypes)
        .map(String::trim)
        .filter(String::isNotBlank)
        .distinct()
}

private fun sameAddonTransport(first: String, second: String): Boolean =
    normalizedAddonIdentity(first) == normalizedAddonIdentity(second)

private fun normalizedAddonIdentity(value: String): String = value.trim()
    .trimEnd('/')
    .removeSuffix("/manifest.json")
    .trimEnd('/')
    .lowercase()

private fun canonicalNuvioContentType(type: String): String = when (type.trim().lowercase()) {
    "series", "show", "tv", "anime" -> "series"
    else -> "movie"
}

private fun NuvioLibraryItemDto.nuvioLibraryIdentity(): String =
    "${canonicalNuvioContentType(contentType)}:${contentId.trim()}"

private fun NuvioWatchProgressDto.nuvioLibraryIdentity(): String =
    "${canonicalNuvioContentType(contentType)}:${contentId.trim()}"

private fun Meta.episodeLocator(videoId: String?): Pair<Int, Int>? {
    val fromId = videoId?.split(':')?.let { parts ->
        if (parts.size < 3) null else {
            val season = parts[parts.lastIndex - 1].toIntOrNull()
            val episode = parts.last().toIntOrNull()
            if (season != null && episode != null) season to episode else null
        }
    }
    if (fromId != null) return fromId
    val label = lastEpisodeName ?: return null
    val match = Regex("(?i)\\bS(\\d+)\\s*E(\\d+)\\b").find(label) ?: return null
    return match.groupValues[1].toIntOrNull()?.let { season ->
        match.groupValues[2].toIntOrNull()?.let { episode -> season to episode }
    }
}

private fun Meta.nuvioProgressKey(): String {
    val contentId = id.trim()
    val locator = episodeLocator(lastVideoId)
    return if (canonicalNuvioContentType(type) == "series" && locator != null) {
        "${contentId}_s${locator.first}e${locator.second}"
    } else {
        contentId
    }
}

private fun <T> Response<T>.bodyOrNull(): T? = if (isSuccessful) body() else null

private fun Response<*>.requireSuccess() {
    if (!isSuccessful) throw IllegalStateException("Nuvio request failed (${code()})")
}
