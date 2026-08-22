package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.DetailTrailer
import com.fluxa.app.shared.feature.player.JvmTrailerPlaybackResolver
import org.junit.Assert.assertEquals
import org.junit.Test

class TrailerSelectionPolicyTest {
    @Test
    fun selectsTheHighestDeclaredQualityDirectTrailer() {
        val trailers = listOf(
            trailer("Plex 1080p", "https://video.example/plex.mp4"),
            trailer("MUBI 720p", "https://video.example/mubi.mp4"),
            trailer("IMDb 4K", "https://video.example/imdb.m3u8")
        )

        assertEquals("https://video.example/imdb.m3u8", selectBestDirectTrailerUrl(trailers))
    }

    @Test
    fun desktopQualityCapPrefers1080pOver4kTrailerioVariant() {
        val trailers = listOf(
            trailer("IMDb 4K", "https://video.example/imdb-4k.m3u8"),
            trailer("Trailerio 1080p", "https://video.example/trailerio-1080.mp4"),
            trailer("Trailerio 720p", "https://video.example/trailerio-720.mp4")
        )

        assertEquals(
            "https://video.example/trailerio-1080.mp4",
            JvmTrailerPlaybackResolver.selectBestDirectTrailerUrl(trailers, maxHeight = 1080)
        )
    }

    @Test
    fun keepsAddonOrderWhenDirectTrailerQualityIsEqual() {
        val first = trailer("First 1080p", "https://video.example/first.mp4")
        val second = trailer("Second 1080p", "https://video.example/second.mp4")

        assertEquals(first.url, selectBestDirectTrailerUrl(listOf(first, second)))
    }

    private fun trailer(title: String, url: String) = DetailTrailer(
        id = url,
        title = title,
        type = "Trailer",
        url = url,
        source = "Trailerio"
    )
}
