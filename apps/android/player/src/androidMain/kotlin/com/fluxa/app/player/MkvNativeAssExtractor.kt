package com.fluxa.app.player

import androidx.media3.common.C
import androidx.media3.common.DataReader
import androidx.media3.common.Format
import androidx.media3.common.MimeTypes
import androidx.media3.common.util.ParsableByteArray
import androidx.media3.common.util.UnstableApi
import androidx.media3.extractor.DefaultExtractorInput
import androidx.media3.extractor.Extractor
import androidx.media3.extractor.ExtractorOutput
import androidx.media3.extractor.PositionHolder
import androidx.media3.extractor.SeekMap
import androidx.media3.extractor.TrackOutput
import androidx.media3.extractor.mkv.MatroskaExtractor
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.InputStream
import java.net.URI
import java.util.Locale
import java.util.concurrent.TimeUnit

@UnstableApi
object MkvNativeAssExtractor {
    private const val ATTACHMENT_PREFIX_BYTES = 8L * 1024L * 1024L
    private const val HEAD_SCAN_BYTES = 64L * 1024L
    private const val MAX_ATTACHMENTS_BYTES = 128L * 1024L * 1024L
    private const val MAX_SUBTITLE_SCAN_BYTES = 512L * 1024L * 1024L
    private val client = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(0, TimeUnit.SECONDS)
        .build()
    private val playbackFontClient = OkHttpClient.Builder()
        .connectTimeout(4, TimeUnit.SECONDS)
        .readTimeout(8, TimeUnit.SECONDS)
        .callTimeout(30, TimeUnit.SECONDS)
        .build()

    suspend fun extract(
        url: String,
        headers: Map<String, String> = emptyMap()
    ): List<NativeAssTrack> = withContext(Dispatchers.IO) {
        if (!looksLikeMatroska(url)) {
            LibassDebugLog.d("local MKV ASS extractor skipped non-Matroska url=${LibassDebugLog.urlSummary(url)}")
            return@withContext emptyList()
        }
        LibassDebugLog.d("local MKV ASS extractor start url=${LibassDebugLog.urlSummary(url)} headers=${headers.keys}")
        runCatching {
            val fonts = extractFontAttachments(url, headers)
            val tracks = extractAssTracks(url, headers)
            LibassDebugLog.d("local MKV ASS extractor found tracks=${tracks.size} fonts=${fonts.size}")
            tracks.map { track -> track.copy(fonts = fonts) }
        }.onFailure { error ->
            LibassDebugLog.w("local MKV ASS extractor failed url=${LibassDebugLog.urlSummary(url)}", error)
        }.getOrDefault(emptyList())
    }

    suspend fun extractExternalFontHints(
        subtitleUrl: String
    ): List<NativeAssFont> = withContext(Dispatchers.IO) {
        runCatching {
            if (!subtitleUrl.startsWith("file:", ignoreCase = true) && !subtitleUrl.startsWith("content:", ignoreCase = true)) {
                LibassDebugLog.d("external font hints skipped remote subtitle url=${LibassDebugLog.urlSummary(subtitleUrl)}")
                return@runCatching emptyList()
            }
            val file = File(URI(subtitleUrl))
            val dir = file.parentFile ?: return@runCatching emptyList()
            val fonts = dir.listFiles()
                ?.filter { it.isFile && it.extension.lowercase(Locale.ROOT) in setOf("ttf", "otf", "ttc") }
                ?.map { NativeAssFont(it.name, it.readBytes()) }
                .orEmpty()
            LibassDebugLog.d("external font hints found fonts=${fonts.size} dir=${dir.absolutePath}")
            fonts
        }.getOrDefault(emptyList())
    }

