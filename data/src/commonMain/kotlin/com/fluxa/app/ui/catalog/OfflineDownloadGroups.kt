package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.OfflineDownloadItem

fun List<OfflineDownloadItem>.toOfflineDownloadGroups(
    resolveLocalImage: (String) -> String = { it }
): List<OfflineDownloadGroup> {
    return groupBy { item -> item.metaId.ifBlank { item.title } }
        .values
        .map { episodes ->
            val sorted = episodes.sortedWith(compareBy<OfflineDownloadItem> { it.videoId.orEmpty() }.thenByDescending { it.createdAt })
            val first = sorted.first()
            val artwork = first.localPosterPath?.let(resolveLocalImage)
                ?: first.poster
                ?: first.localBackgroundPath?.let(resolveLocalImage)
                ?: first.background
            OfflineDownloadGroup(
                key = first.metaId.ifBlank { first.title },
                title = first.title,
                poster = artwork,
                episodes = sorted,
                totalBytes = sorted.sumOf { it.totalBytes.takeIf { bytes -> bytes > 0L } ?: it.downloadedBytes }
            )
        }
        .sortedBy { it.title.lowercase() }
}

fun OfflineDownloadItem.effectiveSizeLabel(): String? {
    val size = totalBytes.takeIf { it > 0L } ?: downloadedBytes.takeIf { it > 0L } ?: return null
    return size.formatDownloadBytes()
}

fun Long.formatDownloadBytes(): String = when {
    this >= Gibibyte -> "${formatSingleDecimal(this, Gibibyte)} GB"
    this >= Mebibyte -> "${roundedUnits(this, Mebibyte)} MB"
    this >= Kibibyte -> "${roundedUnits(this, Kibibyte)} KB"
    else -> "$this B"
}

private fun formatSingleDecimal(value: Long, unit: Long): String {
    val tenths = (value * 10 + unit / 2) / unit
    return "${tenths / 10}.${tenths % 10}"
}

private fun roundedUnits(value: Long, unit: Long): Long = (value + unit / 2) / unit

private const val Kibibyte = 1024L
private const val Mebibyte = Kibibyte * 1024L
private const val Gibibyte = Mebibyte * 1024L
