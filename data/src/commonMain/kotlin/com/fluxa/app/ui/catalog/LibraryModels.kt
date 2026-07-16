package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.OfflineDownloadItem

data class OfflineDownloadGroup(
    val key: String,
    val title: String,
    val poster: String?,
    val episodes: List<OfflineDownloadItem>,
    val totalBytes: Long
)
