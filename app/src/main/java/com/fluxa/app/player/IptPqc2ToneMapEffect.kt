@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.player

import android.content.Context
import android.opengl.GLES30
import androidx.media3.common.VideoFrameProcessingException
import androidx.media3.common.util.Size
import androidx.media3.common.util.UnstableApi
import androidx.media3.effect.BaseGlShaderProgram
import androidx.media3.effect.GlEffect
import java.nio.ByteBuffer
import java.nio.ByteOrder

// IPTPQc2 → SDR BT.709 pipeline for Dolby Vision Profile 5 CID=0 content.
//
// P5 CID=0 has no HDR10 fallback layer. Its HEVC base layer is encoded in IPTPQc2 (Dolby's
// ICtCp variant), NOT BT.2020 PQ. When the codec tag is rewritten to hvc1.2.4.L153.B0 so
// the HEVC decoder accepts the stream, the numeric pixel values are still IPTPQc2 signals.
// This effect applies the correct color-space transform and tone mapping before display.
//
// Assumes Media3 delivers the decoded frame as-is (PQ-encoded ICtCp values in [0,1]).
@UnstableApi
internal class IptPqc2ToneMapEffect : GlEffect {
    @Throws(VideoFrameProcessingException::class)
    override fun toGlShaderProgram(context: Context, useHdr: Boolean): IptPqc2ToneMapProgram =
        IptPqc2ToneMapProgram()

    override fun isNoOp(inputWidth: Int, inputHeight: Int): Boolean = false
}

