package com.fluxa.app.data.repository.library

import com.fluxa.app.common.PlatformLog
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ProviderDataOwner
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.providerAccountId
import com.fluxa.app.data.local.isProviderConnected
import com.fluxa.app.data.local.providerDataOwner
import com.fluxa.app.data.local.withProviderLastSyncAt
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.TraktIntegration
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Inject
import javax.inject.Singleton
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * One provider/account snapshot. Every list in this object belongs to exactly
 * one remote account. It is never merged with Fluxa-local state or with a
 * different provider.
 */
data class ThirdPartyProviderSnapshot(
    val providerId: ThirdPartyProviderId,
    val accountId: String,
    val planned: List<Meta> = emptyList(),
    val watching: List<Meta> = emptyList(),
    val completed: List<Meta> = emptyList(),
    val continueWatching: List<Meta> = emptyList(),
    val favorites: List<Meta> = emptyList(),
    val collection: List<Meta> = emptyList(),
    val watchedEpisodeIdsBySeries: Map<String, Set<String>> = emptyMap(),
    val addonCount: Int = 0,
    val syncedAt: Long = 0L,
    val fromCache: Boolean = false
) {
    val libraryItems: List<Meta>
        get() = providerDistinct(planned + watching + completed)

    val libraryCount: Int
        get() = libraryItems.size

    val itemCount: Int
        get() = providerDistinct(
            libraryItems + continueWatching + favorites + collection
        ).size
}

/**
 * Common Android/Desktop boundary for third-party data.
 *
 * Cache identity is `(Fluxa profile, provider, remote account)`. A provider
 * connection never grants access to another provider's cache, and disconnect
 * only clears the matching namespace.
 */
