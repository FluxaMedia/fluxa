package com.fluxa.app.ui

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream

internal data class PlayerLaunchRequest(
    val meta: Meta,
    val videoId: String? = null,
    val initialProgress: Long = 0L,
    val streamIndex: Int = 0,
    val initialStreams: List<Stream> = emptyList(),
    val lastStreamUrl: String? = null,
    val lastStreamTitle: String? = null,
    val preferredBingeGroup: String? = null,
    val returnToSourcesOnError: Boolean = false,
    val showSourceSelection: Boolean = false
)
