package com.fluxa.app.data.stream

import com.fluxa.app.core.rust.models.NativeStreamPlaybackInfo
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Stream

val Stream.legacyYtId: String?
    get() = yt_ID

fun Stream.resolveHeaders(): Map<String, String> {
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

val Stream.rawDisplayTitle: String
    get() = name?.takeIf { it.isNotBlank() }
        ?: title?.takeIf { it.isNotBlank() }
        ?: description?.takeIf { it.isNotBlank() }
        ?: addonName?.takeIf { it.isNotBlank() }
        ?: playableUrl.orEmpty()

val Stream.bingeGroup: String?
    get() = (behaviorHints?.get("bingeGroup") as? String)?.takeIf { it.isNotBlank() }

val Stream.releaseName: String?
    get() {
        val value = title ?: description ?: return null
        return value.lineSequence().first().trim().takeIf { it.length > 5 }
    }

val Stream.skipOffsets: List<IntroTimestamps>?
    get() {
        val offsets = behaviorHints?.get("skipOffsets") as? Map<*, *> ?: return null
        return offsets.mapNotNull { (key, value) ->
            val start = (value as? Number)?.toLong() ?: return@mapNotNull null
            if (start <= 0) return@mapNotNull null
            IntroTimestamps(start * 1000, (start + 90) * 1000, key.toString().lowercase())
        }.takeIf { it.isNotEmpty() }
    }

private val Stream.playbackInfo: NativeStreamPlaybackInfo
    get() = resolveStreamPlaybackInfo(this)

val Stream.effectiveVideoHash: String?
    get() = playbackInfo.effectiveVideoHash

val Stream.effectiveVideoSize: Long?
    get() = playbackInfo.effectiveVideoSize

val Stream.effectiveFilename: String?
    get() = playbackInfo.effectiveFilename

val Stream.playableUrl: String?
    get() = playbackInfo.playableUrl

val Stream.isLikelyPlayerCompatible: Boolean
    get() = playbackInfo.isLikelyPlayerCompatible

val Stream.isDebrid: Boolean
    get() = url?.let { rawUrl ->
        val normalized = rawUrl.lowercase()
        normalized.contains("premiumize") ||
            normalized.contains("real-debrid") ||
            normalized.contains("alldebrid")
    } == true

internal expect fun resolveStreamPlaybackInfo(stream: Stream): NativeStreamPlaybackInfo
