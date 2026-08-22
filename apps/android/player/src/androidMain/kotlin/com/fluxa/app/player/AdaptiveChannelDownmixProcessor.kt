@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.player

import androidx.media3.common.C
import androidx.media3.common.audio.AudioProcessor
import androidx.media3.common.audio.BaseAudioProcessor
import com.fluxa.app.shared.feature.player.AudioDownmixMatrix
import com.fluxa.app.shared.feature.player.AudioPeakLimiter
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * A conservative PCM fallback for outputs that cannot accept the source layout.
 * Encoded passthrough never enters an AudioProcessor chain, so this cannot alter
 * TrueHD/DTS/AC3 bitstreams. Reference playback remains untouched for layouts the
 * active sink can represent.
 */
internal class AdaptiveChannelDownmixProcessor(
    private val maxOutputChannels: Int,
) : BaseAudioProcessor() {
    private var targetChannels = 0
    private var active = false
    private var frameSamples = FloatArray(0)
    private var nextFrameSamples = FloatArray(0)
    private var mixedSamples = FloatArray(0)
    private var nextMixedSamples = FloatArray(0)
    private var previousMixedSamples = FloatArray(0)

    override fun onConfigure(inputAudioFormat: AudioProcessor.AudioFormat): AudioProcessor.AudioFormat {
        if (!PcmSampleReader.isSupported(inputAudioFormat.encoding)) {
            targetChannels = inputAudioFormat.channelCount
            active = false
            return inputAudioFormat
        }
        targetChannels = targetChannelCount(inputAudioFormat.channelCount)
        active = targetChannels != inputAudioFormat.channelCount
        frameSamples = FloatArray(inputAudioFormat.channelCount.coerceAtLeast(0))
        nextFrameSamples = FloatArray(inputAudioFormat.channelCount.coerceAtLeast(0))
        mixedSamples = FloatArray(targetChannels.coerceAtLeast(0))
        nextMixedSamples = FloatArray(targetChannels.coerceAtLeast(0))
        previousMixedSamples = FloatArray(targetChannels.coerceAtLeast(0))
        return if (targetChannels == inputAudioFormat.channelCount) {
            inputAudioFormat
        } else {
            AudioProcessor.AudioFormat(
                inputAudioFormat.sampleRate,
                targetChannels,
                // Keep headroom while applying the channel matrix. This also
                // avoids truncating 24/32-bit PCM back to integer samples.
                C.ENCODING_PCM_FLOAT,
            )
        }
    }

    override fun isActive(): Boolean = active

    override fun queueInput(input: ByteBuffer) {
        val format = inputAudioFormat
        val outputFormat = outputAudioFormat
        if (format == AudioProcessor.AudioFormat.NOT_SET || outputFormat == AudioProcessor.AudioFormat.NOT_SET) return

        val bytesPerSample = PcmSampleReader.bytesPerSample(format.encoding)
        val inputChannels = format.channelCount
        if (inputChannels <= 0 || targetChannels <= 0) {
            input.position(input.limit())
            return
        }
        val outputFrames = input.remaining() / (bytesPerSample * inputChannels)
        val frameBytes = bytesPerSample * inputChannels
        val output = replaceOutputBuffer(outputFrames * targetChannels * 4)
            .order(ByteOrder.LITTLE_ENDIAN)
        val source = input.slice().order(ByteOrder.LITTLE_ENDIAN)
        val lookahead = source.duplicate().order(ByteOrder.LITTLE_ENDIAN)
        repeat(outputFrames) {
            repeat(inputChannels) { channel ->
                frameSamples[channel] = PcmSampleReader.read(source, format.encoding)
            }
            AudioDownmixMatrix.mixInto(frameSamples, targetChannels, mixedSamples)
            val hasNextFrame = source.remaining() >= frameBytes
            if (hasNextFrame) {
                lookahead.position(source.position())
                repeat(inputChannels) { channel ->
                    nextFrameSamples[channel] = PcmSampleReader.read(lookahead, format.encoding)
                }
                AudioDownmixMatrix.mixInto(nextFrameSamples, targetChannels, nextMixedSamples)
            }
            val linkedGain = AudioPeakLimiter.linkedGain(
                mixedSamples,
                previousMixedSamples,
                next = nextMixedSamples.takeIf { hasNextFrame },
            )
            repeat(targetChannels) { channel ->
                val value = mixedSamples[channel] * linkedGain
                val limited = AudioDownmixMatrix.softLimit(value)
                output.putFloat(limited)
                previousMixedSamples[channel] = limited
            }
        }
        input.position(input.limit())
        output.flip()
    }

    override fun onFlush() {
        previousMixedSamples.fill(0f)
    }

    private fun targetChannelCount(inputChannels: Int): Int {
        if (inputChannels <= maxOutputChannels) return inputChannels
        return when {
            maxOutputChannels >= 8 -> 8
            maxOutputChannels >= 7 -> 7
            maxOutputChannels >= 6 -> 6
            maxOutputChannels >= 2 -> maxOutputChannels.coerceAtMost(5)
            else -> 1
        }
    }

}
