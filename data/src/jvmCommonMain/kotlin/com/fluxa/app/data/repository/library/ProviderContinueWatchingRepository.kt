package com.fluxa.app.data.repository.library

import com.fluxa.app.common.PlatformLog
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.providerDataOwner
import com.fluxa.app.data.local.safeContinueWatchingSource
import com.fluxa.app.data.remote.Meta
import javax.inject.Inject
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Singleton

data class ProviderContinueWatchingSnapshot(
    val providerId: ThirdPartyProviderId,
    val accountId: String,
    val items: List<Meta>,
    val fromCache: Boolean
)

/**
 * Strict source-of-truth repository. Exactly one selected provider is read.
 * It never merges local data, another provider, or metadata-add-on results.
 */
@Singleton
class ProviderContinueWatchingRepository @Inject constructor(
    private val adapters: ProviderAdapters,
    private val store: ProviderDataStore
) {
    private val scopeEpochs = ConcurrentHashMap<String, Long>()

    suspend fun loadSelected(profile: UserProfile, refresh: Boolean = true): ProviderContinueWatchingSnapshot? {
        val providerId = ThirdPartyProviderId.from(profile.safeContinueWatchingSource) ?: return null
        return load(profile, providerId, refresh)
    }

    suspend fun load(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        refresh: Boolean = true
    ): ProviderContinueWatchingSnapshot? {
        val adapter = adapters.byId(providerId.key) ?: return null
        val owner = adapter.dataOwner(profile) ?: return null
        val expectedEpoch = currentEpoch(profile.id, providerId)
        val writeLease = store.lease(owner)
        if (!adapter.isConnected(profile) || ProviderCapability.CONTINUE_WATCHING !in adapter.capabilities) {
            return ProviderContinueWatchingSnapshot(providerId, owner.providerAccountId, emptyList(), fromCache = false)
        }

        if (refresh) {
            val remote = runCatching { adapter.fetchContinueWatching(profile).orEmpty() }
                .onFailure { PlatformLog.w("ProviderContinueWatching", "${providerId.key} sync failed", it) }
                .getOrNull()
            if (remote != null && isEpochCurrent(profile.id, providerId, expectedEpoch)) {
                val pure = FluxaCoreNative.mergeContinueWatchingDuplicates(
                    remote.map { it.withProviderReason(providerId) }
                )
                if (!store.replaceContinueWatching(writeLease, pure)) {
                    return ProviderContinueWatchingSnapshot(
                        providerId,
                        owner.providerAccountId,
                        store.getContinueWatching(owner),
                        fromCache = true
                    )
                }
                return ProviderContinueWatchingSnapshot(providerId, owner.providerAccountId, pure, fromCache = false)
            }
        }

        return ProviderContinueWatchingSnapshot(
            providerId = providerId,
            accountId = owner.providerAccountId,
            items = FluxaCoreNative.mergeContinueWatchingDuplicates(store.getContinueWatching(owner)),
            fromCache = true
        )
    }

    suspend fun replace(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        items: List<Meta>
    ): ProviderContinueWatchingSnapshot? {
        val adapter = adapters.byId(providerId.key) ?: return null
        val owner = adapter.dataOwner(profile) ?: return null
        val expectedEpoch = currentEpoch(profile.id, providerId)
        val writeLease = store.lease(owner)
        val pure = FluxaCoreNative.mergeContinueWatchingDuplicates(
            items.map { it.withProviderReason(providerId) }
        )
        if (!isEpochCurrent(profile.id, providerId, expectedEpoch)) return null
        if (!store.replaceContinueWatching(writeLease, pure)) return null
        return ProviderContinueWatchingSnapshot(providerId, owner.providerAccountId, pure, fromCache = false)
    }


    suspend fun cached(
        profile: UserProfile,
        providerId: ThirdPartyProviderId
    ): ProviderContinueWatchingSnapshot? {
        val adapter = adapters.byId(providerId.key) ?: return null
        val owner = adapter.dataOwner(profile) ?: return null
        return ProviderContinueWatchingSnapshot(
            providerId = providerId,
            accountId = owner.providerAccountId,
            items = FluxaCoreNative.mergeContinueWatchingDuplicates(store.getContinueWatching(owner)),
            fromCache = true
        )
    }

    fun invalidate(profileId: String, providerId: ThirdPartyProviderId) {
        scopeEpochs.compute(scopeKey(profileId, providerId)) { _, current -> (current ?: 0L) + 1L }
    }

    private fun currentEpoch(profileId: String, providerId: ThirdPartyProviderId): Long =
        scopeEpochs[scopeKey(profileId, providerId)] ?: 0L

    private fun isEpochCurrent(
        profileId: String,
        providerId: ThirdPartyProviderId,
        expectedEpoch: Long
    ): Boolean = currentEpoch(profileId, providerId) == expectedEpoch

}


private fun scopeKey(profileId: String, providerId: ThirdPartyProviderId): String =
    "$profileId|${providerId.key}"

private fun Meta.withProviderReason(providerId: ThirdPartyProviderId): Meta = copy(
    reason = providerId.reasonLabel
)
