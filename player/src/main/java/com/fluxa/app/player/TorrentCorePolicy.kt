package com.fluxa.app.player

import com.fluxa.app.common.Constants
import com.fluxa.app.core.rust.FluxaCoreNative

object TorrentCorePolicy {
    fun plan(
        link: String,
        title: String,
        requestedFileIdx: Int?,
        preferredFilename: String?,
        sources: List<String>,
        fileStats: List<TorrFileStat>,
        rejectedIndex: Int? = null,
        baseUrl: String = Constants.LocalServer.TORR_SERVER_BASE_URL,
        play: Boolean = true,
        stat: Boolean = false
    ): NativeTorrentRuntimeInfo {
        return FluxaCoreNative.torrentRuntimeInfo(
            link = link,
            title = title,
            requestedFileIdx = requestedFileIdx,
            preferredFilename = preferredFilename,
            sources = sources,
            fileStats = fileStats,
            rejectedIndex = rejectedIndex,
            baseUrl = baseUrl,
            play = play,
            stat = stat
        )
    }

    fun statusInfo(status: TorrStatus): NativeTorrentStatusInfo {
        return FluxaCoreNative.torrentStatusInfo(status)
    }
}
