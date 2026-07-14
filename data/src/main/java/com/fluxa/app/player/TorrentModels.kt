package com.fluxa.app.player

import com.google.gson.annotations.SerializedName

data class TorrentStatus(
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
    @SerializedName("file_stats") val fileStats: List<TorrentFileStat>?
)

data class TorrentFileStat(
    @SerializedName("id") val id: Int,
    @SerializedName("path") val path: String,
    @SerializedName("length") val length: Long
)

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
