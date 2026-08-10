@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.player

import androidx.media3.common.C
import androidx.media3.common.Format
import androidx.media3.common.MimeTypes
import androidx.media3.decoder.DecoderInputBuffer
import androidx.media3.exoplayer.BaseRenderer
import androidx.media3.exoplayer.FormatHolder
import androidx.media3.exoplayer.RendererCapabilities
import androidx.media3.exoplayer.source.MediaSource
import com.fluxa.app.player.subtitle.SubtitleCoordinator
import com.fluxa.app.player.subtitle.SubtitleFormat
import com.fluxa.app.player.subtitle.cueParserFor
import java.nio.charset.StandardCharsets

internal class EmbeddedTextInterceptor(
    private val coordinator: SubtitleCoordinator
) : BaseRenderer(C.TRACK_TYPE_TEXT) {
    private val formatHolder = FormatHolder()
    private val inputBuffer = DecoderInputBuffer(DecoderInputBuffer.BUFFER_REPLACEMENT_MODE_NORMAL)
    private var inputStreamEnded = false
    private var pendingStartUs = C.TIME_UNSET
    private var pendingText: String? = null
    private var currentFormat: SubtitleFormat = SubtitleFormat.SRT

    override fun getName(): String = "EmbeddedTextInterceptor"

    override fun supportsFormat(format: Format): Int {
        val mimeType = format.sampleMimeType
        return when {
            mimeType == MimeTypes.APPLICATION_SUBRIP ||
                mimeType == MimeTypes.TEXT_VTT ||
                mimeType == MimeTypes.APPLICATION_TTML -> RendererCapabilities.create(
                if (format.cryptoType == C.CRYPTO_TYPE_NONE) C.FORMAT_HANDLED else C.FORMAT_UNSUPPORTED_DRM
            )
            MimeTypes.isText(mimeType) -> RendererCapabilities.create(C.FORMAT_UNSUPPORTED_SUBTYPE)
            else -> RendererCapabilities.create(C.FORMAT_UNSUPPORTED_TYPE)
        }
    }

    override fun onStreamChanged(
        formats: Array<Format>,
        startPositionUs: Long,
        offsetUs: Long,
        mediaPeriodId: MediaSource.MediaPeriodId
    ) {
        closePending(pendingStartUs)
        coordinator.resetEmbedded()
    }

    override fun onPositionReset(positionUs: Long, joining: Boolean, sampleStreamIsResetToKeyFrame: Boolean) {
        inputStreamEnded = false
        closePending(pendingStartUs)
        coordinator.resetEmbedded()
    }

    override fun render(positionUs: Long, elapsedRealtimeUs: Long) {
        if (inputStreamEnded) return
        while (true) {
            inputBuffer.clear()
            when (readSource(formatHolder, inputBuffer, 0)) {
                C.RESULT_FORMAT_READ -> {
                    currentFormat = subtitleFormatFor(formatHolder.format?.sampleMimeType)
                }
                C.RESULT_BUFFER_READ -> {
                    if (inputBuffer.isEndOfStream) {
                        inputStreamEnded = true
                        closePending(pendingStartUs.takeIf { it != C.TIME_UNSET }?.let { it + DEFAULT_TAIL_DURATION_US })
                        return
                    }
                    val data = inputBuffer.data
                    val text = if (data != null) {
                        String(data.array(), data.arrayOffset() + data.position(), data.remaining(), StandardCharsets.UTF_8)
                    } else null
                    if (text != null) handleSample(inputBuffer.timeUs, text)
                }
                C.RESULT_NOTHING_READ -> return
                else -> return
            }
        }
    }

    private fun handleSample(timeUs: Long, text: String) {
        val wholeFileCues = cueParserFor(currentFormat).parse(text)
        if (wholeFileCues.isNotEmpty()) {
            closePending(null)
            coordinator.loadEmbeddedCues(wholeFileCues)
            return
        }
        closePending(timeUs)
        pendingStartUs = timeUs
        pendingText = text
    }

    private fun closePending(endUs: Long?) {
        val start = pendingStartUs
        val text = pendingText
        if (start != C.TIME_UNSET && text != null && endUs != null && endUs > start) {
            coordinator.onEmbeddedSample(start, endUs, text)
        }
        pendingStartUs = C.TIME_UNSET
        pendingText = null
    }

    private fun subtitleFormatFor(mimeType: String?): SubtitleFormat = when (mimeType) {
        MimeTypes.TEXT_VTT -> SubtitleFormat.WEBVTT
        MimeTypes.APPLICATION_TTML -> SubtitleFormat.TTML
        else -> SubtitleFormat.SRT
    }

    override fun isEnded(): Boolean = inputStreamEnded

    override fun isReady(): Boolean = true

    private companion object {
        const val DEFAULT_TAIL_DURATION_US = 5_000_000L
    }
}
