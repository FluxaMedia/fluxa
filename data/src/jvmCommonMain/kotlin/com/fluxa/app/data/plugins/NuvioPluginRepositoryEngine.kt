package com.fluxa.app.data.plugins

import com.fluxa.app.common.PlatformLog
import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.core.rust.FluxaHeadlessAppRuntime
import com.fluxa.app.core.rust.FluxaHeadlessRuntimeFactory
import com.fluxa.app.core.rust.HeadlessPlatformEnvironment
import com.fluxa.app.core.rust.PluginHttpClientImpl
import com.fluxa.app.data.platform.PlatformKeyValueStore
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.SubtitleAttributes
import com.fluxa.app.data.remote.SubtitleData
import com.google.gson.Gson
import com.google.gson.JsonParser
import com.google.gson.reflect.TypeToken
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import okhttp3.HttpUrl.Companion.toHttpUrlOrNull
import okhttp3.OkHttpClient
import okhttp3.Request

private const val KEY_PERSISTED_STATE = "persisted_state"

data class NuvioPluginRepositoryUiModel(
    val manifestUrl: String,
    val name: String,
    val description: String?,
    val scraperCount: Int,
)

data class NuvioPluginScraperUiModel(
    val id: String,
    val name: String,
    val repositoryUrl: String,
    val filename: String,
    val enabled: Boolean,
    val supportedTypes: List<String>,
    val hasSettings: Boolean = false,
    val settings: Map<String, Any?> = emptyMap(),
)

data class NuvioPluginsUiState(
    val repositories: List<NuvioPluginRepositoryUiModel> = emptyList(),
    val scrapers: List<NuvioPluginScraperUiModel> = emptyList(),
    val addingRepositoryUrl: String? = null,
    val error: String? = null,
)

data class NuvioPluginSettingsOptionUiModel(
    val label: String,
    val value: String,
)

data class NuvioPluginSettingsFieldUiModel(
    val key: String,
    val type: String,
    val label: String,
    val description: String? = null,
    val placeholder: String? = null,
    val isPassword: Boolean = false,
    val defaultValue: String? = null,
    val defaultBoolean: Boolean = false,
    val options: List<NuvioPluginSettingsOptionUiModel> = emptyList(),
)

private data class PersistedScraperOverride(
    val enabled: Boolean = true,
    val settings: Map<String, Any?> = emptyMap(),
)

private data class PersistedPluginsState(
    /** Legacy union retained for migration from older Fluxa builds. */
    val repositoryUrls: List<String> = emptyList(),
    val manualRepositoryUrls: List<String> = emptyList(),
    val nuvioRepositoryUrls: List<String> = emptyList(),
    val scraperOverrides: Map<String, PersistedScraperOverride> = emptyMap(),
)

private data class PluginExecutionPlan(
    val contentId: String = "",
    val mediaType: String = "",
    val season: Int? = null,
    val episode: Int? = null,
    val scrapers: List<Map<String, Any?>> = emptyList(),
)

/**
 * JVM-shared implementation of the Nuvio/Fluxa plugin repository lifecycle.
 *
 * Android and desktop intentionally expose thin platform wrappers around this
 * class. Repository persistence, headless-core actions, scraper-code caching,
 * settings parsing and JavaScript execution therefore have one source of truth.
 */
