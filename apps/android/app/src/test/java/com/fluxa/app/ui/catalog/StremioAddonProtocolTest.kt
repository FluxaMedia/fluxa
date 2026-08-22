package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.core.rust.FluxaCoreNative

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.net.ServerSocket
import kotlin.concurrent.thread

class StremioAddonProtocolTest {

    @Test
    fun addonAssetUrlsPreferHttpsForImages() {
        assertEquals(
            "https://hdhub.thevolecitor.qzz.io/logo.png",
            StremioAddonUrls.preferHttpsAssetUrl("http://hdhub.thevolecitor.qzz.io/logo.png")
        )
        assertEquals(
            "https://cdn.example/logo.png",
            StremioAddonUrls.preferHttpsAssetUrl("//cdn.example/logo.png")
        )
    }

    @Test
    fun stringResourceUsesManifestTypeAndPrefixFilters() {
        val addon = AddonDescriptor(
            manifest = AddonManifest(
                id = "org.test",
                name = "Test",
                resources = listOf("stream"),
                types = listOf("movie"),
                catalogs = emptyList(),
                idPrefixes = listOf("tt")
            ),
            transportUrl = "https://example.com/manifest.json"
        )

        assertTrue(addon.supportsStremioResource("stream", "movie", "tt1254207"))
        assertFalse(addon.supportsStremioResource("stream", "series", "tt1254207:1:1"))
        assertFalse(addon.supportsStremioResource("stream", "movie", "tmdb:123"))
    }

    @Test
    fun objectResourceCanOverrideTypeAndPrefixFilters() {
        val addon = AddonDescriptor(
            manifest = AddonManifest(
                id = "org.test",
                name = "Test",
                resources = listOf(
                    mapOf(
                        "name" to "stream",
                        "types" to listOf("series"),
                        "idPrefixes" to listOf("kitsu:")
                    )
                ),
                types = listOf("movie"),
                catalogs = emptyList(),
                idPrefixes = listOf("tt")
            ),
            transportUrl = "https://example.com/manifest.json"
        )

        assertTrue(addon.supportsStremioResource("stream", "series", "kitsu:42:1:2"))
        assertFalse(addon.supportsStremioResource("stream", "movie", "tt1254207"))
    }

    @Test
    fun resourceNameMatchesSingularAndPluralSdkNames() {
        val manifest = AddonManifest(
            id = "org.test",
            name = "Test",
            resources = listOf("subtitles"),
            types = listOf("movie"),
            catalogs = emptyList()
        )

        assertTrue(manifest.hasStremioResource("subtitle"))
        assertTrue(manifest.hasStremioResource("subtitles"))
    }

    @Test
    fun resourceNameMatchesMetadataAlias() {
        val addon = AddonDescriptor(
            manifest = AddonManifest(
                id = "org.test",
                name = "Test",
                resources = listOf("metadata"),
                types = listOf("movie"),
                catalogs = emptyList()
            ),
            transportUrl = "https://example.com/manifest.json"
        )

        assertTrue(addon.supportsStremioResource("meta", "movie", "tt1254207"))
    }

    @Test
    fun objectResourceAcceptsSingularTypeAndPrefixKeys() {
        val addon = AddonDescriptor(
            manifest = AddonManifest(
                id = "org.test",
                name = "Test",
                resources = listOf(
                    mapOf(
                        "name" to "stream",
                        "type" to "movie",
                        "idPrefix" to "tt"
                    )
                ),
                types = listOf("series"),
                catalogs = emptyList(),
                idPrefixes = listOf("kitsu:")
            ),
            transportUrl = "https://example.com/manifest.json"
        )

        assertTrue(addon.supportsStremioResource("stream", "movie", "tt1254207"))
        assertFalse(addon.supportsStremioResource("stream", "series", "kitsu:42:1:2"))
    }

    @Test
    fun catalogExtraHelpersSupportModernAndLegacyExtraDefinitions() {
        val modern = AddonCatalog(
            type = "movie",
            id = "modern",
            extra = listOf(CatalogExtra(name = "search", isRequired = true))
        )
        val legacy = AddonCatalog(
            type = "movie",
            id = "legacy",
            extraSupported = listOf("genre")
        )

        assertTrue(modern.supportsCatalogExtra("search"))
        assertTrue(modern.requiresCatalogExtra("search"))
        assertTrue(legacy.supportsCatalogExtra("genre"))
        assertFalse(legacy.requiresCatalogExtra("genre"))
    }

    @Test
    fun requiredCatalogExtrasCanBeScopedToAllowedRequest() {
        val catalog = AddonCatalog(
            type = "movie",
            id = "search-only",
            extra = listOf(
                CatalogExtra(name = "search", isRequired = true),
                CatalogExtra(name = "genre", isRequired = false)
            )
        )

        assertFalse(catalog.hasRequiredCatalogExtraExcept(setOf("search")))
        assertTrue(catalog.hasRequiredCatalogExtraExcept())
    }

    @Test
    fun manifestVersionDoesNotDefaultToSyntheticValue() {
        val manifest = AddonManifest(
            id = "org.test",
            name = "Test",
            resources = listOf("catalog"),
            types = listOf("movie"),
            catalogs = emptyList()
        )

        assertNull(manifest.version)
    }

    @Test
    fun addonCapabilityRowsMergeCatalogAndMetaResourcesForSameTypes() {
        val manifest = AddonManifest(
            id = "com.linvo.cinemeta",
            name = "Cinemeta",
            resources = listOf("catalog", "meta", "addon_catalog"),
            types = listOf("movie", "series"),
            catalogs = listOf(
                AddonCatalog(type = "movie", id = "top"),
                AddonCatalog(type = "series", id = "top")
            ),
            idPrefixes = listOf("tt")
        )

        val rows = addonManifestCapabilityRows(manifest, "en")

        assertEquals(1, rows.size)
        val resources = rows.single().split(" - ")[1].split(", ")
        assertEquals(listOf("catalog", "meta", "addon_catalog"), resources)
    }

