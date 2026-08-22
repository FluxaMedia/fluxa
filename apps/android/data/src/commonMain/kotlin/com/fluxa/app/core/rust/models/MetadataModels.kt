package com.fluxa.app.core.rust.models

import com.fluxa.app.data.remote.Meta

data class NativeDirectPlaybackPlan(
    val meta: Meta? = null,
    val targetVideoId: String? = null,
    val lookupId: String = ""
)
