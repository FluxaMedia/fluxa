package com.fluxa.app.core.rust

import com.fluxa.app.common.TtlMemoryCache
import com.fluxa.app.data.remote.AddonManifest
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.domain.discovery.StremioAddonUrls
import com.fluxa.app.domain.discovery.hasStremioResource
import com.fluxa.app.player.TorrentFileStat
import com.fluxa.app.player.TorrentStatus
import com.fluxa.app.player.TorrentCorePolicy
import java.net.HttpURLConnection
import java.net.ServerSocket
import java.net.Socket
import java.net.URL
import java.util.concurrent.Callable
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger
import kotlin.math.max
import kotlin.random.Random
import org.junit.After
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

class FluxaCoreBenchmarkTest {
    private val sessions = mutableListOf<FluxaLocalStreamServer.Session>()
    private val servers = mutableListOf<StaticByteServer>()

    @After
    fun tearDown() {
        sessions.forEach { it.stop() }
        servers.forEach { it.stop() }
    }

    @Test
    fun benchmarkKotlinCacheWithoutNativeCalls() {
        val cache = TtlMemoryCache<String>(
            maxEntries = 512,
            ttlMillis = 60_000L,
            nowMillis = { 1_000L }
        )

        val put = benchmark("kotlin-cache-put", iterations = 20_000) { index ->
            cache.put("stream|$index", """{"url":"https://cdn.example/$index.mp4"}""")
        }
        val get = benchmark("kotlin-cache-get", iterations = 20_000) { index ->
            assertNotNull(cache.get("stream|${20_000 - 512 + (index % 512)}"))
        }
        val prefix = benchmark("kotlin-cache-prefix", iterations = 1_000) { _ ->
            cache.firstWithPrefix("stream|")
        }

        assertEquals(512, cache.size())
        assertTrue(put.averageMicros > 0.0)
        assertTrue(get.averageMicros > 0.0)
        assertTrue(prefix.averageMicros > 0.0)
    }

    @Test
    fun benchmarkNativeParseAndStremioHelpers() {
        val manifestBody = """
            {
              "id": "org.fluxa.benchmark",
              "name": "Benchmark",
              "version": "1.0.0",
              "resources": ["catalog", "meta", "stream"],
              "types": ["movie", "series"],
              "idPrefixes": ["tt"],
              "catalogs": [
                {
                  "type": "movie",
                  "id": "top",
                  "name": "Top",
                  "extra": [{"name": "search", "isRequired": true}]
                }
              ],
              "logo": "/logo.png"
            }
        """.trimIndent()

        val parse = benchmark("native-manifest-parse", iterations = 2_000) { _ ->
            val descriptor = FluxaCoreNative.parseManifestJson(
                manifestBody,
                "https://addon.example/manifest.json",
                "Unknown"
            )
            assertEquals("org.fluxa.benchmark", descriptor?.manifest?.id)
        }
        val urlHelpers = benchmark("native-url-helpers", iterations = 20_000) { index ->
            assertEquals(
                "https://addon.example/manifest.json",
                StremioAddonUrls.normalizeManifestUrl("stremio://addon.example")
            )
            assertEquals(
                "https://addon.example/stream/movie/tt$index.json",
                FluxaCoreNative.buildResourceUrl(
                    "https://addon.example/manifest.json",
                    "stream",
                    "movie",
                    "tt$index",
                    emptyMap()
                )
            )
        }
        val supports = benchmark("native-resource-support", iterations = 20_000) { _ ->
            val manifest = AddonManifest(
                id = "org.fluxa.benchmark",
                name = "Benchmark",
                resources = listOf("stream"),
                types = listOf("movie"),
                catalogs = emptyList(),
                idPrefixes = listOf("tt")
            )
            assertTrue(manifest.hasStremioResource("stream"))
        }
        val sourceSelectionStreams = List(48) { index ->
            Stream(
                name = "${720 + index}p WEB-DL",
                title = "Show S01E02 Source $index",
                url = "https://cdn.example/$index.mkv",
                behaviorHints = mapOf("bingeGroup" to "group-$index")
            )
        }
        val sourceSelection = benchmark("native-stream-source-selection", iterations = 5_000) { _ ->
            assertEquals(
                47,
                FluxaCoreNative.selectStreamIndex(
                    streams = sourceSelectionStreams,
                    currentVideoId = "tt123:1:2",
                    initialStreamIndex = 0,
                    savedUrl = null,
                    savedTitle = null,
                    sourceSelectionMode = "regex",
                    regexPattern = "group-47",
                    preferredBingeGroup = null
                )
            )
        }
        val torrentStats = List(64) { index ->
            TorrentFileStat(
                id = index,
                path = if (index == 42) {
                    "Show/Show.S01E02.1080p.mkv"
                } else {
                    "Show/Show.S01E${(index % 10) + 10}.mkv"
                },
                length = 100_000L + index
            )
        }
        val torrentRuntime = benchmark("native-torrent-runtime-policy", iterations = 5_000) { _ ->
            val info = TorrentCorePolicy.plan(
                link = "stremio://torrent/ABCDEF1234567890ABCDEF1234567890ABCDEF12/4",
                title = "tt123:1:2",
                requestedFileIdx = null,
                preferredFilename = null,
                sources = listOf("tracker:udp://tracker.example:1337/announce"),
                fileStats = torrentStats
            )
            assertEquals(42, info.selectedFileIdx)
        }
        val torrentStatus = benchmark("native-torrent-status-policy", iterations = 20_000) { _ ->
            val info = TorrentCorePolicy.statusInfo(
                TorrentStatus(
                    hash = "hash",
                    title = "title",
                    downloadSpeed = 0.0,
                    activePeers = 2,
                    totalPeers = 4,
                    progress = 5.0,
                    stat = 1,
                    preload = 0,
                    loadedSize = 256L * 1024L,
                    preloadSize = 512L * 1024L,
                    fileStats = null
                )
            )
            assertEquals(50, info.bufferProgress)
        }
        assertTrue(parse.averageMicros > 0.0)
        assertTrue(urlHelpers.averageMicros > 0.0)
        assertTrue(supports.averageMicros > 0.0)
        assertTrue(sourceSelection.averageMicros > 0.0)
        assertTrue(torrentRuntime.averageMicros > 0.0)
        assertTrue(torrentStatus.averageMicros > 0.0)
    }

