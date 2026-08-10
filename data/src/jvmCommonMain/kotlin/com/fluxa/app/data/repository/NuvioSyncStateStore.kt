package com.fluxa.app.data.repository

import com.fluxa.app.data.platform.PlatformKeyValueStore
import com.fluxa.app.data.remote.NuvioLibraryDeltaEventDto
import com.fluxa.app.data.remote.NuvioLibraryItemDto
import com.fluxa.app.data.remote.NuvioWatchProgressDeltaEventDto
import com.fluxa.app.data.remote.NuvioWatchProgressDto
import com.fluxa.app.data.remote.NuvioWatchedItemDeltaEventDto
import com.fluxa.app.data.remote.NuvioWatchedItemDto
import com.google.gson.Gson
import com.google.gson.JsonArray
import com.google.gson.JsonObject
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import javax.inject.Inject
import javax.inject.Named
import javax.inject.Singleton

/**
 * Persistent raw Nuvio state used by the event-cursor sync engine.
 *
 * This cache deliberately stores exact server rows. Core owns their identity,
 * event application, ordering, and Continue Watching projection.
 */
@Singleton
class NuvioSyncStateStore @Inject constructor(
    @param:Named("NuvioSyncDelta") private val cache: PlatformKeyValueStore,
    private val gson: Gson,
) {
    private val mutex = Mutex()

    internal suspend fun read(scope: String): NuvioSyncState = mutex.withLock {
        readUnlocked(scope)
    }

    internal suspend fun replaceLibraryBootstrap(
        scope: String,
        items: List<NuvioLibraryItemDto>,
        cursor: Long,
    ): NuvioSyncState {
        val state = read(scope)
        return replaceLibraryCore(
            scope,
            NuvioCoreBridge.applyDeltaSync(
                resource = "library",
                state = state.libraryCoreState(gson),
                snapshot = gson.toJsonTree(items),
                snapshotCursor = cursor,
                events = JsonArray(),
            ),
        )
    }

    internal suspend fun replaceHistoryBootstrap(
        scope: String,
        items: List<NuvioWatchedItemDto>,
        cursor: Long,
    ): NuvioSyncState {
        val state = read(scope)
        return replaceHistoryCore(
            scope,
            NuvioCoreBridge.applyDeltaSync(
                resource = "history",
                state = state.historyCoreState(gson),
                snapshot = gson.toJsonTree(items),
                snapshotCursor = cursor,
                events = JsonArray(),
            ),
        )
    }

    /** Snapshot-only fallback. It remains uninitialized so the next healthy call bootstraps with a cursor fence. */
    internal suspend fun cacheLibrarySnapshot(scope: String, items: List<NuvioLibraryItemDto>): NuvioSyncState =
        update(scope) { state -> state.copy(library = items) }

    internal suspend fun cacheProgressSnapshot(scope: String, items: List<NuvioWatchProgressDto>): NuvioSyncState =
        update(scope) { state -> state.copy(progress = items) }

    internal suspend fun cacheHistorySnapshot(scope: String, items: List<NuvioWatchedItemDto>): NuvioSyncState =
        update(scope) { state -> state.copy(history = items) }

    internal suspend fun applyLibraryEvents(
        scope: String,
        events: List<NuvioLibraryDeltaEventDto>,
    ): NuvioSyncState {
        val state = read(scope)
        return replaceLibraryCore(
            scope,
            NuvioCoreBridge.applyDeltaSync(
                resource = "library",
                state = state.libraryCoreState(gson),
                snapshot = JsonArray(),
                snapshotCursor = null,
                events = gson.toJsonTree(events),
            ),
        )
    }

    internal suspend fun replaceProgressCore(scope: String, coreState: JsonObject): NuvioSyncState = update(scope) { state ->
        state.copy(
            progressInitialized = coreState.get("initialized")?.asBoolean ?: state.progressInitialized,
            progressCursor = coreState.get("cursor")?.asLong?.coerceAtLeast(0L) ?: state.progressCursor,
            progress = coreState.getAsJsonArray("items")?.let { gson.fromJson(it, Array<NuvioWatchProgressDto>::class.java).toList() } ?: state.progress,
        )
    }

    internal suspend fun replaceLibraryCore(scope: String, coreState: JsonObject): NuvioSyncState = update(scope) { state ->
        state.copy(
            libraryInitialized = coreState.get("initialized")?.asBoolean ?: state.libraryInitialized,
            libraryCursor = coreState.get("cursor")?.asLong?.coerceAtLeast(0L) ?: state.libraryCursor,
            library = coreState.getAsJsonArray("items")?.let { gson.fromJson(it, Array<NuvioLibraryItemDto>::class.java).toList() } ?: state.library,
        )
    }

    internal suspend fun replaceHistoryCore(scope: String, coreState: JsonObject): NuvioSyncState = update(scope) { state ->
        state.copy(
            historyInitialized = coreState.get("initialized")?.asBoolean ?: state.historyInitialized,
            historyCursor = coreState.get("cursor")?.asLong?.coerceAtLeast(0L) ?: state.historyCursor,
            history = coreState.getAsJsonArray("items")?.let { gson.fromJson(it, Array<NuvioWatchedItemDto>::class.java).toList() } ?: state.history,
        )
    }

    internal suspend fun applyHistoryEvents(
        scope: String,
        events: List<NuvioWatchedItemDeltaEventDto>,
    ): NuvioSyncState {
        val state = read(scope)
        return replaceHistoryCore(
            scope,
            NuvioCoreBridge.applyDeltaSync(
                resource = "history",
                state = state.historyCoreState(gson),
                snapshot = JsonArray(),
                snapshotCursor = null,
                events = gson.toJsonTree(events),
            ),
        )
    }

    suspend fun clear(scope: String) {
        mutex.withLock { cache.remove(cacheKey(scope)) }
    }

    private suspend fun update(scope: String, transform: (NuvioSyncState) -> NuvioSyncState): NuvioSyncState =
        mutex.withLock {
            val next = transform(readUnlocked(scope)).copy(schemaVersion = SCHEMA_VERSION)
            cache.write(cacheKey(scope), gson.toJson(next))
            next
        }

    private suspend fun readUnlocked(scope: String): NuvioSyncState {
        val raw = cache.read(cacheKey(scope)) ?: return NuvioSyncState()
        return runCatching { gson.fromJson(raw, NuvioSyncState::class.java) }
            .getOrNull()
            ?.takeIf { it.schemaVersion == SCHEMA_VERSION }
            ?: NuvioSyncState()
    }

    private fun cacheKey(scope: String): String = "state:${scope.trim()}"

    private companion object {
        const val SCHEMA_VERSION = 1
    }
}

internal data class NuvioSyncState(
    val schemaVersion: Int = 1,
    val libraryInitialized: Boolean = false,
    val libraryCursor: Long = 0L,
    val library: List<NuvioLibraryItemDto> = emptyList(),
    val progressInitialized: Boolean = false,
    val progressCursor: Long = 0L,
    val progress: List<NuvioWatchProgressDto> = emptyList(),
    val historyInitialized: Boolean = false,
    val historyCursor: Long = 0L,
    val history: List<NuvioWatchedItemDto> = emptyList(),
)

data class NuvioProgressSyncResult(
    val progress: List<NuvioWatchProgressDto>,
    val continueWatching: List<NuvioWatchProgressDto>,
)

private fun NuvioSyncState.libraryCoreState(gson: Gson): JsonObject = JsonObject().apply {
    addProperty("initialized", libraryInitialized)
    addProperty("cursor", libraryCursor)
    add("items", gson.toJsonTree(library))
}

private fun NuvioSyncState.historyCoreState(gson: Gson): JsonObject = JsonObject().apply {
    addProperty("initialized", historyInitialized)
    addProperty("cursor", historyCursor)
    add("items", gson.toJsonTree(history))
}
