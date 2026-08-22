package com.fluxa.app.player

data class NativeTorrentRuntimeInfo(
    val normalizedLink: String = "",
    val selectedFileIdx: Int? = null,
    val selectedReason: String? = null,
    val fallbackFileIndexes: List<Int> = emptyList(),
    val streamUrl: String = ""
)

data class NativeTorrentStatusInfo(
    val bufferProgress: Int = 0,
    val isPlayableEnough: Boolean = false,
    val statusKey: String = ""
)
