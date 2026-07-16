package com.fluxa.app.core.rust.models

import com.fluxa.app.data.remote.Stream

data class NativePlayerFlowResult(
    val state: NativePlayerFlowState = NativePlayerFlowState(),
    val effects: List<NativePlayerFlowEffect> = emptyList()
)

data class NativePlayerFlowState(
    val currentVideoId: String? = null,
    val currentStreams: List<Stream> = emptyList(),
    val currentStreamIndex: Int = 0,
    val currentUrl: String? = null,
    val zeroSpeedTicks: Int = 0,
    val isBuffering: Boolean = false,
    val isVideoRendered: Boolean = false,
    val playerError: String? = null,
    val preferredBingeGroup: String? = null
)
