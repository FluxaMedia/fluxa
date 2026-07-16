package com.fluxa.app.player

data class TorrentStreamStatus(
    val bufferProgress: Int = 0,
    val detailedStatus: String = "",
    val downloadSpeed: Double = 0.0,
    val activePeers: Int = 0,
    val totalPeers: Int = 0
)
