package com.fluxa.app.domain.discovery

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.AddonCatalog
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.AddonManifest

fun AddonDescriptor.supportsStremioResource(
    resourceName: String,
    type: String? = null,
    id: String? = null
): Boolean {
    return FluxaCoreNative.supportsResource(manifest, resourceName, type, id)
}

fun AddonManifest.hasStremioResource(resourceName: String): Boolean {
    return FluxaCoreNative.supportsResource(this, resourceName, null, null)
}

fun AddonCatalog.supportsCatalogExtra(extraName: String): Boolean {
    return FluxaCoreNative.catalogSupportsExtra(this, extraName)
}

fun AddonCatalog.requiresCatalogExtra(extraName: String): Boolean {
    return FluxaCoreNative.catalogRequiresExtra(this, extraName)
}

fun AddonCatalog.hasRequiredCatalogExtraExcept(allowedNames: Set<String> = emptySet()): Boolean {
    return FluxaCoreNative.catalogHasRequiredExtraExcept(this, allowedNames)
}
