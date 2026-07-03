package com.fluxa.app.data.remote

import com.fluxa.app.core.rust.FluxaCoreNative
import com.google.gson.annotations.SerializedName

data class Stream(
    val name: String?, 
    val title: String?, 
    val description: String? = null,
    val url: String? = null, 
    val ytId: String? = null,
    @SerializedName("yt_ID") val legacyYtId: String? = null,
    val externalUrl: String? = null,
    val infoHash: String? = null, 
    val fileIdx: Int? = null, 
    val fileMustInclude: String? = null,
    val filename: String? = null,
    val videoHash: String? = null,
    val videoSize: Long? = null,
    val subtitles: List<SubtitleData>? = null,
    val sources: List<String>? = null, //  NEW: Trackers
    val behaviorHints: Map<String, Any>? = null,
    var addonName: String? = null,
    var isAutoSelected: Boolean = false
) {
    fun getHeaders(): Map<String, String> {
        val headers = mutableMapOf<String, String>()
        behaviorHints?.let { hints ->
            // Standard Stremio headers
            (hints["requestHeaders"] as? Map<*, *>)?.forEach { (k, v) -> 
                headers[k.toString()] = v.toString() 
            }
            // Proxy-style headers (used by some advanced addons)
            (hints["proxyHeaders"] as? Map<*, *>)?.let { proxy ->
                (proxy["request"] as? Map<*, *>)?.forEach { (k, v) -> 
                    headers[k.toString()] = v.toString() 
                }
            }
            // Direct referer hint
            (hints["referer"] as? String)?.let { headers["Referer"] = it }
        }
        return headers
    }

    private fun metadataText(): String = listOfNotNull(name, title, description).joinToString(" ").replace('\n', ' ').trim()

    private fun normalizedMetadataText(): String = metadataText().lowercase()

    val rawDisplayTitle: String get() = name?.takeIf { it.isNotBlank() }
        ?: title?.takeIf { it.isNotBlank() }
        ?: description?.takeIf { it.isNotBlank() }
        ?: addonName?.takeIf { it.isNotBlank() }
        ?: playableUrl.orEmpty()

    val bingeGroup: String? get() = (behaviorHints?.get("bingeGroup") as? String)?.takeIf { it.isNotBlank() }

    val releaseName: String? get() {
        val t = title ?: description ?: return null
        // First line is often the release name
        val firstLine = t.split("\n").first().trim()
        return if (firstLine.length > 5) firstLine else null
    }

    val skipOffsets: List<IntroTimestamps>? get() {
        val hints = behaviorHints ?: return null
        val results = mutableListOf<IntroTimestamps>()
        
        //  FORMAT 1: { "skipOffsets": { "intro": 30, "outro": 1200 } }
        (hints["skipOffsets"] as? Map<*, *>)?.let { offsets ->
            offsets.forEach { (key, value) ->
                val type = key.toString().lowercase()
                val start = (value as? Number)?.toLong() ?: 0L
                if (start > 0) {
                    // If only start is provided, we assume a standard 90s intro or until end for outro
                    results.add(IntroTimestamps(start * 1000, (start + 90) * 1000, type))
                }
            }
        }
        
        return results.takeIf { it.isNotEmpty() }
    }

    val effectiveVideoHash: String? get() {
        return FluxaCoreNative.streamPlaybackInfo(this).effectiveVideoHash
    }

    val effectiveVideoSize: Long? get() {
        return FluxaCoreNative.streamPlaybackInfo(this).effectiveVideoSize
    }

    val effectiveFilename: String? get() {
        return FluxaCoreNative.streamPlaybackInfo(this).effectiveFilename
    }

    val isDebrid: Boolean get() = url?.contains("premiumize") == true || url?.contains("real-debrid") == true || url?.contains("alldebrid") == true
    val playableUrl: String? get() {
        return FluxaCoreNative.streamPlaybackInfo(this).playableUrl
    }
    val isLikelyPlayerCompatible: Boolean get() {
        return FluxaCoreNative.streamPlaybackInfo(this).isLikelyPlayerCompatible
    }
}
