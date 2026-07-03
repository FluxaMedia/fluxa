@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
package com.fluxa.app.player

import android.net.Uri
import androidx.media3.common.C
import androidx.media3.common.Format
import androidx.media3.common.MimeTypes
import androidx.media3.common.DataReader
import androidx.media3.common.util.ParsableByteArray
import androidx.media3.extractor.Extractor
import androidx.media3.extractor.ExtractorInput
import androidx.media3.extractor.ExtractorOutput
import androidx.media3.extractor.ExtractorsFactory
import androidx.media3.extractor.PositionHolder
import androidx.media3.extractor.SeekMap
import androidx.media3.extractor.TrackOutput
import androidx.media3.extractor.mkv.MatroskaExtractor
import java.io.ByteArrayOutputStream

class LibassInjectingExtractorsFactory(
    private val relay: LibassEventRelay,
    private val baseFactory: ExtractorsFactory,
    private val fontsDir: String? = null,
    private val fontAttachmentsProvider: (() -> List<NativeAssFont>)? = null,
    private val onExtractorsCreated: (() -> Unit)? = null
) : ExtractorsFactory {

    override fun createExtractors(): Array<Extractor> =
        baseFactory.createExtractors().map(::wrapIfMkv).toTypedArray()

    override fun createExtractors(uri: Uri, responseHeaders: Map<String, List<String>>): Array<Extractor> {
        LibassDebugLog.d("extractor factory create uri=${LibassDebugLog.urlSummary(uri.toString())} responseHeaders=${responseHeaders.keys}")
        onExtractorsCreated?.invoke()
        return baseFactory.createExtractors(uri, responseHeaders).map(::wrapIfMkv).toTypedArray()
    }

    private fun wrapIfMkv(extractor: Extractor): Extractor {
        val wrapped = extractor is MatroskaExtractor
        if (wrapped) LibassDebugLog.d("wrapping MatroskaExtractor for raw ASS relay fontsDir=$fontsDir")
        return if (extractor is MatroskaExtractor) {
            LibassInjectingMkvExtractor(extractor, relay, fontsDir, fontAttachmentsProvider)
        } else {
            extractor
        }
    }
}

private class LibassInjectingMkvExtractor(
    private val delegate: MatroskaExtractor,
    private val relay: LibassEventRelay,
    private val fontsDir: String?,
    private val fontAttachmentsProvider: (() -> List<NativeAssFont>)?
) : Extractor {
    private var interceptingOutput: LibassInterceptingExtractorOutput? = null
    private var captureInput: PrefixCapturingExtractorInput? = null
    private val prefixCapture = PrefixCaptureBuffer()

    override fun sniff(input: ExtractorInput): Boolean = delegate.sniff(input)

    override fun init(output: ExtractorOutput) {
        LibassDebugLog.d("mkv extractor init with libass intercepting output")
        val passiveFontsProvider = {
            val bytes = prefixCapture.bytes()
            if (bytes.isEmpty()) {
                LibassDebugLog.d("passive MKV prefix font scan has no captured bytes")
                emptyList()
            } else {
                val fonts = MkvNativeAssExtractor.scanFontAttachments(bytes)
                LibassDebugLog.d("passive MKV prefix font scan bytes=${bytes.size} fonts=${fonts.size}")
                fonts
            }
        }
        val intercepting = LibassInterceptingExtractorOutput(output, relay, fontsDir, fontAttachmentsProvider, passiveFontsProvider)
        interceptingOutput = intercepting
        delegate.init(intercepting)
    }

    override fun read(input: ExtractorInput, seekPosition: PositionHolder): Int {
        val capturingInput = captureInput?.takeIf { it.delegate === input }
            ?: PrefixCapturingExtractorInput(input, prefixCapture).also { captureInput = it }
        return delegate.read(capturingInput, seekPosition)
    }

    override fun seek(position: Long, timeUs: Long) {
        LibassDebugLog.d("mkv extractor seek position=$position timeUs=$timeUs")
        interceptingOutput?.onSeek()
        delegate.seek(position, timeUs)
    }

    override fun release() {
        LibassDebugLog.d("mkv extractor release")
        delegate.release()
    }
}

