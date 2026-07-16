package com.fluxa.app.core.rust.models

import com.fluxa.app.data.remote.Stream

data class NativeDetailStreamResultPlan(
    val streams: List<Stream> = emptyList(),
    val availableAddons: List<String> = emptyList(),
    val resolvedRequestId: String? = null,
    val hasStreamProviders: Boolean = false
)
