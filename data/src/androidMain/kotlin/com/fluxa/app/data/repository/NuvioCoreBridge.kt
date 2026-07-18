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

    private fun invoke(method: String, args: JsonObject): JsonElement =
        FluxaCoreUniFfi.coreInvokeValue(method, args.toString())
}