private class LibassInterceptingExtractorOutput(
    private val delegate: ExtractorOutput,
    private val relay: LibassEventRelay,
    val fontsDir: String?,
    private val fontAttachmentsProvider: (() -> List<NativeAssFont>)?,
    private val passiveFontAttachmentsProvider: (() -> List<NativeAssFont>)?
) : ExtractorOutput {
    private val textTracks = mutableListOf<LibassInterceptingTrackOutput>()
    val collectedFonts = mutableListOf<NativeAssFont>()
    private var providerFontsLoaded = false

    override fun track(id: Int, type: Int): TrackOutput {
        val delegateTrack = delegate.track(id, type)
        return when (type) {
            C.TRACK_TYPE_TEXT -> {
                LibassDebugLog.d("intercepting text track id=$id")
                LibassInterceptingTrackOutput(delegateTrack, relay, this, id).also { textTracks.add(it) }
            }
            C.TRACK_TYPE_METADATA -> {
                LibassDebugLog.d("intercepting metadata track id=$id for font attachments")
                LibassFontCollectingTrackOutput(delegateTrack) { font ->
                    collectedFonts.add(font)
                    LibassDebugLog.d("collected font attachment name=${font.name} bytes=${font.data.size} total=${collectedFonts.size}")
                }
            }
            else -> delegateTrack
        }
    }

    override fun endTracks() = delegate.endTracks()
    override fun seekMap(seekMap: SeekMap) = delegate.seekMap(seekMap)

    fun onSeek() {
        LibassDebugLog.d("intercepting output seek flush textTracks=${textTracks.size}")
        relay.clearEvents()
        textTracks.forEach { it.onSeekFlush() }
    }

    fun fontsForRenderer(): List<NativeAssFont> {
        if (!providerFontsLoaded) {
            val providerFonts = fontAttachmentsProvider?.let { provider ->
                runCatching { provider() }
                    .onFailure { LibassDebugLog.w("font attachment provider failed", it) }
                    .getOrDefault(emptyList())
            }.orEmpty()
            val passiveFonts = passiveFontAttachmentsProvider?.let { provider ->
                runCatching { provider() }
                    .onFailure { LibassDebugLog.w("passive font attachment provider failed", it) }
                    .getOrDefault(emptyList())
            }.orEmpty()
            if (passiveFonts.isNotEmpty()) {
                collectedFonts += passiveFonts
                LibassDebugLog.d("passive font attachment provider supplied fonts=${passiveFonts.size} total=${collectedFonts.size}")
            }
            if (providerFonts.isNotEmpty()) {
                collectedFonts += providerFonts
                LibassDebugLog.d("font attachment provider supplied fonts=${providerFonts.size} total=${collectedFonts.size}")
            } else if (passiveFonts.isEmpty()) {
                LibassDebugLog.d("font attachment provider supplied no fonts")
            }
            providerFontsLoaded = collectedFonts.isNotEmpty()
        }
        return collectedFonts.distinctBy { it.name }
    }
}

private class PrefixCaptureBuffer {
    private val capture = ByteArray(MAX_CAPTURE_BYTES)
    private var capturedLength = 0

    @Synchronized
    fun bytes(): ByteArray = capture.copyOf(capturedLength)

    @Synchronized
    fun capture(position: Long, buffer: ByteArray, offset: Int, length: Int) {
        if (!shouldCapture(position) || length <= 0) return
        val targetOffset = position.toInt()
        val writable = minOf(length, MAX_CAPTURE_BYTES - targetOffset)
        if (writable > 0) {
            System.arraycopy(buffer, offset, capture, targetOffset, writable)
            capturedLength = maxOf(capturedLength, targetOffset + writable)
        }
    }

    fun shouldCapture(position: Long): Boolean = position in 0 until MAX_CAPTURE_BYTES.toLong()

    private companion object {
        const val MAX_CAPTURE_BYTES = 8 * 1024 * 1024
    }
}

