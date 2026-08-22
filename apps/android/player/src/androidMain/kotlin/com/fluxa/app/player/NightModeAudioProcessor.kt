@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.player

import androidx.media3.common.C
import androidx.media3.common.audio.AudioProcessor
import androidx.media3.common.audio.BaseAudioProcessor
import com.fluxa.app.shared.feature.player.AudioDialogueBoost
import com.fluxa.app.shared.feature.player.AudioPeakLimiter
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.math.abs
import kotlin.math.exp

/**
 * Deliberately modest night processing. It only runs after decoding to PCM;
 * encoded passthrough remains untouched. The center channel gets a small lift
 * because film dialogue is normally carried there, followed by a soft-knee
 * compressor with fixed headroom.
 */
internal class NightModeAudioProcessor : BaseAudioProcessor() {
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
        envelopeAttack = coefficient(sampleRate, 0.020f)
        envelopeRelease = coefficient(sampleRate, 0.250f)
        gainAttack = coefficient(sampleRate, 0.020f)
        gainRelease = coefficient(sampleRate, 0.250f)
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
            val samples = frameSamples
            repeat(channels) { channel ->
                samples[channel] = PcmSampleReader.read(source, format.encoding).takeIf { it.isFinite() } ?: 0f
            }
            AudioDialogueBoost.applyInPlace(samples, channels)
            samples.forEachIndexed { channel, sample -> samples[channel] = sample * 1.08f }
            val peak = samples.maxOfOrNull { abs(it) } ?: 0f
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
                AudioDialogueBoost.applyInPlace(nextSamples, channels)
                nextSamples.forEachIndexed { channel, sample -> nextSamples[channel] = sample * 1.08f }
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
                ceiling = 0.92f,
                next = nextSamples.takeIf { hasNextFrame },
            )
            samples.forEachIndexed { channel, sample ->
                val limitedSample = sample * linkedLimiterGain
                samples[channel] = limitedSample
                output.putFloat(limitedSample
                    .takeIf { it.isFinite() }?.coerceIn(-0.92f, 0.92f) ?: 0f)
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
        val threshold = 0.55f
        if (peak <= threshold || peak <= 0f) return 1f
        val compressedPeak = if (peak <= 1f) {
            threshold + (peak - threshold) / 3f
        } else {
            threshold + (1f - threshold) / 3f
        }
        return (compressedPeak / peak).coerceAtMost(1f)
    }

}
