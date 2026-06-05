package com.fluxa.app.core.rust.models

data class NativePlayerBackendSelection(
    val backend: String = "exoplayer",
    val reason: String = "default"
)

data class NativeTorrentFallbackFilePolicy(
    val fallbackFileIndexes: List<Int> = emptyList(),
    val rejectedIndex: Int? = null
)

data class NativePlayerBufferTargets(
    val forwardBufferMs: Long = 120_000L,
    val backBufferMs: Long = 30_000L,
    val cacheSizeBytes: Long = 100_000_000L
)

data class NativePlayerRetryPolicy(
    val shouldRetry: Boolean = false,
    val fallbackAction: String = "show_error",
    val delayMs: Long = 0L,
    val retryCount: Int = 0
)
