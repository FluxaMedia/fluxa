package com.fluxa.app.data.repository

import android.content.Context
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.BuildConfig
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.SimklAllItemsResponse
import com.fluxa.app.data.remote.TraktApi
import com.google.gson.Gson
import com.google.gson.JsonObject
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

data class SimklSyncSnapshot(val resources: Map<String, SimklAllItemsResponse>)

@Singleton
class SimklSyncCoordinator @Inject constructor(
    @ApplicationContext context: Context,
    private val gson: Gson,
    private val traktApi: TraktApi
) {
    private val cache = context.getSharedPreferences("simkl_sync_delta", Context.MODE_PRIVATE)
    private val mutex = Mutex()

    suspend fun snapshot(profile: UserProfile): SimklSyncSnapshot = withContext(Dispatchers.IO) {
        mutex.withLock { snapshotLocked(profile) }
    }

    private suspend fun snapshotLocked(profile: UserProfile): SimklSyncSnapshot {
        val token = profile.simklAccessToken?.takeIf(String::isNotBlank) ?: return SimklSyncSnapshot(emptyMap())
        if (BuildConfig.SIMKL_CLIENT_ID.isBlank()) return SimklSyncSnapshot(emptyMap())
        val key = "snapshot:${profile.id}"
        val previous = cache.getString(key, null)?.let { gson.fromJson(it, SimklCache::class.java) }
        val activities = runCatching { traktApi.getSimklActivities("Bearer $token", BuildConfig.SIMKL_CLIENT_ID) }.getOrNull()
            ?: return SimklSyncSnapshot(previous?.resources.orEmpty())
        val definitions = listOf(
            Definition("showsWatching", "shows", "watching"),
            Definition("moviesWatching", "movies", "watching"),
            Definition("showsPlanToWatch", "shows", "plantowatch"),
            Definition("moviesPlanToWatch", "movies", "plantowatch"),
            Definition("showsCompleted", "shows", "completed"),
            Definition("moviesCompleted", "movies", "completed")
        )
        val plan = FluxaCoreNative.simklResourceSyncPlan(
            previous?.activities,
            activities,
            definitions.map { definition ->
                mapOf("key" to definition.key, "type" to if (definition.type == "shows") "tv_shows" else "movies", "status" to definition.status, "hasCached" to previous?.resources.orEmpty().containsKey(definition.key))
            }
        ).associateBy { it.key }
        val resources = definitions.associate { definition ->
            val entry = plan[definition.key]
            val cached = previous?.resources?.get(definition.key)
            val next = runCatching {
                when (entry?.action) {
                    "unchanged" -> cached
                    "delta" -> {
                        val changes = traktApi.getSimklAllItems(definition.type, definition.status, "Bearer $token", BuildConfig.SIMKL_CLIENT_ID, dateFrom = entry.dateFrom)
                        gson.fromJson(FluxaCoreNative.simklMergeDelta(cached, changes), SimklAllItemsResponse::class.java)
                    }
                    else -> traktApi.getSimklAllItems(definition.type, definition.status, "Bearer $token", BuildConfig.SIMKL_CLIENT_ID)
                }
            }.getOrNull()
            definition.key to (next ?: cached ?: SimklAllItemsResponse())
        }
        cache.edit().putString(key, gson.toJson(SimklCache(activities, resources))).apply()
        return SimklSyncSnapshot(resources)
    }

    private data class Definition(val key: String, val type: String, val status: String)
    private data class SimklCache(val activities: JsonObject, val resources: Map<String, SimklAllItemsResponse>)
}
