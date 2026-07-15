package com.fluxa.app.ui.routes

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.ui.Screen

internal fun Meta.detailScreen(sourceAddonTransportUrl: String? = null, sourceAddonCatalogType: String? = null): Screen.Detail {
    return Screen.Detail(
        type = type,
        id = id,
        initialProgress = timeOffset,
        lastVideoId = lastVideoId,
        lastStreamIndex = lastStreamIndex,
        lastStreamUrl = lastStreamUrl,
        lastStreamTitle = lastStreamTitle,
        sourceAddonTransportUrl = sourceAddonTransportUrl,
        sourceAddonCatalogType = sourceAddonCatalogType,
        initialMeta = this
    )
}

internal fun Meta.resumePlayerScreen(returnToSourcesOnError: Boolean): Screen.Player {
    return Screen.Player(
        meta = this,
        videoId = lastVideoId,
        initialProgress = timeOffset ?: 0L,
        streamIndex = lastStreamIndex ?: 0,
        lastStreamUrl = lastStreamUrl,
        lastStreamTitle = lastStreamTitle,
        preferredBingeGroup = lastBingeGroup,
        returnToSourcesOnError = returnToSourcesOnError
    )
}
