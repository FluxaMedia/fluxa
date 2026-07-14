package com.fluxa.app.data.remote

data class AddonDescriptor(
    val manifest: AddonManifest,
    val transportUrl: String
)

data class AddonManifest(
    val id: String,
    val name: String,
    val description: String? = null,
    val version: String? = null,
    val resources: List<Any>?,
    val types: List<String>?,
    val catalogs: List<AddonCatalog>?,
    val idPrefixes: List<String>? = null,
    val logo: String? = null,
    val background: String? = null,
    val configurable: Boolean? = null
)

data class AddonCatalog(
    val type: String? = null,
    val id: String? = null,
    val name: String? = null,
    val extra: List<CatalogExtra>? = null,
    val extraSupported: List<String>? = null,
    val genres: List<String>? = null
)

data class CatalogExtra(
    val name: String? = null,
    val options: List<String>? = null,
    val isRequired: Boolean? = null,
    val optionsLimit: Int? = null
)

data class AddonCollectionResponse(val result: AddonCollectionResult)

data class AddonCollectionResult(val addons: List<AddonDescriptor>)
