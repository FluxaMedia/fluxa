package com.fluxa.app.shared.feature.localmedia

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

class LocalMediaFilenameParserTest {
    @Test
    fun parsesMovieReleaseNoiseAndYear() {
        val parsed = LocalMediaFilenameParser.parse(
            "Dune.Part.Two.2024.2160p.BluRay.REMUX.DV.mkv",
            parentHints = listOf("Dune Part Two (2024)", "Movies"),
            kind = LocalMediaKind.Movies,
        )

        assertNotNull(parsed)
        assertEquals("Dune Part Two", parsed.title)
        assertEquals(2024, parsed.year)
    }

    @Test
    fun seriesUsesShowFolderInsteadOfSeasonFolder() {
        val parsed = LocalMediaFilenameParser.parse(
            "Breaking.Bad.S02E05.1080p.WEB-DL.mkv",
            parentHints = listOf("Season 02", "Breaking Bad", "TV Shows"),
            kind = LocalMediaKind.TvShows,
        )

        assertNotNull(parsed)
        assertEquals("Breaking Bad", parsed.title)
        assertEquals(2, parsed.season)
        assertEquals(5, parsed.episode)
    }

    @Test
    fun flatSeriesRootFallsBackToFilenameTitle() {
        val parsed = LocalMediaFilenameParser.parse(
            "Severance.S01E03.2160p.WEB-DL.mkv",
            parentHints = listOf("TV Shows", "Media"),
            kind = LocalMediaKind.TvShows,
        )

        assertNotNull(parsed)
        assertEquals("Severance", parsed.title)
        assertEquals(1, parsed.season)
        assertEquals(3, parsed.episode)
    }

    @Test
    fun animeAbsoluteEpisodeUsesAnimeFolderTitle() {
        val parsed = LocalMediaFilenameParser.parse(
            "[SubsPlease] Jujutsu Kaisen - 27 (1080p).mkv",
            parentHints = listOf("Jujutsu Kaisen", "Anime"),
            kind = LocalMediaKind.Anime,
        )

        assertNotNull(parsed)
        assertEquals("Jujutsu Kaisen", parsed.title)
        assertEquals(27, parsed.absoluteEpisode)
    }

    @Test
    fun explicitImdbIdIsPreserved() {
        val parsed = LocalMediaFilenameParser.parse(
            "The.Matrix.1999.[imdb-tt0133093].mkv",
            parentHints = listOf("Movies"),
            kind = LocalMediaKind.Movies,
        )

        assertNotNull(parsed)
        assertEquals("tt0133093", parsed.explicitMetadataId)
        assertEquals("imdb", parsed.explicitMetadataProvider)
    }
}
