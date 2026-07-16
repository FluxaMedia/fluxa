@file:androidx.annotation.OptIn(
    androidx.media3.common.util.UnstableApi::class,
    androidx.media3.common.util.ExperimentalApi::class
)
package com.fluxa.app.player

import androidx.media3.common.C
import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.audio.AudioRendererEventListener
import androidx.media3.exoplayer.audio.AudioSink
import androidx.media3.exoplayer.audio.MediaCodecAudioRenderer
import androidx.media3.exoplayer.mediacodec.MediaCodecSelector
import androidx.media3.exoplayer.Renderer
import androidx.media3.exoplayer.text.TextOutput
import androidx.media3.exoplayer.text.TextRenderer
import androidx.media3.exoplayer.video.VideoRendererEventListener
import android.content.Context
import android.util.Log
import android.os.Handler
import android.os.Looper
import androidx.media3.common.AudioAttributes
import androidx.media3.common.AuxEffectInfo
import androidx.media3.common.Format
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackParameters
import androidx.media3.common.util.Clock
import androidx.media3.decoder.DecoderInputBuffer
import androidx.media3.exoplayer.BaseRenderer
import androidx.media3.exoplayer.FormatHolder
import androidx.media3.exoplayer.RendererCapabilities
import androidx.media3.decoder.ffmpeg.FfmpegAudioRenderer
import androidx.media3.decoder.ffmpeg.FfmpegLibrary
import androidx.media3.exoplayer.analytics.PlayerId
import androidx.media3.exoplayer.audio.AudioOffloadSupport
import androidx.media3.exoplayer.source.MediaSource
import android.media.AudioDeviceInfo
import java.nio.ByteBuffer
import java.util.concurrent.atomic.AtomicLong

object PlayerDelayController {
    private val audioDelayMs = AtomicLong(0L)

    fun setAudioDelayMs(value: Long) {
        audioDelayMs.set(value.coerceIn(-5_000L, 5_000L))
    }

    fun audioDelayUs(): Long = audioDelayMs.get() * 1_000L
}

class AppAudioRendererFactory(context: Context, private val audioDecoderMode: String = "hw_prefer") : DefaultRenderersFactory(context) {
    override fun buildVideoRenderers(
        context: Context,
        extensionRendererMode: Int,
        mediaCodecSelector: MediaCodecSelector,
        enableDecoderFallback: Boolean,
        eventHandler: Handler,
        eventListener: VideoRendererEventListener,
        allowedVideoJoiningTimeMs: Long,
        out: ArrayList<Renderer>
    ) {
        super.buildVideoRenderers(
            context,
            extensionRendererMode,
            mediaCodecSelector,
            enableDecoderFallback,
            eventHandler,
            eventListener,
            allowedVideoJoiningTimeMs,
            out
        )
    }
    
    override fun buildAudioRenderers(
        context: Context,
        extensionRendererMode: Int,
        mediaCodecSelector: MediaCodecSelector,
        enableDecoderFallback: Boolean,
        audioSink: AudioSink,
        eventHandler: Handler,
        eventListener: AudioRendererEventListener,
        out: ArrayList<Renderer>
    ) {
        val delayedAudioSink = DelayedAudioSink(audioSink)
        val mediaCodecRenderer = MediaCodecAudioRenderer(
            context,
            mediaCodecSelector,
            enableDecoderFallback,
            eventHandler,
            eventListener,
            delayedAudioSink
        )
        val useFfmpeg = audioDecoderMode != "hw_only" && extensionRendererMode != EXTENSION_RENDERER_MODE_OFF
        val ffmpegRenderer = if (useFfmpeg && FfmpegLibrary.isAvailable()) {
            FfmpegAudioRenderer(eventHandler, eventListener, delayedAudioSink)
        } else {
            if (useFfmpeg) {
                Log.w("AppAudioRendererFactory", "FFmpeg audio renderer requested but FFmpeg library is unavailable")
            }
            null
        }

        if (audioDecoderMode == "sw_only" && ffmpegRenderer != null) {
            out.add(ffmpegRenderer)
            out.add(mediaCodecRenderer)
        } else {
            out.add(mediaCodecRenderer)
            ffmpegRenderer?.let(out::add)
        }
    }

    override fun buildTextRenderers(
        context: Context,
        output: TextOutput,
        outputLooper: Looper,
        extensionRendererMode: Int,
        out: ArrayList<Renderer>
    ) {
        LibassDebugLog.d("building text renderers with NativeAssTextRenderer before Media3 TextRenderer")
        out.add(NativeAssTextRenderer())
        out.add(
            TextRenderer(output, outputLooper).apply {
                experimentalSetLegacyDecodingEnabled(true)
            }
        )
    }
}

private class NativeAssTextRenderer : BaseRenderer(C.TRACK_TYPE_TEXT) {
    private val formatHolder = FormatHolder()
    private val inputBuffer = DecoderInputBuffer(DecoderInputBuffer.BUFFER_REPLACEMENT_MODE_NORMAL)
    private var inputStreamEnded = false
    private var sampleCount = 0

    override fun getName(): String = "NativeAssTextRenderer"

