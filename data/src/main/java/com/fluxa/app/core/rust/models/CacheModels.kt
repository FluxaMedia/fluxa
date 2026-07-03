package com.fluxa.app.core.rust.models

data class NativeCacheEntryPolicy(
    val key: String = "",
    val storedAtMillis: Long = 0L,
    val expiresAtMillis: Long = 0L,
    val isExpired: Boolean = false
)

data class NativeCacheTrimPolicy(
    val expiredKeys: List<String> = emptyList(),
    val evictedKeys: List<String> = emptyList()
)

data class NativeAddonStoreSearchPolicy(
    val normalizedQuery: String = "",
    val url: String = "",
    val useCache: Boolean = false,
    val shouldFetch: Boolean = false
)

data class NativeDataFailurePolicy(
    val operation: String = "",
    val kind: String = "",
    val message: String = "",
    val retryable: Boolean = false,
    val staleFallbackAllowed: Boolean = false
)
