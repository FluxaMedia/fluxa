package com.fluxa.app.player

data class TorrentStatus(
    val hash: String,
    val title: String,
    val downloadSpeed: Double,
    val activePeers: Int,
    val totalPeers: Int,
    val progress: Double,
    val stat: Int = 0,
    val statString: String = "",
    val preload: Int = 0,
    val loadedSize: Long = 0,
    val preloadSize: Long = 0,
    val fileStats: List<TorrentFileStat>?
)

data class TorrentFileStat(
    val id: Int,
    val path: String,
    val length: Long
)
