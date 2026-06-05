package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.MetaDetailResponse
import com.google.gson.GsonBuilder
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class StremioAddonResourceClientTest {

    @Test
    fun aiometadataMetaDetailKeepsEpisodesTrailersAndNullableSeasonPosters() {
        val json = """
            {
              "meta": {
                "id": "tt30460310",
                "name": "Spider-Noir",
                "type": "series",
                "genres": ["Crime"],
                "poster": "poster",
                "background": "background",
                "description": "description",
                "director": "",
                "releaseInfo": "2026-",
                "runtime": "44min",
                "videos": [
                  {
                    "id": "tt30460310:1:1",
                    "title": "Step Into My Office",
                    "season": 1,
                    "episode": 1,
                    "thumbnail": "thumb",
                    "overview": "It won't end the way you want it to.",
                    "released": "2026-05-25T08:00:00.000Z"
                  }
                ],
                "trailers": [
                  { "source": "HgMbkitzhEM", "type": "Trailer", "name": "Spider-Noir" }
                ],
                "app_extras": {
                  "seasonPosters": [null, "season-one-poster"]
                }
              }
            }
        """.trimIndent()

        val detail = GsonBuilder().create()
            .fromJson(json, MetaDetailResponse::class.java)
            .meta!!

        val episode = detail.videos!!.single()
        assertEquals("Step Into My Office", episode.name)
        assertEquals(1, episode.season)
        assertEquals(1, episode.number)
        assertNull(detail.director)
        assertEquals("https://www.youtube.com/watch?v=HgMbkitzhEM", detail.trailers!!.single().url)

        val posters = detail.appExtras!!.seasonPosters!!
        assertNull(posters[0])
        assertEquals(mapOf("1" to "season-one-poster"), mapAppExtraSeasonPosters(detail.videos.orEmpty(), 1, posters))
    }

    @Test
    fun metaDetailCastParserKeepsFullActorNames() {
        val json = """
            {
              "meta": {
                "id": "tt-cast",
                "name": "Cast Test",
                "type": "movie",
                "cast": [
                  { "name": "Nicolas", "surname": "Cage" },
                  { "firstName": "Pedro", "lastName": "Pascal" },
                  { "name": { "first": "Bella", "last": "Ramsey" } }
                ]
              }
            }
        """.trimIndent()

        val detail = GsonBuilder().create()
            .fromJson(json, MetaDetailResponse::class.java)
            .meta!!

        assertEquals(
            listOf("Nicolas Cage", "Pedro Pascal", "Bella Ramsey"),
            detail.cast!!.map { it.name }
        )
    }
}
