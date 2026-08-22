package com.fluxa.app.shared.feature.plugins

import com.fluxa.app.data.plugins.NuvioPluginRepositoryUiModel
import com.fluxa.app.data.plugins.NuvioPluginScraperUiModel
import com.fluxa.app.data.plugins.NuvioPluginSettingsFieldUiModel
import com.fluxa.app.data.plugins.NuvioPluginsUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine

/** Shared Android/Desktop presentation adapter for Nuvio plugin repositories. */
class JvmNuvioPluginsUiController(
    private val repositoryState: StateFlow<NuvioPluginsUiState>,
    private val refreshAllRepositories: suspend () -> Unit,
    private val addRepositoryCommand: suspend (String) -> Unit,
    private val removeRepositoryCommand: suspend (String) -> Unit,
    private val refreshRepositoryCommand: suspend (String) -> Unit,
    private val toggleScraperCommand: suspend (String, Boolean) -> Unit,
    private val updateScraperSettingsCommand: suspend (String, Map<String, Any?>) -> Unit,
    private val settingsLayout: suspend (NuvioPluginScraperUiModel) -> List<NuvioPluginSettingsFieldUiModel>,
) {
    private val settingsSheet = MutableStateFlow<PluginScraperSettingsUiState?>(null)

    fun observePlugins(): Flow<PluginsUiState> = combine(repositoryState, settingsSheet) { state, sheet ->
        state.toPluginsUiState(sheet)
    }

    suspend fun refresh() = refreshAllRepositories()
    suspend fun addRepository(manifestUrl: String) = addRepositoryCommand(manifestUrl)
    suspend fun removeRepository(manifestUrl: String) = removeRepositoryCommand(manifestUrl)
    suspend fun refreshRepository(manifestUrl: String) = refreshRepositoryCommand(manifestUrl)
    suspend fun toggleScraper(scraperId: String, enabled: Boolean) = toggleScraperCommand(scraperId, enabled)

    suspend fun requestScraperSettings(scraperId: String) {
        val scraper = repositoryState.value.scrapers.find { it.id == scraperId } ?: return
        settingsSheet.value = PluginScraperSettingsUiState(
            scraper = scraper.toPluginScraperUiModel(),
            loading = true,
            fields = emptyList(),
        )
        val fields = settingsLayout(scraper)
            .map(NuvioPluginSettingsFieldUiModel::toPluginSettingsFieldUiModel)
        val current = settingsSheet.value
        if (current?.scraper?.id == scraperId) {
            settingsSheet.value = current.copy(loading = false, fields = fields)
        }
    }

    fun dismissScraperSettings() {
        settingsSheet.value = null
    }

    suspend fun saveScraperSettings(scraperId: String, values: Map<String, Any?>) {
        updateScraperSettingsCommand(scraperId, values)
        settingsSheet.value = null
    }
}

private fun NuvioPluginsUiState.toPluginsUiState(
    settingsSheet: PluginScraperSettingsUiState?,
): PluginsUiState = PluginsUiState(
    repositories = repositories.map(NuvioPluginRepositoryUiModel::toPluginRepositoryUiModel),
    scrapers = scrapers.map(NuvioPluginScraperUiModel::toPluginScraperUiModel),
    addingRepositoryUrl = addingRepositoryUrl,
    repositoryError = error,
    scraperSettingsSheet = settingsSheet,
)

private fun NuvioPluginRepositoryUiModel.toPluginRepositoryUiModel(): PluginRepositoryUiModel =
    PluginRepositoryUiModel(
        manifestUrl = manifestUrl,
        name = name,
        description = description,
        scraperCount = scraperCount,
    )

private fun NuvioPluginScraperUiModel.toPluginScraperUiModel(): PluginScraperUiModel =
    PluginScraperUiModel(
        id = id,
        name = name,
        repositoryUrl = repositoryUrl,
        enabled = enabled,
        supportedTypes = supportedTypes,
        hasSettings = hasSettings,
        settings = settings,
    )

private fun NuvioPluginSettingsFieldUiModel.toPluginSettingsFieldUiModel(): PluginSettingsFieldUiModel =
    PluginSettingsFieldUiModel(
        key = key,
        type = type,
        label = label,
        description = description,
        placeholder = placeholder,
        isPassword = isPassword,
        defaultValue = defaultValue,
        defaultBoolean = defaultBoolean,
        options = options.map { option ->
            PluginSettingsOptionUiModel(label = option.label, value = option.value)
        },
    )
