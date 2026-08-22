package com.fluxa.app.player

import com.google.gson.annotations.SerializedName

data class AndroidTorrentStatus(
    @SerializedName("hash") val hash: String,
    @SerializedName("title") val title: String,
    @SerializedName("download_speed") val downloadSpeed: Double,
    @SerializedName("active_peers") val activePeers: Int,
    @SerializedName("total_peers") val totalPeers: Int,
    @SerializedName("progress") val progress: Double,
    @SerializedName("stat") val stat: Int = 0,
    @SerializedName("stat_string") val statString: String = "",
    @SerializedName("preload") val preload: Int = 0,
    @SerializedName("loaded_size") val loadedSize: Long = 0,
    @SerializedName("preload_size") val preloadSize: Long = 0,
    @SerializedName("file_stats") val fileStats: List<AndroidTorrentFileStat>?
) {
    fun toShared(): TorrentStatus = TorrentStatus(
        hash = hash,
        title = title,
        downloadSpeed = downloadSpeed,
        activePeers = activePeers,
        totalPeers = totalPeers,
        progress = progress,
        stat = stat,
        statString = statString,
        preload = preload,
        loadedSize = loadedSize,
        preloadSize = preloadSize,
        fileStats = fileStats?.map(AndroidTorrentFileStat::toShared)
    )
}

data class AndroidTorrentFileStat(
    @SerializedName("id") val id: Int,
    @SerializedName("path") val path: String,
    @SerializedName("length") val length: Long
) {
    fun toShared(): TorrentFileStat = TorrentFileStat(id = id, path = path, length = length)
}
