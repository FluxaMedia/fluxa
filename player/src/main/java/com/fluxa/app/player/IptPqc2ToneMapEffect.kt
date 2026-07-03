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

internal object IptPqc2L1State {
    @Volatile var available: Boolean = false
    @Volatile var sdrWhiteLinear: Float = 0.01f
    @Volatile var hdrPeakLinear: Float = 0.40f
}

@UnstableApi
internal class IptPqc2ToneMapEffect(private val outputHdr: Boolean = false) : GlEffect {
    @Throws(VideoFrameProcessingException::class)
    override fun toGlShaderProgram(context: Context, useHdr: Boolean): IptPqc2ToneMapProgram =
        IptPqc2ToneMapProgram(outputHdr)

    override fun isNoOp(inputWidth: Int, inputHeight: Int): Boolean = false
}

@UnstableApi
internal class IptPqc2ToneMapProgram(private val outputHdr: Boolean = false) : BaseGlShaderProgram(
    /* useHighPrecisionColorComponents = */ true,
    /* texturePoolCapacity = */ 1
) {
    private var glProgram = 0
    private var positionAttrib = 0
    private var textureUniform = 0
    private var sdrWhiteUniform = -1
    private var hdrPeakUniform = -1

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
        val fragmentSrc = if (outputHdr) FRAGMENT_SHADER_HDR else FRAGMENT_SHADER_SDR
        val vs = compileShader(GLES30.GL_VERTEX_SHADER, VERTEX_SHADER)
        val fs = compileShader(GLES30.GL_FRAGMENT_SHADER, fragmentSrc)
        glProgram = GLES30.glCreateProgram().also { prog ->
            GLES30.glAttachShader(prog, vs)
            GLES30.glAttachShader(prog, fs)
            GLES30.glLinkProgram(prog)
            GLES30.glDeleteShader(vs)
            GLES30.glDeleteShader(fs)
        }
        positionAttrib = GLES30.glGetAttribLocation(glProgram, "aPosition")
        textureUniform = GLES30.glGetUniformLocation(glProgram, "uTexture")
        sdrWhiteUniform = GLES30.glGetUniformLocation(glProgram, "uSdrWhite")
        hdrPeakUniform = GLES30.glGetUniformLocation(glProgram, "uHdrPeak")
        return Size(inputWidth, inputHeight)
    }

    @Throws(VideoFrameProcessingException::class)
    override fun drawFrame(inputTexId: Int, presentationTimeUs: Long) {
        GLES30.glUseProgram(glProgram)
        if (sdrWhiteUniform >= 0) GLES30.glUniform1f(sdrWhiteUniform, IptPqc2L1State.sdrWhiteLinear)
        if (hdrPeakUniform >= 0) GLES30.glUniform1f(hdrPeakUniform, IptPqc2L1State.hdrPeakLinear)
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

        private const val FRAGMENT_SHADER_SDR = """
            #version 300 es
            precision highp float;

            uniform sampler2D uTexture;
            uniform float uSdrWhite;
            uniform float uHdrPeak;
            in  vec2 vUv;
            out vec4 fragColor;

            float pqEotf(float E) {
                float Ep  = pow(max(E, 0.0), 1.0 / 78.84375);
                float num = max(Ep - 0.8359375, 0.0);
                float den = max(18.8515625 - 18.6875 * Ep, 1e-6);
                return pow(num / den, 1.0 / 0.1593017578125);
            }

            const mat3 M2_INV = mat3(
                 1.0,           1.0,           1.0,
                 0.0086053084, -0.0086053084,  0.5600319713,
                 0.1110296250, -0.1110296250, -0.3206271744
            );

            const mat3 M_LMS_XYZ = mat3(
                 2.0701800566, 0.3650292938, -0.0495955500,
                -1.3264812278, 0.6804494857, -0.0494211680,
                 0.2066101766,-0.0454801996,  1.1879355232
            );

            const mat3 M_XYZ_BT709 = mat3(
                 3.2404542,-0.9692660, 0.0556434,
                -1.5371385, 1.8760108,-0.2040259,
                -0.4985314, 0.0415560, 1.0572252
            );

            float toneMapLuminance(float x, float sdrWhite, float hdrPeak) {
                const float SDR_OUT = 0.90;
                if (x <= 0.0)        return 0.0;
                if (x <= sdrWhite)   return x * (SDR_OUT / sdrWhite);
                float t = clamp((x - sdrWhite) / (hdrPeak - sdrWhite), 0.0, 1.0);
                return SDR_OUT + (1.0 - SDR_OUT) * (t * t * (3.0 - 2.0 * t));
            }

            float bt709Oetf(float L) {
                L = max(L, 0.0);
                return L < 0.0031308 ? 12.92 * L : 1.055 * pow(L, 1.0 / 2.4) - 0.055;
            }

            void main() {
                vec3 itp = texture(uTexture, vUv).rgb;
                vec3 lmsPq = M2_INV * itp;
                vec3 lms = vec3(pqEotf(lmsPq.r), pqEotf(lmsPq.g), pqEotf(lmsPq.b));
                vec3 xyz = M_LMS_XYZ * lms;
                vec3 rgb = max(M_XYZ_BT709 * xyz, 0.0);

                float sdrW = max(uSdrWhite, 0.001);
                float hdrP = max(uHdrPeak, sdrW + 0.001);
                float Y  = max(dot(rgb, vec3(0.2126, 0.7152, 0.0722)), 0.0);
                float Ym = toneMapLuminance(Y, sdrW, hdrP);
                float sc = (Y > 1e-4) ? (Ym / Y) : (0.90 / sdrW);
                rgb = clamp(rgb * sc, 0.0, 1.0);

                fragColor = vec4(bt709Oetf(rgb.r), bt709Oetf(rgb.g), bt709Oetf(rgb.b), 1.0);
            }
        """

        private const val FRAGMENT_SHADER_HDR = """
            #version 300 es
            precision highp float;

            uniform sampler2D uTexture;
            in  vec2 vUv;
            out vec4 fragColor;

            float pqEotf(float E) {
                float Ep  = pow(max(E, 0.0), 1.0 / 78.84375);
                float num = max(Ep - 0.8359375, 0.0);
                float den = max(18.8515625 - 18.6875 * Ep, 1e-6);
                return pow(num / den, 1.0 / 0.1593017578125);
            }

            float pqOetf(float L) {
                float Lm1 = pow(max(L, 0.0), 0.1593017578125);
                return pow((0.8359375 + 18.8515625 * Lm1) / (1.0 + 18.6875 * Lm1), 78.84375);
            }

            const mat3 M2_INV = mat3(
                 1.0,           1.0,           1.0,
                 0.0086053084, -0.0086053084,  0.5600319713,
                 0.1110296250, -0.1110296250, -0.3206271744
            );

            const mat3 M_LMS_XYZ = mat3(
                 2.0701800566, 0.3650292938, -0.0495955500,
                -1.3264812278, 0.6804494857, -0.0494211680,
                 0.2066101766,-0.0454801996,  1.1879355232
            );

            const mat3 M_XYZ_BT2020 = mat3(
                 1.7166512, -0.6666844,  0.0176399,
                -0.3556708,  1.6164812, -0.0427706,
                -0.0428226,  0.0157685,  0.9421031
            );

            void main() {
                vec3 itp = texture(uTexture, vUv).rgb;
                vec3 lmsPq = M2_INV * itp;
                vec3 lms = vec3(pqEotf(lmsPq.r), pqEotf(lmsPq.g), pqEotf(lmsPq.b));
                vec3 xyz = M_LMS_XYZ * lms;
                vec3 rgb = max(M_XYZ_BT2020 * xyz, 0.0);
                fragColor = vec4(pqOetf(rgb.r), pqOetf(rgb.g), pqOetf(rgb.b), 1.0);
            }
        """
    }
}
