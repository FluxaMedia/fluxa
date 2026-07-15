package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Stream

internal const val STREAM_SOURCE_MODE_MANUAL = "manual"
internal const val STREAM_SOURCE_MODE_FIRST = "first"

internal fun selectStreamIndex(
    streams: List<Stream>,
    currentVideoId: String?,
    initialStreamIndex: Int,
    savedUrl: String?,
    savedTitle: String?,
    sourceSelectionMode: String,
    regexPattern: String?,
    preferredBingeGroup: String?
): Int {
    return FluxaCoreNative.selectStreamIndex(
        streams = streams,
        currentVideoId = currentVideoId,
        initialStreamIndex = initialStreamIndex,
        savedUrl = savedUrl,
        savedTitle = savedTitle,
        sourceSelectionMode = sourceSelectionMode,
        regexPattern = regexPattern,
        preferredBingeGroup = preferredBingeGroup
    )
}