    fun extractFontAttachmentsForPlayback(
        url: String,
        headers: Map<String, String> = emptyMap(),
        shouldAbort: () -> Boolean = { false }
    ): List<NativeAssFont> {
        LibassDebugLog.d("playback font attachment scan start url=${LibassDebugLog.urlSummary(url)} headers=${headers.keys}")
        repeat(3) { attempt ->
            if (attempt > 0) Thread.sleep(attempt * 8000L)
            if (shouldAbort()) {
                LibassDebugLog.d("playback font attachment scan aborted, fonts already available")
                return emptyList()
            }
            val fonts = runCatching { fetchAttachmentsViaSeekHead(url, headers) }
                .onFailure { LibassDebugLog.w("seek head font fetch failed attempt=$attempt", it) }
                .getOrNull()
                ?: runCatching {
                    if (shouldAbort()) return@runCatching emptyList()
                    LibassDebugLog.d("attachments seek head unavailable, falling back to prefix scan")
                    val bytes = openStream(url, headers, range = 0L until ATTACHMENT_PREFIX_BYTES, httpClient = playbackFontClient).use { it.readBytes() }
                    EbmlAttachmentScanner.scanFonts(bytes)
                }.onFailure { LibassDebugLog.w("prefix font scan failed attempt=$attempt", it) }.getOrNull()
            if (fonts != null) {
                LibassDebugLog.d(
                    "playback font attachment scan found fonts=${fonts.size} " +
                        "names=${fonts.joinToString(prefix = "[", postfix = "]") { "${it.name}:${it.data.size}" }}"
                )
                return fonts
            }
        }
        LibassDebugLog.w("playback font attachment scan gave up url=${LibassDebugLog.urlSummary(url)}")
        return emptyList()
    }

    private fun fetchAttachmentsViaSeekHead(url: String, headers: Map<String, String>): List<NativeAssFont>? {
        val head = openStream(url, headers, range = 0L until HEAD_SCAN_BYTES, httpClient = playbackFontClient).use { it.readBytes() }
        val position = EbmlAttachmentScanner.findAttachmentsPosition(head) ?: return null
        val blob = openStream(url, headers, range = position until position + MAX_ATTACHMENTS_BYTES, httpClient = playbackFontClient)
            .use(::readAttachmentsElement)
            ?: return null
        return EbmlAttachmentScanner.scanFonts(blob)
    }

    private fun readAttachmentsElement(stream: InputStream): ByteArray? {
        val header = ByteArray(16)
        var headerRead = 0
        while (headerRead < header.size) {
            val n = stream.read(header, headerRead, header.size - headerRead)
            if (n <= 0) return null
            headerRead += n
        }
        val length = EbmlAttachmentScanner.attachmentsElementLength(header) ?: return null
        if (length > MAX_ATTACHMENTS_BYTES) {
            LibassDebugLog.w("attachments element too large length=$length")
            return null
        }
        val out = ByteArrayOutputStream(length.toInt())
        out.write(header)
        var remaining = length - header.size
        val chunk = ByteArray(64 * 1024)
        while (remaining > 0) {
            val n = stream.read(chunk, 0, minOf(chunk.size.toLong(), remaining).toInt())
            if (n <= 0) break
            out.write(chunk, 0, n)
            remaining -= n
        }
        if (remaining > 0) return null
        return out.toByteArray()
    }

    fun scanFontAttachments(bytes: ByteArray): List<NativeAssFont> {
        return EbmlAttachmentScanner.scanFonts(bytes)
    }

    private fun looksLikeMatroska(url: String): Boolean {
        val path = url.substringBefore('?').lowercase(Locale.ROOT)
        return path.endsWith(".mkv") || path.endsWith(".mks") || path.endsWith(".mka") || path.endsWith(".mk3d") || path.contains(".mkv")
    }

    private fun extractAssTracks(url: String, headers: Map<String, String>): List<NativeAssTrack> {
        val output = RawAssExtractorOutput()
        openStream(url, headers, range = null).use { stream ->
            val reader = LimitedInputStreamDataReader(stream, MAX_SUBTITLE_SCAN_BYTES)
            val input = DefaultExtractorInput(reader, 0L, C.LENGTH_UNSET.toLong())
            val extractor = MatroskaExtractor(
                MatroskaExtractor.FLAG_DISABLE_SEEK_FOR_CUES or MatroskaExtractor.FLAG_EMIT_RAW_SUBTITLE_DATA
            )
            extractor.init(output)
            val seek = PositionHolder()
            var result: Int
            do {
                result = extractor.read(input, seek)
            } while (result == Extractor.RESULT_CONTINUE)
            extractor.release()
        }
        return output.buildTracks()
    }

    private fun extractFontAttachments(url: String, headers: Map<String, String>): List<NativeAssFont> {
        val bytes = openStream(url, headers, range = 0L until ATTACHMENT_PREFIX_BYTES).use { it.readBytes() }
        return EbmlAttachmentScanner.scanFonts(bytes)
    }

    private fun openStream(
        url: String,
        headers: Map<String, String>,
        range: LongRange?,
        httpClient: OkHttpClient = client
    ): InputStream {
        val request = Request.Builder().url(url).apply {
            StreamRequestPolicy.headersFor(url, headers).forEach { (key, value) -> header(key, value) }
            if (range != null) header("Range", "bytes=${range.first}-${range.last}")
        }.build()
        val response = httpClient.newCall(request).execute()
        if (!response.isSuccessful) {
            response.close()
            error("HTTP ${response.code}")
        }
        return response.body.byteStream()
    }

