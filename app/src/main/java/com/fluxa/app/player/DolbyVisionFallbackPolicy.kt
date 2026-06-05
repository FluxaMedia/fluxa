package com.fluxa.app.player

import android.content.Context
import android.hardware.display.DisplayManager
import android.media.MediaCodecList
import android.view.Display
import androidx.media3.common.MimeTypes
import java.util.Locale

internal data class DolbyVisionCapabilities(
    val displaySupportsDolbyVision: Boolean,
    val mediaCodecSupportsDolbyVision: Boolean
)

internal data class DolbyVisionManifestRewrite(
    val manifest: String,
    val decision: String,
    val requiresIptPqc2ToneMap: Boolean = false
)

internal data class DvContainerInfo(val profile: Int, val compatId: Int) {
    // Mirrors the HLS manifest logic: P5 CID=1 has an HDR10 base layer and CAN fall back.
    val notHasHdrFallback: Boolean get() =
        profile == 4 || (profile == 5 && compatId != 1) || (profile == 10 && compatId in setOf(0, 2, 3))
}

internal object DolbyVisionFallbackPolicy {
    private const val HEVC_HDR_BASE = "hvc1.2.4.L153.B0"
    private const val DVCC_SCAN_WINDOW = 65536

    // Only matches HEVC-based DV codecs (dvhe/dvh1). AV1-based DV (dva1/dvav) is excluded:
    // we have no valid AV1 fallback codec string, so rewriting them to hvc1 would break decoding.
    private val dolbyVisionCodec = Regex(
        "(?i)\\b(?:dvhe|dvh1)\\.(\\d{2})(?:\\.(\\d{2}))?(?:\\.[A-Za-z0-9]+)*"
    )

    fun capabilities(context: Context): DolbyVisionCapabilities {
        return DolbyVisionCapabilities(
            displaySupportsDolbyVision = displaySupportsDolbyVision(context),
            mediaCodecSupportsDolbyVision = mediaCodecSupportsDolbyVision()
        )
    }

    fun rewriteManifest(
        manifest: String,
        mode: DolbyVisionFallbackMode,
        capabilities: DolbyVisionCapabilities
    ): DolbyVisionManifestRewrite {
        if (mode == DolbyVisionFallbackMode.Off) {
            return DolbyVisionManifestRewrite(manifest, "dv_fallback=off")
        }
        var changed = false
        var iptpqc2Required = false
        val decisions = linkedSetOf<String>()
        val rewritten = dolbyVisionCodec.replace(manifest) { match ->
            val token = match.value
            val profile = match.groupValues.getOrNull(1)?.toIntOrNull()
            val compatibilityId = match.groupValues.getOrNull(2)?.takeIf { it.isNotBlank() }?.toIntOrNull()
            // P5 CID=1 has an HDR10 base layer and CAN fall back to hvc1 — do not mark as noHdrFallback.
            val noHdrFallback = profile == 4 ||
                (profile == 5 && compatibilityId != 1) ||
                (profile == 10 && compatibilityId in setOf(0, 2, 3))
            val replacement = when (mode) {
                DolbyVisionFallbackMode.Off -> null
                DolbyVisionFallbackMode.Auto -> when {
                    capabilities.mediaCodecSupportsDolbyVision &&
                        (capabilities.displaySupportsDolbyVision || noHdrFallback) -> null
                    !noHdrFallback -> HEVC_HDR_BASE
                    // P5 CID=0 (no HDR base layer): rewrite to HEVC so the decoder accepts the
                    // stream. Uses a unique codec marker so onVideoInputFormatChanged can activate
                    // the IPTPQc2 → SDR shader without relying on a separate boolean flag.
                    profile == 5 && compatibilityId != 1 -> HEVC_HDR_BASE.also { iptpqc2Required = true }
                    else -> null
                }
                DolbyVisionFallbackMode.ConvertToDv81 -> when {
                    // Full DV device: passthrough — stream plays natively, no rewrite needed.
                    capabilities.mediaCodecSupportsDolbyVision && capabilities.displaySupportsDolbyVision -> null
                    // DV decoder present but no DV display.
                    capabilities.mediaCodecSupportsDolbyVision -> when {
                        // P7: rewrite codec string to DV8.1 so the decoder expects single-layer RPU.
                        // The proxy handles the per-sample RPU conversion. Only the profile digits
                        // change (07 → 08); the compat-id and level fields are preserved.
                        // Anchored to start so it only replaces the profile field, not compat-id.
                        profile == 7 -> token.replace(Regex("(?i)^(dvhe|dvh1)\\.07"), "$1.08")
                        // All other profiles: DV decoder handles them natively → keep as DV.
                        else -> null
                    }
                    // No DV decoder: same fallback paths as Auto.
                    !noHdrFallback -> HEVC_HDR_BASE
                    profile == 5 && compatibilityId != 1 -> HEVC_HDR_BASE.also { iptpqc2Required = true }
                    else -> null
                }
            }
            val profileLabel = profile?.let { "profile=$it" } ?: "profile=unknown"
            if (replacement == null) {
                decisions += "dv_fallback=keep_dolby_vision / $profileLabel / compatibility=${compatibilityId ?: "unknown"} / display_dv=${capabilities.displaySupportsDolbyVision} / codec_dv=${capabilities.mediaCodecSupportsDolbyVision}"
                token
            } else {
                changed = true
                decisions += "dv_fallback=${fallbackLabel(replacement)} / $profileLabel / compatibility=${compatibilityId ?: "unknown"} / display_dv=${capabilities.displaySupportsDolbyVision} / codec_dv=${capabilities.mediaCodecSupportsDolbyVision}"
                replacement
            }
        }
        return DolbyVisionManifestRewrite(
            manifest = rewritten,
            decision = decisions.joinToString("\n").ifBlank {
                if (changed) "dv_fallback=rewritten" else "dv_fallback=no_dolby_vision_manifest_codec"
            },
            requiresIptPqc2ToneMap = iptpqc2Required
        )
    }