    @Test
    fun benchmarkRustLocalStreamServerDirectHttpReads() {
        val payload = ByteArray(2 * 1024 * 1024) { index -> (index % 251).toByte() }
        val upstreamRequests = AtomicInteger(0)
        val upstream = startStaticServer(payload, upstreamRequests)
        val upstreamUrl = "http://127.0.0.1:${upstream.port}/video.bin"

        val session = FluxaLocalStreamServer.start(
            targetUrl = upstreamUrl,
            headers = mapOf("X-Test" to "benchmark")
        )
        assertNotNull(session)
        sessions += session!!

        val fullRead = benchmark("rust-local-stream-full-read", iterations = 8) { _ ->
            assertEquals(payload.size, readBytes(session.url).size)
        }
        val rangeRead = benchmark("rust-local-stream-range-read", iterations = 64) { index ->
            val start = index * 1024
            val end = start + 4095
            val bytes = readBytes(session.url, "bytes=$start-$end")
            assertArrayEquals(payload.copyOfRange(start, end + 1), bytes)
        }

        assertEquals(76, upstreamRequests.get())
        assertTrue(fullRead.averageMicros > 0.0)
        assertTrue(rangeRead.averageMicros > 0.0)
    }

    @Test
    fun benchmarkRustLocalStreamServerLargePayloadRanges() {
        val payloadSizes = listOf(
            "10mb" to 10L * 1024 * 1024,
            "100mb" to 100L * 1024 * 1024,
            "1gb-simulated" to 1024L * 1024 * 1024
        )
        val rangeLength = 1024 * 1024

        payloadSizes.forEach { (label, payloadSize) ->
            val upstreamRequests = AtomicInteger(0)
            val upstream = startStaticServer(payloadSize, upstreamRequests)
            val session = FluxaLocalStreamServer.start(
                targetUrl = "http://127.0.0.1:${upstream.port}/video.bin",
                headers = emptyMap()
            )
            assertNotNull(session)
            sessions += session!!

            val starts = mapOf(
                "start" to 0L,
                "middle" to payloadSize / 2,
                "near-end" to payloadSize - rangeLength
            )
            starts.forEach { (position, start) ->
                val end = start + rangeLength - 1
                val result = benchmark("rust-local-stream-$label-$position-range", iterations = 4) { _ ->
                    val bytes = readBytes(session.url, "bytes=$start-$end")
                    assertArrayEquals(expectedBytes(start, rangeLength), bytes)
                }
                assertTrue(result.averageMicros > 0.0)
            }

            val parallelResult = benchmark("rust-local-stream-$label-4-parallel-ranges", iterations = 4) { _ ->
                val executor = Executors.newFixedThreadPool(4)
                try {
                    val parallelStarts = listOf(
                        0L,
                        payloadSize / 4,
                        payloadSize / 2,
                        payloadSize - rangeLength
                    )
                    val futures = executor.invokeAll(parallelStarts.map { start ->
                        Callable {
                            val end = start + rangeLength - 1
                            start to readBytes(session.url, "bytes=$start-$end")
                        }
                    })
                    futures.forEach { future ->
                        val (start, bytes) = future.get()
                        assertArrayEquals(expectedBytes(start, rangeLength), bytes)
                    }
                } finally {
                    executor.shutdownNow()
                }
            }
            assertTrue(parallelResult.averageMicros > 0.0)
            assertEquals(35, upstreamRequests.get())
        }
    }

