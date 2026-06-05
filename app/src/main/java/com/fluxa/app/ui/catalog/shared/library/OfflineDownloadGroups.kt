@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.OfflineDownloadItem
import java.io.File
import java.util.Locale

internal fun List<OfflineDownloadItem>.toOfflineDownloadGroups(): List<OfflineDownloadGroup> {
    return groupBy { item -> item.metaId.ifBlank { item.title } }
        .values
        .map { episodes ->
            val sorted = episodes.sortedWith(compareBy<OfflineDownloadItem> { it.videoId.orEmpty() }.thenByDescending { it.createdAt })
            val first = sorted.first()
            OfflineDownloadGroup(
                key = first.metaId.ifBlank { first.title },
                title = first.title,
                poster = first.localPosterPath?.toFileImageModel() ?: first.poster ?: first.localBackgroundPath?.toFileImageModel() ?: first.background,
                episodes = sorted,
                totalBytes = sorted.sumOf { it.totalBytes.takeIf { bytes -> bytes > 0L } ?: it.downloadedBytes }
            )
        }
        .sortedBy { it.title.lowercase(Locale.ROOT) }
}

internal fun OfflineDownloadItem.effectiveSizeLabel(): String? {
    val size = totalBytes.takeIf { it > 0L } ?: downloadedBytes.takeIf { it > 0L } ?: return null
    return size.formatDownloadBytes()
}

internal fun String.toFileImageModel(): String {
    return File(this).takeIf { it.exists() }?.toURI()?.toString() ?: this
}

internal fun Long.formatDownloadBytes(): String {
    val value = this.toDouble()
    return when {
        this >= 1024L * 1024L * 1024L -> String.format(Locale.US, "%.1f GB", value / (1024.0 * 1024.0 * 1024.0))
        this >= 1024L * 1024L -> String.format(Locale.US, "%.0f MB", value / (1024.0 * 1024.0))
        this >= 1024L -> String.format(Locale.US, "%.0f KB", value / 1024.0)
        else -> "$this B"
    }
}