    override fun supportsFormat(format: Format): Int {
        val mimeType = format.sampleMimeType
        return when {
            mimeType == MimeTypes.TEXT_SSA -> {
                LibassDebugLog.d("NativeAssTextRenderer handles ${LibassDebugLog.formatSummary(format)}")
                RendererCapabilities.create(
                    if (format.cryptoType == C.CRYPTO_TYPE_NONE) C.FORMAT_HANDLED else C.FORMAT_UNSUPPORTED_DRM
                )
            }
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
        LibassDebugLog.d(
            "NativeAssTextRenderer stream changed startUs=$startPositionUs offsetUs=$offsetUs formats=${
                formats.joinToString { LibassDebugLog.formatSummary(it) }
            }"
        )
    }

    override fun render(positionUs: Long, elapsedRealtimeUs: Long) {
        if (inputStreamEnded) return

        while (true) {
            inputBuffer.clear()
            when (readSource(formatHolder, inputBuffer, 0)) {
                C.RESULT_FORMAT_READ -> Unit
                C.RESULT_BUFFER_READ -> {
                    if (inputBuffer.isEndOfStream) {
                        inputStreamEnded = true
                        LibassDebugLog.d("NativeAssTextRenderer reached end of stream samples=$sampleCount")
                        return
                    }
                    sampleCount++
                    if (sampleCount <= 8 || sampleCount % 50 == 0) {
                        LibassDebugLog.d("NativeAssTextRenderer drained SSA sample count=$sampleCount timeUs=${inputBuffer.timeUs} bytes=${inputBuffer.data?.limit() ?: 0}")
                    }
                }
                C.RESULT_NOTHING_READ -> return
                else -> return
            }
        }
    }

    override fun isEnded(): Boolean = inputStreamEnded

    override fun isReady(): Boolean = true

    override fun onPositionReset(positionUs: Long, joining: Boolean, sampleStreamIsResetToKeyFrame: Boolean) {
        inputStreamEnded = false
        sampleCount = 0
        LibassDebugLog.d("NativeAssTextRenderer position reset positionUs=$positionUs joining=$joining keyFrameReset=$sampleStreamIsResetToKeyFrame")
    }
}

private class DelayedAudioSink(private val delegate: AudioSink) : AudioSink {
    override fun setListener(listener: AudioSink.Listener) = delegate.setListener(listener)
    override fun setPlayerId(playerId: PlayerId?) = delegate.setPlayerId(playerId)
    override fun setClock(clock: Clock) = delegate.setClock(clock)
    override fun supportsFormat(format: Format): Boolean = delegate.supportsFormat(format)
    override fun getFormatSupport(format: Format): Int = delegate.getFormatSupport(format)
    override fun getFormatOffloadSupport(format: Format): AudioOffloadSupport = delegate.getFormatOffloadSupport(format)
    override fun getCurrentPositionUs(sourceEnded: Boolean): Long = delegate.getCurrentPositionUs(sourceEnded)
    override fun configure(inputFormat: Format, specifiedBufferSize: Int, outputChannels: IntArray?) = delegate.configure(inputFormat, specifiedBufferSize, outputChannels)
    override fun play() = delegate.play()
    override fun handleDiscontinuity() = delegate.handleDiscontinuity()
    override fun handleBuffer(buffer: ByteBuffer, presentationTimeUs: Long, encodedAccessUnitCount: Int): Boolean {
        return delegate.handleBuffer(buffer, presentationTimeUs + PlayerDelayController.audioDelayUs(), encodedAccessUnitCount)
    }
    override fun playToEndOfStream() = delegate.playToEndOfStream()
    override fun isEnded(): Boolean = delegate.isEnded
    override fun hasPendingData(): Boolean = delegate.hasPendingData()
    override fun setPlaybackParameters(playbackParameters: PlaybackParameters) = delegate.setPlaybackParameters(playbackParameters)
    override fun getPlaybackParameters(): PlaybackParameters = delegate.playbackParameters
    override fun setSkipSilenceEnabled(skipSilenceEnabled: Boolean) = delegate.setSkipSilenceEnabled(skipSilenceEnabled)
    override fun getSkipSilenceEnabled(): Boolean = delegate.skipSilenceEnabled
    override fun setAudioAttributes(audioAttributes: AudioAttributes) = delegate.setAudioAttributes(audioAttributes)
    override fun getAudioAttributes(): AudioAttributes? = delegate.audioAttributes
    override fun setAudioSessionId(audioSessionId: Int) = delegate.setAudioSessionId(audioSessionId)
    override fun setAuxEffectInfo(auxEffectInfo: AuxEffectInfo) = delegate.setAuxEffectInfo(auxEffectInfo)
    override fun setPreferredDevice(audioDeviceInfo: AudioDeviceInfo?) = delegate.setPreferredDevice(audioDeviceInfo)
    override fun setOutputStreamOffsetUs(outputStreamOffsetUs: Long) = delegate.setOutputStreamOffsetUs(outputStreamOffsetUs)
    override fun getAudioTrackBufferSizeUs(): Long = delegate.audioTrackBufferSizeUs
    override fun enableTunnelingV21() = delegate.enableTunnelingV21()
    override fun disableTunneling() = delegate.disableTunneling()
    override fun setOffloadMode(offloadMode: Int) = delegate.setOffloadMode(offloadMode)

    override fun setOffloadDelayPadding(delayInFrames: Int, paddingInFrames: Int) =
        delegate.setOffloadDelayPadding(delayInFrames, paddingInFrames)
    override fun setVolume(volume: Float) = delegate.setVolume(volume)
    override fun pause() = delegate.pause()
    override fun flush() = delegate.flush()
    override fun reset() = delegate.reset()
    override fun release() = delegate.release()
}