    private class LimitedInputStreamDataReader(
        private val input: InputStream,
        private val maxBytes: Long
    ) : DataReader {
        private var readBytes = 0L

        override fun read(buffer: ByteArray, offset: Int, length: Int): Int {
            if (readBytes >= maxBytes) return C.RESULT_END_OF_INPUT
            val allowed = minOf(length.toLong(), maxBytes - readBytes).toInt()
            val count = input.read(buffer, offset, allowed)
            if (count > 0) readBytes += count
            return count
        }
    }

    private class RawAssExtractorOutput : ExtractorOutput {
        private val outputs = linkedMapOf<Int, RawAssTrackOutput>()

        override fun track(id: Int, type: Int): TrackOutput {
            return outputs.getOrPut(id) { RawAssTrackOutput(id) }
        }

        override fun endTracks() = Unit
        override fun seekMap(seekMap: SeekMap) = Unit

        fun buildTracks(): List<NativeAssTrack> {
            val tracks = outputs.values.mapNotNull { it.buildTrack() }
            LibassDebugLog.d("raw ASS extractor output build tracks=${tracks.size}")
            return tracks
        }
    }

    private class RawAssTrackOutput(private val id: Int) : TrackOutput {
        private var format: Format? = null
        private val pendingSample = ByteArrayOutputStream()
        private val blocks = mutableListOf<Pair<Long, String>>()

        override fun format(format: Format) {
            this.format = format
        }

        override fun sampleData(input: DataReader, length: Int, allowEndOfInput: Boolean, sampleDataPart: Int): Int {
            val buffer = ByteArray(length)
            val read = input.read(buffer, 0, length)
            if (read == C.RESULT_END_OF_INPUT) {
                if (allowEndOfInput) return C.RESULT_END_OF_INPUT
                error("Unexpected end of input")
            }
            if (sampleDataPart == TrackOutput.SAMPLE_DATA_PART_MAIN && read > 0) {
                pendingSample.write(buffer, 0, read)
            }
            return read
        }

        override fun sampleData(data: ParsableByteArray, length: Int, sampleDataPart: Int) {
            if (sampleDataPart != TrackOutput.SAMPLE_DATA_PART_MAIN) {
                data.skipBytes(length)
                return
            }
            val bytes = ByteArray(length)
            data.readBytes(bytes, 0, length)
            pendingSample.write(bytes)
        }

        override fun sampleMetadata(timeUs: Long, flags: Int, size: Int, offset: Int, cryptoData: TrackOutput.CryptoData?) {
            val currentFormat = format ?: return resetSample()
            if (currentFormat.sampleMimeType != MimeTypes.TEXT_SSA) return resetSample()
            val body = pendingSample.toByteArray().decodeToString().trim()
            if (body.isNotEmpty()) blocks += Pair(timeUs, body)
            resetSample()
        }

        fun buildTrack(): NativeAssTrack? {
            val currentFormat = format ?: return null
            if (currentFormat.sampleMimeType != MimeTypes.TEXT_SSA || blocks.isEmpty()) return null
            val codecPrivate = currentFormat.initializationData.getOrNull(1)?.decodeToString().orEmpty()
            val dialogueLines = mutableListOf<String>()
            for (i in blocks.indices) {
                val (startUs, body) = blocks[i]
                if (body.startsWith("Dialogue:", ignoreCase = true)) {
                    dialogueLines += body
                    continue
                }
                val endUs = blocks.getOrNull(i + 1)?.first ?: (startUs + 5_000_000L)
                val fields = body.split(',', limit = 9)
                if (fields.size < 9) continue
                dialogueLines += "Dialogue: ${fields[1]},${formatAssTime(startUs)},${formatAssTime(endUs)},${fields[2]},${fields[3]},${fields[4]},${fields[5]},${fields[6]},${fields[7]},${fields[8]}"
            }
            if (dialogueLines.isEmpty()) return null
            LibassDebugLog.d("raw ASS track built id=$id dialogueLines=${dialogueLines.size} format=${LibassDebugLog.formatSummary(currentFormat)}")
            val eventsIdx = codecPrivate.indexOf("[Events]", ignoreCase = true)
            val scriptHeader = if (eventsIdx >= 0) codecPrivate.substring(0, eventsIdx).trimEnd() else codecPrivate.trimEnd()
            val assDoc = buildString {
                appendLine(scriptHeader)
                appendLine("[Events]")
                appendLine("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text")
                dialogueLines.forEach { appendLine(it) }
            }
            return NativeAssTrack(
                id = "embedded_ass_$id",
                label = currentFormat.label ?: currentFormat.id ?: "ASS $id",
                language = currentFormat.language,
                assData = assDoc.toByteArray()
            )
        }

