package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.player.StreamSelectionRequest
import com.fluxa.app.player.StreamSourceSelectionPolicy

internal object AndroidStreamSourceSelectionPolicy : StreamSourceSelectionPolicy {
    override fun select(request: StreamSelectionRequest): Int = FluxaCoreNative.selectStreamIndex(
        streams = request.streams,
        currentVideoId = request.currentVideoId,
        initialStreamIndex = request.initialStreamIndex,
        savedUrl = request.savedUrl,
        savedTitle = request.savedTitle,
        sourceSelectionMode = request.sourceSelectionMode,
        regexPattern = request.regexPattern,
        preferredBingeGroup = request.preferredBingeGroup
    )
}

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
    return AndroidStreamSourceSelectionPolicy.select(
        StreamSelectionRequest(
            streams,
            currentVideoId,
            initialStreamIndex,
            savedUrl,
            savedTitle,
            sourceSelectionMode,
            regexPattern,
            preferredBingeGroup
        )
    )
}
