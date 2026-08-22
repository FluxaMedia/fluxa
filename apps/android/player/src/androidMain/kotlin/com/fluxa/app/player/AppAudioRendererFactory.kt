@file:androidx.annotation.OptIn(
    androidx.media3.common.util.UnstableApi::class,
    androidx.media3.common.util.ExperimentalApi::class
)
package com.fluxa.app.player

import androidx.media3.common.C
import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.audio.AudioRendererEventListener
import androidx.media3.exoplayer.audio.AudioSink
import androidx.media3.exoplayer.audio.DefaultAudioSink
import androidx.media3.exoplayer.audio.MediaCodecAudioRenderer
import androidx.media3.exoplayer.mediacodec.MediaCodecSelector
import androidx.media3.exoplayer.mediacodec.MediaCodecInfo
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

object AudioDecoderSupport {
    fun isFfmpegAvailable(): Boolean = FfmpegLibrary.isAvailable()
}

/**
 * Media3's decoder fallback only retries another MediaCodec.  Keep a small
 * runtime blacklist so a codec that actually failed can be skipped on the
 * next preparation and the FFmpeg extension renderer can take the track.
 */
internal class RuntimeAudioCodecSelector : MediaCodecSelector {
    private val failedCodecNames = java.util.Collections.synchronizedSet(mutableSetOf<String>())
    private val failedMimeTypes = java.util.Collections.synchronizedSet(mutableSetOf<String>())

    override fun getDecoderInfos(
        mimeType: String,
        requiresSecureDecoder: Boolean,
        requiresTunnelingDecoder: Boolean,
    ): List<MediaCodecInfo> {
        if (mimeType.lowercase() in failedMimeTypes) return emptyList()
        return MediaCodecSelector.DEFAULT
            .getDecoderInfos(mimeType, requiresSecureDecoder, requiresTunnelingDecoder)
            .filterNot { it.name in failedCodecNames }
    }

    fun blacklist(codecName: String): Boolean = failedCodecNames.add(codecName)

    fun blacklistMimeType(mimeType: String?): Boolean {
        val normalized = mimeType?.trim()?.lowercase()?.takeIf { it.isNotBlank() } ?: return false
        return failedMimeTypes.add(normalized)
    }

    fun clearFailures() {
        failedCodecNames.clear()
        failedMimeTypes.clear()
    }
}

