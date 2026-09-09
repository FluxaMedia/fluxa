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
    FluxaCoreNative.cs3PluginFeedKey(apiName)

fun cs3CatalogFeedKey(pluginName: String, catalogName: String, catalogIndex: Int): String =
    FluxaCoreNative.cs3CatalogFeedKey(pluginName, catalogName, catalogIndex)

fun buildCs3MetadataFeedOptions(catalogs: List<Cs3CatalogFeedDescriptor>): List<MetadataFeedOption> =
    FluxaCoreNative.cs3MetadataFeedOptions(catalogs)

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

fun moveMetadataFeedOrder(options: List<MetadataFeedOption>, currentOrder: List<String>?, key: String, delta: Int): List<String> {
    return FluxaCoreNative.moveMetadataFeedOrder(options.map { it.key }, currentOrder, key, delta)
}