        private fun resetSample() {
            pendingSample.reset()
        }

        private fun formatAssTime(timeUs: Long): String {
            val totalCentis = timeUs.coerceAtLeast(0L) / 10_000L
            val centis = totalCentis % 100
            val totalSeconds = totalCentis / 100
            val seconds = totalSeconds % 60
            val totalMinutes = totalSeconds / 60
            val minutes = totalMinutes % 60
            val hours = totalMinutes / 60
            return "%d:%02d:%02d.%02d".format(Locale.US, hours, minutes, seconds, centis)
        }
    }

    private object EbmlAttachmentScanner {
        private const val ID_SEGMENT = 0x18538067
        private const val ID_SEEK_HEAD = 0x114D9B74
        private const val ID_SEEK = 0x4DBB
        private const val ID_SEEK_ID = 0x53AB
        private const val ID_SEEK_POSITION = 0x53AC
        private const val ID_ATTACHMENTS = 0x1941A469
        private const val ID_ATTACHED_FILE = 0x61A7
        private const val ID_FILE_NAME = 0x466E
        private const val ID_FILE_MIME_TYPE = 0x4660
        private const val ID_FILE_DATA = 0x465C

        fun scanFonts(bytes: ByteArray): List<NativeAssFont> {
            val fonts = mutableListOf<NativeAssFont>()
            scan(bytes, 0, bytes.size, fonts, null)
            return fonts.distinctBy { it.name }
        }

        fun findAttachmentsPosition(head: ByteArray): Long? {
            var offset = 0
            var segmentDataStart = -1L
            while (offset < head.size - 2) {
                val id = readElementId(head, offset) ?: return null
                val sizeInfo = readElementSize(head, offset + id.length) ?: return null
                val dataStart = offset + id.length + sizeInfo.length
                when (id.value) {
                    ID_SEGMENT -> {
                        segmentDataStart = dataStart.toLong()
                        offset = dataStart
                    }
                    ID_SEEK_HEAD -> {
                        if (segmentDataStart < 0) return null
                        val dataEnd = (dataStart + sizeInfo.value).coerceAtMost(head.size.toLong()).toInt()
                        val position = seekPositionForAttachments(head, dataStart, dataEnd) ?: return null
                        return segmentDataStart + position
                    }
                    else -> {
                        val next = dataStart + sizeInfo.value
                        if (next > head.size) return null
                        offset = next.toInt()
                    }
                }
            }
            return null
        }

        fun attachmentsElementLength(bytes: ByteArray): Long? {
            val id = readElementId(bytes, 0) ?: return null
            if (id.value != ID_ATTACHMENTS) return null
            val sizeInfo = readElementSize(bytes, id.length) ?: return null
            return id.length + sizeInfo.length + sizeInfo.value
        }

        private fun seekPositionForAttachments(bytes: ByteArray, start: Int, end: Int): Long? {
            var offset = start
            while (offset < end - 2) {
                val id = readElementId(bytes, offset) ?: return null
                val sizeInfo = readElementSize(bytes, offset + id.length) ?: return null
                val dataStart = offset + id.length + sizeInfo.length
                val dataEnd = (dataStart + sizeInfo.value).coerceAtMost(end.toLong()).toInt()
                if (dataEnd < dataStart) return null
                if (id.value == ID_SEEK) {
                    val entry = parseSeekEntry(bytes, dataStart, dataEnd)
                    if (entry != null && entry.first == ID_ATTACHMENTS) return entry.second
                }
                offset = dataEnd
            }
            return null
        }

