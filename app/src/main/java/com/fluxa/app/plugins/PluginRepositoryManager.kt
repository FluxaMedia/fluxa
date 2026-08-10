package com.fluxa.app.plugins

import com.fluxa.app.shared.feature.plugins.enabledManifestUrls
import com.fluxa.app.core.rust.FluxaAndroidHeadlessEnvironment
import com.fluxa.app.core.rust.PluginHttpClientImpl
import com.fluxa.app.data.platform.PlatformKeyValueStore
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.plugins.NuvioPluginRepositoryEngine
import com.fluxa.app.data.plugins.NuvioPluginRepositoryUiModel
import com.fluxa.app.data.plugins.NuvioPluginScraperUiModel
import com.fluxa.app.data.plugins.NuvioPluginSettingsFieldUiModel
import com.fluxa.app.data.plugins.NuvioPluginSettingsOptionUiModel
import com.fluxa.app.data.plugins.NuvioPluginsUiState
import com.fluxa.app.data.plugins.resolveNuvioPluginTmdbId
import com.fluxa.app.data.remote.NuvioPluginDto
import com.fluxa.app.data.remote.TmdbService
import com.fluxa.app.data.remote.Stream
import com.google.gson.Gson
import kotlinx.coroutines.flow.StateFlow
import okhttp3.OkHttpClient
import javax.inject.Inject
import javax.inject.Named
import javax.inject.Singleton

typealias PluginRepositoryUiModel = NuvioPluginRepositoryUiModel
typealias PluginScraperUiModel = NuvioPluginScraperUiModel
typealias PluginsUiState = NuvioPluginsUiState
typealias PluginSettingsOptionUiModel = NuvioPluginSettingsOptionUiModel
typealias PluginSettingsFieldUiModel = NuvioPluginSettingsFieldUiModel

/** Android dependency-injection wrapper around the JVM-shared Nuvio plugin engine. */
@Singleton
class PluginRepositoryManager @Inject constructor(
    @param:Named("PluginRepositoryState") prefsStore: PlatformKeyValueStore,
    environment: dagger.Lazy<FluxaAndroidHeadlessEnvironment>,
    pluginHttpClient: PluginHttpClientImpl,
    @param:Named("PluginScraperClient") scraperCodeClient: OkHttpClient,
    tmdbService: TmdbService,
    profileManager: ProfileManager,
    gson: Gson,
) {
    private val delegate = NuvioPluginRepositoryEngine(
        prefsStore = prefsStore,
        environmentProvider = environment::get,
        pluginHttpClient = pluginHttpClient,
        scraperCodeClient = scraperCodeClient,
        gson = gson,
        pluginTmdbIdResolver = { contentId, mediaType ->
            val activeProfileId = profileManager.getLastActiveProfileId()
            val apiKey = profileManager.getProfiles()
                .firstOrNull { it.id == activeProfileId }
                ?.safeTmdbApiKey
                .orEmpty()
            resolveNuvioPluginTmdbId(tmdbService, contentId, mediaType, apiKey)
        },
        logTag = "PluginRepositoryManager",
    )

    val state: StateFlow<PluginsUiState> get() = delegate.state

    suspend fun addRepository(manifestUrl: String) = delegate.addRepository(manifestUrl)

    /** Reconciles enabled repository URLs from the currently active Nuvio profile. */
    suspend fun syncNuvioPlugins(plugins: List<NuvioPluginDto>) {
        delegate.syncNuvioRepositoryUrls(plugins.enabledManifestUrls())
    }

    @Deprecated("Use syncNuvioPlugins")
    suspend fun importNuvioPlugins(plugins: List<NuvioPluginDto>) = syncNuvioPlugins(plugins)

    suspend fun removeRepository(manifestUrl: String) = delegate.removeRepository(manifestUrl)

    suspend fun refreshRepository(manifestUrl: String) = delegate.refreshRepository(manifestUrl)

    suspend fun refreshAllRepositories() = delegate.refreshAllRepositories()

    suspend fun toggleScraper(scraperId: String, enabled: Boolean) =
        delegate.toggleScraper(scraperId, enabled)

    suspend fun updateScraperSettings(scraperId: String, settings: Map<String, Any?>) =
        delegate.updateScraperSettings(scraperId, settings)

    suspend fun executeEnabledScrapers(
        contentId: String,
        mediaType: String,
        season: Int?,
        episode: Int?,
    ): List<Stream> = delegate.executeEnabledScrapers(contentId, mediaType, season, episode)

    fun hasCompatibleEnabledScrapers(mediaType: String): Boolean =
        delegate.hasCompatibleEnabledScrapers(mediaType)

    suspend fun executeScraper(
        scraper: PluginScraperUiModel,
        tmdbId: String,
        mediaType: String,
        season: Int?,
        episode: Int?,
    ): List<Stream> = delegate.executeScraper(scraper, tmdbId, mediaType, season, episode)

    suspend fun getSettingsLayout(
        scraper: PluginScraperUiModel,
    ): List<PluginSettingsFieldUiModel> = delegate.getSettingsLayout(scraper)
}
