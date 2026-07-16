package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.AddonManifest

fun addonManifestCapabilityRows(manifest: AddonManifest, language: String?): List<String> {
    val resourceOrder = manifest.resources.orEmpty().distinct()
    val types = (manifest.types.orEmpty() + manifest.catalogs.orEmpty().mapNotNull { it.type })
        .filter { it.isNotBlank() }
        .distinct()
    if (resourceOrder.isEmpty() || types.isEmpty()) return emptyList()
    return listOf("${types.joinToString(", ")} - ${resourceOrder.joinToString(", ")}")
}
