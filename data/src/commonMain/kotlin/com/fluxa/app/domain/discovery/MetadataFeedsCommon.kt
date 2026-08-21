package com.fluxa.app.domain.discovery

fun isMetadataFeedEnabled(selectedKeys: List<String>?, key: String): Boolean =
    selectedKeys == null || key in selectedKeys

fun effectiveMetadataFeedSelection(selectedKeys: List<String>?, availableKeys: List<String>): List<String>? =
    selectedKeys?.filter { it in availableKeys }

fun effectiveHomeMetadataFeedSelection(selectedKeys: List<String>?, availableKeys: List<String>): List<String>? {
    val selection = effectiveMetadataFeedSelection(selectedKeys, availableKeys)
    return if (
        selection != null &&
        selection.isEmpty() &&
        !selectedKeys.isNullOrEmpty() &&
        availableKeys.isNotEmpty() &&
        selectedKeys.none { it in availableKeys }
    ) {
        null
    } else {
        selection
    }
}

fun orderedMetadataFeeds(options: List<MetadataFeedOption>, order: List<String>?): List<MetadataFeedOption> {
    val optionByKey = options.associateBy { it.key }
    val ordered = order.orEmpty().mapNotNull(optionByKey::get).distinctBy { it.key }
    return ordered + options.filterNot { it.key in ordered.map { item -> item.key } }
}

fun metadataFeedHomeTitle(label: String): String {
    val parts = label.split(" - ").map { it.trim() }.filter { it.isNotEmpty() }
    return when {
        parts.size >= 3 -> parts.drop(1).joinToString(" ")
        parts.size == 2 -> parts[1]
        else -> label
    }
}
