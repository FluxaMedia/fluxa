package com.fluxa.app.shared.feature.plugins

import kotlinx.coroutines.flow.Flow

data class PluginRepositoryUiModel(
    val manifestUrl: String,
    val name: String,
    val description: String?,
    val scraperCount: Int
)

data class PluginScraperUiModel(
    val id: String,
    val name: String,
    val repositoryUrl: String,
    val enabled: Boolean,
    val supportedTypes: List<String>,
    val hasSettings: Boolean = false,
    val settings: Map<String, Any?> = emptyMap()
)

data class PluginSettingsOptionUiModel(
    val label: String,
    val value: String
)

data class PluginSettingsFieldUiModel(
    val key: String,
    val type: String,
    val label: String,
    val description: String? = null,
    val placeholder: String? = null,
    val isPassword: Boolean = false,
    val defaultValue: String? = null,
    val defaultBoolean: Boolean = false,
    val options: List<PluginSettingsOptionUiModel> = emptyList()
)

data class PluginScraperSettingsUiState(
    val scraper: PluginScraperUiModel,
    val loading: Boolean,
    val fields: List<PluginSettingsFieldUiModel>
)

data class CloudstreamRepoUiModel(
    val name: String,
    val url: String,
    val iconUrl: String? = null
)

data class CloudstreamPluginUiModel(
    val internalName: String,
    val name: String,
    val description: String,
    val iconUrl: String? = null,
    val typesLabel: String = "",
    val isInstalled: Boolean = false
)

data class PluginsUiState(
    val repositories: List<PluginRepositoryUiModel> = emptyList(),
    val scrapers: List<PluginScraperUiModel> = emptyList(),
    val addingRepositoryUrl: String? = null,
    val repositoryError: String? = null,
    val scraperSettingsSheet: PluginScraperSettingsUiState? = null,
    val cloudstreamRepos: List<CloudstreamRepoUiModel> = emptyList(),
    val isAddingCloudstreamRepo: Boolean = false,
    val cloudstreamRepoError: String? = null,
    val openRepoUrl: String? = null,
    val openRepoName: String? = null,
    val openRepoPlugins: List<CloudstreamPluginUiModel> = emptyList(),
    val isLoadingRepoPlugins: Boolean = false,
    val installingPluginKeys: Set<String> = emptySet(),
    val repoDialogError: String? = null
)

sealed interface PluginsAction {
    data object Refresh : PluginsAction
    data class RepositoryAdded(val manifestUrl: String) : PluginsAction
    data class RepositoryRemoved(val manifestUrl: String) : PluginsAction
    data class RepositoryRefreshed(val manifestUrl: String) : PluginsAction
    data class ScraperToggled(val scraperId: String, val enabled: Boolean) : PluginsAction
    data class ScraperSettingsRequested(val scraperId: String) : PluginsAction
    data object ScraperSettingsDismissed : PluginsAction
    data class ScraperSettingsSaved(val scraperId: String, val values: Map<String, Any?>) : PluginsAction
    data class CloudstreamRepoAdded(val url: String) : PluginsAction
    data class RepoOpened(val url: String) : PluginsAction
    data object RepoDialogDismissed : PluginsAction
    data class RepoRemoved(val url: String) : PluginsAction
    data class RepoPluginToggled(val repoUrl: String, val internalName: String) : PluginsAction
}

interface PluginsDataSource {
    fun observePlugins(): Flow<PluginsUiState>
    suspend fun refresh()
    suspend fun addRepository(manifestUrl: String)
    suspend fun removeRepository(manifestUrl: String)
    suspend fun refreshRepository(manifestUrl: String)
    suspend fun toggleScraper(scraperId: String, enabled: Boolean)
    suspend fun requestScraperSettings(scraperId: String)
    suspend fun dismissScraperSettings()
    suspend fun saveScraperSettings(scraperId: String, values: Map<String, Any?>)
    suspend fun addCloudstreamRepository(url: String)
    suspend fun openRepo(url: String)
    suspend fun dismissRepoDialog()
    suspend fun removeRepo(url: String)
    suspend fun toggleRepoPlugin(repoUrl: String, internalName: String)
}
