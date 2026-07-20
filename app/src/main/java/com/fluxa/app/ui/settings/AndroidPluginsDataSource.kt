package com.fluxa.app.ui.settings

import com.fluxa.app.common.AppStrings
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.plugins.PluginRepositoryManager
import com.fluxa.app.plugins.PluginScraperUiModel as NuvioScraperUiModel
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import com.fluxa.app.plugins.cloudstream.PluginInfo
import com.fluxa.app.shared.feature.plugins.CloudstreamPluginUiModel
import com.fluxa.app.shared.feature.plugins.CloudstreamRepoUiModel
import com.fluxa.app.shared.feature.plugins.PluginRepositoryUiModel
import com.fluxa.app.shared.feature.plugins.PluginScraperSettingsUiState
import com.fluxa.app.shared.feature.plugins.PluginScraperUiModel
import com.fluxa.app.shared.feature.plugins.PluginSettingsFieldUiModel
import com.fluxa.app.shared.feature.plugins.PluginSettingsOptionUiModel
import com.fluxa.app.shared.feature.plugins.PluginsDataSource
import com.fluxa.app.shared.feature.plugins.PluginsUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update

class AndroidPluginsDataSource(
    private val pluginRepositoryManager: PluginRepositoryManager,
    private val pluginManager: PluginManager,
    private val language: () -> String
) : PluginsDataSource {

    private data class Extras(
        val scraperSettingsSheet: PluginScraperSettingsUiState? = null,
        val isAddingCloudstreamRepo: Boolean = false,
        val cloudstreamRepoError: String? = null,
        val openRepoUrl: String? = null,
        val openRepoName: String? = null,
        val openRepoPlugins: List<PluginInfo> = emptyList(),
        val isLoadingRepoPlugins: Boolean = false,
        val installingPluginKeys: Set<String> = emptySet(),
        val repoDialogError: String? = null
    )

    private val extras = MutableStateFlow(Extras())
    private var latestNuvioScrapers: List<NuvioScraperUiModel> = emptyList()

    override fun observePlugins(): Flow<PluginsUiState> = combine(
        pluginRepositoryManager.state,
        pluginManager.installedPlugins,
        pluginManager.repositories,
        extras
    ) { nuvio, installedPlugins, repos, ex ->
        latestNuvioScrapers = nuvio.scrapers
        PluginsUiState(
            repositories = nuvio.repositories.map {
                PluginRepositoryUiModel(
                    manifestUrl = it.manifestUrl,
                    name = it.name,
                    description = it.description,
                    scraperCount = it.scraperCount
                )
            },
            scrapers = nuvio.scrapers.map {
                PluginScraperUiModel(
                    id = it.id,
                    name = it.name,
                    repositoryUrl = it.repositoryUrl,
                    enabled = it.enabled,
                    supportedTypes = it.supportedTypes,
                    hasSettings = it.hasSettings,
                    settings = it.settings
                )
            },
            addingRepositoryUrl = nuvio.addingRepositoryUrl,
            repositoryError = nuvio.error,
            scraperSettingsSheet = ex.scraperSettingsSheet,
            cloudstreamRepos = repos.map { CloudstreamRepoUiModel(name = it.name, url = it.url, iconUrl = it.iconUrl) },
            isAddingCloudstreamRepo = ex.isAddingCloudstreamRepo,
            cloudstreamRepoError = ex.cloudstreamRepoError,
            openRepoUrl = ex.openRepoUrl,
            openRepoName = ex.openRepoName,
            openRepoPlugins = mapCloudstreamPlugins(ex, installedPlugins),
            isLoadingRepoPlugins = ex.isLoadingRepoPlugins,
            installingPluginKeys = ex.installingPluginKeys,
            repoDialogError = ex.repoDialogError
        )
    }

    private fun mapCloudstreamPlugins(ex: Extras, installedPlugins: List<InstalledPlugin>): List<CloudstreamPluginUiModel> =
        ex.openRepoPlugins.map { plugin ->
            val key = pluginManager.pluginInstallKey(ex.openRepoUrl, plugin.internalName)
            CloudstreamPluginUiModel(
                internalName = plugin.internalName,
                name = plugin.name,
                description = plugin.description,
                iconUrl = plugin.iconUrl,
                typesLabel = plugin.getDisplayTypes(),
                isInstalled = installedPlugins.any { pluginManager.pluginInstallKey(it.repositoryUrl, it.internalName) == key }
            )
        }

    override suspend fun refresh() = Unit

    override suspend fun addRepository(manifestUrl: String) {
        pluginRepositoryManager.addRepository(manifestUrl)
    }

    override suspend fun removeRepository(manifestUrl: String) {
        pluginRepositoryManager.removeRepository(manifestUrl)
    }

    override suspend fun refreshRepository(manifestUrl: String) {
        pluginRepositoryManager.refreshRepository(manifestUrl)
    }

    override suspend fun toggleScraper(scraperId: String, enabled: Boolean) {
        pluginRepositoryManager.toggleScraper(scraperId, enabled)
    }

    override suspend fun requestScraperSettings(scraperId: String) {
        val scraper = latestNuvioScrapers.find { it.id == scraperId } ?: return
        val sharedScraper = PluginScraperUiModel(
            id = scraper.id,
            name = scraper.name,
            repositoryUrl = scraper.repositoryUrl,
            enabled = scraper.enabled,
            supportedTypes = scraper.supportedTypes,
            hasSettings = scraper.hasSettings,
            settings = scraper.settings
        )
        extras.update {
            it.copy(scraperSettingsSheet = PluginScraperSettingsUiState(scraper = sharedScraper, loading = true, fields = emptyList()))
        }
        val fields = pluginRepositoryManager.getSettingsLayout(scraper).map { field ->
            PluginSettingsFieldUiModel(
                key = field.key,
                type = field.type,
                label = field.label,
                description = field.description,
                placeholder = field.placeholder,
                isPassword = field.isPassword,
                defaultValue = field.defaultValue,
                defaultBoolean = field.defaultBoolean,
                options = field.options.map { PluginSettingsOptionUiModel(label = it.label, value = it.value) }
            )
        }
        extras.update { current ->
            val sheet = current.scraperSettingsSheet
            if (sheet?.scraper?.id == scraperId) {
                current.copy(scraperSettingsSheet = sheet.copy(loading = false, fields = fields))
            } else {
                current
            }
        }
    }

    override suspend fun dismissScraperSettings() {
        extras.update { it.copy(scraperSettingsSheet = null) }
    }

    override suspend fun saveScraperSettings(scraperId: String, values: Map<String, Any?>) {
        pluginRepositoryManager.updateScraperSettings(scraperId, values)
        extras.update { it.copy(scraperSettingsSheet = null) }
    }

    override suspend fun addCloudstreamRepository(url: String) {
        extras.update { it.copy(isAddingCloudstreamRepo = true, cloudstreamRepoError = null) }
        val normalizedUrl = FluxaCoreNative.normalizeCloudstreamRepoUrl(url)
        val result = pluginManager.addRepository(normalizedUrl)
        extras.update {
            if (result.isSuccess) {
                it.copy(isAddingCloudstreamRepo = false)
            } else {
                it.copy(
                    isAddingCloudstreamRepo = false,
                    cloudstreamRepoError = result.exceptionOrNull()?.message
                        ?: AppStrings.t(language(), "addons.repository_add_failed")
                )
            }
        }
    }

    override suspend fun openRepo(url: String) {
        val repoName = pluginManager.repositories.value.find { it.url == url }?.name?.takeIf { it.isNotBlank() }
        extras.update {
            it.copy(openRepoUrl = url, openRepoName = repoName, isLoadingRepoPlugins = true, openRepoPlugins = emptyList(), repoDialogError = null)
        }
        val plugins = pluginManager.getPluginsFromRepository(url)
        extras.update { it.copy(openRepoPlugins = plugins, isLoadingRepoPlugins = false) }
    }

    override suspend fun dismissRepoDialog() {
        extras.update { it.copy(openRepoUrl = null, openRepoName = null, openRepoPlugins = emptyList(), repoDialogError = null) }
    }

    override suspend fun removeRepo(url: String) {
        pluginManager.removeRepository(url)
    }

    override suspend fun toggleRepoPlugin(repoUrl: String, internalName: String) {
        val key = pluginManager.pluginInstallKey(repoUrl, internalName)
        val plugin = extras.value.openRepoPlugins.find { it.internalName == internalName } ?: return
        val isInstalled = pluginManager.installedPlugins.value.any {
            pluginManager.pluginInstallKey(it.repositoryUrl, it.internalName) == key
        }
        extras.update { it.copy(installingPluginKeys = it.installingPluginKeys + key, repoDialogError = null) }
        if (isInstalled) {
            pluginManager.uninstallPlugin(repoUrl, internalName)
        } else {
            val result = pluginManager.installPlugin(plugin, repoUrl)
            if (result.isFailure) {
                extras.update {
                    it.copy(repoDialogError = result.exceptionOrNull()?.message ?: AppStrings.t(language(), "auto.install_failed"))
                }
            }
        }
        extras.update { it.copy(installingPluginKeys = it.installingPluginKeys - key) }
    }
}
