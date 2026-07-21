package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.MetaDetailResponse
import com.fluxa.app.data.remote.decodeMetaDetailPayload
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

        val detail = decodeMetaDetailPayload(json)!!

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

        val detail = decodeMetaDetailPayload(json)!!

        assertEquals(
            listOf("Nicolas Cage", "Pedro Pascal", "Bella Ramsey"),
            detail.cast!!.map { it.name }
        )
    }

    @Test
    fun metaDetailParsesTrailerioDirectVideoLinksAsTrailers() {
        val detail = decodeMetaDetailPayload(
            """
                {
                  "meta": {
                    "id": "tt0944947",
                    "name": "Game of Thrones",
                    "type": "series",
                    "links": [
                      {
                        "trailers": "https://video.fandango.com/trailer.mp4",
                        "provider": "Rotten Tomatoes 1080p"
                      },
                      {
                        "trailers": "https://imdb-video.media-imdb.com/trailer.m3u8",
                        "provider": "IMDb SD"
                      }
                    ]
                  }
                }
            """.trimIndent()
        )!!

        assertEquals(2, detail.trailers!!.size)
        assertEquals("Rotten Tomatoes 1080p", detail.trailers!![0].title)
        assertEquals("https://imdb-video.media-imdb.com/trailer.m3u8", detail.trailers!![1].url)
    }
}