@UnstableApi
internal class IptPqc2ToneMapProgram : BaseGlShaderProgram(
    /* useHighPrecisionColorComponents = */ true,
    /* texturePoolCapacity = */ 1
) {
    private var glProgram = 0
    private var positionAttrib = 0
    private var textureUniform = 0

    // Full-screen quad as triangle strip: BL, BR, TL, TR
    private val quadBuffer = ByteBuffer
        .allocateDirect(QUAD_POSITIONS.size * Float.SIZE_BYTES)
        .order(ByteOrder.nativeOrder())
        .asFloatBuffer()
        .apply { put(QUAD_POSITIONS); rewind() }

    @Throws(VideoFrameProcessingException::class)
    override fun configure(inputWidth: Int, inputHeight: Int): Size {
        if (glProgram != 0) {
            GLES30.glDeleteProgram(glProgram)
            glProgram = 0
        }
        val vs = compileShader(GLES30.GL_VERTEX_SHADER, VERTEX_SHADER)
        val fs = compileShader(GLES30.GL_FRAGMENT_SHADER, FRAGMENT_SHADER)
        glProgram = GLES30.glCreateProgram().also { prog ->
            GLES30.glAttachShader(prog, vs)
            GLES30.glAttachShader(prog, fs)
            GLES30.glLinkProgram(prog)
            GLES30.glDeleteShader(vs)
            GLES30.glDeleteShader(fs)
        }
        positionAttrib = GLES30.glGetAttribLocation(glProgram, "aPosition")
        textureUniform = GLES30.glGetUniformLocation(glProgram, "uTexture")
        return Size(inputWidth, inputHeight)
    }

    @Throws(VideoFrameProcessingException::class)
    override fun drawFrame(inputTexId: Int, presentationTimeUs: Long) {
        GLES30.glUseProgram(glProgram)
        GLES30.glActiveTexture(GLES30.GL_TEXTURE0)
        GLES30.glBindTexture(GLES30.GL_TEXTURE_2D, inputTexId)
        GLES30.glUniform1i(textureUniform, 0)
        GLES30.glEnableVertexAttribArray(positionAttrib)
        GLES30.glVertexAttribPointer(positionAttrib, 2, GLES30.GL_FLOAT, false, 0, quadBuffer)
        GLES30.glDrawArrays(GLES30.GL_TRIANGLE_STRIP, 0, 4)
        GLES30.glDisableVertexAttribArray(positionAttrib)
    }

    @Throws(VideoFrameProcessingException::class)
    override fun release() {
        if (glProgram != 0) {
            GLES30.glDeleteProgram(glProgram)
            glProgram = 0
        }
    }

    private fun compileShader(type: Int, src: String): Int {
        val shader = GLES30.glCreateShader(type)
        GLES30.glShaderSource(shader, src)
        GLES30.glCompileShader(shader)
        return shader
    }

    companion object {
        private val QUAD_POSITIONS = floatArrayOf(-1f, -1f, 1f, -1f, -1f, 1f, 1f, 1f)

        private const val VERTEX_SHADER = """
            #version 300 es
            in vec4 aPosition;
            out vec2 vUv;
            void main() {
                gl_Position = aPosition;
                vUv = vec2(aPosition.x * 0.5 + 0.5, aPosition.y * 0.5 + 0.5);
            }
        """

        // Full IPTPQc2 → SDR BT.709 conversion pipeline.
        //
        //   Input:  ICtCp values, PQ-encoded, normalized [0,1]  (Dolby Vision IPTPQc2)
        //   Output: BT.709 gamma-encoded SDR [0,1]
        //
        // Steps:
        //   1. ICtCp → PQ-encoded LMS     (M2⁻¹, ITU-R BT.2100-2)
        //   2. PQ EOTF per channel        (ST 2084; 1.0 = 10 000 nit)
        //   3. LMS → XYZ D65              (inverse of BT.2100 HPE matrix)
        //   4. XYZ → linear BT.709
        //   5. Luminance-based tone map   (linear for SDR range, smoothstep rolloff for HDR)
        //   6. BT.709 OETF                (gamma encode for SDR display)
        //
        // All mat3 literals below are GLSL column-major (transposed from the row-major spec values).
        private const val FRAGMENT_SHADER = """
            #version 300 es
            precision highp float;

            uniform sampler2D uTexture;
            in  vec2 vUv;
            out vec4 fragColor;

            // ST 2084 (PQ) EOTF — PQ-encoded signal → linear luminance [0,1] (1.0 = 10 000 nit)
            float pqEotf(float E) {
                float Ep  = pow(max(E, 0.0), 1.0 / 78.84375);
                float num = max(Ep - 0.8359375, 0.0);
                float den = max(18.8515625 - 18.6875 * Ep, 1e-6);
                return pow(num / den, 1.0 / 0.1593017578125);
            }

            // ICtCp → LMS'  (M2⁻¹ from ITU-R BT.2100-2)
            const mat3 M2_INV = mat3(
                 1.0,           1.0,           1.0,
                 0.0086053084, -0.0086053084,  0.5600319713,
                 0.1110296250, -0.1110296250, -0.3206271744
            );

            // Linear LMS → XYZ D65  (inverse of BT.2100 HPE matrix)
            const mat3 M_LMS_XYZ = mat3(
                 2.0701800566, 0.3650292938, -0.0495955500,
                -1.3264812278, 0.6804494857, -0.0494211680,
                 0.2066101766,-0.0454801996,  1.1879355232
            );

            // XYZ D65 → linear BT.709  (IEC 61966-2-1)
            const mat3 M_XYZ_BT709 = mat3(
                 3.2404542,-0.9692660, 0.0556434,
                -1.5371385, 1.8760108,-0.2040259,
                -0.4985314, 0.0415560, 1.0572252
            );

            // Piecewise tone map:
            //   [0, 100 nit]        → [0, 0.90]  linear   (SDR content at SDR brightness)
            //   [100, ~4000 nit]    → [0.90, 1.0] smoothstep rolloff
            // Applied per-luminance (preserves hue and saturation).
            float toneMapLuminance(float x) {
                const float SDR_WHITE = 0.01;
                const float HDR_PEAK  = 0.40;
                const float SDR_OUT   = 0.90;
                if (x <= 0.0)       return 0.0;
                if (x <= SDR_WHITE) return x * (SDR_OUT / SDR_WHITE);
                float t = clamp((x - SDR_WHITE) / (HDR_PEAK - SDR_WHITE), 0.0, 1.0);
                return SDR_OUT + (1.0 - SDR_OUT) * (t * t * (3.0 - 2.0 * t));
            }

            // BT.709 OETF (linear → gamma-encoded)
            float bt709Oetf(float L) {
                L = max(L, 0.0);
                return L < 0.0031308 ? 12.92 * L : 1.055 * pow(L, 1.0 / 2.4) - 0.055;
            }

            void main() {
                vec3 itp = texture(uTexture, vUv).rgb;

                // 1. ICtCp → PQ-encoded LMS
                vec3 lmsPq = M2_INV * itp;

                // 2. PQ EOTF → absolute linear LMS  (1.0 = 10 000 nit)
                vec3 lms = vec3(pqEotf(lmsPq.r), pqEotf(lmsPq.g), pqEotf(lmsPq.b));

                // 3. LMS → XYZ
                vec3 xyz = M_LMS_XYZ * lms;

                // 4. XYZ → linear BT.709
                vec3 rgb = max(M_XYZ_BT709 * xyz, 0.0);

                // 5. Luminance-based tone map
                float Y  = max(dot(rgb, vec3(0.2126, 0.7152, 0.0722)), 0.0);
                float Ym = toneMapLuminance(Y);
                float sc = (Y > 1e-4) ? (Ym / Y) : (0.90 / 0.01);
                rgb = clamp(rgb * sc, 0.0, 1.0);

                // 6. BT.709 OETF
                fragColor = vec4(bt709Oetf(rgb.r), bt709Oetf(rgb.g), bt709Oetf(rgb.b), 1.0);
            }
        """
    }
}
