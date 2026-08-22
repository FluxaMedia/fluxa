@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.player

import androidx.media3.common.C
import androidx.media3.common.audio.AudioProcessor
import androidx.media3.common.audio.BaseAudioProcessor
import com.fluxa.app.shared.feature.player.AudioPeakLimiter
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.math.abs
import kotlin.math.exp

/**
 * A deliberately mild PCM-only compressor for Balanced mode. It never enters
 * an encoded passthrough path and leaves Reference mode completely untouched.
 */
internal class BalancedAudioProcessor : BaseAudioProcessor() {
    private var enabled = false
    private var frameSamples = FloatArray(0)
    private var nextSamples = FloatArray(0)
    private var previousSamples = FloatArray(0)
    private var envelope = 0f
    private var gain = 1f
    private var envelopeAttack = 0f
    private var envelopeRelease = 0f
    private var gainAttack = 0f
    private var gainRelease = 0f

    override fun onConfigure(inputAudioFormat: AudioProcessor.AudioFormat): AudioProcessor.AudioFormat {
        enabled = PcmSampleReader.isSupported(inputAudioFormat.encoding)
        frameSamples = FloatArray(inputAudioFormat.channelCount.coerceAtLeast(0))
        nextSamples = FloatArray(inputAudioFormat.channelCount.coerceAtLeast(0))
        previousSamples = FloatArray(inputAudioFormat.channelCount.coerceAtLeast(0))
        val sampleRate = inputAudioFormat.sampleRate.coerceAtLeast(1)
        envelopeAttack = coefficient(sampleRate, 0.030f)
        envelopeRelease = coefficient(sampleRate, 0.300f)
        gainAttack = coefficient(sampleRate, 0.030f)
        gainRelease = coefficient(sampleRate, 0.300f)
        resetDynamics()
        return if (enabled) {
            AudioProcessor.AudioFormat(
                inputAudioFormat.sampleRate,
                inputAudioFormat.channelCount,
                C.ENCODING_PCM_FLOAT,
            )
        } else {
            inputAudioFormat
        }
    }

    override fun isActive(): Boolean = enabled

    override fun queueInput(input: ByteBuffer) {
        val format = inputAudioFormat
        val inputBytesPerSample = PcmSampleReader.bytesPerSample(format.encoding)
        val channels = format.channelCount
        if (channels <= 0) {
            input.position(input.limit())
            return
        }
        val frames = input.remaining() / (inputBytesPerSample * channels)
        val frameBytes = inputBytesPerSample * channels
        val output = replaceOutputBuffer(frames * channels * 4).order(ByteOrder.LITTLE_ENDIAN)
        val source = input.slice().order(ByteOrder.LITTLE_ENDIAN)
        val lookahead = source.duplicate().order(ByteOrder.LITTLE_ENDIAN)
        repeat(frames) {
            // Link the compressor across the complete frame. Independent
            // per-channel compression changes the stereo/surround image.
            val samples = frameSamples
            var peak = 0f
            repeat(channels) { channel ->
                val sample = PcmSampleReader.read(source, format.encoding).takeIf { it.isFinite() } ?: 0f
                samples[channel] = sample
                peak = maxOf(peak, abs(sample))
            }
            envelope = smooth(envelope, peak, if (peak > envelope) envelopeAttack else envelopeRelease)
            val targetGain = compressionGain(envelope)
            gain = smooth(gain, targetGain, if (targetGain < gain) gainAttack else gainRelease)
            samples.forEachIndexed { channel, sample -> samples[channel] = sample * gain }
            val hasNextFrame = source.remaining() >= frameBytes
            if (hasNextFrame) {
                lookahead.position(source.position())
                repeat(channels) { channel ->
                    nextSamples[channel] = PcmSampleReader.read(lookahead, format.encoding).takeIf { it.isFinite() } ?: 0f
                }
                // Preview the same compressor step for look-ahead. Feeding
                // raw PCM here makes the limiter react to a signal that will
                // never reach the sink in Balanced mode.
                var nextPeak = 0f
                nextSamples.forEach { nextPeak = maxOf(nextPeak, abs(it)) }
                val nextEnvelope = smooth(envelope, nextPeak, if (nextPeak > envelope) envelopeAttack else envelopeRelease)
                val nextTargetGain = compressionGain(nextEnvelope)
                val nextGain = smooth(gain, nextTargetGain, if (nextTargetGain < gain) gainAttack else gainRelease)
                nextSamples.forEachIndexed { channel, sample -> nextSamples[channel] = sample * nextGain }
            }
            val linkedLimiterGain = AudioPeakLimiter.linkedGain(
                samples,
                previousSamples,
                next = nextSamples.takeIf { hasNextFrame },
            )
            samples.forEachIndexed { channel, sample ->
                val limitedSample = sample * linkedLimiterGain
                samples[channel] = limitedSample
                output.putFloat(limitedSample
                    .takeIf { it.isFinite() }?.coerceIn(-0.98f, 0.98f) ?: 0f)
            }
            samples.copyInto(previousSamples)
        }
        input.position(input.limit())
        output.flip()
    }

    override fun onFlush() {
        resetDynamics()
    }

    private fun resetDynamics() {
        envelope = 0f
        gain = 1f
        previousSamples.fill(0f)
    }

    private fun coefficient(sampleRate: Int, seconds: Float): Float =
        exp(-1f / (sampleRate * seconds)).coerceIn(0f, 1f)

    private fun smooth(previous: Float, target: Float, coefficient: Float): Float =
        coefficient * previous + (1f - coefficient) * target

    private fun compressionGain(peak: Float): Float {
        val threshold = 0.78f
        if (peak <= threshold || peak <= 0f) return 1f
        val compressedPeak = if (peak <= 1f) {
            threshold + (peak - threshold) / 1.5f
        } else {
            threshold + (1f - threshold) / 1.5f
        }
        return (compressedPeak / peak).coerceAtMost(1f)
    }

}
