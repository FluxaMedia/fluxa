package com.fluxa.app.data.repository.library

import com.fluxa.app.data.local.ProviderDataOwner
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.local.WatchedContentDurationRecord
import com.fluxa.app.data.remote.Meta
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Inject
import javax.inject.Singleton
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Account-scoped mirror of remote provider data.
 *
 * All writes are serialized per `(Fluxa profile, provider)` scope. A lease is
 * captured before a remote request starts. Disconnect increments the scope
 * generation while holding the same lock, so an older in-flight request cannot
 * recreate data after the provider was disconnected or switched to another
 * account.
 */
@Singleton
class ProviderDataStore @Inject constructor(
    private val watchlistManager: WatchlistManager
) {
    private val generations = ConcurrentHashMap<String, Long>()
    private val scopeLocks = ConcurrentHashMap<String, Mutex>()

    data class WriteLease internal constructor(
        val owner: ProviderDataOwner,
        internal val generation: Long
    )

    fun lease(owner: ProviderDataOwner): WriteLease = WriteLease(
        owner = owner,
        generation = currentGeneration(owner.scopeKey())
    )

    suspend fun replaceContinueWatching(lease: WriteLease, items: List<Meta>): Boolean =
        commit(lease) { watchlistManager.replaceExternalContinueWatching(lease.owner, items) }

    suspend fun replaceContinueWatching(owner: ProviderDataOwner, items: List<Meta>): Boolean =
        replaceContinueWatching(lease(owner), items)

    suspend fun getContinueWatching(owner: ProviderDataOwner): List<Meta> =
        watchlistManager.getExternalContinueWatchingSnapshot(owner)

    suspend fun replaceWatchedEpisodes(
        lease: WriteLease,
        episodesBySeries: Map<String, Set<String>>
    ): Boolean = commit(lease) {
        watchlistManager.replaceExternalWatchedEpisodes(lease.owner, episodesBySeries)
    }

    suspend fun replaceWatchedEpisodes(
        owner: ProviderDataOwner,
        episodesBySeries: Map<String, Set<String>>
    ): Boolean = replaceWatchedEpisodes(lease(owner), episodesBySeries)

    suspend fun replaceSnapshot(
        lease: WriteLease,
        continueWatching: List<Meta>,
        watchedEpisodesBySeries: Map<String, Set<String>>
    ): Boolean = commit(lease) {
        watchlistManager.replaceExternalProviderSnapshot(
            owner = lease.owner,
            continueWatching = continueWatching,
            watchedEpisodesBySeries = watchedEpisodesBySeries
        )
    }

    suspend fun replaceWatchedDurations(
        lease: WriteLease,
        records: Collection<WatchedContentDurationRecord>
    ): Boolean = commit(lease) {
        watchlistManager.replaceExternalWatchedContentDurations(lease.owner, records)
    }

    suspend fun replaceWatchedDurations(
        owner: ProviderDataOwner,
        records: Collection<WatchedContentDurationRecord>
    ): Boolean = replaceWatchedDurations(lease(owner), records)

    suspend fun getWatchedEpisodeIds(owner: ProviderDataOwner, seriesId: String): Set<String> =
        watchlistManager.getProviderWatchedVideoIds(owner, seriesId)

    suspend fun clear(owner: ProviderDataOwner) {
        val scopeKey = owner.scopeKey()
        lockFor(scopeKey).withLock {
            invalidateLocked(scopeKey)
            watchlistManager.clearProviderData(owner)
        }
    }

    suspend fun clearAllAccounts(profileId: String, providerId: ThirdPartyProviderId) {
        val scopeKey = scopeKey(profileId, providerId)
        lockFor(scopeKey).withLock {
            invalidateLocked(scopeKey)
            watchlistManager.clearAllProviderAccounts(profileId, providerId)
        }
    }

    private suspend fun commit(lease: WriteLease, block: suspend () -> Unit): Boolean {
        val scopeKey = lease.owner.scopeKey()
        return lockFor(scopeKey).withLock {
            if (currentGeneration(scopeKey) != lease.generation) return@withLock false
            block()
            true
        }
    }

    private fun currentGeneration(scopeKey: String): Long = generations[scopeKey] ?: 0L

    private fun invalidateLocked(scopeKey: String) {
        generations[scopeKey] = currentGeneration(scopeKey) + 1L
    }

    private fun lockFor(scopeKey: String): Mutex = scopeLocks.computeIfAbsent(scopeKey) { Mutex() }
}

private fun ProviderDataOwner.scopeKey(): String = scopeKey(appProfileId, providerId)

private fun scopeKey(profileId: String, providerId: ThirdPartyProviderId): String =
    "$profileId|${providerId.key}"
