package com.fluxa.app.plugins

import android.content.Context
import android.util.Log
import com.fluxa.app.core.rust.FluxaAndroidHeadlessEnvironment
import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.core.rust.FluxaHeadlessRuntimeFactory
import com.fluxa.app.core.rust.PluginHttpClientImpl
import com.fluxa.app.data.remote.Stream
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import okhttp3.Request
import javax.inject.Inject
import javax.inject.Named
import javax.inject.Singleton

private const val PREFS_NAME = "fluxa_plugin_repository_manager"
private const val KEY_PERSISTED_STATE = "persisted_state"

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
    val filename: String,
    val enabled: Boolean,
    val supportedTypes: List<String>,
    val hasSettings: Boolean = false,
    val settings: Map<String, Any?> = emptyMap()
)

data class PluginsUiState(
    val repositories: List<PluginRepositoryUiModel> = emptyList(),
    val scrapers: List<PluginScraperUiModel> = emptyList(),
    val addingRepositoryUrl: String? = null,
    val error: String? = null
)

private data class PersistedScraperOverride(
    val enabled: Boolean = true,
    val settings: Map<String, Any?> = emptyMap()
)

private data class PersistedPluginsState(
    val repositoryUrls: List<String> = emptyList(),
    val scraperOverrides: Map<String, PersistedScraperOverride> = emptyMap()
)

