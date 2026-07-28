package com.fluxa.app.domain.discovery

data class MetadataFeedOption(
    val key: String,
    val label: String,
    val transportUrl: String,
    val type: String,
    val id: String,
    val genre: String? = null
)

data class DiscoverCatalogExtra(
    val name: String = "",
    val options: List<String> = emptyList(),
    val default: String? = null,
    val isRequired: Boolean = false
)

data class DiscoverCatalogOption(
    val key: String,
    val label: String,
    val transportUrl: String,
    val type: String,
    val id: String,
    val genres: List<String>,
    val requiresGenre: Boolean = false,
    val defaultGenre: String? = null,
    val extras: List<DiscoverCatalogExtra> = emptyList()
)

data class Cs3CatalogFeedDescriptor(
    val pluginName: String,
    val catalogName: String,
    val catalogIndex: Int
)
