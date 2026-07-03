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
    private val fontsDir: String? = null
) : ExtractorsFactory {

    override fun createExtractors(): Array<Extractor> =
        baseFactory.createExtractors().map(::wrapIfMkv).toTypedArray()

    override fun createExtractors(uri: Uri, responseHeaders: Map<String, List<String>>): Array<Extractor> =
        baseFactory.createExtractors(uri, responseHeaders).map(::wrapIfMkv).toTypedArray()

    private fun wrapIfMkv(extractor: Extractor): Extractor =
        if (extractor is MatroskaExtractor) LibassInjectingMkvExtractor(extractor, relay, fontsDir) else extractor
}

private class LibassInjectingMkvExtractor(
    private val delegate: MatroskaExtractor,
    private val relay: LibassEventRelay,
    private val fontsDir: String?
) : Extractor {
    private var interceptingOutput: LibassInterceptingExtractorOutput? = null

    override fun sniff(input: ExtractorInput): Boolean = delegate.sniff(input)

    override fun init(output: ExtractorOutput) {
        val intercepting = LibassInterceptingExtractorOutput(output, relay, fontsDir)
        interceptingOutput = intercepting
        delegate.init(intercepting)
    }

    override fun read(input: ExtractorInput, seekPosition: PositionHolder): Int =
        delegate.read(input, seekPosition)

    override fun seek(position: Long, timeUs: Long) {
        interceptingOutput?.onSeek()
        delegate.seek(position, timeUs)
    }

    override fun release() = delegate.release()
}

private class LibassInterceptingExtractorOutput(
    private val delegate: ExtractorOutput,
    private val relay: LibassEventRelay,
    val fontsDir: String?
) : ExtractorOutput {
    private val textTracks = mutableListOf<LibassInterceptingTrackOutput>()
    val collectedFonts = mutableListOf<NativeAssFont>()

    override fun track(id: Int, type: Int): TrackOutput {
        val delegateTrack = delegate.track(id, type)
        return when (type) {
            C.TRACK_TYPE_TEXT -> LibassInterceptingTrackOutput(delegateTrack, relay, this, id).also { textTracks.add(it) }
            C.TRACK_TYPE_METADATA -> LibassFontCollectingTrackOutput(delegateTrack) { font -> collectedFonts.add(font) }
            else -> delegateTrack
        }
    }

    override fun endTracks() = delegate.endTracks()
    override fun seekMap(seekMap: SeekMap) = delegate.seekMap(seekMap)

    fun onSeek() {
        relay.clearEvents()
        textTracks.forEach { it.onSeekFlush() }
    }
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

    override fun format(format: Format) {
        this.format = format
        if (format.sampleMimeType == MimeTypes.TEXT_SSA) {
            cachedHeader = buildAssHeader(format.initializationData)
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
                pendingSample.reset()
                delegate.sampleMetadata(timeUs, flags, size, offset, cryptoData)
                return
            }

            val body = pendingSample.toByteArray()
            if (body.isNotEmpty()) {
                val currentGeneration = relay.headerGeneration()
                if (primedGeneration != currentGeneration) {
                    cachedHeader?.let { relay.setHeader(it, extractorOutput.collectedFonts, extractorOutput.fontsDir) }
                    primedGeneration = currentGeneration
                }
                val prevTimeUs = pendingTimeUs
                val prevBody = pendingBody
                if (prevTimeUs != Long.MIN_VALUE && prevBody != null) {
                    val durationMs = ((timeUs - prevTimeUs) / 1000L).coerceIn(100L, 30_000L)
                    lastInterEventMs = durationMs
                    relay.addEvent(prevTimeUs / 1000L, durationMs, prevBody)
                }
                pendingTimeUs = timeUs
                pendingBody = body
            }
        }
        pendingSample.reset()
        delegate.sampleMetadata(timeUs, flags, size, offset, cryptoData)
    }

    fun onSeekFlush() {
        val prevBody = pendingBody
        val prevTimeUs = pendingTimeUs
        if (prevTimeUs != Long.MIN_VALUE && prevBody != null) {
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
}
