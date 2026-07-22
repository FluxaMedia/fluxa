package com.fluxa.app.domain.discovery

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.core.rust.FluxaCoreNative

fun buildMetadataFeedOptions(addons: List<AddonDescriptor>, language: String? = "en"): List<MetadataFeedOption> {
    return FluxaCoreNative.buildMetadataFeedOptions(addons)
}

fun buildDiscoverContentTypes(addons: List<AddonDescriptor>): List<String> {
    return FluxaCoreNative.discoverContentTypes(addons)
}

fun buildDiscoverCatalogOptions(addons: List<AddonDescriptor>, selectedType: String): List<DiscoverCatalogOption> {
    return FluxaCoreNative.discoverCatalogOptions(addons, selectedType)
}

fun cs3PluginFeedKey(apiName: String): String =
    "cs3_plugin_${apiName.replace(Regex("[^a-zA-Z0-9]"), "_").lowercase()}"

fun cs3CatalogFeedKey(pluginName: String, catalogName: String, catalogIndex: Int): String =
    "cs3_catalog_${pluginName.stableFeedPart()}:${catalogIndex}:${catalogName.stableFeedPart()}"

fun buildCs3MetadataFeedOptions(catalogs: List<Cs3CatalogFeedDescriptor>): List<MetadataFeedOption> =
    catalogs.map { catalog ->
        val key = cs3CatalogFeedKey(catalog.pluginName, catalog.catalogName, catalog.catalogIndex)
        MetadataFeedOption(
            key = key,
            label = "${catalog.catalogName} - ${catalog.pluginName}",
            transportUrl = "cs3://$key",
            type = "all",
            id = key
        )
    }

fun isMetadataFeedEnabled(selectedKeys: List<String>?, key: String): Boolean {
    return selectedKeys == null || selectedKeys.contains(key)
}

fun effectiveMetadataFeedSelection(selectedKeys: List<String>?, availableKeys: List<String>): List<String>? {
    return FluxaCoreNative.effectiveMetadataFeedSelection(selectedKeys, availableKeys)
}

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

fun toggleMetadataFeed(selectedKeys: List<String>?, availableKeys: List<String>, key: String): List<String> {
    return FluxaCoreNative.toggleMetadataFeed(selectedKeys, availableKeys, key)
}

fun toggleMetadataFeed(selectedKeys: List<String>?, availableKeys: List<String>, key: String, maxEnabled: Int): List<String> {
    return FluxaCoreNative.toggleMetadataFeed(selectedKeys, availableKeys, key, maxEnabled)
}

fun setMetadataFeedGroupEnabled(
    selectedKeys: List<String>?,
    availableKeys: List<String>,
    groupKeys: List<String>,
    enabled: Boolean
): List<String> {
    return FluxaCoreNative.setMetadataFeedGroupEnabled(selectedKeys, availableKeys, groupKeys, enabled)
}

fun orderedMetadataFeeds(options: List<MetadataFeedOption>, order: List<String>?): List<MetadataFeedOption> {
    val optionByKey = options.associateBy { it.key }
    return FluxaCoreNative.orderedMetadataFeedKeys(options.map { it.key }, order).mapNotNull { optionByKey[it] }
}

fun moveMetadataFeedOrder(options: List<MetadataFeedOption>, currentOrder: List<String>?, key: String, delta: Int): List<String> {
    return FluxaCoreNative.moveMetadataFeedOrder(options.map { it.key }, currentOrder, key, delta)
}

fun metadataFeedHomeTitle(label: String): String {
    val parts = label.split(" - ").map { it.trim() }.filter { it.isNotEmpty() }
    return when {
        parts.size >= 3 -> parts.drop(1).joinToString(" ")
        parts.size == 2 -> parts[1]
        else -> label
    }
}
