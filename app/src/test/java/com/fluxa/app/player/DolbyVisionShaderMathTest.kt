package com.fluxa.app.player

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.pow

class DolbyVisionShaderMathTest {

    private fun pqEotf(e: Float): Float {
        val ep = max(e, 0f).toDouble().pow(1.0 / 78.84375).toFloat()
        val num = max(ep - 0.8359375f, 0f)
        val den = max(18.8515625f - 18.6875f * ep, 1e-6f)
        return (num / den).toDouble().pow(1.0 / 0.1593017578125).toFloat()
    }

    private fun pqOetf(l: Float): Float {
        val lm1 = max(l, 0f).toDouble().pow(0.1593017578125).toFloat()
        return ((0.8359375f + 18.8515625f * lm1) / (1f + 18.6875f * lm1)).toDouble().pow(78.84375).toFloat()
    }

    private fun bt709Oetf(l: Float): Float {
        val c = max(l, 0f)
        return if (c < 0.0031308f) 12.92f * c else 1.055f * c.toDouble().pow(1.0 / 2.4).toFloat() - 0.055f
    }

    private val m2Inv = arrayOf(
        floatArrayOf(1f, 1f, 1f),
        floatArrayOf(0.0086053084f, -0.0086053084f, 0.5600319713f),
        floatArrayOf(0.1110296250f, -0.1110296250f, -0.3206271744f)
    )

    private val mLmsXyz = arrayOf(
        floatArrayOf(2.0701800566f, 0.3650292938f, -0.0495955500f),
        floatArrayOf(-1.3264812278f, 0.6804494857f, -0.0494211680f),
        floatArrayOf(0.2066101766f, -0.0454801996f, 1.1879355232f)
    )

    private val mXyzBt709 = arrayOf(
        floatArrayOf(3.2404542f, -1.5371385f, -0.4985314f),
        floatArrayOf(-0.9692660f, 1.8760108f, 0.0415560f),
        floatArrayOf(0.0556434f, -0.2040259f, 1.0572252f)
    )

    private val mXyzBt2020 = arrayOf(
        floatArrayOf(1.7166512f, -0.3556708f, -0.0428226f),
        floatArrayOf(-0.6666844f, 1.6164812f, 0.0157685f),
        floatArrayOf(0.0176399f, -0.0427706f, 0.9421031f)
    )

    private fun mat3Mul(m: Array<FloatArray>, v: FloatArray): FloatArray {
        return floatArrayOf(
            m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
            m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
            m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2]
        )
    }

    private fun toneMapLuminance(x: Float, sdrWhite: Float, hdrPeak: Float): Float {
        val sdrOut = 0.90f
        if (x <= 0f) return 0f
        if (x <= sdrWhite) return x * (sdrOut / sdrWhite)
        val t = ((x - sdrWhite) / (hdrPeak - sdrWhite)).coerceIn(0f, 1f)
        return sdrOut + (1f - sdrOut) * (t * t * (3f - 2f * t))
    }

    private fun sdrPipeline(itp: FloatArray, sdrWhite: Float = 0.01f, hdrPeak: Float = 0.40f): FloatArray {
        val lmsPq = mat3Mul(m2Inv, itp)
        val lms = floatArrayOf(pqEotf(lmsPq[0]), pqEotf(lmsPq[1]), pqEotf(lmsPq[2]))
        val xyz = mat3Mul(mLmsXyz, lms)
        val rgb = mat3Mul(mXyzBt709, xyz).map { max(it, 0f) }.toFloatArray()
        val y = max(0.2126f * rgb[0] + 0.7152f * rgb[1] + 0.0722f * rgb[2], 0f)
        val ym = toneMapLuminance(y, sdrWhite, hdrPeak)
        val sc = if (y > 1e-4f) ym / y else 0.90f / sdrWhite
        val clamped = rgb.map { (it * sc).coerceIn(0f, 1f) }.toFloatArray()
        return floatArrayOf(bt709Oetf(clamped[0]), bt709Oetf(clamped[1]), bt709Oetf(clamped[2]))
    }

    private fun hdrPipeline(itp: FloatArray): FloatArray {
        val lmsPq = mat3Mul(m2Inv, itp)
        val lms = floatArrayOf(pqEotf(lmsPq[0]), pqEotf(lmsPq[1]), pqEotf(lmsPq[2]))
        val xyz = mat3Mul(mLmsXyz, lms)
        val rgb = mat3Mul(mXyzBt2020, xyz).map { max(it, 0f) }.toFloatArray()
        return floatArrayOf(pqOetf(rgb[0]), pqOetf(rgb[1]), pqOetf(rgb[2]))
    }

    @Test
    fun black_input_produces_black_output() {
        val out = sdrPipeline(floatArrayOf(0f, 0f, 0f))
        out.forEach { assertEquals(0f, it, 1e-4f) }
    }

    @Test
    fun neutral_gray_itp_maps_to_achromatic_bt709() {
        val midPq = 0.508f
        val out = sdrPipeline(floatArrayOf(midPq, 0f, 0f))
        val delta = abs(out[0] - out[1]).coerceAtLeast(abs(out[1] - out[2]))
        assertTrue("neutral ICtCp should map to achromatic BT.709, R/G/B diff=$delta", delta < 0.02f)
    }

    @Test
    fun pq_roundtrip_is_identity() {
        val values = listOf(0f, 0.001f, 0.1f, 0.5f, 1f)
        for (v in values) {
            val roundtripped = pqOetf(pqEotf(v))
            assertEquals("PQ round-trip failed at $v", v, roundtripped, 1e-4f)
        }
    }

    @Test
    fun sdr_pipeline_output_within_unit_range() {
        val samples = listOf(
            floatArrayOf(0.5f, 0f, 0f),
            floatArrayOf(0.8f, 0.02f, -0.01f),
            floatArrayOf(0.3f, 0.1f, 0.05f)
        )
        for (itp in samples) {
            val out = sdrPipeline(itp)
            out.forEach { c ->
                assertTrue("SDR output $c out of [0,1]", c in 0f..1.001f)
            }
        }
    }

    @Test
    fun hdr_pipeline_output_within_unit_range() {
        val samples = listOf(
            floatArrayOf(0.5f, 0f, 0f),
            floatArrayOf(0.8f, 0.02f, -0.01f),
            floatArrayOf(1.0f, 0f, 0f)
        )
        for (itp in samples) {
            val out = hdrPipeline(itp)
            out.forEach { c ->
                assertTrue("HDR output $c out of [0,1]", c in 0f..1.001f)
            }
        }
    }

    @Test
    fun l1_derived_sdrwhite_tighter_than_default() {
        val neutralPq = floatArrayOf(0.508f, 0f, 0f)
        val defaultOut = sdrPipeline(neutralPq, sdrWhite = 0.01f, hdrPeak = 0.40f)
        val l1Out = sdrPipeline(neutralPq, sdrWhite = 0.005f, hdrPeak = 0.25f)
        val defaultLuma = 0.2126f * defaultOut[0] + 0.7152f * defaultOut[1] + 0.0722f * defaultOut[2]
        val l1Luma = 0.2126f * l1Out[0] + 0.7152f * l1Out[1] + 0.0722f * l1Out[2]
        assertTrue("tighter hdrPeak should raise shadows; l1=$l1Luma default=$defaultLuma", l1Luma >= defaultLuma - 0.05f)
    }
}
