package com.fluxa.app.data.remote

import com.fluxa.app.core.rust.models.NativeStreamPlaybackInfo

data class Stream(
    val name: String?,
    val title: String?,
    val description: String? = null,
    val url: String? = null,
    val ytId: String? = null,
    val yt_ID: String? = null,
    val externalUrl: String? = null,
    val infoHash: String? = null,
    val fileIdx: Int? = null,
    val fileMustInclude: String? = null,
    val filename: String? = null,
    val videoHash: String? = null,
    val videoSize: Long? = null,
    val subtitles: List<SubtitleData>? = null,
    val sources: List<String>? = null,
    val behaviorHints: Map<String, Any>? = null,
    val headers: Map<String, String>? = null,
    val quality: String? = null,
    val size: String? = null,
    val provider: String? = null,
    val seeders: Int? = null,
    val peers: Int? = null,
    var addonName: String? = null,
    var isAutoSelected: Boolean = false
) {
    val legacyYtId: String? get() = yt_ID

    fun resolveHeaders(): Map<String, String> {
        val result = mutableMapOf<String, String>()
        headers?.forEach { (key, value) -> result[key] = value }
        behaviorHints?.let { hints ->
            (hints["requestHeaders"] as? Map<*, *>)?.forEach { (key, value) ->
                result[key.toString()] = value.toString()
            }
            (hints["proxyHeaders"] as? Map<*, *>)?.let { proxy ->
                (proxy["request"] as? Map<*, *>)?.forEach { (key, value) ->
                    result[key.toString()] = value.toString()
                }
            }
            (hints["referer"] as? String)
                ?.takeIf { it.isNotBlank() }
                ?.let { result["referer"] = it }
        }
        return result
    }

    private fun metadataText(): String = listOfNotNull(name, title, description)
        .joinToString(" ")
        .replace('\n', ' ')
        .trim()

    val rawDisplayTitle: String get() = name?.takeIf { it.isNotBlank() }
        ?: title?.takeIf { it.isNotBlank() }
        ?: description?.takeIf { it.isNotBlank() }
        ?: addonName?.takeIf { it.isNotBlank() }
        ?: playableUrl.orEmpty()

    val bingeGroup: String? get() = (behaviorHints?.get("bingeGroup") as? String)?.takeIf { it.isNotBlank() }

    val releaseName: String? get() {
        val value = title ?: description ?: return null
        val firstLine = value.lineSequence().first().trim()
        return firstLine.takeIf { it.length > 5 }
    }

    val skipOffsets: List<IntroTimestamps>? get() {
        val offsets = behaviorHints?.get("skipOffsets") as? Map<*, *> ?: return null
        return offsets.mapNotNull { (key, value) ->
            val start = (value as? Number)?.toLong() ?: return@mapNotNull null
            if (start <= 0) return@mapNotNull null
            IntroTimestamps(start * 1000, (start + 90) * 1000, key.toString().lowercase())
        }.takeIf { it.isNotEmpty() }
    }

    private val playbackInfo: NativeStreamPlaybackInfo get() = resolveStreamPlaybackInfo(this)
    val effectiveVideoHash: String? get() = playbackInfo.effectiveVideoHash
    val effectiveVideoSize: Long? get() = playbackInfo.effectiveVideoSize
    val effectiveFilename: String? get() = playbackInfo.effectiveFilename
    val playableUrl: String? get() = playbackInfo.playableUrl
    val isLikelyPlayerCompatible: Boolean get() = playbackInfo.isLikelyPlayerCompatible
    val isDebrid: Boolean get() = url?.let { it.contains("premiumize") || it.contains("real-debrid") || it.contains("alldebrid") } == true
}

internal expect fun resolveStreamPlaybackInfo(stream: Stream): NativeStreamPlaybackInfo
