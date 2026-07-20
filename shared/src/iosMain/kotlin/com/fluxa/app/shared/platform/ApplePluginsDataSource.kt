package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.plugins.PluginsDataSource
import com.fluxa.app.shared.feature.plugins.PluginsUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

data class ApplePluginsActionSnapshot(
    val type: String,
    val manifestUrl: String? = null,
    val scraperId: String? = null,
    val enabled: Boolean? = null,
    val settingsJson: String? = null,
    val url: String? = null,
    val repoUrl: String? = null,
    val internalName: String? = null
)

class ApplePluginsDataSource : PluginsDataSource {
    private val state = MutableStateFlow(PluginsUiState())
    private var onActionRequested: (ApplePluginsActionSnapshot) -> Unit = {}

    override fun observePlugins(): Flow<PluginsUiState> = state.asStateFlow()

    override suspend fun refresh() {
        postAction(ApplePluginsActionSnapshot(type = "refresh"))
    }

    override suspend fun addRepository(manifestUrl: String) {
        postAction(ApplePluginsActionSnapshot(type = "addRepository", manifestUrl = manifestUrl))
    }

    override suspend fun removeRepository(manifestUrl: String) {
        postAction(ApplePluginsActionSnapshot(type = "removeRepository", manifestUrl = manifestUrl))
    }

    override suspend fun refreshRepository(manifestUrl: String) {
        postAction(ApplePluginsActionSnapshot(type = "refreshRepository", manifestUrl = manifestUrl))
    }

    override suspend fun toggleScraper(scraperId: String, enabled: Boolean) {
        postAction(ApplePluginsActionSnapshot(type = "toggleScraper", scraperId = scraperId, enabled = enabled))
    }

    override suspend fun requestScraperSettings(scraperId: String) {
        postAction(ApplePluginsActionSnapshot(type = "requestScraperSettings", scraperId = scraperId))
    }

    override suspend fun dismissScraperSettings() {
        state.value = state.value.copy(scraperSettingsSheet = null)
    }

    override suspend fun saveScraperSettings(scraperId: String, values: Map<String, Any?>) {
        postAction(ApplePluginsActionSnapshot(type = "saveScraperSettings", scraperId = scraperId))
    }

    override suspend fun addCloudstreamRepository(url: String) {
        postAction(ApplePluginsActionSnapshot(type = "addCloudstreamRepository", url = url))
    }

    override suspend fun openRepo(url: String) {
        state.value = state.value.copy(openRepoUrl = url, isLoadingRepoPlugins = true)
        postAction(ApplePluginsActionSnapshot(type = "openRepo", url = url))
    }

    override suspend fun dismissRepoDialog() {
        state.value = state.value.copy(openRepoUrl = null, openRepoName = null, openRepoPlugins = emptyList(), repoDialogError = null)
    }

    override suspend fun removeRepo(url: String) {
        postAction(ApplePluginsActionSnapshot(type = "removeRepo", url = url))
    }

    override suspend fun toggleRepoPlugin(repoUrl: String, internalName: String) {
        postAction(ApplePluginsActionSnapshot(type = "toggleRepoPlugin", repoUrl = repoUrl, internalName = internalName))
    }

    fun setOnActionRequested(handler: (ApplePluginsActionSnapshot) -> Unit) {
        onActionRequested = handler
    }

    private fun postAction(action: ApplePluginsActionSnapshot) {
        onActionRequested(action)
    }
}
