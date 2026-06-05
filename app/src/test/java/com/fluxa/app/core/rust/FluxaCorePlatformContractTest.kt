package com.fluxa.app.core.rust

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.AddonManifest
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.domain.discovery.StremioAddonUrls
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class FluxaCorePlatformContractTest {
    @Test
    fun stremioUrlContractIsOwnedByNativeCore() {
        assertEquals(
            "https://addon.example/manifest.json",
            StremioAddonUrls.normalizeManifestUrl("stremio://addon.example")
        )
        assertEquals(
            "http://127.0.0.1:7000/manifest.json",
            StremioAddonUrls.normalizeManifestUrl("127.0.0.1:7000")
        )
        assertEquals(
            listOf("http://127.0.0.1:7000/manifest.json"),
            StremioAddonUrls.manifestCandidates("127.0.0.1:7000")
        )
        assertEquals(
            "https://addon.example/",
            StremioAddonUrls.baseUrl("https://addon.example/manifest.json")
        )
    }

    @Test
    fun resourceUrlContractEncodesPathSegmentsWithoutDroppingExtras() {
        val url = FluxaCoreNative.buildResourceUrl(
            transportUrl = "https://addon.example/root/manifest.json",
            resource = "stream",
            type = "series",
            id = "tt123:1:2",
            extraArgs = mapOf(
                "search" to "one two",
                "genre" to "crime/drama",
                "skip" to null
            )
        )

        assertTrue(url.startsWith("https://addon.example/root/stream/series/tt123%3A1%3A2/"))
        assertTrue(url.endsWith(".json"))
        assertTrue(url.contains("search=one%20two"))
        assertTrue(url.contains("genre=crime%2Fdrama"))
        assertFalse(url.contains("skip="))
    }

    @Test
    fun episodeContractIsOwnedByNativeCore() {
        val locator = FluxaCoreNative.parseEpisodeLocator("tmdb:12345:2:7")

        assertEquals("tmdb:12345", locator?.baseId)
        assertEquals(2, locator?.season)
        assertEquals(7, locator?.episode)
        assertTrue(FluxaCoreNative.episodeTextMatches("Show.Name.S02E07.1080p.mkv", 2, 7))
        assertFalse(FluxaCoreNative.episodeTextMatches("Show.Name.S02E08.1080p.mkv", 2, 7))
    }

    @Test
    fun streamEpisodeMatcherDoesNotRankOrRewriteAddonResults() {
        assertTrue(
            FluxaCoreNative.streamMatchesEpisode(
                videoId = "tt123:1:2",
                title = "Provider item without explicit episode marker",
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
    fun offlineDownloadPlanningIsOwnedByNativeCore() {
        val plan = FluxaCoreNative.offlineDownloadPlan(
            meta = Meta(id = "tt1", name = "Movie: Name", type = "movie", poster = null),
            video = Video(id = "tt1:1:2", name = null, season = 1, number = 2, released = null, thumbnail = null),
            videoId = null,
            stream = Stream(name = null, title = "1080p", url = "https://cdn.example/video.mkv"),
            subtitleUrl = "https://sub.example/file.vtt",
            downloadId = "offline-id"
        )

        assertTrue(plan.supported)
        assertEquals("https://cdn.example/video.mkv", plan.playbackUrl)
        assertEquals("Movie Name S1 E2-offline-id.mkv", plan.videoFileName)
        assertEquals("Movie Name S1 E2-offline-id.vtt", plan.subtitleFileName)
        assertEquals("tt1:1:2", plan.videoId)

        val rejected = FluxaCoreNative.offlineDownloadPlan(
            meta = Meta(id = "tt1", name = "Movie", type = "movie", poster = null),
            video = null,
            videoId = null,
            stream = Stream(name = null, title = null, url = "https://cdn.example/master.m3u8"),
            subtitleUrl = null,
            downloadId = "offline-id"
        )

        assertFalse(rejected.supported)
        assertEquals("unsupported_source", rejected.reason)
    }

    @Test
    fun profileSafePrefsAreOwnedByNativeCore() {
        val profile = UserProfile(
            id = "p1",
            email = "guest",
            authKey = "",
            language = null,
            subtitleSize = 20f,
            preferredPlayer = "internal",
            cardLayout = "episode",
            continueWatchingLayout = "inherit",
            playerBufferCacheMb = 50,
            playerForwardBufferSeconds = 999,
            playerBackBufferSeconds = -1,
            detailEpisodeViewMode = "unknown",
            dolbyVisionFallbackMode = null,
            streamSourceSelectionMode = "invalid"
        )
        val prefs = FluxaCoreNative.profileSafePrefs(profile)

        assertEquals("en", prefs.language)
        assertEquals(100f, prefs.subtitleSizePercent)
        assertEquals("exoplayer", prefs.preferredPlayer)
        assertEquals("horizontal", prefs.cardLayout)
        assertEquals("horizontal", prefs.resolvedContinueWatchingLayout)
        assertEquals(100, prefs.playerBufferCacheMb)
        assertEquals(600L, prefs.playerForwardBufferSeconds)
        assertEquals(0L, prefs.playerBackBufferSeconds)
        assertEquals("modern", prefs.detailEpisodeViewMode)
        assertEquals("dv8", prefs.dolbyVisionFallbackMode)
        assertEquals("manual", prefs.streamSourceSelectionMode)
    }

    @Test
    fun appCoreStateReducerOwnsSharedStateTransitions() {
        FluxaCoreNative.createAppCoreState(
            mapOf(
                "homeSearch" to mapOf("searchHistory" to listOf(mapOf("id" to "tt1"))),
                "player" to mapOf(
                    "currentStreamIndex" to 3,
                    "lastSavedPosition" to 4100,
                    "playbackEnded" to true,
                    "isBuffering" to false
                )
            )
        ).use { state ->
            val homeSnapshot = state.dispatch(
                mapOf(
                    "type" to "setSearchResults",
                    "value" to listOf(
                        mapOf("id" to "tt2"),
                        mapOf("id" to "tt1")
                    )
                )
            )
            assertTrue(homeSnapshot.contains(""""searchResults":[{"id":"tt2"},{"id":"tt1"}]"""))

            val playerSnapshot = state.dispatch(
                mapOf(
                    "type" to "playerResetForEpisode",
                    "videoId" to "tt123:1:2"
                )
            )
            assertTrue(playerSnapshot.contains(""""currentVideoId":"tt123:1:2""""))
            assertTrue(playerSnapshot.contains(""""currentStreamIndex":0"""))
            assertTrue(playerSnapshot.contains(""""lastSavedPosition":0"""))
            assertTrue(playerSnapshot.contains(""""playbackEnded":false"""))
            assertTrue(playerSnapshot.contains(""""isBuffering":true"""))
        }
    }

    @Test
    fun repositoryFlowPoliciesAreOwnedByNativeCore() {
        val metaPlan = FluxaCoreNative.repositoryMetaDetailPlan(
            useConfiguredAddons = true,
            authKey = "",
            localAddons = listOf("https://addon.example/manifest.json")
        )
        assertTrue(metaPlan.preferAddonMetaDetail)
        assertTrue(metaPlan.fallbackToStremioMetaDetail)

        val videos = FluxaCoreNative.repositorySeasonVideos(
            MetaDetail(
                id = "tt1",
                type = "series",
                name = "Show",
                genres = null,
                poster = null,
                background = null,
                logo = null,
                description = null,
                releaseInfo = null,
                runtime = null,
                videos = listOf(
                    Video(id = "s2e1", name = null, season = 2, number = 1, released = null, thumbnail = null),
                    Video(id = "s1e1", name = null, season = 1, number = 1, released = null, thumbnail = null),
                    Video(id = "s2e2", name = null, season = 2, number = 2, released = null, thumbnail = null)
                )
            ),
            seasonNumber = 2
        )
        assertEquals(listOf("s2e1", "s2e2"), videos.map { it.id })

        assertEquals(
            "memory",
            FluxaCoreNative.manifestFetchDecision(forceRefresh = false, memoryHit = true, persistentHit = true).phase
        )
        assertEquals(
            "fetch",
            FluxaCoreNative.manifestFetchDecision(forceRefresh = true, memoryHit = true, persistentHit = true).phase
        )
    }

    @Test
    fun addonResourceRequestAndStreamProviderPoliciesAreNative() {
        val subtitles = FluxaCoreNative.addonResourceRequestPlan(
            transportUrl = "https://addon.example/manifest.json",
            resource = "subtitles",
            type = "movie",
            id = "tt1",
            extraRaw = "videoHash=abc&filename=File Name.mkv"
        )
        assertEquals("https://addon.example/subtitles/movie/tt1.json", subtitles.urls.first())
        assertTrue(subtitles.urls.last().contains("filename=File%20Name.mkv"))

        val catalog = FluxaCoreNative.addonResourceRequestPlan(
            transportUrl = "https://addon.example/manifest.json",
            resource = "catalog",
            type = "movie",
            id = "top",
            extraArgs = mapOf("search" to "matrix", "skip" to "100")
        )
        assertTrue(catalog.urls.single().contains("catalog/movie/top/"))
        assertTrue(catalog.urls.single().contains("search=matrix"))
        assertTrue(catalog.urls.single().contains("skip=100"))

        val streamsJson = FluxaCoreNative.addonStreamsWithProvider(
            streamsJson = """[{"title":"A","behaviorHints":{"videoHash":"abc","videoSize":12,"proxyHeaders":{"request":{"X-Proxy":"1"}}}},{"title":"B"}]""",
            addonName = "Provider"
        )
        assertTrue(streamsJson.contains(""""addonName":"Provider""""))
        assertTrue(streamsJson.contains(""""videoHash":"abc""""))
        assertTrue(streamsJson.contains(""""X-Proxy":"1""""))
        assertTrue(streamsJson.indexOf(""""title":"A"""") < streamsJson.indexOf(""""title":"B""""))

        val subtitlesJson = FluxaCoreNative.normalizeAddonSubtitles(
            subtitlesJson = """[{"id":"one","url":"/subs/one.vtt","lang":"en","attributes":{"languages":[]}},{"id":"two","attributes":{"url":"two.srt"}}]""",
            resourceUrl = "https://addon.example/subtitles/movie/tt1.json"
        )
        assertTrue(subtitlesJson.indexOf(""""id":"one"""") < subtitlesJson.indexOf(""""id":"two""""))
        assertTrue(subtitlesJson.contains(""""url":"https://addon.example/subs/one.vtt""""))
        assertTrue(subtitlesJson.contains(""""languages":["en"]"""))
        assertTrue(subtitlesJson.contains(""""url":"https://addon.example/subtitles/movie/two.srt""""))
    }

    @Test
    fun streamDiscoveryExecutionPolicyIsNative() {
        val addonA = AddonDescriptor(
            transportUrl = "https://a.example/manifest.json",
            manifest = AddonManifest(
                id = "a",
                name = "A",
                resources = listOf("stream"),
                types = listOf("movie"),
                catalogs = emptyList()
            )
        )
        val addonB = AddonDescriptor(
            transportUrl = "https://b.example/manifest.json",
            manifest = AddonManifest(
                id = "b",
                name = "B",
                resources = listOf("stream"),
                types = listOf("movie"),
                catalogs = emptyList()
            )
        )
        val policy = FluxaCoreNative.streamDiscoveryExecutionPolicy(
            type = "movie",
            id = "tt1",
            language = "tr",
            preferFastStart = false,
            addonRequestTimeoutMs = 1000,
            fastAddonRequestTimeoutMs = 500,
            cloudstreamTimeoutMs = 2000,
            addons = listOf(addonA, addonB),
            cs3PluginNames = listOf("cs"),
            cs3SearchQuery = "Movie",
            cs3OriginalName = null,
            cs3Year = 2024
        )

        assertEquals("movie|tt1|tr", policy.cacheLookupPrefix)
        assertEquals("movie|tt1|tr", FluxaCoreNative.streamDiscoveryCachePrefix("movie", "tt1", "tr"))
        // Concurrency now computed by Rust as min(addon_count, 32) when input ≤ 0.
        // Two addons → 2.
        assertEquals(2L, policy.maxConcurrentAddonRequests)
        assertEquals(1L, policy.cacheWriteMinimumResultCount)
        assertTrue(policy.emitCachedResult)
        assertTrue(policy.emitPartialNonEmptyResults)
        assertEquals(listOf("A", "B"), policy.addonRequests.map { it.addonName })
        assertEquals("Movie", policy.cloudstreamRequest?.title)
    }

    @Test
    fun nativeCoreSurfaceDoesNotExposeAddonResultMutationApis() {
        val publicMethodNames = FluxaCoreNative::class.java.methods.map { it.name.lowercase() }
        val mutationWords = listOf("rank", "sort", "reorder", "filterstreams", "filtermeta")

        assertFalse(publicMethodNames.any { method -> mutationWords.any(method::contains) })
    }

    @Test
    fun playerFlowReturnsEffectsAndSelectsStreamsWithoutReorderingProviderResults() {
        val requested = FluxaCoreNative.playerFlowDispatch(
            state = emptyMap<String, Any?>(),
            action = mapOf(
                "type" to "loadStreamsRequested",
                "contentType" to "movie",
                "id" to "tt1",
                "currentVideoId" to "tt1",
                "initialVideoId" to null,
                "initialStreams" to emptyList<Stream>(),
                "initialStreamIndex" to 1
            )
        )

        assertTrue(requested.state.isBuffering)
        assertEquals("loadStreams", requested.effects.single().type)
        assertFalse(requested.effects.single().useInitialStreams)

        val loaded = FluxaCoreNative.playerFlowDispatch(
            state = requested.state,
            action = mapOf(
                "type" to "streamsLoaded",
                "streams" to listOf(
                    Stream(name = "Provider", title = "A", url = "http://a"),
                    Stream(name = "Provider", title = "B", url = "http://b")
                ),
                "currentVideoId" to "tt1",
                "initialStreamIndex" to 1,
                "sourceSelectionMode" to "manual"
            )
        )

        assertEquals(1, loaded.state.currentStreamIndex)
        assertEquals("http://b", loaded.state.currentUrl)
        assertEquals("A", loaded.state.currentStreams[0].title)
        assertEquals("B", loaded.state.currentStreams[1].title)
    }

    @Test
    fun headlessEngineEmitsEffectsAndAcceptsEffectCompletion() {
        FluxaCoreNative.createHeadlessEngine().use { engine ->
            val requested = engine.dispatch(
                mapOf(
                    "type" to "detailLoadRequested",
                    "contentType" to "movie",
                    "id" to "tt1",
                    "language" to "en"
                )
            )

            assertEquals("fetchMetaDetail", requested.effects[0].type)
            assertEquals("readPlaybackProgress", requested.effects[1].type)

            val completed = engine.completeEffect(
                mapOf(
                    "effectId" to requested.effects[0].id,
                    "status" to "ok",
                    "value" to mapOf("id" to "tt1", "name" to "Movie")
                )
            )

            val detail = completed.state["detail"] as Map<*, *>
            val meta = detail["meta"] as Map<*, *>
            assertEquals(false, detail["isLoading"])
            assertEquals("Movie", meta["name"])
        }
    }

    @Test
    fun headlessEngineOwnsDetailSelectedAddonVisibleStreamsWithoutMutatingStreams() {
        FluxaCoreNative.createHeadlessEngine().use { engine ->
            val requested = engine.dispatch(
                mapOf(
                    "type" to "detailStreamsRequested",
                    "contentType" to "movie",
                    "requestIds" to listOf("tt1"),
                    "detail" to null,
                    "seasonEpisodes" to emptyList<Video>(),
                    "language" to "en"
                )
            )
            val completed = engine.completeEffect(
                mapOf(
                    "effectId" to requested.effects.single().id,
                    "status" to "ok",
                    "value" to mapOf(
                        "streams" to listOf(
                            mapOf("title" to "A", "addonName" to "One"),
                            mapOf("title" to "B", "addonName" to "Two"),
                            mapOf("title" to "C", "addonName" to "One")
                        ),
                        "availableAddons" to listOf("One", "Two"),
                        "hasStreamProviders" to true
                    )
                )
            )
            val loadedDetail = completed.state["detail"] as Map<*, *>
            assertEquals(3, (loadedDetail["streams"] as List<*>).size)
            assertEquals(3, (loadedDetail["visibleStreams"] as List<*>).size)

            val selected = engine.dispatch(
                mapOf(
                    "type" to "detailSelectedAddonChanged",
                    "addon" to "one"
                )
            )
            val selectedDetail = selected.state["detail"] as Map<*, *>
            assertEquals(3, (selectedDetail["streams"] as List<*>).size)
            val visible = selectedDetail["visibleStreams"] as List<*>
            assertEquals(2, visible.size)
            assertEquals("A", (visible[0] as Map<*, *>)["title"])
            assertEquals("C", (visible[1] as Map<*, *>)["title"])
        }
    }

    @Test
    fun headlessEngineOwnsPlaybackResolveEffects() {
        FluxaCoreNative.createHeadlessEngine().use { engine ->
            val torrent = engine.dispatch(
                mapOf(
                    "type" to "playerResolvePlaybackRequested",
                    "url" to "stremio://torrent/abc",
                    "stream" to mapOf("title" to "Torrent"),
                    "currentVideoId" to "tt1",
                    "title" to "Movie"
                )
            )
            assertEquals("startTorrentStream", torrent.effects.single().type)

            val completed = engine.completeEffect(
                mapOf(
                    "effectId" to torrent.effects.single().id,
                    "status" to "ok",
                    "value" to mapOf("url" to "http://127.0.0.1:8090/stream")
                )
            )
            val player = completed.state["player"] as Map<*, *>
            assertEquals("http://127.0.0.1:8090/stream", player["resolvedUrl"])

            val direct = engine.dispatch(
                mapOf(
                    "type" to "playerResolvePlaybackRequested",
                    "url" to "https://video.example/file.mp4",
                    "title" to "Movie"
                )
            )
            val directPlayer = direct.state["player"] as Map<*, *>
            assertEquals("stopTorrent", direct.effects.single().type)
            assertEquals("https://video.example/file.mp4", directPlayer["resolvedUrl"])
        }
    }

    @Test
    fun headlessEngineOwnsHomeAndLibraryStorageEffects() {
        FluxaCoreNative.createHeadlessEngine(
            mapOf("profile" to mapOf("activeProfileId" to "p1"))
        ).use { engine ->
            val home = engine.dispatch(
                mapOf(
                    "type" to "homeLoadRequested",
                    "profile" to mapOf("id" to "p1"),
                    "language" to "tr",
                    "force" to true
                )
            )
            assertEquals("readHomeBootstrap", home.effects.single().type)
            assertEquals("p1", home.effects.single().payload["profileId"])
            assertEquals("tr", home.effects.single().payload["language"])

            val library = engine.dispatch(
                mapOf(
                    "type" to "toggleWatchlistRequested",
                    "item" to mapOf("id" to "tt1", "name" to "Movie", "type" to "movie")
                )
            )
            assertEquals("writeLibraryCommand", library.effects.single().type)
            assertEquals("p1", library.effects.single().payload["profileId"])

            val progress = engine.dispatch(
                mapOf(
                    "type" to "savePlaybackProgressRequested",
                    "meta" to mapOf("id" to "tt1", "name" to "Movie", "type" to "movie"),
                    "timeOffset" to -1,
                    "duration" to 100L,
                    "lastVideoId" to "tt1",
                    "lastStreamIndex" to 0
                )
            )
            val progressPayload = progress.effects.single().payload["progress"] as Map<*, *>
            assertEquals("writePlaybackProgress", progress.effects.single().type)
            assertEquals(0.0, progressPayload["timeOffset"])
        }
    }

    @Test
    fun headlessEngineOwnsWatchedSyncEffectPlanning() {
        FluxaCoreNative.createHeadlessEngine(
            mapOf("profile" to mapOf("activeProfileId" to "p1"))
        ).use { engine ->
            val result = engine.dispatch(
                mapOf(
                    "type" to "markWatchedRequested",
                    "seriesId" to "tt1",
                    "videoIds" to listOf("tt1:1:2", "tt1:1:2", ""),
                    "watched" to false,
                    "meta" to mapOf("id" to "tt1", "name" to "Show", "type" to "series"),
                    "episodes" to listOf(mapOf("id" to "tt1:1:2", "season" to 1, "number" to 2)),
                    "profile" to mapOf("id" to "p1", "authKey" to "auth", "isGuest" to false)
                )
            )

            assertEquals(listOf("writeLibraryCommand", "syncWatchedState"), result.effects.map { it.type })
            val writePayload = result.effects[0].payload["command"] as Map<*, *>
            assertEquals(listOf("tt1:1:2"), writePayload["videoIds"])
            assertEquals(false, result.effects[1].payload["watched"])
        }
    }

    @Test
    fun headlessEngineOwnsAppBackboneEffectPlanning() {
        FluxaCoreNative.createHeadlessEngine(
            mapOf("profile" to mapOf("activeProfileId" to "p1"))
        ).use { engine ->
            val addon = engine.dispatch(
                mapOf(
                    "type" to "addonInstallRequested",
                    "transportUrl" to "https://addon.example/manifest.json",
                    "forceRefresh" to true
                )
            )
            assertEquals("fetchAddonManifest", addon.effects.single().type)
            assertEquals("https://addon.example/manifest.json", addon.effects.single().payload["transportUrl"])

            val resource = engine.dispatch(
                mapOf(
                    "type" to "addonResourceRequested",
                    "transportUrl" to "https://addon.example/manifest.json",
                    "resource" to "stream",
                    "contentType" to "movie",
                    "id" to "tt1",
                    "extra" to mapOf("search" to "matrix")
                )
            )
            assertEquals("fetchAddonResource", resource.effects.single().type)
            assertEquals("stream", resource.effects.single().payload["resource"])

            val search = engine.dispatch(
                mapOf("type" to "searchRequested", "query" to "matrix", "language" to "en")
            )
            assertEquals("runSearch", search.effects.single().type)
            assertEquals("p1", search.effects.single().payload["profileId"])

            val discover = engine.dispatch(
                mapOf(
                    "type" to "discoverRequested",
                    "contentType" to "movie",
                    "filters" to mapOf("genre" to "sci-fi")
                )
            )
            assertEquals("runDiscover", discover.effects.single().type)

            val season = engine.dispatch(
                mapOf("type" to "detailSeasonRequested", "seriesId" to "tt1", "season" to 2)
            )
            assertEquals("fetchSeasonEpisodes", season.effects.single().type)

            val subtitles = engine.dispatch(
                mapOf(
                    "type" to "subtitleLoadRequested",
                    "stream" to mapOf("url" to "http://a"),
                    "contentType" to "movie",
                    "id" to "tt1",
                    "extraArgs" to "videoHash=abc"
                )
            )
            assertEquals("fetchSubtitles", subtitles.effects.single().type)

            val sync = engine.dispatch(
                mapOf("type" to "externalSyncRequested", "provider" to "trakt", "language" to "tr")
            )
            assertEquals("runExternalSync", sync.effects.single().type)

            val calendar = engine.dispatch(
                mapOf("type" to "calendarMonthRequested", "year" to 2026, "month" to 20)
            )
            assertEquals("readCalendarMonth", calendar.effects.single().type)
            assertEquals(12.0, calendar.effects.single().payload["month"])

            val offline = engine.dispatch(
                mapOf(
                    "type" to "offlineDownloadRequested",
                    "meta" to mapOf("id" to "tt1"),
                    "stream" to mapOf("url" to "http://a"),
                    "videoId" to "tt1"
                )
            )
            assertEquals("enqueueOfflineDownload", offline.effects.single().type)
        }
    }

    @Test
    fun coreCapabilitiesSeparatePortableBrainFromNativeRuntime() {
        val android = FluxaCoreNative.coreCapabilities(portable = false)
        val portable = FluxaCoreNative.coreCapabilities(portable = true)

        assertTrue(android.http)
        assertTrue(android.storage)
        assertTrue(android.player)
        assertTrue(android.plugins)
        assertTrue(android.torrent)
        assertTrue(android.localStream)

        assertTrue(portable.http)
        assertTrue(portable.storage)
        assertTrue(portable.player)
        assertFalse(portable.plugins)
        assertFalse(portable.torrent)
        assertFalse(portable.localStream)
    }
}
