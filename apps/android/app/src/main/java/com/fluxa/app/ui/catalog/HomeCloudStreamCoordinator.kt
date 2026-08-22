package com.fluxa.app.ui.catalog

import android.util.Log
import com.fluxa.app.data.local.*
import com.fluxa.app.domain.discovery.buildCs3MetadataFeedOptions
import com.fluxa.app.domain.discovery.effectiveHomeMetadataFeedSelection
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

internal class HomeCloudStreamCoordinator(
    private val scope: CoroutineScope,
    private val gateway: HomePlatformContentGateway,
    private val hasLoadedHome: StateFlow<Boolean>,
    private val activeProfile: () -> UserProfile?,
    private val categories: () -> List<HomeCategory>,
    private val setCategories: (List<HomeCategory>) -> Unit,
    private val billboardIsEmpty: () -> Boolean,
    private val refreshBillboard: suspend (UserProfile?) -> Unit
) {
    private var refreshJob: Job? = null

    fun bind() {
        scope.launch {
            gateway.loadedApis
                .map { apis -> apis.toCs3CatalogFeedDescriptors().map { "${it.pluginName}:${it.catalogIndex}:${it.catalogName}" }.toSet() }
                .distinctUntilChanged()
                .collect {
                    refresh()
                    if (billboardIsEmpty()) scope.launch { refreshBillboard(activeProfile()) }
                }
        }
    }

    fun refresh() {
        refreshJob?.cancel()
        refreshJob = scope.launch {
            try {
                hasLoadedHome.first { it }
                val profile = activeProfile()
                val toggles = profile?.homeFeedToggles
                val apis = gateway.loadedApis.value.filter { it.hasMainPage }
                val feedOptions = buildCs3MetadataFeedOptions(apis.toCs3CatalogFeedDescriptors())
                val enabledKeys = when {
                    toggles == null || profile.cs3FeedsConfigured != true -> null
                    toggles.any { it.startsWith("cs3_catalog_") } -> effectiveHomeMetadataFeedSelection(toggles, feedOptions.map { it.key })?.toSet()
                    toggles.any { it.startsWith("cs3_plugin_") } -> toggles.toSet()
                    else -> null
                }
                if (apis.isEmpty()) {
                    val current = categories()
                    val retained = current.filterNot { it.id.startsWith("cs3_") }
                    if (retained.size != current.size) setCategories(retained)
                    return@launch
                }
                val icons = gateway.installedPlugins.value
                    .mapNotNull { plugin -> plugin.iconUrl?.takeIf { it.isNotBlank() }?.let { plugin.name to it } }
                    .toMap()
                val rows = gateway.cloudHomeCategories(apis, icons, enabledKeys)
                if (!isActive) return@launch
                val rowsById = rows.associateBy { it.id }
                val current = categories()
                val existingIds = current.filter { it.id.startsWith("cs3_") }.mapTo(mutableSetOf()) { it.id }
                val merged = current.mapNotNull { category -> if (category.id.startsWith("cs3_")) rowsById[category.id] else category }
                setCategories(merged + rows.filter { it.id !in existingIds })
            } catch (error: Throwable) {
                Log.w("HomeCloudStream", "Refresh failed", error)
            }
        }
    }
}