class NuvioPluginRepositoryEngine(
    private val prefsStore: PlatformKeyValueStore,
    private val environmentProvider: () -> HeadlessPlatformEnvironment,
    private val pluginHttpClient: PluginHttpClientImpl,
    private val scraperCodeClient: OkHttpClient,
    private val gson: Gson,
    private val pluginTmdbIdResolver: suspend (String, String) -> String? = { _, _ -> null },
    private val logTag: String = "NuvioPluginRepository",
) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private val runtime: FluxaHeadlessAppRuntime by lazy(LazyThreadSafetyMode.SYNCHRONIZED) {
        FluxaHeadlessRuntimeFactory.createUniFfi(environmentProvider())
    }
    private val streamListType = object : TypeToken<List<Stream>>() {}.type
    private val codeCacheMutex = Mutex()
    private val codeCache = mutableMapOf<String, String>()
    private val repositorySyncMutex = Mutex()
    private var manualRepositoryUrls: Set<String> = emptySet()
    private var nuvioRepositoryUrls: Set<String> = emptySet()
    private val restoreJob: Job

    val state: StateFlow<NuvioPluginsUiState> by lazy {
        runtime.state
            .map { it.toPluginsUiState() }
            .stateIn(scope, SharingStarted.Eagerly, NuvioPluginsUiState())
    }

    init {
        restoreJob = scope.launch { restorePersistedRepositories() }
    }

    private suspend fun awaitRestored() {
        restoreJob.join()
    }

    private suspend fun restorePersistedRepositories() {
        val saved = loadPersistedState()
        nuvioRepositoryUrls = saved.nuvioRepositoryUrls.map(String::trim).filter(String::isNotEmpty).toSet()
        manualRepositoryUrls = when {
            saved.manualRepositoryUrls.isNotEmpty() || saved.nuvioRepositoryUrls.isNotEmpty() ->
                saved.manualRepositoryUrls.map(String::trim).filter(String::isNotEmpty).toSet()
            else -> saved.repositoryUrls.map(String::trim).filter(String::isNotEmpty).toSet()
        }
        (manualRepositoryUrls + nuvioRepositoryUrls).forEach { manifestUrl ->
            dispatchAddRepository(manifestUrl)
        }
        saved.scraperOverrides.forEach { (scraperId, override) ->
            if (!override.enabled) {
                runtime.dispatch(
                    mapOf(
                        "type" to "pluginScraperToggled",
                        "scraperId" to scraperId,
                        "enabled" to false,
                    ),
                )
            }
            if (override.settings.isNotEmpty()) {
                runtime.dispatch(
                    mapOf(
                        "type" to "pluginScraperSettingsUpdated",
                        "scraperId" to scraperId,
                        "settings" to override.settings,
                    ),
                )
            }
        }
    }

    suspend fun addRepository(manifestUrl: String) {
        awaitRestored()
        val normalized = manifestUrl.trim()
        if (normalized.isEmpty()) return
        manualRepositoryUrls = manualRepositoryUrls + normalized
        dispatchAddRepository(normalized)
        persistCurrentState()
    }

    /** Reconciles only the repositories sourced from the currently active Nuvio profile. */
    suspend fun syncNuvioRepositoryUrls(manifestUrls: Iterable<String>) {
        awaitRestored()
        repositorySyncMutex.withLock {
            val next = manifestUrls
                .asSequence()
                .map(String::trim)
                .filter(String::isNotEmpty)
                .distinct()
                .toSet()
            val previous = nuvioRepositoryUrls
            val currentUrls = state.value.repositories.map { it.manifestUrl }.toSet()

            (previous - next)
                .filterNot { it in manualRepositoryUrls }
                .forEach { manifestUrl ->
                    clearCodeCache()
                    runtime.dispatch(
                        mapOf(
                            "type" to "pluginRepositoryRemoveRequested",
                            "manifestUrl" to manifestUrl,
                        ),
                    )
                }

            (next - currentUrls).forEach { dispatchAddRepository(it) }
            nuvioRepositoryUrls = next
            persistCurrentState()
        }
    }

    @Deprecated("Use syncNuvioRepositoryUrls for Nuvio account snapshots")
    suspend fun importRepositoryUrls(manifestUrls: Iterable<String>) = syncNuvioRepositoryUrls(manifestUrls)

    suspend fun removeRepository(manifestUrl: String) {
        awaitRestored()
        manualRepositoryUrls = manualRepositoryUrls - manifestUrl
        nuvioRepositoryUrls = nuvioRepositoryUrls - manifestUrl
        clearCodeCache()
        runtime.dispatch(
            mapOf(
                "type" to "pluginRepositoryRemoveRequested",
                "manifestUrl" to manifestUrl,
            ),
        )
        persistCurrentState()
    }

    suspend fun refreshRepository(manifestUrl: String) {
        awaitRestored()
        clearCodeCache()
        dispatchAddRepository(manifestUrl)
        persistCurrentState()
    }

    suspend fun refreshAllRepositories() {
        awaitRestored()
        clearCodeCache()
        state.value.repositories.forEach { dispatchAddRepository(it.manifestUrl) }
        persistCurrentState()
    }

    suspend fun toggleScraper(scraperId: String, enabled: Boolean) {
        awaitRestored()
        runtime.dispatch(
            mapOf(
                "type" to "pluginScraperToggled",
                "scraperId" to scraperId,
                "enabled" to enabled,
            ),
        )
        persistCurrentState()
    }

    suspend fun updateScraperSettings(scraperId: String, settings: Map<String, Any?>) {
        awaitRestored()
        runtime.dispatch(
            mapOf(
                "type" to "pluginScraperSettingsUpdated",
                "scraperId" to scraperId,
                "settings" to settings,
            ),
        )
        persistCurrentState()
    }

    suspend fun executeEnabledScrapers(
        contentId: String,
        mediaType: String,
        season: Int?,
        episode: Int?,
    ): List<Stream> = supervisorScope {
        awaitRestored()
        val normalizedType = normalizeMediaType(mediaType)
        val currentScrapers = state.value.scrapers.filter { scraper ->
            scraper.enabled && (
                scraper.supportedTypes.isEmpty() ||
                    scraper.supportedTypes.any { normalizeMediaType(it) == normalizedType }
                )
        }
        if (currentScrapers.isEmpty()) return@supervisorScope emptyList()

        val plan = buildExecutionPlan(
            currentScrapers = currentScrapers,
            contentId = contentId,
            mediaType = mediaType,
            season = season,
            episode = episode,
        ) ?: return@supervisorScope emptyList()

        val byId = currentScrapers.associateBy { it.id }
        plan.scrapers
            .mapNotNull { it["id"] as? String }
            .distinct()
            .mapNotNull(byId::get)
            .map { scraper ->
                async(Dispatchers.IO) {
                    executeScraper(
                        scraper = scraper,
                        tmdbId = plan.contentId,
                        mediaType = plan.mediaType,
                        season = plan.season,
                        episode = plan.episode,
                    )
                }
            }
            .awaitAll()
            .flatten()
    }

    fun hasCompatibleEnabledScrapers(mediaType: String): Boolean {
        val normalizedType = normalizeMediaType(mediaType)
        return state.value.scrapers.any { scraper ->
            scraper.enabled && (
                scraper.supportedTypes.isEmpty() ||
                    scraper.supportedTypes.any { normalizeMediaType(it) == normalizedType }
                )
        }
    }

    suspend fun executeScraper(
        scraper: NuvioPluginScraperUiModel,
        tmdbId: String,
        mediaType: String,
        season: Int?,
        episode: Int?,
    ): List<Stream> = withContext(Dispatchers.IO) {
        awaitRestored()
        val code = fetchScraperCode(scraper) ?: return@withContext emptyList()
        try {
            val normalizedMediaType = normalizeMediaType(mediaType)
            val baseContentId = nuvioPluginContentId(tmdbId, season, episode)
            val resolvedContentId = pluginTmdbIdResolver(baseContentId, normalizedMediaType)
                ?.takeIf(String::isNotBlank)
                ?: baseContentId
            PlatformLog.d(
                logTag,
                "execute scraper=${scraper.name} type=$normalizedMediaType input=$baseContentId resolved=$resolvedContentId season=$season episode=$episode"
            )
            val rawJson = FluxaCoreUniFfi.executePluginScraper(
                pluginHttpClient,
                code,
                scraper.repositoryUrl,
                scraper.id,
                gson.toJson(scraper.settings),
                resolvedContentId,
                normalizedMediaType,
                season,
                episode,
            )

            // Match NuvioMobile's runtime-result contract first. Its parser accepts both
            // a direct string URL and { url: ... }, plus headers/subtitles/quality metadata.
            // Keep the native normalizer as a compatibility fallback for older Fluxa plugins.
            val nuvioStreams = parseNuvioCompatibleStreams(rawJson, scraper.name)
            val streams = if (nuvioStreams.isNotEmpty() || rawJson.trim() == "[]") {
                nuvioStreams
            } else {
                val normalized = FluxaCoreUniFfi.coreInvokeValue("pluginStreamResultsToStreams", rawJson)
                val fallback: List<Stream> = gson.fromJson(normalized, streamListType) ?: emptyList()
                fallback.map { it.copy(addonName = scraper.name) }
            }
            PlatformLog.d(logTag, "scraper ${scraper.name} returned ${streams.size} streams")
            streams
        } catch (error: Exception) {
            PlatformLog.w(logTag, "scraper ${scraper.id} failed", error)
            emptyList()
        }
    }

    suspend fun getSettingsLayout(
        scraper: NuvioPluginScraperUiModel,
    ): List<NuvioPluginSettingsFieldUiModel> = withContext(Dispatchers.IO) {
        awaitRestored()
        val code = fetchScraperCode(scraper) ?: return@withContext emptyList()
        try {
            val layoutJson = FluxaCoreUniFfi.getPluginScraperSettingsLayout(code, scraper.id)
            parseSettingsLayout(layoutJson)
        } catch (error: Exception) {
            PlatformLog.w(logTag, "settings layout for ${scraper.id} failed", error)
            emptyList()
        }
    }

    private suspend fun dispatchAddRepository(manifestUrl: String) {
        runtime.dispatch(
            mapOf(
                "type" to "pluginRepositoryAddRequested",
                "manifestUrl" to manifestUrl,
            ),
        )
    }

    private fun buildExecutionPlan(
        currentScrapers: List<NuvioPluginScraperUiModel>,
        contentId: String,
        mediaType: String,
        season: Int?,
        episode: Int?,
    ): PluginExecutionPlan? = runCatching {
        val normalizedScrapers = currentScrapers.map { scraper ->
            scraper.copy(supportedTypes = scraper.supportedTypes.map(::normalizeMediaType).distinct())
        }
        val payload = mapOf(
            "scrapers" to normalizedScrapers,
            "contentId" to contentId,
            "mediaType" to normalizeMediaType(mediaType),
            "season" to season,
            "episode" to episode,
        )
        val value = FluxaCoreUniFfi.coreInvokeValue("pluginExecutionPlan", gson.toJson(payload))
        gson.fromJson(value, PluginExecutionPlan::class.java)
    }.onFailure { error ->
        PlatformLog.w(logTag, "plugin execution plan failed", error)
    }.getOrNull()

    private suspend fun persistCurrentState() {
        val current = runtime.state.value.toPluginsUiState()
        val persisted = PersistedPluginsState(
            repositoryUrls = current.repositories.map { it.manifestUrl },
            manualRepositoryUrls = manualRepositoryUrls.toList(),
            nuvioRepositoryUrls = nuvioRepositoryUrls.toList(),
            scraperOverrides = current.scrapers.associate { scraper ->
                scraper.id to PersistedScraperOverride(
                    enabled = scraper.enabled,
                    settings = scraper.settings,
                )
            },
        )
        prefsStore.write(KEY_PERSISTED_STATE, gson.toJson(persisted))
    }

    private suspend fun loadPersistedState(): PersistedPluginsState {
        val json = prefsStore.read(KEY_PERSISTED_STATE) ?: return PersistedPluginsState()
        return try {
            gson.fromJson(json, PersistedPluginsState::class.java) ?: PersistedPluginsState()
        } catch (error: Exception) {
            PlatformLog.w(logTag, "failed to parse persisted plugin state", error)
            PersistedPluginsState()
        }
    }

    private fun parseSettingsLayout(layoutJson: String): List<NuvioPluginSettingsFieldUiModel> {
        val elements = try {
            JsonParser.parseString(layoutJson).asJsonArray
        } catch (_: Exception) {
            return emptyList()
        }
        return elements.mapNotNull { element ->
            val field = element.asJsonObject
            val type = field.get("type")?.asString ?: return@mapNotNull null
            val defaultValueElement = field.get("defaultValue")
            val defaultIsBoolean = defaultValueElement?.isJsonPrimitive == true &&
                defaultValueElement.asJsonPrimitive.isBoolean
            val options = field.get("options")
                ?.takeIf { it.isJsonArray }
                ?.asJsonArray
                ?.map { option ->
                    val optionObject = option.asJsonObject
                    NuvioPluginSettingsOptionUiModel(
                        label = optionObject.get("label")?.asString.orEmpty(),
                        value = optionObject.get("value")?.asString.orEmpty(),
                    )
                }
                .orEmpty()

            NuvioPluginSettingsFieldUiModel(
                key = field.get("key")?.asString.orEmpty(),
                type = type,
                label = field.get("label")?.asString.orEmpty(),
                description = field.get("description")?.asString,
                placeholder = field.get("placeholder")?.asString,
                isPassword = field.get("isPassword")?.asBoolean ?: false,
                defaultValue = if (defaultIsBoolean) {
                    null
                } else {
                    defaultValueElement?.takeIf { !it.isJsonNull }?.asString
                },
                defaultBoolean = if (defaultIsBoolean) defaultValueElement.asBoolean else false,
                options = options,
            )
        }
    }

    private suspend fun fetchScraperCode(scraper: NuvioPluginScraperUiModel): String? =
        codeCacheMutex.withLock {
            val url = resolveScraperUrl(scraper.repositoryUrl, scraper.filename) ?: return@withLock null
            codeCache[url]?.let { return@withLock it }
            try {
                val request = Request.Builder().url(url).build()
                scraperCodeClient.newCall(request).execute().use { response ->
                    if (!response.isSuccessful) return@withLock null
                    val code = response.body.string()
                    if (code.isBlank()) return@withLock null
                    codeCache[url] = code
                    code
                }
            } catch (error: Exception) {
                PlatformLog.w(logTag, "failed to fetch scraper code for ${scraper.id}", error)
                null
            }
        }

    private suspend fun clearCodeCache() {
        codeCacheMutex.withLock { codeCache.clear() }
    }

    private fun resolveScraperUrl(repositoryUrl: String, filename: String): String? {
        if (filename.startsWith("http://") || filename.startsWith("https://")) return filename
        val manifestUrl = repositoryUrl.toHttpUrlOrNull() ?: return null
        return manifestUrl.resolve(filename)?.toString()
    }

    private fun normalizeMediaType(mediaType: String): String = normalizeNuvioPluginType(mediaType)

    private fun parseNuvioCompatibleStreams(rawJson: String, scraperName: String): List<Stream> {
        val root = runCatching { JsonParser.parseString(rawJson) }.getOrNull() ?: return emptyList()
        val array = when {
            root.isJsonArray -> root.asJsonArray
            root.isJsonPrimitive && root.asJsonPrimitive.isString ->
                runCatching { JsonParser.parseString(root.asString).asJsonArray }.getOrNull()
            else -> null
        } ?: return emptyList()

        return array.mapNotNull stream@{ element ->
            val item = element.takeIf { it.isJsonObject }?.asJsonObject ?: return@stream null
            val urlElement = item.get("url")
            val url = when {
                urlElement == null || urlElement.isJsonNull -> null
                urlElement.isJsonPrimitive -> runCatching { urlElement.asString }.getOrNull()?.takeIf(String::isNotBlank)
                urlElement.isJsonObject -> urlElement.asJsonObject.get("url")
                    ?.takeIf { it.isJsonPrimitive }
                    ?.let { runCatching { it.asString }.getOrNull() }
                    ?.takeIf(String::isNotBlank)
                else -> null
            } ?: return@stream null

            fun string(key: String): String? = item.get(key)
                ?.takeIf { it.isJsonPrimitive }
                ?.let { runCatching { it.asString }.getOrNull() }
                ?.trim()
                ?.takeIf { it.isNotBlank() && !it.contains("[object") }

            fun int(key: String): Int? = item.get(key)
                ?.takeIf { it.isJsonPrimitive }
                ?.let { primitive ->
                    runCatching { primitive.asInt }.getOrNull()
                        ?: runCatching { primitive.asString.toIntOrNull() }.getOrNull()
                }

            val headersObject = item.get("headers")?.takeIf { it.isJsonObject }?.asJsonObject
            val headers = headersObject
                ?.entrySet()
                ?.mapNotNull header@{ (key, value) ->
                    val headerName = key.trim()
                    val headerValue = value
                        .takeIf { it.isJsonPrimitive }
                        ?.let { runCatching { it.asString }.getOrNull() }
                        ?.trim()
                        .orEmpty()
                    if (
                        headerName.isBlank() ||
                        headerValue.isBlank() ||
                        headerName.equals("Range", ignoreCase = true)
                    ) return@header null
                    headerName to headerValue
                }
                ?.toMap()
                ?.takeIf { it.isNotEmpty() }

            val subtitleArray = item.get("subtitles")?.takeIf { it.isJsonArray }?.asJsonArray
            val subtitles = subtitleArray
                ?.mapNotNull subtitle@{ subtitleElement ->
                    val subtitle = subtitleElement.takeIf { it.isJsonObject }?.asJsonObject ?: return@subtitle null
                    val subtitleUrl = subtitle.get("url")
                        ?.takeIf { it.isJsonPrimitive }
                        ?.let { runCatching { it.asString }.getOrNull() }
                        ?.trim()
                        ?.takeIf(String::isNotBlank)
                        ?: return@subtitle null
                    val language = subtitle.get("language")
                        ?.takeIf { it.isJsonPrimitive }
                        ?.let { runCatching { it.asString }.getOrNull() }
                        ?.trim()
                        ?.takeIf(String::isNotBlank)
                        ?: "Unknown"
                    val subtitleName = subtitle.get("name")
                        ?.takeIf { it.isJsonPrimitive }
                        ?.let { runCatching { it.asString }.getOrNull() }
                        ?.trim()
                        ?.takeIf(String::isNotBlank)
                    SubtitleData(
                        id = subtitleName,
                        url = subtitleUrl,
                        lang = language,
                        attributes = SubtitleAttributes(
                            url = subtitleUrl,
                            languages = listOf(language),
                        ),
                    )
                }
                ?.takeIf { it.isNotEmpty() }

            val title = string("title") ?: string("name") ?: "Unknown"
            val name = string("name") ?: title
            val quality = string("quality")
            val size = string("size")
            val language = string("language")
            val type = string("type")
            val description = listOfNotNull(quality, size, language, type)
                .distinct()
                .joinToString(" • ")
                .ifBlank { null }
            Stream(
                name = name,
                title = title,
                description = description,
                url = url,
                infoHash = string("infoHash"),
                subtitles = subtitles,
                headers = headers,
                quality = quality,
                size = size,
                provider = string("provider"),
                seeders = int("seeders"),
                peers = int("peers"),
                addonName = scraperName,
            )
        }.filter { !it.url.isNullOrBlank() }
    }

    @Suppress("UNCHECKED_CAST")
    private fun Map<String, Any?>.toPluginsUiState(): NuvioPluginsUiState {
        val plugins = this["plugins"] as? Map<String, Any?> ?: return NuvioPluginsUiState()
        val repositories = (plugins["repositories"] as? List<Map<String, Any?>>).orEmpty().map { repo ->
            NuvioPluginRepositoryUiModel(
                manifestUrl = repo["manifestUrl"] as? String ?: "",
                name = repo["name"] as? String ?: "",
                description = repo["description"] as? String,
                scraperCount = (repo["scraperCount"] as? Number)?.toInt() ?: 0,
            )
        }
        val scrapers = (plugins["scrapers"] as? List<Map<String, Any?>>).orEmpty().map { scraper ->
            NuvioPluginScraperUiModel(
                id = scraper["id"] as? String ?: "",
                name = scraper["name"] as? String ?: "",
                repositoryUrl = scraper["repositoryUrl"] as? String ?: "",
                filename = scraper["filename"] as? String ?: "",
                enabled = scraper["enabled"] as? Boolean ?: true,
                supportedTypes = (scraper["supportedTypes"] as? List<String>).orEmpty(),
                hasSettings = scraper["hasSettings"] as? Boolean ?: false,
                settings = (scraper["settings"] as? Map<String, Any?>).orEmpty(),
            )
        }
        val error = when (val rawError = plugins["error"]) {
            is String -> rawError
            is Map<*, *> -> rawError["code"] as? String
            else -> null
        }
        return NuvioPluginsUiState(
            repositories = repositories,
            scrapers = scrapers,
            addingRepositoryUrl = plugins["addingRepositoryUrl"] as? String,
            error = error,
        )
    }
}