class AppAudioRendererFactory(
    context: Context,
    private val audioDecoderMode: String = "hw_prefer",
    private val audioProcessingMode: String = "reference",
    private val subtitleCoordinator: com.fluxa.app.player.subtitle.SubtitleCoordinator? = null
) : DefaultRenderersFactory(context) {
    internal val runtimeAudioCodecSelector = RuntimeAudioCodecSelector()
    private var audioSinkContext: Context? = null
    private var audioSinkFloatOutput = false
    private var audioSinkPlaybackParameters = false
    private var delayedAudioSink: DelayedAudioSink? = null
    private var audioRouteSignature: String? = null
    @Volatile private var configuredAudioFormat: Format? = null
    @Volatile private var configuredOutputChannels: IntArray? = null

    internal fun audioSinkRuntimeStatus(): String? {
        val format = configuredAudioFormat ?: return null
        val channels = configuredOutputChannels?.joinToString(",") ?: "auto"
        return "mime=${format.sampleMimeType ?: "unknown"} rate=${format.sampleRate} source_channels=${format.channelCount} output_channels=$channels"
    }

    internal fun blacklistAudioCodec(codecName: String): Boolean =
        runtimeAudioCodecSelector.blacklist(codecName)

    internal fun blacklistAudioMimeType(mimeType: String?): Boolean =
        runtimeAudioCodecSelector.blacklistMimeType(mimeType)

    internal fun resetAudioCodecFailures() = runtimeAudioCodecSelector.clearFailures()

    override fun buildAudioSink(
        context: Context,
        enableFloatOutput: Boolean,
        enableAudioOutputPlaybackParameters: Boolean
    ): AudioSink {
        audioSinkContext = context.applicationContext
        audioSinkFloatOutput = enableFloatOutput
        audioSinkPlaybackParameters = enableAudioOutputPlaybackParameters
        return buildConfiguredAudioSink(context, enableFloatOutput, enableAudioOutputPlaybackParameters)
    }

    private fun buildConfiguredAudioSink(
        context: Context,
        enableFloatOutput: Boolean,
        enableAudioOutputPlaybackParameters: Boolean,
    ): AudioSink {
        val audioAttributes = AudioCapabilityResolver.mediaAudioAttributes()
        val capabilities = AudioCapabilityResolver.resolve(context, audioAttributes)
        audioRouteSignature = capabilities.routeSignature()
        Log.i("AppAudioRendererFactory", "audio output ${capabilities.routeDescription} pcm=${capabilities.maxPcmChannels} passthrough=${capabilities.passthroughEncodings}")
        return DefaultAudioSink.Builder(context)
            .setAudioCapabilities(
                capabilities.sinkCapabilities(audioProcessingMode)
            )
            .setAudioProcessors(
                buildList {
                    // Apply DSP while the source channel layout is still
                    // intact. Night mode must be able to lift the real center
                    // channel of 5.1/7.1 dialogue before a stereo fallback
                    // folds it into L/R. Downmix is deliberately last.
                    when (audioProcessingMode) {
                        "balanced" -> add(BalancedAudioProcessor())
                        "night" -> add(NightModeAudioProcessor())
                    }
                    add(AdaptiveChannelDownmixProcessor(capabilities.maxPcmChannels))
                }.toTypedArray()
            )
            .setEnableFloatOutput(enableFloatOutput)
            .setEnableAudioOutputPlaybackParameters(enableAudioOutputPlaybackParameters)
            .build()
    }

    internal fun refreshAudioRoute(): Boolean {
        val context = audioSinkContext ?: return false
        val currentCapabilities = AudioCapabilityResolver.resolve(
            context,
            AudioCapabilityResolver.mediaAudioAttributes()
        )
        if (currentCapabilities.routeSignature() == audioRouteSignature) return false
        val replacement = buildConfiguredAudioSink(context, audioSinkFloatOutput, audioSinkPlaybackParameters)
        delayedAudioSink?.replaceDelegate(replacement)
        return true
    }

    private fun AudioOutputCapabilities.routeSignature(): String =
        "$routeDescription|$maxPcmChannels|${pcmSampleRates.sorted()}|${passthroughEncodings.sorted()}|$speakerLayoutMasks|$spatialLayoutMasks"

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
        val delayedAudioSink = DelayedAudioSink(audioSink) { format, outputChannels ->
            configuredAudioFormat = format
            configuredOutputChannels = outputChannels?.copyOf()
            Log.i(
                "AppAudioRendererFactory",
                "audio sink configured mime=${format.sampleMimeType} rate=${format.sampleRate} channels=${format.channelCount} output=${outputChannels?.contentToString() ?: "auto"}",
            )
        }
        this.delayedAudioSink = delayedAudioSink
        val mediaCodecRenderer = MediaCodecAudioRenderer(
            context,
            runtimeAudioCodecSelector,
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
        subtitleCoordinator?.let { out.add(EmbeddedTextInterceptor(it)) }
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

private class DelayedAudioSink(
    private var delegate: AudioSink,
    private val onConfigured: (Format, IntArray?) -> Unit,
) : AudioSink {
    private var listener: AudioSink.Listener? = null
    private var playerId: PlayerId? = null
    private var clock: Clock? = null
    private var audioAttributes: AudioAttributes? = null
    private var playbackParameters: PlaybackParameters? = null
    private var skipSilence = false
    private var volume = 1f
    private var preferredDevice: AudioDeviceInfo? = null
    private var audioSessionId: Int? = null
    private var auxEffectInfo: AuxEffectInfo? = null
    private var outputStreamOffsetUs: Long? = null
    private var tunnelingEnabled = false
    private var offloadMode: Int? = null
    private var offloadDelayPadding: Pair<Int, Int>? = null

    @Synchronized
    fun replaceDelegate(replacement: AudioSink) {
        val old = delegate
        listener?.let(replacement::setListener)
        playerId?.let(replacement::setPlayerId)
        clock?.let(replacement::setClock)
        audioAttributes?.let(replacement::setAudioAttributes)
        playbackParameters?.let(replacement::setPlaybackParameters)
        replacement.setSkipSilenceEnabled(skipSilence)
        replacement.setVolume(volume)
        preferredDevice?.let(replacement::setPreferredDevice)
        audioSessionId?.let(replacement::setAudioSessionId)
        auxEffectInfo?.let(replacement::setAuxEffectInfo)
        outputStreamOffsetUs?.let(replacement::setOutputStreamOffsetUs)
        if (tunnelingEnabled) replacement.enableTunnelingV21()
        offloadMode?.let(replacement::setOffloadMode)
        offloadDelayPadding?.let { (delay, padding) -> replacement.setOffloadDelayPadding(delay, padding) }
        runCatching { old.reset() }
        runCatching { old.release() }
        delegate = replacement
    }

    override fun setListener(listener: AudioSink.Listener) {
        this.listener = listener
        delegate.setListener(listener)
    }
    override fun setPlayerId(playerId: PlayerId?) {
        this.playerId = playerId
        delegate.setPlayerId(playerId)
    }
    override fun setClock(clock: Clock) {
        this.clock = clock
        delegate.setClock(clock)
    }
    override fun supportsFormat(format: Format): Boolean = delegate.supportsFormat(format)
    override fun getFormatSupport(format: Format): Int = delegate.getFormatSupport(format)
    override fun getFormatOffloadSupport(format: Format): AudioOffloadSupport = delegate.getFormatOffloadSupport(format)
    override fun getCurrentPositionUs(sourceEnded: Boolean): Long = delegate.getCurrentPositionUs(sourceEnded)
    override fun configure(inputFormat: Format, specifiedBufferSize: Int, outputChannels: IntArray?) {
        onConfigured(inputFormat, outputChannels)
        delegate.configure(inputFormat, specifiedBufferSize, outputChannels)
    }
    override fun play() = delegate.play()
    override fun handleDiscontinuity() = delegate.handleDiscontinuity()
    override fun handleBuffer(buffer: ByteBuffer, presentationTimeUs: Long, encodedAccessUnitCount: Int): Boolean {
        return delegate.handleBuffer(buffer, presentationTimeUs + PlayerDelayController.audioDelayUs(), encodedAccessUnitCount)
    }
    override fun playToEndOfStream() = delegate.playToEndOfStream()
    override fun isEnded(): Boolean = delegate.isEnded
    override fun hasPendingData(): Boolean = delegate.hasPendingData()
    override fun setPlaybackParameters(playbackParameters: PlaybackParameters) {
        this.playbackParameters = playbackParameters
        delegate.setPlaybackParameters(playbackParameters)
    }
    override fun getPlaybackParameters(): PlaybackParameters = delegate.playbackParameters
    override fun setSkipSilenceEnabled(skipSilenceEnabled: Boolean) {
        skipSilence = skipSilenceEnabled
        delegate.setSkipSilenceEnabled(skipSilenceEnabled)
    }
    override fun getSkipSilenceEnabled(): Boolean = delegate.skipSilenceEnabled
    override fun setAudioAttributes(audioAttributes: AudioAttributes) {
        this.audioAttributes = audioAttributes
        delegate.setAudioAttributes(audioAttributes)
    }
    override fun getAudioAttributes(): AudioAttributes? = delegate.audioAttributes
    override fun setAudioSessionId(audioSessionId: Int) {
        this.audioSessionId = audioSessionId
        delegate.setAudioSessionId(audioSessionId)
    }
    override fun setAuxEffectInfo(auxEffectInfo: AuxEffectInfo) {
        this.auxEffectInfo = auxEffectInfo
        delegate.setAuxEffectInfo(auxEffectInfo)
    }
    override fun setPreferredDevice(audioDeviceInfo: AudioDeviceInfo?) {
        preferredDevice = audioDeviceInfo
        delegate.setPreferredDevice(audioDeviceInfo)
    }
    override fun setOutputStreamOffsetUs(outputStreamOffsetUs: Long) {
        this.outputStreamOffsetUs = outputStreamOffsetUs
        delegate.setOutputStreamOffsetUs(outputStreamOffsetUs)
    }
    override fun getAudioTrackBufferSizeUs(): Long = delegate.audioTrackBufferSizeUs
    override fun enableTunnelingV21() {
        tunnelingEnabled = true
        delegate.enableTunnelingV21()
    }
    override fun disableTunneling() {
        tunnelingEnabled = false
        delegate.disableTunneling()
    }
    override fun setOffloadMode(offloadMode: Int) {
        this.offloadMode = offloadMode
        delegate.setOffloadMode(offloadMode)
    }

    override fun setOffloadDelayPadding(delayInFrames: Int, paddingInFrames: Int) {
        offloadDelayPadding = delayInFrames to paddingInFrames
        delegate.setOffloadDelayPadding(delayInFrames, paddingInFrames)
    }
    override fun setVolume(volume: Float) {
        this.volume = volume
        delegate.setVolume(volume)
    }
    override fun pause() = delegate.pause()
    override fun flush() = delegate.flush()
    override fun reset() = delegate.reset()
    override fun release() = delegate.release()
}
