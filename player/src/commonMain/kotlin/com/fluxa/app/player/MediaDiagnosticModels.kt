package com.fluxa.app.player

data class LastMediaDebugInfo(
    val url: String = "",
    val title: String = "",
    val technicalInfo: String = "",
    val updatedAtMs: Long = 0L
) {
    val hasInfo: Boolean get() = technicalInfo.isNotBlank()
}