@Singleton
class ThirdPartyProviderRepository @Inject constructor(
    private val adapters: ProviderAdapters,
    private val continueWatchingRepository: ProviderContinueWatchingRepository,
    private val store: ProviderDataStore,
    private val profileManager: ProfileManager
) {
    private val snapshots = ConcurrentHashMap<String, ThirdPartyProviderSnapshot>()
    private val scopeEpochs = ConcurrentHashMap<String, Long>()
    private val scopeLocks = ConcurrentHashMap<String, Mutex>()

    suspend fun loadSelectedLibrary(
        profile: UserProfile,
        refresh: Boolean = true
    ): ThirdPartyProviderSnapshot? {
        val providerId = ThirdPartyProviderId.from(profile.integrationLibrarySource) ?: return null
        return load(profile, providerId, refresh)
    }

    suspend fun load(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        refresh: Boolean = true
    ): ThirdPartyProviderSnapshot? {
        val adapter = adapters.byId(providerId.key) ?: return null
        val owner = adapter.dataOwner(profile) ?: return null
        val key = owner.cacheKey()
        val expectedEpoch = currentEpoch(profile.id, providerId)
        val writeLease = store.lease(owner)

        if (!adapter.isConnected(profile)) {
            snapshots.remove(key)
            return emptySnapshot(owner)
        }

        if (!refresh) {
            return snapshots[key]?.copy(fromCache = true)
                ?: loadFromProvider(profile, adapter, owner, expectedEpoch, writeLease)
        }

        return runCatching { loadFromProvider(profile, adapter, owner, expectedEpoch, writeLease) }
            .onFailure {
                PlatformLog.w(
                    "ThirdPartyProvider",
                    "${providerId.key} snapshot failed",
                    it
                )
            }
            .getOrElse {
                snapshots[key]?.copy(fromCache = true)
                    ?: emptySnapshot(owner, fromCache = true)
            }
    }

    /** Loads each connected provider into its own independent snapshot. */
    suspend fun loadConnected(
        profile: UserProfile,
        refresh: Boolean = true
    ): Map<String, ThirdPartyProviderSnapshot> = coroutineScope {
        adapters.all
            .filter { it.isConnected(profile) }
            .map { adapter ->
                async {
                    adapter.id to load(profile, adapter.providerId, refresh)
                }
            }
            .awaitAll()
            .mapNotNull { (id, snapshot) -> snapshot?.let { id to it } }
            .toMap()
    }

    fun cached(
        profile: UserProfile,
        providerId: ThirdPartyProviderId
    ): ThirdPartyProviderSnapshot? {
        val owner = profile.providerDataOwner(providerId) ?: return null
        return snapshots[owner.cacheKey()]?.copy(fromCache = true)
    }

    suspend fun clear(profile: UserProfile, providerId: ThirdPartyProviderId) {
        val scopeKey = scopeKey(profile.id, providerId)
        scopeLock(scopeKey).withLock {
            invalidateScope(profile.id, providerId)
            continueWatchingRepository.invalidate(profile.id, providerId)
            val owner = profile.providerDataOwner(providerId)
            if (owner != null) {
                snapshots.remove(owner.cacheKey())
                store.clear(owner)
            } else {
                val prefix = "${profile.id}|${providerId.key}|"
                snapshots.keys.removeIf { it.startsWith(prefix) }
                store.clearAllAccounts(profile.id, providerId)
            }
        }
    }

    suspend fun clearAll(profile: UserProfile) {
        ThirdPartyProviderId.entries.forEach { clear(profile, it) }
    }

    suspend fun pushWatchlist(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        expectedAccountId: String?,
        item: Meta,
        add: Boolean
    ): Boolean {
        val adapter = strictAdapter(profile, providerId, expectedAccountId) ?: return false
        if (ProviderCapability.LIBRARY !in adapter.capabilities) return false
        return runCatching {
            if (!adapter.pushWatchlist(profile, item, add)) return@runCatching false
            refreshAfterWrite(profile, providerId, expectedAccountId)
        }.onFailure { PlatformLog.w("ThirdPartyProvider", "${providerId.key} watchlist write failed", it) }
            .getOrDefault(false)
    }

    suspend fun pushWatched(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        expectedAccountId: String?,
        item: Meta,
        episodes: List<Video> = emptyList(),
        watched: Boolean
    ): Boolean {
        val adapter = strictAdapter(profile, providerId, expectedAccountId) ?: return false
        if (ProviderCapability.WATCH_HISTORY !in adapter.capabilities) return false
        return runCatching {
            if (!adapter.pushWatched(profile, item, episodes, watched)) return@runCatching false
            refreshAfterWrite(profile, providerId, expectedAccountId)
        }.onFailure { PlatformLog.w("ThirdPartyProvider", "${providerId.key} watched write failed", it) }
            .getOrDefault(false)
    }

    /** Fan playback completion/history writes out to every connected provider that supports them. */
    suspend fun pushWatchedToConnected(
        profile: UserProfile,
        item: Meta,
        episodes: List<Video> = emptyList(),
        watched: Boolean
    ): Boolean = coroutineScope {
        val targets = ThirdPartyProviderId.entries.filter(profile::isProviderConnected)
        if (targets.isEmpty()) return@coroutineScope false
        targets.map { providerId ->
            async {
                pushWatched(
                    profile = profile,
                    providerId = providerId,
                    expectedAccountId = profile.providerAccountId(providerId),
                    item = item,
                    episodes = episodes,
                    watched = watched
                )
            }
        }.awaitAll().any { it }
    }

    /** Clear remote resume progress from every connected provider; Continue Watching source is read-only policy. */
    suspend fun removeContinueWatchingFromConnected(
        profile: UserProfile,
        item: Meta
    ): Boolean = coroutineScope {
        val targets = ThirdPartyProviderId.entries.filter(profile::isProviderConnected)
        if (targets.isEmpty()) return@coroutineScope true
        targets.map { providerId ->
            async {
                removeContinueWatching(
                    profile = profile,
                    providerId = providerId,
                    expectedAccountId = profile.providerAccountId(providerId),
                    item = item
                )
            }
        }.awaitAll().any { it }
    }

    suspend fun pushPlaybackProgressToConnected(
        profile: UserProfile,
        item: Meta,
        videoId: String?,
        positionMs: Long,
        durationMs: Long,
        action: PlaybackSyncAction
    ): Boolean = coroutineScope {
        if (durationMs <= 0L) return@coroutineScope false
        val targets = adapters.all.filter { adapter ->
            ProviderCapability.PUSH_PROGRESS in adapter.capabilities && adapter.isConnected(profile)
        }
        if (targets.isEmpty()) return@coroutineScope false
        targets.map { adapter ->
            async {
                runCatching {
                    adapter.pushPlaybackProgress(profile, item, videoId, positionMs, durationMs, action)
                }.onFailure {
                    PlatformLog.w("ThirdPartyProvider", "${adapter.id} playback progress write failed", it)
                }.getOrDefault(false)
            }
        }.awaitAll().any { it }
    }

    suspend fun pushFavorite(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        expectedAccountId: String?,
        item: Meta,
        favorite: Boolean
    ): Boolean {
        val adapter = strictAdapter(profile, providerId, expectedAccountId) ?: return false
        if (ProviderCapability.FAVORITES !in adapter.capabilities) return false
        return runCatching {
            if (!adapter.pushFavorite(profile, item, favorite)) return@runCatching false
            refreshAfterWrite(profile, providerId, expectedAccountId)
        }.onFailure { PlatformLog.w("ThirdPartyProvider", "${providerId.key} favorite write failed", it) }
            .getOrDefault(false)
    }

    suspend fun removeContinueWatching(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        expectedAccountId: String?,
        item: Meta
    ): Boolean {
        val adapter = strictAdapter(profile, providerId, expectedAccountId) ?: return false
        if (ProviderCapability.CONTINUE_WATCHING !in adapter.capabilities) return false
        return runCatching {
            if (!adapter.clearPlaybackProgress(profile, item)) return@runCatching false
            val snapshot = refreshAfterWriteSnapshot(profile, providerId, expectedAccountId)
                ?: return@runCatching false
            val removedIdentity = TraktIntegration.contentIdentityKey(item)
            snapshot.continueWatching.none {
                TraktIntegration.contentIdentityKey(it) == removedIdentity
            }
        }.onFailure {
            PlatformLog.w("ThirdPartyProvider", "${providerId.key} progress clear failed", it)
        }.getOrDefault(false)
    }

    private suspend fun refreshAfterWrite(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        expectedAccountId: String?
    ): Boolean = refreshAfterWriteSnapshot(profile, providerId, expectedAccountId) != null

    private suspend fun refreshAfterWriteSnapshot(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        expectedAccountId: String?
    ): ThirdPartyProviderSnapshot? {
        val snapshot = load(profile, providerId, refresh = true) ?: return null
        return snapshot.takeIf {
            !it.fromCache &&
                expectedAccountId != null &&
                it.accountId == expectedAccountId
        }
    }

    private fun strictAdapter(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        expectedAccountId: String?
    ): ProviderAdapter? {
        val adapter = adapters.byId(providerId.key)
            ?.takeIf { it.isConnected(profile) }
            ?: return null
        val owner = adapter.dataOwner(profile) ?: return null
        return adapter.takeIf { expectedAccountId == null || owner.providerAccountId == expectedAccountId }
    }

    private suspend fun loadFromProvider(
        profile: UserProfile,
        adapter: ProviderAdapter,
        owner: ProviderDataOwner,
        expectedEpoch: Long,
        writeLease: ProviderDataStore.WriteLease
    ): ThirdPartyProviderSnapshot = coroutineScope {
        val addonCount = async {
            if (ProviderCapability.ADDONS !in adapter.capabilities) 0
            else adapter.fetchAddonCount(profile) ?: 0
        }
        val planned = async {
            fetchIf(adapter, ProviderCapability.LIBRARY) {
                adapter.fetchWatchlist(profile)
            }
        }
        val watching = async {
            fetchIf(adapter, ProviderCapability.LIBRARY) {
                adapter.fetchWatching(profile).orEmpty()
            }
        }
        val completed = async {
            fetchIf(adapter, ProviderCapability.WATCH_HISTORY) {
                adapter.fetchWatched(profile).orEmpty()
            }
        }
        val continueWatching = async {
            if (ProviderCapability.CONTINUE_WATCHING !in adapter.capabilities) {
                emptyList()
            } else {
                adapter.fetchContinueWatching(profile).orEmpty()
            }
        }
        val favorites = async {
            fetchIf(adapter, ProviderCapability.FAVORITES) {
                adapter.fetchFavorites(profile).orEmpty()
            }
        }
        val collection = async {
            fetchIf(adapter, ProviderCapability.COLLECTION) {
                adapter.fetchCollection(profile).orEmpty()
            }
        }
        val watchedEpisodeTimestamps = async {
            if (ProviderCapability.WATCH_HISTORY !in adapter.capabilities) null
            else adapter.fetchWatchedEpisodeTimestamps(profile)
        }

        val addonCountValue = addonCount.await()
        val plannedItems = planned.await().providerTagged(adapter.providerId)
        val watchingItems = watching.await().providerTagged(adapter.providerId)
        val completedItems = completed.await().providerTagged(adapter.providerId)
        val continueWatchingItems = FluxaCoreNative.mergeContinueWatchingDuplicates(
            continueWatching.await().providerTagged(adapter.providerId)
        )
        val favoriteItems = favorites.await().providerTagged(adapter.providerId)
        val collectionItems = collection.await().providerTagged(adapter.providerId)
        val watchedBySeries = watchedEpisodeTimestamps.await()?.toWatchedIdsBySeries().orEmpty()

        if (!isEpochCurrent(owner.appProfileId, adapter.providerId, expectedEpoch)) {
            return@coroutineScope emptySnapshot(owner, fromCache = true)
        }
        if (!store.replaceSnapshot(writeLease, continueWatchingItems, watchedBySeries)) {
            return@coroutineScope emptySnapshot(owner, fromCache = true)
        }

        val syncedAt = System.currentTimeMillis()
        val snapshot = ThirdPartyProviderSnapshot(
            providerId = adapter.providerId,
            accountId = owner.providerAccountId,
            planned = plannedItems,
            watching = watchingItems,
            completed = completedItems,
            continueWatching = continueWatchingItems,
            favorites = favoriteItems,
            collection = collectionItems,
            watchedEpisodeIdsBySeries = watchedBySeries,
            addonCount = addonCountValue,
            syncedAt = syncedAt,
            fromCache = false
        )
        val published = scopeLock(scopeKey(owner.appProfileId, adapter.providerId)).withLock {
            if (!isEpochCurrent(owner.appProfileId, adapter.providerId, expectedEpoch)) {
                false
            } else {
                snapshots[owner.cacheKey()] = snapshot
                profileManager.updateProfile(owner.appProfileId) { current ->
                    if (
                        adapter.isConnected(current) &&
                        current.providerAccountId(adapter.providerId) == owner.providerAccountId
                    ) {
                        current.withProviderLastSyncAt(adapter.providerId, syncedAt)
                    } else {
                        current
                    }
                }
                true
            }
        }
        if (published) snapshot else emptySnapshot(owner, fromCache = true)
    }

    private fun currentEpoch(profileId: String, providerId: ThirdPartyProviderId): Long =
        scopeEpochs[scopeKey(profileId, providerId)] ?: 0L

    private fun invalidateScope(profileId: String, providerId: ThirdPartyProviderId) {
        scopeEpochs.compute(scopeKey(profileId, providerId)) { _, current -> (current ?: 0L) + 1L }
    }

    private fun isEpochCurrent(
        profileId: String,
        providerId: ThirdPartyProviderId,
        expectedEpoch: Long
    ): Boolean = currentEpoch(profileId, providerId) == expectedEpoch

    private suspend fun fetchIf(
        adapter: ProviderAdapter,
        capability: ProviderCapability,
        block: suspend () -> List<Meta>
    ): List<Meta> = if (capability in adapter.capabilities) block() else emptyList()

    private fun scopeLock(scopeKey: String): Mutex =
        scopeLocks.computeIfAbsent(scopeKey) { Mutex() }

    private fun emptySnapshot(
        owner: ProviderDataOwner,
        fromCache: Boolean = false
    ) = ThirdPartyProviderSnapshot(
        providerId = owner.providerId,
        accountId = owner.providerAccountId,
        fromCache = fromCache
    )
}