    @Test
    fun nativeManifestParserKeepsSdkResourceObjectsAndCatalogExtras() {
        val body = """
            {
              "id": "org.test.native",
              "name": "Native Test",
              "resources": [
                {"name": "stream", "types": ["movie"], "idPrefixes": ["tt"]},
                "catalog"
              ],
              "types": ["movie", "series"],
              "catalogs": [
                {
                  "type": "movie",
                  "id": "top",
                  "name": "Top",
                  "extra": [
                    {"name": "search", "isRequired": true, "optionsLimit": 20},
                    {"name": "genre", "options": ["action", "drama"]}
                  ]
                }
              ],
              "behaviorHints": {
                "configurable": true
              },
              "logo": "/logo.png"
            }
        """.trimIndent()

        val descriptor = FluxaCoreNative.parseManifestJson(
            body = body,
            transportUrl = "https://addon.example/root/manifest.json",
            unknownName = "Unknown"
        )

        assertEquals("org.test.native", descriptor?.manifest?.id)
        assertEquals("Native Test", descriptor?.manifest?.name)
        assertEquals(true, descriptor?.manifest?.configurable)
        assertEquals("https://addon.example/logo.png", descriptor?.manifest?.logo)
        assertTrue(descriptor!!.supportsStremioResource("stream", "movie", "tt123"))
        assertFalse(descriptor.supportsStremioResource("stream", "series", "tt123"))
        val catalog = descriptor.manifest.catalogs.orEmpty().single()
        assertTrue(catalog.requiresCatalogExtra("search"))
        assertTrue(catalog.supportsCatalogExtra("genre"))
    }

    @Test
    fun nativeResourceUrlBuilderEncodesExtraPathSegments() {
        val url = FluxaCoreNative.buildResourceUrl(
            transportUrl = "https://addon.example/manifest.json",
            resource = "catalog",
            type = "movie",
            id = "top movies",
            extraArgs = mapOf(
                "search" to "breaking bad",
                "genre" to "crime/drama",
                "skip" to null
            )
        )

        assertTrue(url.startsWith("https://addon.example/catalog/movie/top%20movies/"))
        assertTrue(url.endsWith(".json"))
        assertTrue(url.contains("search=breaking%20bad"))
        assertTrue(url.contains("genre=crime%2Fdrama"))
        assertFalse(url.contains("skip="))
    }

    @Test
    fun nativeEpisodeMatcherReadsStremioAndFilenameLocators() {
        val locator = FluxaCoreNative.parseEpisodeLocator("tmdb:12345:2:7")

        assertEquals("tmdb:12345", locator?.baseId)
        assertEquals(2, locator?.season)
        assertEquals(7, locator?.episode)
        assertTrue(FluxaCoreNative.episodeTextMatches("Show.Name.S02E07.1080p.mkv", 2, 7))
        assertTrue(FluxaCoreNative.episodeTextMatches("Show Name Season 02 Episode 07", 2, 7))
        assertFalse(FluxaCoreNative.episodeTextMatches("Show.Name.S02E08.1080p.mkv", 2, 7))
    }

    @Test
    fun nativeStreamMatcherRejectsWrongEpisodeWithoutReorderingResults() {
        assertTrue(
            FluxaCoreNative.streamMatchesEpisode(
                videoId = "tt123:1:2",
                title = "Show S01E02",
                name = null,
                description = null,
                filename = null,
                effectiveFilename = null
            )
        )
        assertFalse(
            FluxaCoreNative.streamMatchesEpisode(
                videoId = "tt123:1:2",
                title = "Show S01E03",
                name = null,
                description = null,
                filename = null,
                effectiveFilename = null
            )
        )
    }


    @Test
    fun streamBodyPrefersSdkDescriptionOverLegacyTitleAndNameLines() {
        val stream = Stream(
            name = "4KHDHub 1080p\nIgnored name line",
            title = "Legacy title",
            description = "[Download] Breaking Bad S01E02\nDownload | 4KHDHub",
            url = "https://example.com/video.mkv"
        )

        assertEquals("[Download] Breaking Bad S01E02\nDownload | 4KHDHub", stream.streamRawBody())
    }

    @Test
    fun streamReadsSubtitleHintsFromBehaviorHints() {
        val stream = Stream(
            name = "Source",
            title = null,
            url = "https://example.com/Breaking%20Bad.mkv",
            behaviorHints = mapOf(
                "videoHash" to "abc123",
                "videoSize" to 42L,
                "filename" to "Custom.mkv"
            )
        )

        assertEquals("abc123", stream.effectiveVideoHash)
        assertEquals(42L, stream.effectiveVideoSize)
        assertEquals("Custom.mkv", stream.effectiveFilename)
        assertEquals("videoHash=abc123&videoSize=42&filename=Custom.mkv", stream.subtitleExtraArgs())
    }

    @Test
    fun streamBuildsPlaybackUrlFromStremioSourceFields() {
        assertEquals(
            "https://www.youtube.com/watch?v=abc123",
            Stream(name = "YouTube", title = null, ytId = "abc123").playableUrl
        )
        assertEquals(
            "stremio://torrent/deadbeef/2",
            Stream(name = "Torrent", title = null, infoHash = "deadbeef", fileIdx = 2).playableUrl
        )
        assertTrue("stremio://torrent/deadbeef/2".isTorrentPlaybackUrl())
    }
}
