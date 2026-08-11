@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
package com.fluxa.app.player

import android.net.Uri
import android.util.Base64
import androidx.media3.common.C
import androidx.media3.common.DataReader
import androidx.media3.common.Format
import androidx.media3.common.MimeTypes
import androidx.media3.common.util.ParsableByteArray
import androidx.media3.extractor.Extractor
import androidx.media3.extractor.ExtractorInput
import androidx.media3.extractor.ExtractorOutput
import androidx.media3.extractor.ExtractorsFactory
import androidx.media3.extractor.PositionHolder
import androidx.media3.extractor.SeekMap
import androidx.media3.extractor.TrackOutput
import androidx.media3.extractor.mp4.FragmentedMp4Extractor
import androidx.media3.extractor.mp4.Mp4Extractor
import com.fluxa.app.core.rust.FluxaCoreNative
import java.io.ByteArrayOutputStream
import org.json.JSONObject

data class DvSampleTransformConfig(
    val enabled: Boolean,
    val mode: Int,
    val stripHdr10Plus: Boolean
)

class DvSampleTransformExtractorsFactory(
    private val baseFactory: ExtractorsFactory,
    private val config: () -> DvSampleTransformConfig
) : ExtractorsFactory {

    override fun createExtractors(): Array<Extractor> =
        baseFactory.createExtractors().map(::wrapIfMp4).toTypedArray()

    override fun createExtractors(uri: Uri, responseHeaders: Map<String, List<String>>): Array<Extractor> =
        baseFactory.createExtractors(uri, responseHeaders).map(::wrapIfMp4).toTypedArray()

    private fun wrapIfMp4(extractor: Extractor): Extractor = when (extractor) {
        is Mp4Extractor, is FragmentedMp4Extractor -> DvSampleTransformingExtractor(extractor, config)
        else -> extractor
    }
}

private class DvSampleTransformingExtractor(
    private val delegate: Extractor,
    private val config: () -> DvSampleTransformConfig
) : Extractor {
    override fun sniff(input: ExtractorInput): Boolean = delegate.sniff(input)

    override fun init(output: ExtractorOutput) {
        delegate.init(DvInterceptingExtractorOutput(output, config))
    }

    override fun read(input: ExtractorInput, seekPosition: PositionHolder): Int =
        delegate.read(input, seekPosition)

    override fun seek(position: Long, timeUs: Long) = delegate.seek(position, timeUs)

    override fun release() = delegate.release()
}

private class DvInterceptingExtractorOutput(
    private val delegate: ExtractorOutput,
    private val config: () -> DvSampleTransformConfig
) : ExtractorOutput {
    override fun track(id: Int, type: Int): TrackOutput {
        val delegateTrack = delegate.track(id, type)
        return if (type == C.TRACK_TYPE_VIDEO) {
            DvSampleTrackOutput(delegateTrack, config)
        } else {
            delegateTrack
        }
    }

    override fun endTracks() = delegate.endTracks()
    override fun seekMap(seekMap: SeekMap) = delegate.seekMap(seekMap)
}

private class DvSampleTrackOutput(
    private val delegate: TrackOutput,
    private val config: () -> DvSampleTransformConfig
) : TrackOutput {
    private var isDolbyVisionTrack = false
    private val pendingSample = ByteArrayOutputStream()

    override fun format(format: Format) {
        isDolbyVisionTrack = format.sampleMimeType == MimeTypes.VIDEO_DOLBY_VISION
        delegate.format(format)
    }

    override fun sampleData(
        input: DataReader,
        length: Int,
        allowEndOfInput: Boolean,
        sampleDataPart: Int
    ): Int {
        if (!isDolbyVisionTrack || !config().enabled) {
            return delegate.sampleData(input, length, allowEndOfInput, sampleDataPart)
        }
        val buf = ByteArray(length)
        val read = input.read(buf, 0, length)
        if (read == C.RESULT_END_OF_INPUT) {
            if (allowEndOfInput) return C.RESULT_END_OF_INPUT
            error("Unexpected end of input")
        }
        if (sampleDataPart == TrackOutput.SAMPLE_DATA_PART_MAIN && read > 0) {
            pendingSample.write(buf, 0, read)
        }
        return read
    }

    override fun sampleData(data: ParsableByteArray, length: Int, sampleDataPart: Int) {
        if (!isDolbyVisionTrack || !config().enabled) {
            delegate.sampleData(data, length, sampleDataPart)
            return
        }
        val buf = ByteArray(length)
        data.readBytes(buf, 0, length)
        if (sampleDataPart == TrackOutput.SAMPLE_DATA_PART_MAIN) {
            pendingSample.write(buf)
        }
    }

    override fun sampleMetadata(
        timeUs: Long,
        flags: Int,
        size: Int,
        offset: Int,
        cryptoData: TrackOutput.CryptoData?
    ) {
        if (!isDolbyVisionTrack || !config().enabled) {
            delegate.sampleMetadata(timeUs, flags, size, offset, cryptoData)
            return
        }
        val raw = pendingSample.toByteArray()
        pendingSample.reset()
        val transformed = transform(raw, encrypted = cryptoData != null)
        delegate.sampleData(ParsableByteArray(transformed, transformed.size), transformed.size, TrackOutput.SAMPLE_DATA_PART_MAIN)
        delegate.sampleMetadata(timeUs, flags, transformed.size, 0, cryptoData)
    }

    private fun transform(sample: ByteArray, encrypted: Boolean): ByteArray {
        if (sample.isEmpty()) return sample
        val cfg = config()
        val request = JSONObject().apply {
            put("sampleBase64", Base64.encodeToString(sample, Base64.NO_WRAP))
            put("framing", "length_delimited")
            put("nalLengthSize", 4)
            put("mode", cfg.mode)
            put("stripHdr10Plus", cfg.stripHdr10Plus)
            put("encrypted", encrypted)
        }
        val result = runCatching {
            FluxaCoreNative.coreInvoke("dolbyVisionProcessSample", request.toString())
        }.getOrNull() ?: return sample
        val response = runCatching { JSONObject(result) }.getOrNull() ?: return sample
        if (!response.optBoolean("ok", false)) return sample
        val outputBase64 = response.optString("outputBase64", "")
        if (outputBase64.isEmpty()) return sample
        return runCatching { Base64.decode(outputBase64, Base64.NO_WRAP) }.getOrDefault(sample)
    }
}
