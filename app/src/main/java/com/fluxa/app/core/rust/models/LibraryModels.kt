package com.fluxa.app.core.rust.models

data class NativeWatchlistTogglePlan(
    val command: String = "add",
    val itemId: String = "",
    val optimisticIsInWatchlist: Boolean = false,
    val profileId: String? = null
)

data class NativeLibraryCollectionImportValidation(
    val isValid: Boolean = false,
    val validCollections: List<Map<String, Any?>> = emptyList(),
    val issues: List<String> = emptyList()
)

data class NativeLibraryOfflineGrouping(
    val ready: List<Map<String, Any?>> = emptyList(),
    val downloading: List<Map<String, Any?>> = emptyList(),
    val queued: List<Map<String, Any?>> = emptyList(),
    val failed: List<Map<String, Any?>> = emptyList()
)

data class NativeOfflineDownloadPlan(
    val supported: Boolean = false,
    val reason: String? = null,
    val playbackUrl: String = "",
    val baseName: String = "",
    val videoFileName: String = "",
    val subtitleFileName: String? = null,
    val posterFileName: String = "",
    val backgroundFileName: String = "",
    val logoFileName: String = "",
    val videoId: String? = null,
    val streamTitle: String? = null
)
