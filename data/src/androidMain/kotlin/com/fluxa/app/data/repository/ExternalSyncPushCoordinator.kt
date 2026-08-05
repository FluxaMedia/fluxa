package com.fluxa.app.data.repository

import com.fluxa.app.common.PlatformLog
import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.library.ProviderAdapters
import com.google.gson.Gson
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import javax.inject.Inject

private data class ProviderEligibility(
    val trakt: Boolean = false,
    val simkl: Boolean = false,
    val anilist: Boolean = false,
    val stremio: Boolean = false,
    val nuvio: Boolean = false
) {
    fun isEligible(id: String): Boolean = when (id) {
        "trakt" -> trakt
        "simkl" -> simkl
        "anilist" -> anilist
        "stremio" -> stremio
        "nuvio" -> nuvio
        else -> false
    }
}

class ExternalSyncPushCoordinator @Inject constructor(
    private val adapters: ProviderAdapters,
    private val gson: Gson
) {
    suspend fun pushMarkWatched(profile: UserProfile, meta: Meta, episodes: List<Video>, watched: Boolean) = coroutineScope {
        val plan = fetchPlan(
            "markWatched",
            mapOf(
                "profile" to profileJson(profile),
                "videoIds" to episodes.map { it.id },
                "watched" to watched,
                "meta" to meta,
                "nowMs" to System.currentTimeMillis()
            )
        ) ?: return@coroutineScope

        adapters.all.filter { plan.isEligible(it.id) }.forEach { adapter ->
            launch {
                runCatching { adapter.pushWatched(profile, meta, episodes, watched) }
                    .onFailure { PlatformLog.w("ExternalSyncPush", "${adapter.id} pushMarkWatched failed for ${meta.id}", it) }
            }
        }
    }

    suspend fun pushWatchlist(profile: UserProfile, meta: Meta, isInWatchlist: Boolean) = coroutineScope {
        val plan = fetchPlan(
            "watchlist",
            mapOf(
                "profile" to profileJson(profile),
                "item" to meta,
                "command" to if (isInWatchlist) "add" else "remove",
                "nowMs" to System.currentTimeMillis()
            )
        ) ?: return@coroutineScope

        adapters.all.filter { plan.isEligible(it.id) }.forEach { adapter ->
            launch {
                runCatching { adapter.pushWatchlist(profile, meta, isInWatchlist) }
                    .onFailure { PlatformLog.w("ExternalSyncPush", "${adapter.id} pushWatchlist failed for ${meta.id}", it) }
            }
        }
    }

    suspend fun pushFavorite(profile: UserProfile, meta: Meta, isFavorite: Boolean) = coroutineScope {
        val plan = fetchPlan(
            "favorite",
            mapOf(
                "profile" to profileJson(profile),
                "command" to if (isFavorite) "add" else "remove",
                "nowMs" to System.currentTimeMillis()
            )
        ) ?: return@coroutineScope

        adapters.all.filter { plan.isEligible(it.id) }.forEach { adapter ->
            launch {
                runCatching { adapter.pushFavorite(profile, meta, isFavorite) }
                    .onFailure { PlatformLog.w("ExternalSyncPush", "${adapter.id} pushFavorite failed for ${meta.id}", it) }
            }
        }
    }

    private fun profileJson(profile: UserProfile): Map<String, Any?> = mapOf(
        "traktAccessToken" to profile.traktAccessToken,
        "traktTokenExpiresAt" to profile.traktTokenExpiresAt,
        "simklAccessToken" to profile.simklAccessToken,
        "anilistAccessToken" to profile.anilistAccessToken,
        "stremioAuthKey" to profile.authKey,
        "nuvioAccessToken" to profile.nuvioAccessToken
    )

    private fun fetchPlan(kind: String, args: Map<String, Any?>): ProviderEligibility? {
        val payload = args + mapOf("kind" to kind, "integrationSettings" to mapOf("watchProgressSource" to "all"))
        return runCatching {
            val value = FluxaCoreUniFfi.coreInvokeValue("externalProviderActionPlan", gson.toJson(payload))
            gson.fromJson(value, ProviderEligibility::class.java)
        }.onFailure { PlatformLog.w("ExternalSyncPush", "externalProviderActionPlan failed for kind=$kind", it) }
            .getOrNull()
    }
}
