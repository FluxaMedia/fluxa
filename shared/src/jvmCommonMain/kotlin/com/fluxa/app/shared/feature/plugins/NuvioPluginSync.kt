package com.fluxa.app.shared.feature.plugins

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.NuvioPluginDto
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator

/** Canonical ordering/filtering for Nuvio-sourced plugin manifests. */
fun List<NuvioPluginDto>.enabledManifestUrls(): List<String> = asSequence()
    .filter { it.enabled }
    .sortedBy { it.sortOrder }
    .mapNotNull { it.manifestUrl?.takeIf(String::isNotBlank) }
    .distinct()
    .toList()

/**
 * Shared profile -> Nuvio plugin reconciliation used by Android and Desktop.
 * A disconnected profile intentionally clears the Nuvio-managed repository set; a transient
 * network failure leaves the previous known-good set untouched.
 */
suspend fun syncNuvioPluginsForProfile(
    profile: UserProfile?,
    coordinator: NuvioAccountImportCoordinator,
    syncPlugins: suspend (List<NuvioPluginDto>) -> Unit,
) {
    if (profile == null || profile.nuvioAccessToken.isNullOrBlank()) {
        syncPlugins(emptyList())
        return
    }
    val remotePlugins = runCatching { coordinator.pullPluginsForProfile(profile) }.getOrNull() ?: return
    syncPlugins(remotePlugins)
}