private class PrefixCapturingExtractorInput(
    val delegate: ExtractorInput,
    private val captureBuffer: PrefixCaptureBuffer
) : ExtractorInput {
    private val scratch = ByteArray(16 * 1024)

    override fun read(buffer: ByteArray, offset: Int, length: Int): Int {
        val position = delegate.position
        val read = delegate.read(buffer, offset, length)
        if (read > 0) capture(position, buffer, offset, read)
        return read
    }

    override fun readFully(target: ByteArray, offset: Int, length: Int, allowEndOfInput: Boolean): Boolean {
        val position = delegate.position
        val ok = delegate.readFully(target, offset, length, allowEndOfInput)
        if (ok) capture(position, target, offset, length)
        return ok
    }

    override fun readFully(target: ByteArray, offset: Int, length: Int) {
        val position = delegate.position
        delegate.readFully(target, offset, length)
        capture(position, target, offset, length)
    }

    override fun skip(length: Int): Int {
        if (!shouldCapture(delegate.position)) return delegate.skip(length)
        val toRead = minOf(length, scratch.size)
        val read = read(scratch, 0, toRead)
        return read
    }

    override fun skipFully(length: Int, allowEndOfInput: Boolean): Boolean {
        var remaining = length
        while (remaining > 0 && shouldCapture(delegate.position)) {
            val read = read(scratch, 0, minOf(remaining, scratch.size))
            if (read == C.RESULT_END_OF_INPUT) return allowEndOfInput && remaining == length
            remaining -= read
        }
        return if (remaining > 0) delegate.skipFully(remaining, allowEndOfInput) else true
    }

    override fun skipFully(length: Int) {
        if (!skipFully(length, allowEndOfInput = false)) throw java.io.EOFException()
    }

    override fun peek(target: ByteArray, offset: Int, length: Int): Int {
        val position = delegate.peekPosition
        val read = delegate.peek(target, offset, length)
        if (read > 0) capture(position, target, offset, read)
        return read
    }

    override fun peekFully(target: ByteArray, offset: Int, length: Int, allowEndOfInput: Boolean): Boolean {
        val position = delegate.peekPosition
        val ok = delegate.peekFully(target, offset, length, allowEndOfInput)
        if (ok) capture(position, target, offset, length)
        return ok
    }

    override fun peekFully(target: ByteArray, offset: Int, length: Int) {
        val position = delegate.peekPosition
        delegate.peekFully(target, offset, length)
        capture(position, target, offset, length)
    }

    override fun advancePeekPosition(length: Int, allowEndOfInput: Boolean): Boolean =
        delegate.advancePeekPosition(length, allowEndOfInput)

    override fun advancePeekPosition(length: Int) = delegate.advancePeekPosition(length)
    override fun resetPeekPosition() = delegate.resetPeekPosition()
    override fun getPeekPosition(): Long = delegate.peekPosition
    override fun getPosition(): Long = delegate.position
    override fun getLength(): Long = delegate.length
    override fun <E : Throwable> setRetryPosition(position: Long, e: E): Nothing {
        delegate.setRetryPosition(position, e)
        throw e
    }

    private fun capture(position: Long, buffer: ByteArray, offset: Int, length: Int) {
        captureBuffer.capture(position, buffer, offset, length)
    }

    private fun shouldCapture(position: Long): Boolean = captureBuffer.shouldCapture(position)
}

private class LibassFontCollectingTrackOutput(
    private val delegate: TrackOutput,
    private val onFontCollected: (NativeAssFont) -> Unit
) : TrackOutput {
    private var isFontTrack = false
    private var fontName: String? = null
    private val pendingData = ByteArrayOutputStream()

    override fun format(format: Format) {
        val mime = format.sampleMimeType?.lowercase() ?: ""
        isFontTrack = mime.startsWith("font/") ||
            mime == "application/x-truetype-font" ||
            mime == "application/vnd.ms-opentype" ||
            mime == "application/font-sfnt" ||
            mime == "application/x-font-ttf"
        fontName = format.label?.takeIf { it.isNotBlank() } ?: format.id ?: "font"
        if (isFontTrack) {
            LibassDebugLog.d("font track format ${LibassDebugLog.formatSummary(format)}")
        }
        delegate.format(format)
    }

    override fun sampleData(input: DataReader, length: Int, allowEndOfInput: Boolean, sampleDataPart: Int): Int {
        val buf = ByteArray(length)
        val read = input.read(buf, 0, length)
        if (isFontTrack && read > 0 && sampleDataPart == TrackOutput.SAMPLE_DATA_PART_MAIN) {
            pendingData.write(buf, 0, read)
        }
        delegate.sampleData(ParsableByteArray(buf, maxOf(0, read)), maxOf(0, read), sampleDataPart)
        return read
    }

    override fun sampleData(data: ParsableByteArray, length: Int, sampleDataPart: Int) {
        val buf = ByteArray(length)
        data.readBytes(buf, 0, length)
        if (isFontTrack && sampleDataPart == TrackOutput.SAMPLE_DATA_PART_MAIN) {
            pendingData.write(buf)
        }
        delegate.sampleData(ParsableByteArray(buf, length), length, sampleDataPart)
    }

    override fun sampleMetadata(timeUs: Long, flags: Int, size: Int, offset: Int, cryptoData: TrackOutput.CryptoData?) {
        if (isFontTrack) {
            val data = pendingData.toByteArray()
            if (data.isNotEmpty()) onFontCollected(NativeAssFont(fontName ?: "font", data))
            pendingData.reset()
        }
        delegate.sampleMetadata(timeUs, flags, size, offset, cryptoData)
    }
}