    @Test
    fun benchmarkRustLocalStreamServerSlowUpstreamBehavior() {
        val payloadSize = 256L * 1024 * 1024
        val rangeLength = 512 * 1024
        val scenarios = listOf(
            SlowScenario("1mbps", bytesPerSecond = 1L * 1024 * 1024),
            SlowScenario("5mbps", bytesPerSecond = 5L * 1024 * 1024),
            SlowScenario("20mbps", bytesPerSecond = 20L * 1024 * 1024),
            SlowScenario("random-100-500ms-delay", randomDelayMillis = 100L..500L)
        )

        scenarios.forEach { scenario ->
            val upstream = startStaticServer(payloadSize, AtomicInteger(0), scenario)
            val session = FluxaLocalStreamServer.start(
                targetUrl = "http://127.0.0.1:${upstream.port}/video.bin",
                headers = emptyMap()
            )
            assertNotNull(session)
            sessions += session!!

            listOf(
                "start" to 0L,
                "middle" to payloadSize / 2,
                "near-end" to payloadSize - rangeLength
            ).forEach { (position, start) ->
                val result = benchmark(
                    "rust-local-stream-slow-${scenario.name}-$position-range",
                    iterations = 2
                ) { _ ->
                    val bytes = readBytes(session.url, "bytes=$start-${start + rangeLength - 1}")
                    assertArrayEquals(expectedBytes(start, rangeLength), bytes)
                }
                assertTrue(result.averageMicros > 0.0)
            }

            val parallelResult = benchmark(
                "rust-local-stream-slow-${scenario.name}-4-parallel-ranges",
                iterations = 2
            ) { _ ->
                val executor = Executors.newFixedThreadPool(4)
                try {
                    val starts = listOf(
                        0L,
                        payloadSize / 4,
                        payloadSize / 2,
                        payloadSize - rangeLength
                    )
                    val futures = executor.invokeAll(starts.map { start ->
                        Callable {
                            val bytes = readBytes(session.url, "bytes=$start-${start + rangeLength - 1}")
                            start to bytes
                        }
                    })
                    futures.forEach { future ->
                        val (start, bytes) = future.get()
                        assertArrayEquals(expectedBytes(start, rangeLength), bytes)
                    }
                } finally {
                    executor.shutdownNow()
                }
            }
            assertTrue(parallelResult.averageMicros > 0.0)
        }
    }

    @Test
    fun rustLocalStreamServerSurfacesDroppedUpstreamConnection() {
        val payloadSize = 64L * 1024 * 1024
        val rangeLength = 512 * 1024
        val upstream = startStaticServer(
            payloadSize = payloadSize,
            requestCount = AtomicInteger(0),
            scenario = SlowScenario("drop-after-128kb", dropAfterBytes = 128 * 1024)
        )
        val session = FluxaLocalStreamServer.start(
            targetUrl = "http://127.0.0.1:${upstream.port}/video.bin",
            headers = emptyMap()
        )
        assertNotNull(session)
        sessions += session!!

        val connection = URL(session.url).openConnection() as HttpURLConnection
        connection.connectTimeout = 5_000
        connection.readTimeout = 5_000
        connection.setRequestProperty("Range", "bytes=0-${rangeLength - 1}")
        val bytes = runCatching {
            connection.inputStream.use { it.readBytes() }
        }.getOrElse { ByteArray(0) }

        assertTrue(bytes.size < rangeLength)
    }

