package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.plugins.PluginRepositoryUiModel
import com.fluxa.app.shared.feature.plugins.PluginScraperSettingsUiState
import com.fluxa.app.shared.feature.plugins.PluginScraperUiModel
import com.fluxa.app.shared.feature.plugins.PluginSettingsFieldUiModel
import com.fluxa.app.shared.feature.plugins.PluginSettingsOptionUiModel
import com.fluxa.app.shared.feature.plugins.PluginsDataSource
import com.fluxa.app.shared.feature.plugins.PluginsUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.boolean
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.longOrNull

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

data class ApplePluginRepositorySnapshot(
    val manifestUrl: String,
    val name: String,
    val description: String?,
    val scraperCount: Int
)

data class ApplePluginSettingsOptionSnapshot(
    val label: String,
    val value: String
)

data class ApplePluginSettingsFieldSnapshot(
    val key: String,
    val type: String,
    val label: String,
    val description: String?,
    val placeholder: String?,
    val isPassword: Boolean,
    val defaultValue: String?,
    val defaultBoolean: Boolean,
    val options: List<ApplePluginSettingsOptionSnapshot> = emptyList()
)

data class ApplePluginScraperSnapshot(
    val id: String,
    val name: String,
    val repositoryUrl: String,
    val enabled: Boolean,
    val supportedTypes: List<String>,
    val hasSettings: Boolean,
    val settingsJson: String
)

data class ApplePluginScraperSettingsSnapshot(
    val scraper: ApplePluginScraperSnapshot,
    val loading: Boolean,
    val fields: List<ApplePluginSettingsFieldSnapshot> = emptyList()
)

data class ApplePluginsSnapshot(
    val repositories: List<ApplePluginRepositorySnapshot> = emptyList(),
    val scrapers: List<ApplePluginScraperSnapshot> = emptyList(),
    val addingRepositoryUrl: String? = null,
    val repositoryError: String? = null,
    val scraperSettingsSheet: ApplePluginScraperSettingsSnapshot? = null
)

private fun jsonElementToAny(element: JsonElement): Any? = when (element) {
    is JsonNull -> null
    is JsonArray -> element.map(::jsonElementToAny)
    is JsonObject -> element.mapValues { jsonElementToAny(it.value) }
    is JsonPrimitive -> when {
        element.booleanOrNull != null -> element.boolean
        element.longOrNull != null -> element.longOrNull
        element.doubleOrNull != null -> element.doubleOrNull
        else -> element.contentOrNull
    }
}

private fun anyToJsonElement(value: Any?): JsonElement = when (value) {
    null -> JsonNull
    is String -> JsonPrimitive(value)
    is Boolean -> JsonPrimitive(value)
    is Number -> JsonPrimitive(value)
    else -> JsonPrimitive(value.toString())
}

private fun parseSettingsJson(json: String): Map<String, Any?> =
    runCatching { Json.parseToJsonElement(json).jsonObject.mapValues { jsonElementToAny(it.value) } }
        .getOrDefault(emptyMap())

private fun encodeSettingsJson(values: Map<String, Any?>): String =
    Json.encodeToString(JsonObject.serializer(), JsonObject(values.mapValues(::anyToJsonElement)))

private fun ApplePluginRepositorySnapshot.toUiModel(): PluginRepositoryUiModel = PluginRepositoryUiModel(
    manifestUrl = manifestUrl,
    name = name,
    description = description,
    scraperCount = scraperCount
)

private fun ApplePluginScraperSnapshot.toUiModel(): PluginScraperUiModel = PluginScraperUiModel(
    id = id,
    name = name,
    repositoryUrl = repositoryUrl,
    enabled = enabled,
    supportedTypes = supportedTypes,
    hasSettings = hasSettings,
    settings = parseSettingsJson(settingsJson)
)

private fun ApplePluginSettingsOptionSnapshot.toUiModel(): PluginSettingsOptionUiModel = PluginSettingsOptionUiModel(
    label = label,
    value = value
)

private fun ApplePluginSettingsFieldSnapshot.toUiModel(): PluginSettingsFieldUiModel = PluginSettingsFieldUiModel(
    key = key,
    type = type,
    label = label,
    description = description,
    placeholder = placeholder,
    isPassword = isPassword,
    defaultValue = defaultValue,
    defaultBoolean = defaultBoolean,
    options = options.map { it.toUiModel() }
)

private fun ApplePluginScraperSettingsSnapshot.toUiState(): PluginScraperSettingsUiState = PluginScraperSettingsUiState(
    scraper = scraper.toUiModel(),
    loading = loading,
    fields = fields.map { it.toUiModel() }
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
        state.value.scrapers.find { it.id == scraperId }?.let { scraper ->
            state.value = state.value.copy(
                scraperSettingsSheet = PluginScraperSettingsUiState(
                    scraper = scraper,
                    loading = true,
                    fields = emptyList()
                )
            )
        }
        postAction(ApplePluginsActionSnapshot(type = "requestScraperSettings", scraperId = scraperId))
    }

    override suspend fun dismissScraperSettings() {
        state.value = state.value.copy(scraperSettingsSheet = null)
    }

    override suspend fun saveScraperSettings(scraperId: String, values: Map<String, Any?>) {
        postAction(
            ApplePluginsActionSnapshot(
                type = "saveScraperSettings",
                scraperId = scraperId,
                settingsJson = encodeSettingsJson(values)
            )
        )
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

    fun update(snapshot: ApplePluginsSnapshot) {
        state.value = state.value.copy(
            repositories = snapshot.repositories.map { it.toUiModel() },
            scrapers = snapshot.scrapers.map { it.toUiModel() },
            addingRepositoryUrl = snapshot.addingRepositoryUrl,
            repositoryError = snapshot.repositoryError,
            scraperSettingsSheet = snapshot.scraperSettingsSheet?.toUiState()
        )
    }

    private fun postAction(action: ApplePluginsActionSnapshot) {
        onActionRequested(action)
    }
}
