package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject

object NuvioCoreBridge {
    fun buildLocalProfiles(
        sessionProfile: JsonObject,
        nuvioProfiles: JsonElement,
        avatarCatalog: JsonElement,
        existingProfiles: JsonElement
    ): JsonArray = invoke(
        "nuvioBuildLocalProfiles",
        JsonObject().apply {
            add("sessionProfile", sessionProfile)
            add("nuvioProfiles", nuvioProfiles)
            add("avatarCatalog", avatarCatalog)
            add("existingProfiles", existingProfiles)
        }
    ).asJsonArray

    fun libraryToWatchlist(library: JsonElement): JsonArray = invoke(
        "nuvioLibraryToWatchlist",
        JsonObject().apply { add("library", library) }
    ).asJsonArray

    fun progressMetaNeeds(watchProgress: JsonElement, library: JsonElement): JsonArray = invoke(
        "nuvioProgressMetaNeeds",
        JsonObject().apply {
            add("watchProgress", watchProgress)
            add("library", library)
        }
    ).asJsonArray

    fun progressSyncRequestPlan(state: JsonObject): JsonObject = invoke(
        "nuvioProgressSyncRequestPlan",
        JsonObject().apply { add("state", state) }
    ).asJsonObject

    fun applyProgressSync(
        state: JsonObject,
        snapshot: JsonElement,
        snapshotCursor: Long?,
        events: JsonElement,
    ): JsonObject = invoke(
        "nuvioApplyProgressSync",
        JsonObject().apply {
            add("state", state)
            add("snapshot", snapshot)
            snapshotCursor?.let { addProperty("snapshotCursor", it) }
            add("events", events)
        }
    ).asJsonObject

    fun deltaSyncRequestPlan(state: JsonObject): JsonObject = invoke(
        "nuvioDeltaSyncRequestPlan",
        JsonObject().apply { add("state", state) },
    ).asJsonObject

    fun applyDeltaSync(
        resource: String,
        state: JsonObject,
        snapshot: JsonElement,
        snapshotCursor: Long?,
        events: JsonElement,
    ): JsonObject = invoke(
        "nuvioApplyDeltaSync",
        JsonObject().apply {
            addProperty("resource", resource)
            add("state", state)
            add("snapshot", snapshot)
            snapshotCursor?.let { addProperty("snapshotCursor", it) }
            add("events", events)
        },
    ).asJsonObject

    fun importMergePlan(
        library: JsonElement,
        addonMetas: JsonObject,
        watchProgress: JsonElement,
        watchHistory: JsonElement
    ): JsonObject = invoke(
        "nuvioImportMergePlan",
        JsonObject().apply {
            add("progress", JsonObject())
            add("watched", JsonObject())
            add("library", library)
            add("addonMetas", addonMetas)
            add("watchProgress", watchProgress)
            add("watchHistory", watchHistory)
        }
    ).asJsonObject

    fun mapCollections(collections: JsonElement): JsonArray = invoke(
        "nuvioMapCollections",
        JsonObject().apply { add("collections", collections) }
    ).asJsonArray

    fun libraryMutationPlan(remote: JsonElement, item: JsonElement, command: String, nowMs: Long): JsonElement? {
        val result = invoke(
            "nuvioLibraryMutationPlan",
            JsonObject().apply {
                add("remote", remote)
                add("item", item)
                addProperty("command", command)
                addProperty("nowMs", nowMs)
            }
        )
        return result.takeUnless { it.isJsonNull }
    }

    private fun invoke(method: String, args: JsonObject): JsonElement =
        FluxaCoreUniFfi.coreInvokeValue(method, args.toString())
}