    @Test
    fun rustLocalStreamServerRetriesTransientUpstreamFailuresBeforeStreaming() {
        val payloadSize = 8L * 1024 * 1024
        val rangeLength = 256 * 1024
        val upstreamRequests = AtomicInteger(0)
        val upstream = startStaticServer(
            payloadSize = payloadSize,
            requestCount = upstreamRequests,
            scenario = SlowScenario("fail-first-two", failFirstRequests = 2)
        )
        val session = FluxaLocalStreamServer.start(
            targetUrl = "http://127.0.0.1:${upstream.port}/video.bin",
            headers = emptyMap()
        )
        assertNotNull(session)
        sessions += session!!

        val bytes = readBytes(session.url, "bytes=0-${rangeLength - 1}")

        assertArrayEquals(expectedBytes(0, rangeLength), bytes)
        assertEquals(3, upstreamRequests.get())
    }

    @Test
    fun nativeCoreApiOnlyExposesCachePolicyFunctions() {
        val nativeFunctionNames = FluxaCoreNative::class.java.declaredMethods.map { it.name }

        val nativeCacheStorageFunctions = nativeFunctionNames.filter { name ->
            name.contains("cache", ignoreCase = true) &&
                !name.contains("cacheKey", ignoreCase = true) &&
                !name.contains("cachePrefix", ignoreCase = true) &&
                !name.contains("BufferCache", ignoreCase = true) &&
                !name.contains("cacheEntryPolicy", ignoreCase = true) &&
                !name.contains("cacheTrimPolicy", ignoreCase = true)
        }
        assertTrue(nativeCacheStorageFunctions.isEmpty())
        assertTrue(nativeFunctionNames.any { it == "cacheEntryPolicyJsonNative" })
        assertTrue(nativeFunctionNames.any { it == "cacheTrimPolicyJsonNative" })
        assertFalse(nativeFunctionNames.any { it == "startLocalStreamServerNative" })
        assertFalse(nativeFunctionNames.any { it == "stopLocalStreamServerNative" })
        assertTrue(nativeFunctionNames.any { it == "parseManifestJsonNative" })
    }

    private fun startStaticServer(payload: ByteArray, requestCount: AtomicInteger): StaticByteServer {
        val server = StaticByteServer(payload, requestCount)
        servers += server
        return server
    }

    private fun startStaticServer(
        payloadSize: Long,
        requestCount: AtomicInteger,
        scenario: SlowScenario = SlowScenario("default")
    ): StaticByteServer {
        val server = StaticByteServer(payloadSize, requestCount, scenario)
        servers += server
        return server
    }

    private fun readBytes(rawUrl: String, range: String? = null): ByteArray {
        val connection = URL(rawUrl).openConnection() as HttpURLConnection
        connection.connectTimeout = 5_000
        connection.readTimeout = 5_000
        if (range != null) {
            connection.setRequestProperty("Range", range)
        }
        return connection.inputStream.use { it.readBytes() }
    }

    private fun expectedBytes(start: Long, length: Int): ByteArray {
        return ByteArray(length) { index -> byteAt(start + index) }
    }

    private fun byteAt(position: Long): Byte {
        return (position % 251).toByte()
    }

    private data class SlowScenario(
        val name: String,
        val bytesPerSecond: Long? = null,
        val randomDelayMillis: LongRange? = null,
        val dropAfterBytes: Int? = null,
        val failFirstRequests: Int = 0
    )

    private fun benchmark(
        name: String,
        iterations: Int,
        block: (Int) -> Unit
    ): BenchmarkResult {
        repeat(max(1, iterations / 20)) { block(it) }
        val start = System.nanoTime()
        repeat(iterations) { block(it) }
        val elapsedNanos = System.nanoTime() - start
        val result = BenchmarkResult(name, iterations, elapsedNanos)
        println(result)
        return result
    }

    private data class BenchmarkResult(
        val name: String,
        val iterations: Int,
        val elapsedNanos: Long
    ) {
        val averageMicros: Double = elapsedNanos / iterations / 1_000.0

        override fun toString(): String {
            return "$name: iterations=$iterations totalMs=${elapsedNanos / 1_000_000.0} avgMicros=$averageMicros"
        }
    }