    // ── Container-level DVCC stripping (for MKV/MP4 direct streams) ──────────────

    fun scanDvContainerInfo(data: ByteArray): DvContainerInfo? {
        val limit = data.size - 8
        for (i in 0..limit) {
            if (data[i] == 'd'.code.toByte() && data[i + 1] == 'v'.code.toByte() &&
                data[i + 2] == 'c'.code.toByte() && data[i + 3] == 'C'.code.toByte()
            ) {
                val payload = data.drop(i + 4)
                if (payload.size < 5) return null
                val profile = ((payload[2].toInt() and 0xFF) shr 1) and 0x7F
                val compatId = ((payload[4].toInt() and 0xFF) shr 4) and 0x0F
                return DvContainerInfo(profile, compatId)
            }
        }
        return null
    }

    fun mangleDvccFourcc(data: ByteArray) {
        val x = 'X'.code.toByte()
        val limit = data.size - 3  // i < limit ensures data[i+3] is always in bounds
        var i = 0
        while (i < limit) {
            val a = data[i]; val b = data[i + 1]; val c = data[i + 2]; val d = data[i + 3]
            val d_ = 'd'.code.toByte(); val v = 'v'.code.toByte()
            val cC = 'c'.code.toByte(); val bigC = 'C'.code.toByte()
            val h = 'h'.code.toByte()
            val e = 'e'.code.toByte(); val one = '1'.code.toByte()
            if (a == d_ && b == v) {
                val matched = (c == cC && d == bigC) ||  // dvcC
                    (c == v  && d == bigC) ||             // dvvC
                    (c == h  && d == e) ||                // dvhe
                    (c == h  && d == one)                 // dvh1
                if (matched) {
                    data[i] = x; data[i + 1] = x; data[i + 2] = x; data[i + 3] = x
                    i += 4
                    continue
                }
            }
            i++
        }
    }

    fun containerDvccScanWindow(): Int = DVCC_SCAN_WINDOW

    fun parseContentRangeStart(header: String?): Long? {
        header ?: return null
        val s = header.removePrefix("bytes ").trim()
        val parts = s.split('/')
        if (parts.size != 2) return null
        return parts[0].split('-').firstOrNull()?.trim()?.toLongOrNull()
    }

    private fun fallbackLabel(replacement: String): String {
        val lower = replacement.lowercase(Locale.ROOT)
        return when {
            lower.startsWith("hvc1") || lower.startsWith("hev1") -> "dolby_vision_to_hdr_base"
            lower.startsWith("dvhe") || lower.startsWith("dvh1") -> "dolby_vision_p7_to_dv81"
            else -> "dolby_vision_to_$replacement"
        }
    }

    @Suppress("DEPRECATION")
    private fun displaySupportsDolbyVision(context: Context): Boolean {
        val displayManager = context.getSystemService(Context.DISPLAY_SERVICE) as? DisplayManager
        val display = displayManager?.getDisplay(Display.DEFAULT_DISPLAY) ?: return false
        return display.hdrCapabilities.supportedHdrTypes.any { it == Display.HdrCapabilities.HDR_TYPE_DOLBY_VISION }
    }

    private fun mediaCodecSupportsDolbyVision(): Boolean {
        return runCatching {
            MediaCodecList(MediaCodecList.REGULAR_CODECS).codecInfos.any { codecInfo ->
                !codecInfo.isEncoder && codecInfo.supportedTypes.any {
                    it.equals(MimeTypes.VIDEO_DOLBY_VISION, ignoreCase = true)
                }
            }
        }.getOrDefault(false)
    }
}