@Singleton
class PluginRepositoryManager @Inject constructor(
    @param:ApplicationContext private val context: Context,
    environment: dagger.Lazy<FluxaAndroidHeadlessEnvironment>,
    private val pluginHttpClient: PluginHttpClientImpl,
    @param:Named("PluginScraperClient") private val scraperCodeClient: okhttp3.OkHttpClient,
    private val gson: Gson
) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private val runtime by lazy(LazyThreadSafetyMode.SYNCHRONIZED) {
        FluxaHeadlessRuntimeFactory.createUniFfi(environment.get())
    }
    private val streamListType = object : TypeToken<List<Stream>>() {}.type
    private val codeCacheMutex = Mutex()
    private val codeCache = mutableMapOf<String, String>()
    private val prefs by lazy { context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE) }

    val state: StateFlow<PluginsUiState> by lazy {
        runtime.state
            .map { it.toPluginsUiState() }
            .stateIn(scope, SharingStarted.Eagerly, PluginsUiState())
    }

    init {
        scope.launch { restorePersistedRepositories() }
    }

    private suspend fun restorePersistedRepositories() {
        val saved = loadPersistedState()
        saved.repositoryUrls.forEach { manifestUrl ->
            runtime.dispatch(mapOf("type" to "pluginRepositoryAddRequested", "manifestUrl" to manifestUrl))
        }
        saved.scraperOverrides.forEach { (scraperId, override) ->
            if (!override.enabled) {
                runtime.dispatch(mapOf("type" to "pluginScraperToggled", "scraperId" to scraperId, "enabled" to false))
            }
            if (override.settings.isNotEmpty()) {
                runtime.dispatch(mapOf("type" to "pluginScraperSettingsUpdated", "scraperId" to scraperId, "settings" to override.settings))
            }
        }
    }

    suspend fun addRepository(manifestUrl: String) {
        runtime.dispatch(mapOf("type" to "pluginRepositoryAddRequested", "manifestUrl" to manifestUrl))
        persistCurrentState()
    }

    suspend fun removeRepository(manifestUrl: String) {
        runtime.dispatch(mapOf("type" to "pluginRepositoryRemoveRequested", "manifestUrl" to manifestUrl))
        persistCurrentState()
    }

    suspend fun toggleScraper(scraperId: String, enabled: Boolean) {
        runtime.dispatch(mapOf("type" to "pluginScraperToggled", "scraperId" to scraperId, "enabled" to enabled))
        persistCurrentState()
    }

    suspend fun updateScraperSettings(scraperId: String, settings: Map<String, Any?>) {
        runtime.dispatch(mapOf("type" to "pluginScraperSettingsUpdated", "scraperId" to scraperId, "settings" to settings))
        persistCurrentState()
    }

    private fun persistCurrentState() {
        val current = runtime.state.value.toPluginsUiState()
        val persisted = PersistedPluginsState(
            repositoryUrls = current.repositories.map { it.manifestUrl },
            scraperOverrides = current.scrapers.associate { scraper ->
                scraper.id to PersistedScraperOverride(enabled = scraper.enabled, settings = scraper.settings)
            }
        )
        prefs.edit().putString(KEY_PERSISTED_STATE, gson.toJson(persisted)).apply()
    }

    private fun loadPersistedState(): PersistedPluginsState {
        val json = prefs.getString(KEY_PERSISTED_STATE, null) ?: return PersistedPluginsState()
        return try {
            gson.fromJson(json, PersistedPluginsState::class.java) ?: PersistedPluginsState()
        } catch (e: Exception) {
            Log.w("PluginRepositoryManager", "failed to parse persisted plugin state", e)
            PersistedPluginsState()
        }
    }

    suspend fun executeScraper(
        scraper: PluginScraperUiModel,
        tmdbId: String,
        mediaType: String,
        season: Int?,
        episode: Int?
    ): List<Stream> = withContext(Dispatchers.IO) {
        val code = fetchScraperCode(scraper) ?: return@withContext emptyList()
        try {
            val rawJson = FluxaCoreUniFfi.executePluginScraper(pluginHttpClient, code, tmdbId, mediaType, season, episode)
            val normalized = FluxaCoreUniFfi.coreInvokeValue("pluginStreamResultsToStreams", rawJson)
            val streams: List<Stream> = gson.fromJson(normalized, streamListType) ?: emptyList()
            streams.map { it.copy(addonName = scraper.name) }
        } catch (e: Exception) {
            Log.w("PluginRepositoryManager", "scraper ${scraper.id} failed", e)
            emptyList()
        }
    }

    private suspend fun fetchScraperCode(scraper: PluginScraperUiModel): String? = codeCacheMutex.withLock {
        codeCache[scraper.id]?.let { return@withLock it }
        val url = resolveScraperUrl(scraper.repositoryUrl, scraper.filename)
        try {
            val request = Request.Builder().url(url).build()
            scraperCodeClient.newCall(request).execute().use { response ->
                if (!response.isSuccessful) return@withLock null
                val code = response.body.string()
                if (code.isBlank()) return@withLock null
                codeCache[scraper.id] = code
                code
            }
        } catch (e: Exception) {
            Log.w("PluginRepositoryManager", "failed to fetch scraper code for ${scraper.id}", e)
            null
        }
    }

    private fun resolveScraperUrl(repositoryUrl: String, filename: String): String {
        val base = repositoryUrl.substringBeforeLast('/', missingDelimiterValue = repositoryUrl)
        return "$base/$filename"
    }

    @Suppress("UNCHECKED_CAST")
    private fun Map<String, Any?>.toPluginsUiState(): PluginsUiState {
        val plugins = this["plugins"] as? Map<String, Any?> ?: return PluginsUiState()
        val repositories = (plugins["repositories"] as? List<Map<String, Any?>>).orEmpty().map { repo ->
            PluginRepositoryUiModel(
                manifestUrl = repo["manifestUrl"] as? String ?: "",
                name = repo["name"] as? String ?: "",
                description = repo["description"] as? String,
                scraperCount = (repo["scraperCount"] as? Double)?.toInt() ?: 0
            )
        }
        val scrapers = (plugins["scrapers"] as? List<Map<String, Any?>>).orEmpty().map { scraper ->
            PluginScraperUiModel(
                id = scraper["id"] as? String ?: "",
                name = scraper["name"] as? String ?: "",
                repositoryUrl = scraper["repositoryUrl"] as? String ?: "",
                filename = scraper["filename"] as? String ?: "",
                enabled = scraper["enabled"] as? Boolean ?: true,
                supportedTypes = (scraper["supportedTypes"] as? List<String>).orEmpty(),
                hasSettings = scraper["hasSettings"] as? Boolean ?: false,
                settings = (scraper["settings"] as? Map<String, Any?>).orEmpty()
            )
        }
        val error = (plugins["error"] as? Map<*, *>)?.get("code") as? String
        return PluginsUiState(
            repositories = repositories,
            scrapers = scrapers,
            addingRepositoryUrl = plugins["addingRepositoryUrl"] as? String,
            error = error
        )
    }
}
