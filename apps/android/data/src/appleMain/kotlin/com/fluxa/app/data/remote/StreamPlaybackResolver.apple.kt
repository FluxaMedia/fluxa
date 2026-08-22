package com.fluxa.app.data.remote

import com.fluxa.app.core.rust.models.NativeStreamPlaybackInfo

internal actual fun resolveStreamPlaybackInfo(stream: Stream): NativeStreamPlaybackInfo {
    val playableUrl = stream.url
        ?: stream.externalUrl
        ?: stream.ytId?.let { "https://www.youtube.com/watch?v=$it" }
        ?: stream.yt_ID?.let { "https://www.youtube.com/watch?v=$it" }
        ?: stream.infoHash?.let { "magnet:?xt=urn:btih:$it" }
    val isTorrent = stream.infoHash != null || playableUrl?.startsWith("magnet:") == true
    return NativeStreamPlaybackInfo(
        playableUrl = playableUrl,
        effectiveVideoHash = stream.videoHash ?: stream.infoHash,
        effectiveVideoSize = stream.videoSize,
        effectiveFilename = stream.filename,
        isTorrentPlaybackUrl = isTorrent,
        isLikelyPlayerCompatible = !playableUrl.isNullOrBlank()
    )
}
