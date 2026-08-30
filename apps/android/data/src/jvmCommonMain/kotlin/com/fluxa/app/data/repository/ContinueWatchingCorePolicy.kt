package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Meta

fun Meta.isUpNextContinueItemFromCore(): Boolean {
    return FluxaCoreNative.isUpNextContinueWatchingItem(
        type = type,
        lastVideoId = lastVideoId,
        timeOffset = timeOffset,
        duration = duration,
        resumeProgressPercent = resumeProgressPercent,
    )
}
