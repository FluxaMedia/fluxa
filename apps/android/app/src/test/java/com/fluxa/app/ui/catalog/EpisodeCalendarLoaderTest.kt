package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.google.gson.Gson
import com.google.gson.JsonObject
import com.google.gson.reflect.TypeToken
import org.junit.Assert.assertEquals
import org.junit.Test

class EpisodeCalendarLoaderTest {
    @Test
    fun calendarCandidatesMergeLibraryContinueWatchingAndProviders() {
        val library = Meta(id = "tt1", name = "Library", type = "series")
        val continueWatching = library.copy(name = "Continue watching")
        val anilist = Meta(id = "anilist:2", name = "Anime", type = "series", reason = "AniList")
        val nuvio = Meta(id = "tt3", name = "Nuvio", type = "series", reason = "Nuvio")

        val gson = Gson()
        val request = JsonObject().apply {
            add("groups", gson.toJsonTree(listOf(listOf(library), listOf(continueWatching, anilist, nuvio))))
        }
        val result: List<Meta> = gson.fromJson(
            FluxaCoreUniFfi.coreInvokeValue("calendarCandidatePlan", request.toString()),
            object : TypeToken<List<Meta>>() {}.type
        )

        assertEquals(listOf("tt1", "anilist:2", "tt3"), result.map { it.id })
    }
}
