package com.fluxa.app.player

enum class DolbyVisionFallbackMode {
    Auto,
    ConvertToDv81,
    Off
}

internal fun DolbyVisionFallbackMode.toRustString(): String = when (this) {
    DolbyVisionFallbackMode.Off -> "off"
    DolbyVisionFallbackMode.Auto -> "auto"
    DolbyVisionFallbackMode.ConvertToDv81 -> "convert_dv81"
}