    private class StaticByteServer private constructor(
        private val payloadSize: Long,
        private val requestCount: AtomicInteger,
        private val payload: ByteArray?,
        private val scenario: SlowScenario
    ) {
        private val running = AtomicBoolean(true)
        private val serverSocket = ServerSocket(0)
        private val executor: ExecutorService = Executors.newCachedThreadPool()
        private val thread = Thread(::acceptLoop, "fluxa-benchmark-static-byte-server")

        val port: Int = serverSocket.localPort

        constructor(payload: ByteArray, requestCount: AtomicInteger) : this(
            payload.size.toLong(),
            requestCount,
            payload,
            SlowScenario("default")
        )

        constructor(
            payloadSize: Long,
            requestCount: AtomicInteger,
            scenario: SlowScenario = SlowScenario("default")
        ) : this(
            payloadSize,
            requestCount,
            null,
            scenario
        )

        init {
            thread.isDaemon = true
            thread.start()
        }

        fun stop() {
            running.set(false)
            serverSocket.close()
            executor.shutdownNow()
            thread.join(1_000)
        }

        private fun acceptLoop() {
            while (running.get()) {
                runCatching {
                    val socket = serverSocket.accept()
                    executor.execute {
                        socket.use(::handle)
                    }
                }
            }
        }

        private fun handle(socket: Socket) {
            val requestNumber = requestCount.incrementAndGet()
            scenario.randomDelayMillis?.let { delayRange ->
                Thread.sleep(Random.nextLong(delayRange.first, delayRange.last + 1))
            }
            val input = socket.getInputStream().bufferedReader()
            val headers = mutableMapOf<String, String>()
            input.readLine() ?: return
            while (true) {
                val line = input.readLine() ?: return
                if (line.isEmpty()) break
                val separator = line.indexOf(':')
                if (separator > 0) {
                    headers[line.substring(0, separator).trim().lowercase()] =
                        line.substring(separator + 1).trim()
                }
            }

            val range = headers["range"]
            val output = socket.getOutputStream()
            if (requestNumber <= scenario.failFirstRequests) {
                output.write(
                    (
                        "HTTP/1.1 503 Service Unavailable\r\n" +
                            "Content-Length: 0\r\n" +
                            "Connection: close\r\n" +
                            "\r\n"
                        ).toByteArray()
                )
                output.flush()
                return
            }
            if (range?.startsWith("bytes=") == true) {
                val bounds = range.removePrefix("bytes=").split("-", limit = 2)
                val start = bounds[0].toLong()
                val end = bounds.getOrNull(1)?.takeIf { it.isNotBlank() }?.toLong() ?: payloadSize - 1
                val clampedEnd = end.coerceAtMost(payloadSize - 1)
                val length = (clampedEnd - start + 1).toInt()
                output.write((
                    "HTTP/1.1 206 Partial Content\r\n" +
                        "Accept-Ranges: bytes\r\n" +
                        "Content-Range: bytes $start-$clampedEnd/$payloadSize\r\n" +
                        "Content-Length: $length\r\n" +
                        "Connection: close\r\n" +
                        "\r\n"
                    ).toByteArray()
                )
                writeBytes(output, start, length)
                output.flush()
                return
            }

            output.write((
                "HTTP/1.1 200 OK\r\n" +
                    "Accept-Ranges: bytes\r\n" +
                    "Content-Length: $payloadSize\r\n" +
                    "Connection: close\r\n" +
                    "\r\n"
                ).toByteArray()
            )
            writeBytes(output, 0L, payloadSize.toInt())
            output.flush()
        }

        private fun writeBytes(output: java.io.OutputStream, start: Long, length: Int) {
            val maxBytes = scenario.dropAfterBytes?.coerceAtMost(length) ?: length
            val throttleBytesPerSecond = scenario.bytesPerSecond
            val throttleStartNanos = System.nanoTime()
            if (payload != null && start <= Int.MAX_VALUE && start + length <= payload.size) {
                output.write(payload, start.toInt(), maxBytes)
                throttleIfNeeded(throttleBytesPerSecond, maxBytes, throttleStartNanos)
                return
            }
            val buffer = ByteArray(16 * 1024)
            var written = 0
            while (written < maxBytes) {
                val chunk = minOf(buffer.size, maxBytes - written)
                for (index in 0 until chunk) {
                    buffer[index] = ((start + written + index) % 251).toByte()
                }
                output.write(buffer, 0, chunk)
                written += chunk
                throttleIfNeeded(throttleBytesPerSecond, written, throttleStartNanos)
            }
        }

        private fun throttleIfNeeded(bytesPerSecond: Long?, written: Int, startNanos: Long) {
            if (bytesPerSecond == null || bytesPerSecond <= 0) return
            val expectedNanos = written * 1_000_000_000L / bytesPerSecond
            val elapsedNanos = System.nanoTime() - startNanos
            val sleepNanos = expectedNanos - elapsedNanos
            if (sleepNanos <= 0) return
            Thread.sleep(sleepNanos / 1_000_000L, (sleepNanos % 1_000_000L).toInt())
        }
    }
}
