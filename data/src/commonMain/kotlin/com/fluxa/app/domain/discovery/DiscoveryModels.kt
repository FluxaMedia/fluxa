package com.fluxa.app.domain.discovery

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

data class Cs3CatalogFeedDescriptor(
    val pluginName: String,
    val catalogName: String,
    val catalogIndex: Int
)
