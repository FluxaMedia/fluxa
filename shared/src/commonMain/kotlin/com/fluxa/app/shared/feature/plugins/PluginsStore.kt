package com.fluxa.app.shared.feature.plugins

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class PluginsStore(
    private val dataSource: PluginsDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<PluginsUiState> = dataSource.observePlugins()
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), PluginsUiState())

    suspend fun dispatch(action: PluginsAction) {
        when (action) {
            PluginsAction.Refresh -> dataSource.refresh()
            is PluginsAction.RepositoryAdded -> dataSource.addRepository(action.manifestUrl)
            is PluginsAction.RepositoryRemoved -> dataSource.removeRepository(action.manifestUrl)
            is PluginsAction.RepositoryRefreshed -> dataSource.refreshRepository(action.manifestUrl)
            is PluginsAction.ScraperToggled -> dataSource.toggleScraper(action.scraperId, action.enabled)
            is PluginsAction.ScraperSettingsRequested -> dataSource.requestScraperSettings(action.scraperId)
            PluginsAction.ScraperSettingsDismissed -> dataSource.dismissScraperSettings()
            is PluginsAction.ScraperSettingsSaved -> dataSource.saveScraperSettings(action.scraperId, action.values)
            is PluginsAction.CloudstreamRepoAdded -> dataSource.addCloudstreamRepository(action.url)
            is PluginsAction.RepoOpened -> dataSource.openRepo(action.url)
            PluginsAction.RepoDialogDismissed -> dataSource.dismissRepoDialog()
            is PluginsAction.RepoRemoved -> dataSource.removeRepo(action.url)
            is PluginsAction.RepoPluginToggled -> dataSource.toggleRepoPlugin(action.repoUrl, action.internalName)
        }
    }
}
