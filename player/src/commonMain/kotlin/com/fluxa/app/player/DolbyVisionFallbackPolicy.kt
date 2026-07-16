package com.fluxa.app.player


data class DolbyVisionCapabilities(
    val displaySupportsDolbyVision: Boolean,
    val displaySupportsHdr10: Boolean,
    val displaySupportsHdr10Plus: Boolean,
    val displaySupportsHlg: Boolean,
    val decoderNativeP7: Boolean,
    val decoderP5: Boolean,
    val decoderP8: Boolean,
    val decoderAnyDv: Boolean,
) {
    val mediaCodecSupportsDolbyVision: Boolean get() = decoderAnyDv
    val displaySupportsHdr10Family: Boolean get() = displaySupportsHdr10 || displaySupportsHdr10Plus
}

data class DolbyVisionManifestRewrite(
    val manifest: String,
    val decision: String,
    val requiresIptPqc2ToneMap: Boolean = false,
    val hasP81Conversion: Boolean = false,
)

data class DvContainerInfo(val profile: Int, val compatId: Int) {
    val notHasHdrFallback: Boolean get() =
        profile == 4 || (profile == 5 && compatId != 1) || (profile == 10 && compatId in setOf(0, 2, 3))
}

object DolbyVisionFallbackPolicy {
    private const val HEVC_HDR_BASE = "hvc1.2.4.L153.B0"
    private const val DVCC_SCAN_WINDOW = 65536

    private val dolbyVisionCodec = Regex(
        "(?i)\\b(?:dvhe|dvh1)\\.(\\d{2})(?:\\.(\\d{2}))?(?:\\.[A-Za-z0-9]+)*"
    )

    fun rewriteManifest(
        manifest: String,
        mode: DolbyVisionFallbackMode,
        capabilities: DolbyVisionCapabilities
    ): DolbyVisionManifestRewrite {
        if (mode == DolbyVisionFallbackMode.Off) {
            return DolbyVisionManifestRewrite(manifest, "dv_fallback=off")
        }
        if (mode == DolbyVisionFallbackMode.ForceHdr10) {
            val rewritten = dolbyVisionCodec.replace(manifest) { match ->
                val profile = match.groupValues.getOrNull(1)?.toIntOrNull()
                val compatId = match.groupValues.getOrNull(2)?.takeIf { it.isNotBlank() }?.toIntOrNull()
                val noFallback = profile == 4 ||
                    (profile == 5 && compatId != 1) ||
                    (profile == 10 && compatId in setOf(0, 2, 3))
                if (noFallback) match.value else HEVC_HDR_BASE
            }
            return DolbyVisionManifestRewrite(rewritten, "dv_fallback=force_hdr10")
        }
        var iptpqc2Required = false
        var p81Conversion = false
        val decisions = linkedSetOf<String>()
        val rewritten = dolbyVisionCodec.replace(manifest) { match ->
            val token = match.value
            val profile = match.groupValues.getOrNull(1)?.toIntOrNull()
            val compatId = match.groupValues.getOrNull(2)?.takeIf { it.isNotBlank() }?.toIntOrNull()

            val action = decide(profile, compatId, capabilities)

            val replacement = when (action) {
                DvCodecAction.Keep, DvCodecAction.Safety -> null
                DvCodecAction.RewriteToP81 -> {
                    p81Conversion = true
                    token.replace(Regex("(?i)^(dvhe|dvh1)\\.07"), "$1.08")
                }
                is DvCodecAction.StripToHevc -> {
                    if (action.iptPqc2) iptpqc2Required = true
                    HEVC_HDR_BASE
                }
            }

            val label = when (action) {
                DvCodecAction.Keep -> "keep_dolby_vision"
                DvCodecAction.Safety -> "safety_gate"
                DvCodecAction.RewriteToP81 -> "p7_to_p81"
                is DvCodecAction.StripToHevc -> if (action.iptPqc2) "strip_iptpqc2" else "strip_to_hdr10"
            }
            val profileLabel = profile?.let { "profile=$it" } ?: "profile=unknown"
            decisions += "dv_fallback=$label / $profileLabel / cid=${compatId ?: "?"} / decoder_p7=${capabilities.decoderNativeP7} / decoder_p8=${capabilities.decoderP8}"

            replacement ?: token
        }
        return DolbyVisionManifestRewrite(
            manifest = rewritten,
            decision = decisions.joinToString("\n").ifBlank { "dv_fallback=no_dolby_vision_manifest_codec" },
            requiresIptPqc2ToneMap = iptpqc2Required,
            hasP81Conversion = p81Conversion,
        )
    }

    private sealed class DvCodecAction {
        object Keep : DvCodecAction()
        object RewriteToP81 : DvCodecAction()
        data class StripToHevc(val iptPqc2: Boolean = false) : DvCodecAction()
        object Safety : DvCodecAction()
    }

    private fun decide(
        profile: Int?,
        compatId: Int?,
        caps: DolbyVisionCapabilities,
    ): DvCodecAction = when {
        profile == 4 ->
            if (caps.decoderAnyDv) DvCodecAction.Keep else DvCodecAction.Safety

        profile == 5 && compatId != 1 ->
            if (caps.decoderAnyDv) DvCodecAction.Keep
            else DvCodecAction.StripToHevc(iptPqc2 = true)

        profile == 7 -> when {
            caps.decoderNativeP7 -> DvCodecAction.Keep
            caps.decoderP8 -> DvCodecAction.RewriteToP81
            else -> DvCodecAction.StripToHevc()
        }

        profile == 10 && compatId in setOf(0, 2, 3) ->
            if (caps.decoderAnyDv) DvCodecAction.Keep else DvCodecAction.Safety

        else ->
            if (caps.decoderAnyDv) DvCodecAction.Keep
            else DvCodecAction.StripToHevc()
    }

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
        val limit = data.size - 3
        var i = 0
        while (i < limit) {
            val a = data[i]; val b = data[i + 1]; val c = data[i + 2]; val d = data[i + 3]
            val d_ = 'd'.code.toByte(); val v = 'v'.code.toByte()
            val cC = 'c'.code.toByte(); val bigC = 'C'.code.toByte()
            val h = 'h'.code.toByte()
            val e = 'e'.code.toByte(); val one = '1'.code.toByte()
            if (a == d_ && b == v) {
                val matched = (c == cC && d == bigC) ||
                    (c == v  && d == bigC) ||
                    (c == h  && d == e) ||
                    (c == h  && d == one)
                if (matched) {
                    data[i] = x; data[i + 1] = x; data[i + 2] = x; data[i + 3] = x
                    i += 4
                    continue
                }
            }
            i++
        }
    }

    fun containerDvSupportedForCaps(info: DvContainerInfo, caps: DolbyVisionCapabilities): Boolean = when {
        info.profile == 7 -> caps.decoderNativeP7
        else -> caps.decoderAnyDv
    }

    fun containerDvccScanWindow(): Int = DVCC_SCAN_WINDOW

    fun parseContentRangeStart(header: String?): Long? {
        header ?: return null
        val s = header.removePrefix("bytes ").trim()
        val parts = s.split('/')
        if (parts.size != 2) return null
        return parts[0].split('-').firstOrNull()?.trim()?.toLongOrNull()
    }

}
