package com.fluxa.app.player

import kotlinx.coroutines.runBlocking
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

class MkvChapterFetcherTest {
    private lateinit var server: MockWebServer

    @Before
    fun setUp() {
        server = MockWebServer()
        server.start()
    }

    @After
    fun tearDown() {
        server.shutdown()
    }

    @Test
    fun parsesChaptersFromRangeHonoredResponse() = runBlocking {
        val body = segmentWithChapters(listOf(0L to "OP", 90_000L to "Episode"))
        server.enqueue(
            MockResponse()
                .setResponseCode(206)
                .setHeader("Content-Range", "bytes 0-${body.size - 1}/${body.size * 10}")
                .setBody(okio.Buffer().write(body))
        )

        val chapters = MkvChapterFetcher.fetch(server.url("/video.mkv").toString())

        assertEquals(listOf(Chapter("OP", 0), Chapter("Episode", 90_000)), chapters)

        val recorded = server.takeRequest()
        assertEquals("bytes=0-4194303", recorded.getHeader("Range"))
        assertTrue(recorded.getHeader("User-Agent").orEmpty().isNotBlank())
    }

    @Test
    fun bailsOutWhenServerIgnoresRangeAndReturnsWholeFile() = runBlocking {
        val oversized = ByteArray(4 * 1024 * 1024 + 1)
        server.enqueue(
            MockResponse()
                .setResponseCode(200)
                .setBody(okio.Buffer().write(oversized))
        )

        val chapters = MkvChapterFetcher.fetch(server.url("/video.mkv").toString())

        assertEquals(emptyList<Chapter>(), chapters)
    }

    @Test
    fun returnsEmptyOnServerError() = runBlocking {
        server.enqueue(MockResponse().setResponseCode(404))

        val chapters = MkvChapterFetcher.fetch(server.url("/video.mkv").toString())

        assertEquals(emptyList<Chapter>(), chapters)
    }

    private fun ebmlVint(value: Long): ByteArray = when {
        value < 0x7F -> byteArrayOf((0x80 or value.toInt()).toByte())
        value < 0x3FFF -> byteArrayOf((0x40 or (value shr 8).toInt()).toByte(), (value and 0xFF).toByte())
        value < 0x1FFFFF -> byteArrayOf(
            (0x20 or (value shr 16).toInt()).toByte(),
            (value shr 8).toByte(),
            (value and 0xFF).toByte()
        )
        else -> byteArrayOf(
            (0x10 or (value shr 24).toInt()).toByte(),
            (value shr 16).toByte(),
            (value shr 8).toByte(),
            (value and 0xFF).toByte()
        )
    }

    private fun idBytes(id: Long): ByteArray = when {
        id <= 0xFF -> byteArrayOf(id.toByte())
        id <= 0xFFFF -> byteArrayOf((id shr 8).toByte(), (id and 0xFF).toByte())
        id <= 0xFFFFFF -> byteArrayOf((id shr 16).toByte(), (id shr 8).toByte(), (id and 0xFF).toByte())
        else -> byteArrayOf((id shr 24).toByte(), (id shr 16).toByte(), (id shr 8).toByte(), (id and 0xFF).toByte())
    }

    private fun ebmlElement(id: Long, data: ByteArray): ByteArray =
        idBytes(id) + ebmlVint(data.size.toLong()) + data

    private fun chapterAtom(startMs: Long, title: String): ByteArray {
        val timeStart = ebmlElement(0x91, uint64Be(startMs * 1_000_000))
        val chapterString = ebmlElement(0x85, title.toByteArray())
        val display = ebmlElement(0x80, chapterString)
        return ebmlElement(0xB6, timeStart + display)
    }

    private fun uint64Be(value: Long): ByteArray {
        var v = value
        val out = ArrayDeque<Byte>()
        if (v == 0L) return byteArrayOf(0)
        while (v > 0) {
            out.addFirst((v and 0xFF).toByte())
            v = v shr 8
        }
        return out.toByteArray()
    }

    private fun segmentWithChapters(chapters: List<Pair<Long, String>>): ByteArray {
        val atoms = chapters.fold(ByteArray(0)) { acc, (ms, title) -> acc + chapterAtom(ms, title) }
        val editionEntry = ebmlElement(0x45B9, atoms)
        val chaptersElem = ebmlElement(0x1043A770, editionEntry)
        return ebmlElement(0x18538067, chaptersElem)
    }
}
