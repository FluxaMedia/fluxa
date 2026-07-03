@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Immutable
import com.fluxa.app.data.local.OfflineDownloadItem
import com.fluxa.app.data.remote.Meta

@Immutable
internal data class LibraryCollectionUi(
    val title: String,
    val subtitle: String,
    val items: List<Meta>,
    val locked: Boolean = false,
    val userCollectionId: String? = null
)

@Immutable
internal data class LibraryDetailUi(
    val title: String,
    val items: List<Meta>
)

@Immutable
internal data class OfflineDownloadGroup(
    val key: String,
    val title: String,
    val poster: String?,
    val episodes: List<OfflineDownloadItem>,
    val totalBytes: Long
)
