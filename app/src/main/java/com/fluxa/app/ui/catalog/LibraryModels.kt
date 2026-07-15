package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Immutable
import com.fluxa.app.data.local.OfflineDownloadItem

@Immutable
internal data class OfflineDownloadGroup(
    val key: String,
    val title: String,
    val poster: String?,
    val episodes: List<OfflineDownloadItem>,
    val totalBytes: Long
)
