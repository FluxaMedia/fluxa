package com.fluxa.app.data.repository

import android.content.Context
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.BuildConfig
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.SimklAllItemsResponse
import com.fluxa.app.data.remote.ExternalSyncApi
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
    private val externalSyncApi: ExternalSyncApi
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
            ?.takeIf { it.schemaVersion == CACHE_SCHEMA_VERSION }
        val activities = runCatching { externalSyncApi.getSimklActivities("Bearer $token", BuildConfig.SIMKL_CLIENT_ID) }.getOrNull()
            ?: return SimklSyncSnapshot(previous?.resources.orEmpty())
        val definitions = listOf(
            Definition("showsWatching", "shows", "watching"),
            Definition("moviesWatching", "movies", "watching"),
            Definition("animeWatching", "anime", "watching"),
            Definition("showsPlanToWatch", "shows", "plantowatch"),
            Definition("moviesPlanToWatch", "movies", "plantowatch"),
            Definition("animePlanToWatch", "anime", "plantowatch"),
            Definition("showsCompleted", "shows", "completed"),
            Definition("moviesCompleted", "movies", "completed"),
            Definition("animeCompleted", "anime", "completed")
        )
        val plan = FluxaCoreNative.simklResourceSyncPlan(
            previous?.activities,
            activities,
            definitions.map { definition ->
                val activityType = when (definition.type) {
                    "shows" -> "tv_shows"
                    "anime" -> "anime"
                    else -> "movies"
                }
                mapOf("key" to definition.key, "type" to activityType, "status" to definition.status, "hasCached" to previous?.resources.orEmpty().containsKey(definition.key))
            }
        ).associateBy { it.key }
        val resources = definitions.associate { definition ->
            val entry = plan[definition.key]
            val cached = previous?.resources?.get(definition.key)
            val next = runCatching {
                when (entry?.action) {
                    "unchanged" -> cached
                    "delta" -> {
                        val changes = externalSyncApi.getSimklAllItems(definition.type, definition.status, "Bearer $token", BuildConfig.SIMKL_CLIENT_ID, dateFrom = entry.dateFrom)
                        gson.fromJson(FluxaCoreNative.simklMergeDelta(cached, changes), SimklAllItemsResponse::class.java)
                    }
                    else -> externalSyncApi.getSimklAllItems(definition.type, definition.status, "Bearer $token", BuildConfig.SIMKL_CLIENT_ID)
                }
            }.getOrNull()
            definition.key to (next ?: cached ?: SimklAllItemsResponse())
        }
        cache.edit().putString(key, gson.toJson(SimklCache(activities, resources))).apply()
        return SimklSyncSnapshot(resources)
    }

    private data class Definition(val key: String, val type: String, val status: String)
    private data class SimklCache(
        val activities: JsonObject,
        val resources: Map<String, SimklAllItemsResponse>,
        val schemaVersion: Int = CACHE_SCHEMA_VERSION
    )

    private companion object {
        const val CACHE_SCHEMA_VERSION = 2
    }
}
