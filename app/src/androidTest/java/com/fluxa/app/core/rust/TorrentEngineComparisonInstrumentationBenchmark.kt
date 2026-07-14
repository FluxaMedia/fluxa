package com.fluxa.app.core.rust

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.fluxa.app.common.Constants
import com.fluxa.app.player.TorrentServerEngine
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder
import java.util.concurrent.TimeUnit
import org.json.JSONObject
import org.junit.After
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class TorrentEngineComparisonInstrumentationBenchmark {
    private val context = InstrumentationRegistry.getInstrumentation().targetContext
    private var torrentServerEngine: TorrentServerEngine? = null

    @After
    fun tearDown() {
        torrentServerEngine?.stop()
        FluxaStreamingNative.stopTorrentServer()
    }

    @Test
    fun compareTorrentServerAndRustEngineFirstRangeRead() {
        val args = InstrumentationRegistry.getArguments()
        val torrentLink = args.getString("torrentLink").orEmpty()
        assumeTrue("Pass -e torrentLink <magnet-or-torrent-url> to run this benchmark.", torrentLink.isNotBlank())

        val rustInfo = JSONObject(
            FluxaStreamingNative.startTorrentServer(
                cacheDir = context.cacheDir.resolve("rust_torrent_benchmark").absolutePath,
                preferredPort = 0
            )
        )
        val rustBaseUrl = rustInfo.getString("url")
        val rustUrl = torrentStreamUrl(rustBaseUrl, torrentLink)

        torrentServerEngine = TorrentServerEngine(context).also { it.start() }
        waitForHttp(Constants.LocalServer.TORRENT_SERVER_BASE_URL)
        val torrentServerUrl = torrentStreamUrl(Constants.LocalServer.TORRENT_SERVER_BASE_URL, torrentLink)

        val rustDelayMs = measureFirstRangeMs(rustUrl)
        val torrentServerDelayMs = measureFirstRangeMs(torrentServerUrl)

        println("torrent-rust-engine-first-range-ms=$rustDelayMs")
        println("torrent-torrserver-first-range-ms=$torrentServerDelayMs")
        assertTrue(rustDelayMs > 0)
        assertTrue(torrentServerDelayMs > 0)
    }

    private fun torrentStreamUrl(baseUrl: String, torrentLink: String): String {
        return "$baseUrl/stream/fname?link=${URLEncoder.encode(torrentLink, "UTF-8")}&title=benchmark"
    }

    private fun measureFirstRangeMs(url: String): Long {
        val start = System.nanoTime()
        val connection = URL(url).openConnection() as HttpURLConnection
        connection.connectTimeout = 30_000
        connection.readTimeout = 120_000
        connection.setRequestProperty("Range", "bytes=0-262143")
        val bytes = connection.inputStream.use { stream ->
            val buffer = ByteArray(32 * 1024)
            var total = 0
            while (total < 256 * 1024) {
                val read = stream.read(buffer)
                if (read <= 0) break
                total += read
            }
            total
        }
        assertTrue("Expected at least one torrent byte from $url", bytes > 0)
        return (System.nanoTime() - start) / 1_000_000L
    }

    private fun waitForHttp(baseUrl: String) {
        val deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(20)
        while (System.nanoTime() < deadline) {
            runCatching {
                val connection = URL(baseUrl).openConnection() as HttpURLConnection
                connection.connectTimeout = 1_000
                connection.readTimeout = 1_000
                connection.inputStream.close()
                return
            }
            Thread.sleep(250)
        }
        error("Timed out waiting for $baseUrl")
    }
}