        private fun parseSeekEntry(bytes: ByteArray, start: Int, end: Int): Pair<Int, Long>? {
            var offset = start
            var target = 0
            var position = -1L
            while (offset < end) {
                val id = readElementId(bytes, offset) ?: return null
                val sizeInfo = readElementSize(bytes, offset + id.length) ?: return null
                val dataStart = offset + id.length + sizeInfo.length
                val dataEnd = (dataStart + sizeInfo.value).coerceAtMost(end.toLong()).toInt()
                if (dataEnd < dataStart) return null
                when (id.value) {
                    ID_SEEK_ID -> {
                        var value = 0
                        for (i in dataStart until dataEnd) value = (value shl 8) or (bytes[i].toInt() and 0xff)
                        target = value
                    }
                    ID_SEEK_POSITION -> {
                        var value = 0L
                        for (i in dataStart until dataEnd) value = (value shl 8) or (bytes[i].toLong() and 0xff)
                        position = value
                    }
                }
                offset = dataEnd
            }
            return if (position >= 0) target to position else null
        }

        private data class Attachment(
            var name: String? = null,
            var mime: String? = null,
            var data: ByteArray? = null,
            var truncated: Boolean = false
        )

        private fun scan(bytes: ByteArray, start: Int, end: Int, fonts: MutableList<NativeAssFont>, attachment: Attachment?) {
            var offset = start
            while (offset < end - 2) {
                val id = readElementId(bytes, offset) ?: break
                val sizeInfo = readElementSize(bytes, offset + id.length) ?: break
                val dataStart = offset + id.length + sizeInfo.length
                val fullEnd = dataStart + sizeInfo.value
                val dataEnd = fullEnd.coerceAtMost(end.toLong()).toInt()
                if (dataEnd < dataStart || dataStart > end) break
                when (id.value) {
                    ID_ATTACHMENTS -> scan(bytes, dataStart, dataEnd, fonts, null)
                    ID_ATTACHED_FILE -> {
                        val file = Attachment()
                        scan(bytes, dataStart, dataEnd, fonts, file)
                        val name = file.name
                        val data = file.data
                        if (name != null && isFont(name, file.mime)) {
                            if (file.truncated || data == null) {
                                LibassDebugLog.w("dropping truncated font attachment name=$name")
                            } else {
                                fonts += NativeAssFont(name, data)
                            }
                        }
                    }
                    ID_FILE_NAME -> attachment?.name = bytes.copyOfRange(dataStart, dataEnd).decodeToString().trimEnd('\u0000')
                    ID_FILE_MIME_TYPE -> attachment?.mime = bytes.copyOfRange(dataStart, dataEnd).decodeToString().trimEnd('\u0000')
                    ID_FILE_DATA -> {
                        if (fullEnd > end) {
                            attachment?.truncated = true
                        } else {
                            attachment?.data = bytes.copyOfRange(dataStart, dataEnd)
                        }
                    }
                    else -> {
                        if (isLikelyMaster(id.value)) scan(bytes, dataStart, dataEnd, fonts, attachment)
                    }
                }
                offset = dataEnd
            }
        }

        private fun isFont(name: String, mime: String?): Boolean {
            val lowerName = name.lowercase(Locale.ROOT)
            val lowerMime = mime.orEmpty().lowercase(Locale.ROOT)
            return lowerName.endsWith(".ttf") || lowerName.endsWith(".otf") || lowerName.endsWith(".ttc") ||
                lowerMime.contains("font") || lowerMime.contains("truetype") || lowerMime.contains("opentype")
        }

        private fun isLikelyMaster(id: Int): Boolean = id in setOf(0x18538067, 0x114D9B74, 0x1549A966, 0x1654AE6B, 0x1F43B675)

        private data class Vint(val value: Int, val length: Int)
        private data class Size(val value: Long, val length: Int)

        private fun readElementId(bytes: ByteArray, offset: Int): Vint? {
            if (offset >= bytes.size) return null
            val first = bytes[offset].toInt() and 0xff
            val length = when {
                first and 0x80 != 0 -> 1
                first and 0x40 != 0 -> 2
                first and 0x20 != 0 -> 3
                first and 0x10 != 0 -> 4
                else -> return null
            }
            if (offset + length > bytes.size) return null
            var value = 0
            repeat(length) { value = (value shl 8) or (bytes[offset + it].toInt() and 0xff) }
            return Vint(value, length)
        }

        private fun readElementSize(bytes: ByteArray, offset: Int): Size? {
            if (offset >= bytes.size) return null
            val first = bytes[offset].toInt() and 0xff
            var mask = 0x80
            var length = 1
            while (length <= 8 && first and mask == 0) {
                mask = mask shr 1
                length++
            }
            if (length > 8 || offset + length > bytes.size) return null
            var value = (first and mask.inv()).toLong()
            for (i in 1 until length) value = (value shl 8) or (bytes[offset + i].toLong() and 0xff)
            return if (value < 0) null else Size(value, length)
        }
    }
}