private fun scopeKey(profileId: String, providerId: ThirdPartyProviderId): String =
    "$profileId|${providerId.key}"

private fun ProviderDataOwner.cacheKey(): String =
    "$appProfileId|${providerId.key}|$providerAccountId"

private fun List<Meta>.providerTagged(
    providerId: ThirdPartyProviderId
): List<Meta> = map { item -> item.copy(reason = providerId.reasonLabel) }

private fun providerDistinct(items: List<Meta>): List<Meta> {
    val seen = HashSet<String>(items.size)
    return items.filter { seen.add(TraktIntegration.contentIdentityKey(it)) }
}

private fun Map<String, Long>.toWatchedIdsBySeries(): Map<String, Set<String>> =
    keys.mapNotNull { videoId ->
        val parts = videoId.split(':')
        if (parts.size < 3) return@mapNotNull null
        val season = parts[parts.lastIndex - 1].toIntOrNull() ?: return@mapNotNull null
        val episode = parts.last().toIntOrNull() ?: return@mapNotNull null
        val seriesId = parts.dropLast(2).joinToString(":").takeIf(String::isNotBlank) ?: return@mapNotNull null
        seriesId to "$seriesId:$season:$episode"
    }.groupBy({ it.first }, { it.second })
        .mapValues { (_, ids) -> ids.toSet() }
