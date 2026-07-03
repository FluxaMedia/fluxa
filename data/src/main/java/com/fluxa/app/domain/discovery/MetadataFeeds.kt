package com.fluxa.app.domain.discovery

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.core.rust.FluxaCoreNative
import java.util.Locale

data class MetadataFeedOption(
    val key: String,
    val label: String,
    val transportUrl: String,
    val type: String,
    val id: String,
    val genre: String? = null
)

data class DiscoverCatalogOption(
    val key: String,
    val label: String,
    val transportUrl: String,
    val type: String,
    val id: String,
    val genres: List<String>,
    val requiresGenre: Boolean = false
)

fun buildMetadataFeedOptions(addons: List<AddonDescriptor>, language: String? = "en"): List<MetadataFeedOption> {
    val addonFeeds = addons
        .flatMap { addon -> addon.toMetadataFeedOptions() }
    return addonFeeds
}

fun buildDiscoverCatalogOptions(addons: List<AddonDescriptor>, selectedType: String): List<DiscoverCatalogOption> {
    val normalizedType = selectedType.lowercase()
    val rawOptions = addons.filter { it.manifest.hasStremioResource("catalog") }.flatMap { addon ->
        addon.manifest.catalogs.orEmpty().mapNotNull { catalog ->
            val type = catalog.type?.normalizeContentType()?.takeIf { normalizedType == "all" || it == normalizedType }
                ?: return@mapNotNull null
            val id = catalog.id?.takeIf { it.isNotBlank() } ?: return@mapNotNull null
            if (catalog.hasRequiredCatalogExtraExcept(setOf("genre"))) return@mapNotNull null
            val name = discoverCatalogLabel(catalog.name, id)
            val genres = (
                catalog.genres.orEmpty() +
                    catalog.extra.orEmpty()
                        .filter { it.name.equals("genre", ignoreCase = true) }
                        .flatMap { it.options.orEmpty() }
                )
                .map { it.trim() }
                .filter { it.isNotBlank() }
                .distinct()
            val sourceKey = addon.manifest.id.takeIf { it.isNotBlank() } ?: addon.transportUrl
            DiscoverCatalogOption(
                key = "discover:${sourceKey.stableFeedPart()}:${type.stableFeedPart()}:${name.stableFeedPart()}",
                label = name,
                transportUrl = addon.transportUrl,
                type = type,
                id = id,
                genres = genres,
                requiresGenre = catalog.requiresCatalogExtra("genre")
            )
        }
    }

    return if (normalizedType == "all") {
        rawOptions
            .groupBy { "${it.transportUrl}:${it.id}:${it.label.lowercase()}" }
            .map { (_, options) ->
                val first = options.first()
                val types = options.map { it.type }.distinct()
                first.copy(
                    key = "discover:${first.transportUrl.stableFeedPart()}:${first.id.stableFeedPart()}:${first.label.stableFeedPart()}",
                    type = if (types.size > 1) "all" else first.type,
                    genres = options.flatMap { it.genres }.distinct(),
                    requiresGenre = options.any { it.requiresGenre }
                )
            }
            .distinctBy { it.label.lowercase() }
    } else {
        rawOptions.distinctBy { "${it.transportUrl}:${it.type}:${it.label.lowercase()}" }
    }
}

private fun discoverCatalogLabel(rawName: String?, id: String): String {
    val fallback = id.split('_', '-', ' ').filter { it.isNotBlank() }
        .joinToString(" ") { w -> w.replaceFirstChar { if (it.isLowerCase()) it.titlecase(Locale.ROOT) else it.toString() } }
        .ifBlank { id }
    val base = rawName?.trim()?.takeIf { it.isNotEmpty() } ?: fallback
    val label = base
        .replace(Regex("(?i)\\bcinemeta\\b"), "")
        .replace(Regex("(?i)\\b(movie|movies|film|films|series|shows|tv)\\b"), "")
        .replace(Regex("\\s*[-:|/]\\s*"), " ")
        .trim()
        .replace(Regex("\\s+"), " ")
    return label.ifBlank { fallback }
}

fun cs3PluginFeedKey(apiName: String): String =
    "cs3_plugin_${apiName.replace(Regex("[^a-zA-Z0-9]"), "_").lowercase()}"

fun buildCs3MetadataFeedOptions(apiNames: List<String>): List<MetadataFeedOption> =
    apiNames.map { name ->
        MetadataFeedOption(
            key = cs3PluginFeedKey(name),
            label = name,
            transportUrl = "cs3://${cs3PluginFeedKey(name)}",
            type = "all",
            id = cs3PluginFeedKey(name)
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

private fun AddonDescriptor.toMetadataFeedOptions(): List<MetadataFeedOption> {
    if (!manifest.hasStremioResource("catalog")) return emptyList()
    val addonName = manifestName(this)
    return manifest.catalogs.orEmpty().mapNotNull { catalog ->
        val type = catalog.type?.trim()?.takeIf { it.isNotBlank() } ?: return@mapNotNull null
        val id = catalog.id?.takeIf { it.isNotBlank() } ?: return@mapNotNull null
        val baseName = catalog.name?.takeIf { it.isNotBlank() } ?: id.toTitleLabel()
        if (catalog.hasRequiredCatalogExtraExcept()) return@mapNotNull null
        createMetadataFeedOption(addonName, type, id, baseName, genre = null)
    }
}

private fun AddonDescriptor.createMetadataFeedOption(
    addonName: String,
    type: String,
    id: String,
    name: String,
    genre: String?
): MetadataFeedOption {
    val sourceKey = manifest.id.takeIf { it.isNotBlank() } ?: transportUrl
    val genreKey = genre?.let { ":genre:${it.stableFeedPart()}" }.orEmpty()
    val key = "addon:${sourceKey.stableFeedPart()}:${type.stableFeedPart()}:${id.stableFeedPart()}$genreKey"
    return MetadataFeedOption(
        key = key,
        label = "$addonName - $name",
        transportUrl = transportUrl,
        type = type,
        id = id,
        genre = genre
    )
}

private fun String.isContentType(): Boolean {
    return normalizeContentType() != null
}

private fun String.normalizeContentType(): String? {
    return FluxaCoreNative.normalizeContentType(this)
}

private fun manifestName(addon: AddonDescriptor): String {
    return addon.manifest.name.takeIf { it.isNotBlank() }
        ?: addon.manifest.id.takeIf { it.isNotBlank() }
        ?: "Metadata"
}

private fun String.stableFeedPart(): String {
    return FluxaCoreNative.stableFeedPart(this)
}

private fun String.toTitleLabel(): String {
    return split('_', '-', ' ')
        .filter { it.isNotBlank() }
        .joinToString(" ") { word -> word.replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() } }
        .ifBlank { this }
}

fun metadataFeedHomeTitle(label: String): String {
    val parts = label.split(" - ").map { it.trim() }.filter { it.isNotEmpty() }
    return when {
        parts.size >= 3 -> parts.drop(1).joinToString(" ")
        parts.size == 2 -> parts[1]
        else -> label
    }
}
