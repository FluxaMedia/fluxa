package com.fluxa.app.player

import com.fluxa.app.data.remote.Stream

const val STREAM_SOURCE_MODE_MANUAL = "manual"
const val STREAM_SOURCE_MODE_FIRST = "first"
const val STREAM_SOURCE_MODE_REGEX = "regex"

data class StreamSelectionRequest(
    val streams: List<Stream>,
    val currentVideoId: String?,
    val initialStreamIndex: Int,
    val savedUrl: String?,
    val savedTitle: String?,
    val sourceSelectionMode: String,
    val regexPattern: String?,
    val preferredBingeGroup: String?
)

fun interface StreamSourceSelectionPolicy {
    fun select(request: StreamSelectionRequest): Int
}
