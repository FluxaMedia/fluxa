package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Stream

internal fun Stream.streamSourceHeader(): String {
    return name
        ?.takeIf { it.isNotBlank() }
        ?: title?.takeIf { it.isNotBlank() }
        ?: description?.takeIf { it.isNotBlank() }
        ?: addonName?.takeIf { it.isNotBlank() }
        ?: playableUrl.orEmpty()
}

internal fun Stream.streamRawBody(): String? {
    val header = streamSourceHeader()
    return listOf(description, title, addonName)
        .firstOrNull { value -> !value.isNullOrBlank() && value != header }
}

internal fun Stream.cloudstreamPlaybackDetailLine(): String? {
    val addon = addonName?.trim().orEmpty()
    if (addon.isBlank()) return null

    val quality = name
        ?.lines()
        ?.map { it.trim() }
        ?.firstOrNull { line -> line.isNotBlank() && line != addon }
        ?: return null

    val fileName = title?.trim()?.takeIf { it.isNotBlank() }
        ?: effectiveFilename?.trim()?.takeIf { it.isNotBlank() }
        ?: url?.substringBefore('?')?.substringAfterLast('/')?.trim()?.takeIf { it.isNotBlank() }
        ?: return null
    if (fileName.contains(quality, ignoreCase = true)) return fileName
    return listOf(fileName, quality).joinToString(" - ")
}
