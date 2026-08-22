package com.fluxa.app.data.plugins

import kotlin.test.Test
import kotlin.test.assertEquals

class NuvioPluginContentIdsTest {
    @Test
    fun stripsTmdbPrefixAndEpisodeSuffixLikeNuvio() {
        assertEquals("60625", nuvioPluginContentId("tmdb:60625:3:7", season = 3, episode = 7))
        assertEquals("60625", nuvioPluginContentId("tmdb/60625/anything", season = null, episode = null))
    }

    @Test
    fun preservesImdbIdWhenTmdbMappingIsUnavailable() {
        assertEquals("tt2861424", nuvioPluginContentId("tt2861424:1:3", season = 1, episode = 3))
    }

    @Test
    fun normalizesSeriesAliasesToTv() {
        assertEquals("tv", normalizeNuvioPluginType("series"))
        assertEquals("tv", normalizeNuvioPluginType("show"))
        assertEquals("movie", normalizeNuvioPluginType("film"))
    }
}
