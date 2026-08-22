package com.fluxa.app.data.repository

import com.fluxa.app.common.PlatformLog
import com.fluxa.app.data.remote.NuvioLibraryItemDto
import com.fluxa.app.data.remote.NuvioService
import com.fluxa.app.data.remote.NuvioWatchProgressDto
import com.fluxa.app.data.remote.NuvioWatchProgressDeltaEventDto
import com.fluxa.app.data.remote.NuvioWatchedItemDto
import com.google.gson.Gson
import com.google.gson.JsonArray
import com.google.gson.JsonObject
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import retrofit2.Response
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Implements Nuvio Public API v1.2 event-cursor sync for Library, Watch Progress,
 * and Watch History. Snapshot pulls are used only to bootstrap/recover a scope;
 * healthy subsequent reads consume ordered delta events.
 */
@Singleton
class NuvioDeltaSyncEngine @Inject constructor(
    private val service: NuvioService,
    private val stateStore: NuvioSyncStateStore,
    private val gson: Gson,
) {
    private val locks = ConcurrentHashMap<String, Mutex>()
    private val lastSuccessfulSyncAt = ConcurrentHashMap<String, Long>()

    suspend fun syncLibrary(
        authorization: String,
        scope: String,
        profileIndex: Int,
    ): List<NuvioLibraryItemDto> = resourceLock(scope, "library").withLock {
        val cached = stateStore.read(scope)
        if (cached.libraryInitialized && isFresh(scope, "library")) return@withLock cached.library
        runCatching {
            val state = JsonObject().apply {
                addProperty("initialized", cached.libraryInitialized)
                addProperty("cursor", cached.libraryCursor)
                add("items", gson.toJsonTree(cached.library))
            }
            val request = NuvioCoreBridge.deltaSyncRequestPlan(state)
            val result = if (request.get("mode")?.asString == "delta") {
                drainLibraryDelta(authorization, scope, profileIndex, cached.libraryCursor).library
            } else {
                bootstrapLibrary(authorization, scope, profileIndex).library
            }
            markFresh(scope, "library")
            result
        }.onFailure { PlatformLog.w("NuvioDelta", "Library delta sync failed for $scope", it) }
            .getOrElse {
                if (cached.libraryInitialized) cached.library
                else fallbackLibrarySnapshot(authorization, scope, profileIndex, cached.library)
            }
    }

    suspend fun syncProgress(
        authorization: String,
        scope: String,
        profileIndex: Int,
    ): NuvioProgressSyncResult = resourceLock(scope, "progress").withLock {
        val cached = stateStore.read(scope)
        if (cached.progressInitialized && isFresh(scope, "progress")) return@withLock coreProgressResult(cached)
        runCatching {
            val state = JsonObject().apply {
                addProperty("initialized", cached.progressInitialized)
                addProperty("cursor", cached.progressCursor)
                add("items", gson.toJsonTree(cached.progress))
            }
            val request = NuvioCoreBridge.progressSyncRequestPlan(state)
            val snapshot: JsonArray
            val snapshotCursor: Long?
            val events: JsonArray
            if (request.get("mode")?.asString == "delta") {
                snapshot = JsonArray()
                snapshotCursor = null
                events = gson.toJsonTree(pullProgressEvents(authorization, profileIndex, request.get("cursor")?.asLong ?: 0L)).asJsonArray
            } else {
                snapshotCursor = service.getWatchProgressDeltaCursor(
                    authorization,
                    mapOf("p_profile_id" to profileIndex),
                ).requireBody()
                snapshot = gson.toJsonTree(service.pullWatchProgress(
                    authorization,
                    mapOf("p_profile_id" to profileIndex, "p_limit" to PROGRESS_SNAPSHOT_LIMIT),
                ).requireBody()).asJsonArray
                events = gson.toJsonTree(pullProgressEvents(authorization, profileIndex, snapshotCursor)).asJsonArray
            }
            val persisted = stateStore.replaceProgressCore(
                scope,
                NuvioCoreBridge.applyProgressSync(state, snapshot, snapshotCursor, events),
            )
            markFresh(scope, "progress")
            coreProgressResult(persisted)
        }.onFailure { PlatformLog.w("NuvioDelta", "Watch-progress delta sync failed for $scope", it) }
            .getOrElse {
                // The snapshot endpoint is intentionally capped to recent rows. Never
                // replace a successfully bootstrapped raw progress cache with that partial
                // view just because a delta request had a transient failure.
                coreProgressResult(
                    if (cached.progressInitialized) cached
                    else stateStore.cacheProgressSnapshot(
                        scope,
                        service.pullWatchProgress(
                            authorization,
                            mapOf("p_profile_id" to profileIndex, "p_limit" to PROGRESS_SNAPSHOT_LIMIT),
                        ).requireBody(),
                    )
                )
            }
    }

    private fun coreProgressResult(state: NuvioSyncState): NuvioProgressSyncResult {
        val coreState = JsonObject().apply {
            addProperty("initialized", state.progressInitialized)
            addProperty("cursor", state.progressCursor)
            add("items", gson.toJsonTree(state.progress))
        }
        val projection = NuvioCoreBridge.applyProgressSync(
            state = coreState,
            snapshot = JsonArray(),
            snapshotCursor = null,
            events = JsonArray(),
        )
        return NuvioProgressSyncResult(
            progress = state.progress,
            continueWatching = projection.getAsJsonArray("continueWatching")
                ?.let { gson.fromJson(it, Array<NuvioWatchProgressDto>::class.java).toList() }
                .orEmpty(),
        )
    }

    private suspend fun pullProgressEvents(
        authorization: String,
        profileIndex: Int,
        startCursor: Long,
    ): List<NuvioWatchProgressDeltaEventDto> {
        val events = mutableListOf<NuvioWatchProgressDeltaEventDto>()
        var cursor = startCursor
        while (true) {
            val batch = service.pullWatchProgressDelta(
                authorization,
                mapOf("p_profile_id" to profileIndex, "p_since_event_id" to cursor, "p_limit" to EVENT_DELTA_LIMIT),
            ).requireBody()
            if (batch.isEmpty()) return events
            events += batch
            val next = batch.maxOf { it.eventId }
            if (batch.size < EVENT_DELTA_LIMIT || next <= cursor) return events
            cursor = next
        }
    }

    suspend fun syncHistory(
        authorization: String,
        scope: String,
        profileIndex: Int,
    ): List<NuvioWatchedItemDto> = resourceLock(scope, "history").withLock {
        val cached = stateStore.read(scope)
        if (cached.historyInitialized && isFresh(scope, "history")) return@withLock cached.history
        runCatching {
            val state = JsonObject().apply {
                addProperty("initialized", cached.historyInitialized)
                addProperty("cursor", cached.historyCursor)
                add("items", gson.toJsonTree(cached.history))
            }
            val request = NuvioCoreBridge.deltaSyncRequestPlan(state)
            val result = if (request.get("mode")?.asString == "delta") {
                drainHistoryDelta(authorization, scope, profileIndex, cached.historyCursor).history
            } else {
                bootstrapHistory(authorization, scope, profileIndex).history
            }
            markFresh(scope, "history")
            result
        }.onFailure { PlatformLog.w("NuvioDelta", "Watch-history delta sync failed for $scope", it) }
            .getOrElse {
                if (cached.historyInitialized) cached.history
                else fallbackHistorySnapshot(authorization, scope, profileIndex, cached.history)
            }
    }

    /** Marks one resource stale without discarding its persisted event cursor/cache. */
    fun invalidate(scope: String, resource: String) {
        lastSuccessfulSyncAt.remove("${scope.trim()}|${resource.trim()}")
    }

    /** Forces the next read of this account/profile scope to perform a fresh cursor-fenced bootstrap. */
    suspend fun clear(scope: String) {
        lastSuccessfulSyncAt.keys.removeAll { it.startsWith("${scope.trim()}|") }
        stateStore.clear(scope)
    }

    private suspend fun bootstrapLibrary(
        authorization: String,
        scope: String,
        profileIndex: Int,
    ): NuvioSyncState {
        // Capture the fence before the paginated snapshot so mutations that race the
        // snapshot are replayed from the delta log afterwards.
        val fence = service.getLibraryDeltaCursor(
            authorization,
            mapOf("p_profile_id" to profileIndex),
        ).requireBody()
        val snapshot = pullAllLibrary(authorization, profileIndex)
        stateStore.replaceLibraryBootstrap(scope, snapshot, fence)
        return drainLibraryDelta(authorization, scope, profileIndex, fence)
    }

    private suspend fun bootstrapHistory(
        authorization: String,
        scope: String,
        profileIndex: Int,
    ): NuvioSyncState {
        val fence = service.getWatchedItemsDeltaCursor(
            authorization,
            mapOf("p_profile_id" to profileIndex),
        ).requireBody()
        val snapshot = pullAllHistory(authorization, profileIndex)
        stateStore.replaceHistoryBootstrap(scope, snapshot, fence)
        return drainHistoryDelta(authorization, scope, profileIndex, fence)
    }

    private suspend fun drainLibraryDelta(
        authorization: String,
        scope: String,
        profileIndex: Int,
        startCursor: Long,
    ): NuvioSyncState {
        var state = stateStore.read(scope)
        var cursor = maxOf(startCursor, state.libraryCursor)
        while (true) {
            val page = service.pullLibraryDelta(
                authorization,
                mapOf(
                    "p_profile_id" to profileIndex,
                    "p_since_event_id" to cursor,
                    "p_limit" to LIBRARY_DELTA_LIMIT,
                ),
            ).requireBody()
            if (page.isEmpty()) return state
            val previousCursor = cursor
            state = stateStore.applyLibraryEvents(scope, page)
            cursor = state.libraryCursor
            if (page.size < LIBRARY_DELTA_LIMIT || cursor <= previousCursor) return state
        }
    }

    private suspend fun drainHistoryDelta(
        authorization: String,
        scope: String,
        profileIndex: Int,
        startCursor: Long,
    ): NuvioSyncState {
        var state = stateStore.read(scope)
        var cursor = maxOf(startCursor, state.historyCursor)
        while (true) {
            val page = service.pullWatchedItemsDelta(
                authorization,
                mapOf(
                    "p_profile_id" to profileIndex,
                    "p_since_event_id" to cursor,
                    "p_limit" to EVENT_DELTA_LIMIT,
                ),
            ).requireBody()
            if (page.isEmpty()) return state
            val previousCursor = cursor
            state = stateStore.applyHistoryEvents(scope, page)
            cursor = state.historyCursor
            if (page.size < EVENT_DELTA_LIMIT || cursor <= previousCursor) return state
        }
    }

    private suspend fun fallbackLibrarySnapshot(
        authorization: String,
        scope: String,
        profileIndex: Int,
        cached: List<NuvioLibraryItemDto>,
    ): List<NuvioLibraryItemDto> = runCatching {
        val snapshot = pullAllLibrary(authorization, profileIndex)
        stateStore.cacheLibrarySnapshot(scope, snapshot).library
    }.getOrElse { cached }

    private suspend fun fallbackHistorySnapshot(
        authorization: String,
        scope: String,
        profileIndex: Int,
        cached: List<NuvioWatchedItemDto>,
    ): List<NuvioWatchedItemDto> = runCatching {
        val snapshot = pullAllHistory(authorization, profileIndex)
        stateStore.cacheHistorySnapshot(scope, snapshot).history
    }.getOrElse { cached }

    private suspend fun pullAllLibrary(
        authorization: String,
        profileIndex: Int,
    ): List<NuvioLibraryItemDto> {
        val items = ArrayList<NuvioLibraryItemDto>()
        var offset = 0
        while (true) {
            val page = service.pullLibrary(
                authorization,
                mapOf(
                    "p_profile_id" to profileIndex,
                    "p_limit" to LIBRARY_SNAPSHOT_LIMIT,
                    "p_offset" to offset,
                ),
            ).requireBody()
            items += page
            if (page.size < LIBRARY_SNAPSHOT_LIMIT) return items
            offset += page.size
        }
    }

    private suspend fun pullAllHistory(
        authorization: String,
        profileIndex: Int,
    ): List<NuvioWatchedItemDto> {
        val items = ArrayList<NuvioWatchedItemDto>()
        var pageNumber = 1
        while (true) {
            val page = service.pullWatchedItems(
                authorization,
                mapOf(
                    "p_profile_id" to profileIndex,
                    "p_page" to pageNumber,
                    "p_page_size" to HISTORY_SNAPSHOT_LIMIT,
                ),
            ).requireBody()
            items += page
            if (page.size < HISTORY_SNAPSHOT_LIMIT) return items
            pageNumber += 1
        }
    }

    private fun resourceLock(scope: String, resource: String): Mutex =
        locks.computeIfAbsent("${scope.trim()}|$resource") { Mutex() }

    private fun isFresh(scope: String, resource: String): Boolean {
        val at = lastSuccessfulSyncAt["${scope.trim()}|$resource"] ?: return false
        return System.currentTimeMillis() - at <= READ_COALESCE_MS
    }

    private fun markFresh(scope: String, resource: String) {
        lastSuccessfulSyncAt["${scope.trim()}|$resource"] = System.currentTimeMillis()
    }

    private companion object {
        const val READ_COALESCE_MS = 2_000L
        const val LIBRARY_SNAPSHOT_LIMIT = 500
        const val LIBRARY_DELTA_LIMIT = 500
        const val PROGRESS_SNAPSHOT_LIMIT = 200
        const val HISTORY_SNAPSHOT_LIMIT = 500
        const val EVENT_DELTA_LIMIT = 1000
    }
}

private fun <T> Response<T>.requireBody(): T {
    if (!isSuccessful) throw IllegalStateException("Nuvio request failed (${code()})")
    return body() ?: throw IllegalStateException("Nuvio returned an empty response")
}
