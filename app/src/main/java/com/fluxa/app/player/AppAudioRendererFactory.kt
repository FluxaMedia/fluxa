@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
package com.fluxa.app.player

import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.audio.AudioRendererEventListener
import androidx.media3.exoplayer.audio.AudioSink
import androidx.media3.exoplayer.audio.MediaCodecAudioRenderer
import androidx.media3.exoplayer.mediacodec.MediaCodecSelector
import androidx.media3.exoplayer.Renderer
import androidx.media3.exoplayer.video.VideoRendererEventListener
import android.content.Context
import android.util.Log
import android.os.Build
import android.os.Handler
import androidx.media3.common.AudioAttributes
import androidx.media3.common.AuxEffectInfo
import androidx.media3.common.Format
import androidx.media3.common.PlaybackParameters
import androidx.media3.common.util.Clock
import androidx.media3.decoder.ffmpeg.FfmpegAudioRenderer
import androidx.media3.decoder.ffmpeg.FfmpegLibrary
import androidx.media3.exoplayer.analytics.PlayerId
import androidx.media3.exoplayer.audio.AudioOffloadSupport
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