private class LibassInterceptingTrackOutput(
    private val delegate: TrackOutput,
    private val relay: LibassEventRelay,
    private val extractorOutput: LibassInterceptingExtractorOutput,
    private val trackId: Int
) : TrackOutput {
    private var format: Format? = null
    private var cachedHeader: ByteArray? = null
    private var primedGeneration: Int = -1
    private val pendingSample = ByteArrayOutputStream()
    private var pendingTimeUs = Long.MIN_VALUE
    private var pendingBody: ByteArray? = null
    private var lastInterEventMs = 5_000L
    private var sampleCount = 0
    private var skippedBySelectionCount = 0

    override fun format(format: Format) {
        this.format = format
        if (format.sampleMimeType == MimeTypes.TEXT_SSA) {
            cachedHeader = buildAssHeader(format.initializationData)
            LibassDebugLog.d("SSA track format trackId=$trackId ${LibassDebugLog.formatSummary(format)} headerBytes=${cachedHeader?.size ?: 0}")
        } else {
            LibassDebugLog.d("text track format trackId=$trackId ${LibassDebugLog.formatSummary(format)}")
        }
        delegate.format(format)
    }

    override fun sampleData(
        input: DataReader,
        length: Int,
        allowEndOfInput: Boolean,
        sampleDataPart: Int
    ): Int {
        val buf = ByteArray(length)
        val read = input.read(buf, 0, length)
        if (read == C.RESULT_END_OF_INPUT) {
            if (allowEndOfInput) return C.RESULT_END_OF_INPUT
            error("Unexpected end of input")
        }
        if (sampleDataPart == TrackOutput.SAMPLE_DATA_PART_MAIN && read > 0) {
            pendingSample.write(buf, 0, read)
        }
        delegate.sampleData(ParsableByteArray(buf, read), read, sampleDataPart)
        return read
    }

    override fun sampleData(data: ParsableByteArray, length: Int, sampleDataPart: Int) {
        val buf = ByteArray(length)
        data.readBytes(buf, 0, length)
        if (sampleDataPart == TrackOutput.SAMPLE_DATA_PART_MAIN) {
            pendingSample.write(buf)
        }
        delegate.sampleData(ParsableByteArray(buf, length), length, sampleDataPart)
    }

    override fun sampleMetadata(
        timeUs: Long,
        flags: Int,
        size: Int,
        offset: Int,
        cryptoData: TrackOutput.CryptoData?
    ) {
        if (format?.sampleMimeType == MimeTypes.TEXT_SSA) {
            val selectedId = relay.selectedTrackId
            if (selectedId != null && selectedId != trackId) {
                skippedBySelectionCount++
                if (skippedBySelectionCount <= 4 || skippedBySelectionCount % 25 == 0) {
                    LibassDebugLog.d("SSA sample skipped by selection trackId=$trackId selectedTrackId=$selectedId skipped=$skippedBySelectionCount")
                }
                pendingSample.reset()
                delegate.sampleMetadata(timeUs, flags, size, offset, cryptoData)
                return
            }

            val body = pendingSample.toByteArray()
            if (body.isNotEmpty()) {
                sampleCount++
                val currentGeneration = relay.headerGeneration()
                if (primedGeneration != currentGeneration) {
                    cachedHeader?.let {
                        val fonts = extractorOutput.fontsForRenderer()
                        LibassDebugLog.d("priming relay trackId=$trackId generation=$currentGeneration fonts=${fonts.size}")
                        relay.setHeader(it, fonts, extractorOutput.fontsDir)
                    } ?: LibassDebugLog.w("SSA sample received without cached header trackId=$trackId")
                    primedGeneration = currentGeneration
                }
                val textBody = body.decodeToString().trim()
                val media3DurationMs = media3DialogueDurationMs(textBody)
                if (media3DurationMs != null) {
                    if (sampleCount <= 8 || sampleCount % 50 == 0) {
                        LibassDebugLog.d("SSA Media3-prefixed sample trackId=$trackId count=$sampleCount timeUs=$timeUs durationMs=$media3DurationMs bytes=${body.size}")
                    }
                    lastInterEventMs = media3DurationMs
                    relay.addEvent(timeUs / 1000L, media3DurationMs, body)
                    pendingTimeUs = Long.MIN_VALUE
                    pendingBody = null
                } else {
                    val prevTimeUs = pendingTimeUs
                    val prevBody = pendingBody
                    if (prevTimeUs != Long.MIN_VALUE && prevBody != null) {
                        val durationMs = ((timeUs - prevTimeUs) / 1000L).coerceIn(100L, 30_000L)
                        lastInterEventMs = durationMs
                        if (sampleCount <= 8 || sampleCount % 50 == 0) {
                            LibassDebugLog.d("SSA raw sample relayed trackId=$trackId count=$sampleCount startUs=$prevTimeUs nextUs=$timeUs durationMs=$durationMs bytes=${prevBody.size}")
                        }
                        relay.addEvent(prevTimeUs / 1000L, durationMs, prevBody)
                    }
                    pendingTimeUs = timeUs
                    pendingBody = body
                }
            }
        }
        pendingSample.reset()
        delegate.sampleMetadata(timeUs, flags, size, offset, cryptoData)
    }

    fun onSeekFlush() {
        val prevBody = pendingBody
        val prevTimeUs = pendingTimeUs
        if (prevTimeUs != Long.MIN_VALUE && prevBody != null) {
            LibassDebugLog.d("SSA seek flush relays pending sample trackId=$trackId startUs=$prevTimeUs durationMs=${lastInterEventMs.coerceAtMost(5_000L)} bytes=${prevBody.size}")
            relay.addEvent(prevTimeUs / 1000L, lastInterEventMs.coerceAtMost(5_000L), prevBody)
        }
        pendingTimeUs = Long.MIN_VALUE
        pendingBody = null
        pendingSample.reset()
    }

    private fun buildAssHeader(initializationData: List<ByteArray>): ByteArray {
        val codecPrivate = initializationData.getOrNull(1)?.decodeToString().orEmpty()
        val eventsIdx = codecPrivate.indexOf("[Events]", ignoreCase = true)
        val scriptHeader = if (eventsIdx >= 0) codecPrivate.substring(0, eventsIdx).trimEnd() else codecPrivate.trimEnd()
        return buildString {
            appendLine(scriptHeader)
            appendLine("[Events]")
            appendLine("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text")
        }.toByteArray()
    }

    private fun media3DialogueDurationMs(body: String): Long? {
        if (!body.startsWith("Dialogue:", ignoreCase = true)) return null
        val fields = body.substringAfter(':').trim().split(',', limit = 3)
        if (fields.size < 2) return null
        val startMs = parseAssTimeMs(fields[0]) ?: return null
        val endMs = parseAssTimeMs(fields[1]) ?: return null
        return (endMs - startMs).coerceIn(100L, 30_000L)
    }

    private fun parseAssTimeMs(value: String): Long? {
        val parts = value.trim().replace('.', ':').split(':')
        if (parts.size != 4) return null
        val hours = parts[0].toLongOrNull() ?: return null
        val minutes = parts[1].toLongOrNull() ?: return null
        val seconds = parts[2].toLongOrNull() ?: return null
        val centis = parts[3].toLongOrNull() ?: return null
        return (((hours * 60L + minutes) * 60L + seconds) * 1000L) + centis * 10L
    }
}
