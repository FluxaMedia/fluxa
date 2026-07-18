package com.fluxa.app.data.repository

import com.fluxa.app.data.remote.NuvioAddon

data class NuvioAddonState(val installedUrls: List<String>, val disabledUrls: List<String>)

object NuvioImportPolicy {
    fun addonState(addons: List<NuvioAddon>): NuvioAddonState {
        val ordered = addons.sortedBy(NuvioAddon::sortOrder)
        return NuvioAddonState(
            installedUrls = ordered.map(NuvioAddon::url).distinct(),
            disabledUrls = ordered.filterNot(NuvioAddon::enabled).map(NuvioAddon::url).distinct()
        )
    }
}
