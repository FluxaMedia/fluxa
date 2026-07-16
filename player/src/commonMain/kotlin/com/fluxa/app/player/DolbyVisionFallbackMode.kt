package com.fluxa.app.player

enum class DolbyVisionFallbackMode {
    Auto,
    ForceHdr10,
    Off
}

internal fun DolbyVisionFallbackMode.toRustString(): String = when (this) {
    DolbyVisionFallbackMode.Off -> "off"
    DolbyVisionFallbackMode.Auto -> "auto"
    DolbyVisionFallbackMode.ForceHdr10 -> "force_hdr10"
}
