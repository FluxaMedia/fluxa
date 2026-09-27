package com.fluxa.app.plugins

import android.content.Context
import android.util.Log
import com.fluxa.app.BuildConfig
import com.lagradost.cloudstream3.MainAPI
import com.lagradost.cloudstream3.AcraApplication
import com.fluxa.app.plugins.cloudstream.ExternalExtensionLoader
import com.fluxa.app.plugins.cloudstream.ExternalRepoParser
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import com.fluxa.app.plugins.cloudstream.PluginInfo
import com.fluxa.app.plugins.cloudstream.PluginRepositoryEntry
import com.fluxa.app.plugins.cloudstream.RepositoryManifest
import com.fluxa.app.plugins.cloudstream.RepositoryResult
import com.fluxa.app.domain.background.BackgroundTaskScheduler
import com.fluxa.app.plugins.cloudstream.SIGNATURE_ERROR
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

private const val TAG = "PluginManager"

private inline fun logDebug(message: () -> String) {
    if (BuildConfig.DEBUG) Log.d(TAG, message())
}

@Singleton
class PluginManager @Inject constructor(
    @param:ApplicationContext private val context: Context,
    private val backgroundTaskScheduler: BackgroundTaskScheduler,
) {

    private val loader = ExternalExtensionLoader(context)
    private val stateStore = CloudStreamPluginStateStore(context)
    private val runtime = CloudStreamPluginRuntime(loader)
    private val repoParser = ExternalRepoParser()
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    private val mutex = Mutex()

    // State flows for reactive UI updates
    private val _installedPlugins = MutableStateFlow<List<InstalledPlugin>>(emptyList())
    val installedPlugins: StateFlow<List<InstalledPlugin>> = _installedPlugins

    private val _repositories = MutableStateFlow<List<PluginRepositoryEntry>>(emptyList())
    val repositories: StateFlow<List<PluginRepositoryEntry>> = _repositories

    private val _availablePlugins = MutableStateFlow<List<PluginInfo>>(emptyList())
    val availablePlugins: StateFlow<List<PluginInfo>> = _availablePlugins

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading

    private val _automaticUpdatesEnabled = MutableStateFlow(false)
    val automaticUpdatesEnabled: StateFlow<Boolean> = _automaticUpdatesEnabled

    val loadedApis: StateFlow<List<MainAPI>> = runtime.loadedApis

    init {
        // Initialize AcraApplication context for plugins
        AcraApplication.init(context.applicationContext as android.app.Application)

        // Load saved state
        scope.launch {
            loadSavedState()
        }
    }

    // ==================== State Persistence ====================

    private suspend fun loadSavedState() {
        val stored = stateStore.load()
        val trustedRepositories = stored.repositories.filter { it.publisherPublicKey.isNotBlank() }
        val trustedRepositoryUrls = trustedRepositories.map { it.url }.toSet()
        _installedPlugins.value = stored.installedPlugins.filter { plugin ->
            plugin.repositoryUrl == null || plugin.repositoryUrl in trustedRepositoryUrls
        }
        _repositories.value = trustedRepositories
        _automaticUpdatesEnabled.value = stored.automaticUpdatesEnabled
        if (trustedRepositories.size != stored.repositories.size) saveState()

        if (_automaticUpdatesEnabled.value) backgroundTaskScheduler.schedulePluginAutoUpdate()
        runtime.loadAll(_installedPlugins.value)
    }

    private fun saveState() = stateStore.save(
        installedPlugins = _installedPlugins.value,
        repositories = _repositories.value,
    )

    fun setAutomaticUpdatesEnabled(enabled: Boolean) {
        stateStore.setAutomaticUpdatesEnabled(enabled)
        _automaticUpdatesEnabled.value = enabled
        if (enabled) {
            backgroundTaskScheduler.schedulePluginAutoUpdate()
        } else {
            backgroundTaskScheduler.cancelPluginAutoUpdate()
        }
    }

    // ==================== Repository Management ====================

    /**
     * Add a new repository URL
     */
    suspend fun addRepository(url: String, publisherPublicKey: String): Result<RepositoryManifest> = mutex.withLock {
        _isLoading.value = true

        return try {
            val trustedUrl = normalizeCloudStreamRepositoryUrl(expandCloudStreamRepositoryShortcode(url))
            if (!isSecureCloudStreamPluginUrl(trustedUrl)) {
                return Result.failure(Exception("Repository URL must use HTTPS"))
            }
            if (publisherPublicKey.isBlank()) {
                return Result.failure(Exception(SIGNATURE_ERROR))
            }
            // Check if already exists
            if (_repositories.value.any { sameCloudStreamRepositoryUrl(it.url, trustedUrl) }) {
                return Result.failure(Exception("Repository already exists"))
            }

            // Fetch and parse
            when (val result = repoParser.fetchRepository(trustedUrl, publisherPublicKey)) {
                is RepositoryResult.Success -> {
                    val entry = PluginRepositoryEntry(
                        url = trustedUrl,
                        publisherPublicKey = publisherPublicKey,
                        name = result.manifest.name,
                        description = result.manifest.description,
                        iconUrl = result.manifest.iconUrl
                    )

                    _repositories.value = _repositories.value + entry
                    saveState()

                    // Refresh available plugins
                    refreshAvailablePlugins()

                    Result.success(result.manifest)
                }
                is RepositoryResult.Error -> {
                    Result.failure(Exception(result.message))
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error adding repository", e)
            Result.failure(e)
        } finally {
            _isLoading.value = false
        }
    }

    /**
     * Remove a repository
     */
    suspend fun removeRepository(url: String) = mutex.withLock {
        val pluginsFromRepo = _installedPlugins.value.filter { it.repositoryUrl != null && sameCloudStreamRepositoryUrl(it.repositoryUrl, url) }
        pluginsFromRepo.forEach { plugin ->
            runtime.unload(plugin)
            loader.deleteExtensionFile(plugin.cloudStreamInstallKey())
        }
        _installedPlugins.value = _installedPlugins.value.filter { it.repositoryUrl == null || !sameCloudStreamRepositoryUrl(it.repositoryUrl, url) }
        _repositories.value = _repositories.value.filter { !sameCloudStreamRepositoryUrl(it.url, url) }
        saveState()
        refreshAvailablePlugins()
    }

    /**
     * Refresh available plugins from all repositories
     */
    suspend fun refreshAvailablePlugins() {
        _isLoading.value = true

        val allPlugins = mutableListOf<Pair<PluginRepositoryEntry, PluginInfo>>()

        _repositories.value.forEach { repoEntry ->
            try {
                when (val result = repoParser.fetchRepository(repoEntry.url, repoEntry.publisherPublicKey)) {
                    is RepositoryResult.Success -> {
                        allPlugins.addAll(result.manifest.plugins.map { repoEntry to it })
                    }
                    is RepositoryResult.Error -> {
                        Log.w(TAG, "Failed to fetch ${repoEntry.url}: ${result.message}")
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error fetching ${repoEntry.url}", e)
            }
        }

        val installedKeys = _installedPlugins.value.map { it.cloudStreamInstallKey() }.toSet()
        _availablePlugins.value = allPlugins
            .filter { (repo, plugin) -> cloudStreamPluginInstallId(repo.url, plugin.internalName) !in installedKeys }
            .map { it.second }

        _isLoading.value = false
    }

    /**
     * Get plugins from a specific repository URL (for dialog display)
     */
    suspend fun getPluginsFromRepository(repoUrl: String): List<PluginInfo> {
        return try {
            val repository = _repositories.value.firstOrNull { sameCloudStreamRepositoryUrl(it.url, repoUrl) }
                ?: return emptyList()
            when (val result = repoParser.fetchRepository(repository.url, repository.publisherPublicKey)) {
                is RepositoryResult.Success -> result.manifest.plugins
                is RepositoryResult.Error -> {
                    Log.w(TAG, "Failed to fetch repo $repoUrl: ${result.message}")
                    emptyList()
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error fetching repo $repoUrl", e)
            emptyList()
        }
    }

    fun isPluginInstalled(internalName: String): Boolean {
        return _installedPlugins.value.any { it.internalName == internalName }
    }

    fun pluginInstallKey(repositoryUrl: String?, internalName: String): String =
        cloudStreamPluginInstallId(repositoryUrl, internalName)

    fun isPluginInstalled(repositoryUrl: String?, internalName: String): Boolean {
        val installId = cloudStreamPluginInstallId(repositoryUrl, internalName)
        return _installedPlugins.value.any { it.cloudStreamInstallKey() == installId }
    }

    // ==================== Plugin Installation ====================

    /**
     * Install a plugin from a PluginInfo
     */
    suspend fun installPlugin(pluginInfo: PluginInfo, repoUrl: String?): Result<InstalledPlugin> = mutex.withLock {
        _isLoading.value = true

        return try {
            if (repoUrl.isNullOrBlank() || !_repositories.value.any { sameCloudStreamRepositoryUrl(it.url, repoUrl) }) {
                return Result.failure(Exception("Plugin must come from a trusted repository"))
            }
            val trustedPlugin = verifiedRepositoryPlugin(repoUrl, pluginInfo)
                ?: return Result.failure(Exception("Plugin is not present in the signed repository manifest"))
            if (!isSecureCloudStreamPluginUrl(trustedPlugin.url)) {
                return Result.failure(Exception("Plugin download URL must use HTTPS"))
            }
            // Check if already installed
            val installId = cloudStreamPluginInstallId(repoUrl, pluginInfo.internalName)
            if (_installedPlugins.value.any { it.cloudStreamInstallKey() == installId }) {
                return Result.failure(Exception("Plugin already installed"))
            }

            logDebug { "Installing plugin: ${trustedPlugin.internalName} from ${trustedPlugin.url}" }

            // Download the .cs3 file
            val file = loader.downloadExtension(
                scraperId = installId,
                downloadUrl = trustedPlugin.url
            ) ?: run {
                Log.e(TAG, "Failed to download plugin ${trustedPlugin.internalName}")
                return Result.failure(Exception("Failed to download plugin"))
            }
            logDebug { "Downloaded plugin: ${file.absolutePath}" }

            verifyCloudStreamPluginChecksum(file, trustedPlugin.sha256)?.let { error ->
                loader.deleteExtensionFile(installId)
                return Result.failure(Exception(error))
            }
            if (!loader.validateExtension(file)) {
                loader.deleteExtensionFile(installId)
                return Result.failure(Exception("Plugin validation failed"))
            }

            // Create installed plugin record
            val installed = InstalledPlugin(
                installId = installId,
                internalName = trustedPlugin.internalName,
                name = trustedPlugin.name,
                description = trustedPlugin.description,
                version = trustedPlugin.version,
                url = trustedPlugin.url,
                filePath = file.absolutePath,
                repositoryUrl = repoUrl,
                sha256 = trustedPlugin.sha256?.lowercase(),
                iconUrl = trustedPlugin.iconUrl
            )

            logDebug { "Loading plugin: ${trustedPlugin.internalName}" }
            if (!runtime.load(installed)) {
                loader.unloadExtension(installId)
                loader.deleteExtensionFile(installId)
                return Result.failure(Exception("Plugin could not be loaded"))
            }
            logDebug { "Plugin loaded: ${trustedPlugin.internalName}" }

            _installedPlugins.value = _installedPlugins.value + installed
            saveState()
            logDebug { "Saved plugin to installed list: ${trustedPlugin.internalName}" }

            // Refresh available plugins (remove from available)
            _availablePlugins.value = _availablePlugins.value.filter {
                cloudStreamPluginInstallId(repoUrl, it.internalName) != installId
            }

            Result.success(installed)
        } catch (t: Throwable) {
            Log.e(TAG, "Error installing plugin ${pluginInfo.internalName}", t)
            Result.failure(if (t is Exception) t else Exception(t.message, t))
        } finally {
            _isLoading.value = false
        }
    }

    /**
     * Install a plugin from a direct URL (not from repository)
     */
    suspend fun installPluginFromUrl(url: String, name: String): Result<InstalledPlugin> = mutex.withLock {
        _isLoading.value = true

        return try {
            if (!BuildConfig.DEBUG) {
                return Result.failure(Exception("Direct plugin URL installation is disabled"))
            }
            if (!isSecureCloudStreamPluginUrl(url)) {
                return Result.failure(Exception("Plugin download URL must use HTTPS"))
            }
            // Generate internal name from URL
            val internalName = PluginStateCodec.sha256(url).take(16)
            val installId = cloudStreamPluginInstallId(null, internalName)

            // Check if already installed
            if (_installedPlugins.value.any { it.cloudStreamInstallKey() == installId }) {
                return Result.failure(Exception("Plugin already installed"))
            }

            // Download
            val file = loader.downloadExtension(installId, url)
                ?: return Result.failure(Exception("Failed to download plugin"))
            if (!loader.validateExtension(file)) {
                loader.deleteExtensionFile(installId)
                return Result.failure(Exception("Plugin validation failed"))
            }

            val installed = InstalledPlugin(
                installId = installId,
                internalName = internalName,
                name = name,
                description = "Manual installation",
                version = 1,
                url = url,
                filePath = file.absolutePath,
                repositoryUrl = null,
                sha256 = sha256(file)
            )

            if (!runtime.load(installed)) {
                loader.unloadExtension(installId)
                loader.deleteExtensionFile(installId)
                return Result.failure(Exception("Plugin could not be loaded"))
            }

            _installedPlugins.value = _installedPlugins.value + installed
            saveState()

            Result.success(installed)
        } catch (e: Exception) {
            Log.e(TAG, "Error installing plugin from URL", e)
            Result.failure(e)
        } finally {
            _isLoading.value = false
        }
    }

    /**
     * Uninstall a plugin
     */
    suspend fun uninstallPlugin(internalName: String) = mutex.withLock {
        val plugin = _installedPlugins.value.singleOrNull { it.internalName == internalName }
            ?: return@withLock
        uninstallPluginLocked(plugin)
    }

    suspend fun uninstallPlugin(repositoryUrl: String?, internalName: String) = mutex.withLock {
        val installId = cloudStreamPluginInstallId(repositoryUrl, internalName)
        val plugin = _installedPlugins.value.find { it.cloudStreamInstallKey() == installId }
            ?: return@withLock
        uninstallPluginLocked(plugin)
    }

    private suspend fun uninstallPluginLocked(plugin: InstalledPlugin) {
        runtime.unload(plugin)

        // Delete file
        loader.deleteExtensionFile(plugin.cloudStreamInstallKey())

        // Remove from list
        _installedPlugins.value = _installedPlugins.value.filter { it.cloudStreamInstallKey() != plugin.cloudStreamInstallKey() }
        saveState()

        // Refresh available plugins
        refreshAvailablePlugins()
    }

    /**
     * Update a plugin to a newer version - thread-safe public API
     */
    suspend fun updatePlugin(
        internalName: String,
        newPluginInfo: PluginInfo
    ): Result<InstalledPlugin> = mutex.withLock {
        val oldPlugin = _installedPlugins.value.singleOrNull { it.internalName == internalName }
            ?: return@withLock Result.failure(Exception("Plugin is not installed"))
        updatePluginInternal(oldPlugin, newPluginInfo)
    }

    /**
     * Internal update logic without lock - for use when already holding mutex
     */
    private suspend fun updatePluginInternal(
        oldPlugin: InstalledPlugin,
        newPluginInfo: PluginInfo
    ): Result<InstalledPlugin> {
        _isLoading.value = true

        return try {
            val internalName = oldPlugin.internalName
            val installId = oldPlugin.cloudStreamInstallKey()
            if (oldPlugin.repositoryUrl.isNullOrBlank() || _repositories.value.none { sameCloudStreamRepositoryUrl(it.url, oldPlugin.repositoryUrl) }) {
                return Result.failure(Exception("Plugin update repository is not trusted"))
            }
            val trustedPlugin = verifiedRepositoryPlugin(oldPlugin.repositoryUrl, newPluginInfo)
                ?: return Result.failure(Exception("Plugin is not present in the signed repository manifest"))
            if (trustedPlugin.internalName != internalName) {
                return Result.failure(Exception("Plugin update identity does not match"))
            }
            if (!isSecureCloudStreamPluginUrl(trustedPlugin.url)) {
                return Result.failure(Exception("Plugin download URL must use HTTPS"))
            }
            val temporaryFile = loader.downloadExtensionToTemporaryFile(installId, trustedPlugin.url)
                ?: return Result.failure(Exception("Failed to download update"))

            verifyCloudStreamPluginChecksum(temporaryFile, trustedPlugin.sha256)?.let { error ->
                loader.discardTemporaryExtension(temporaryFile)
                return Result.failure(Exception(error))
            }

            if (!loader.validateExtension(temporaryFile)) {
                loader.discardTemporaryExtension(temporaryFile)
                return Result.failure(Exception("Plugin validation failed"))
            }

            runtime.unload(oldPlugin)
            val replacement = loader.promoteTemporaryExtension(installId, temporaryFile)
                ?: run {
                    runtime.load(oldPlugin)
                    return Result.failure(Exception("Failed to install update"))
                }

            val updated = InstalledPlugin(
                installId = installId,
                internalName = internalName,
                name = trustedPlugin.name,
                description = trustedPlugin.description,
                version = trustedPlugin.version,
                url = trustedPlugin.url,
                filePath = replacement.target.absolutePath,
                repositoryUrl = oldPlugin.repositoryUrl,
                sha256 = trustedPlugin.sha256?.lowercase(),
                iconUrl = trustedPlugin.iconUrl
            )

            if (!runtime.load(updated)) {
                loader.restoreExtension(replacement)
                runtime.load(oldPlugin)
                return Result.failure(Exception("Plugin update could not be loaded"))
            }

            _installedPlugins.value = _installedPlugins.value.map {
                if (it.cloudStreamInstallKey() == installId) updated else it
            }
            saveState()
            loader.finalizeExtensionReplacement(replacement)

            Result.success(updated)
        } catch (t: Throwable) {
            Log.e(TAG, "Error updating plugin ${oldPlugin.internalName}", t)
            Result.failure(if (t is Exception) t else Exception(t.message, t))
        } finally {
            _isLoading.value = false
        }
    }

    suspend fun reloadAllPlugins() {
        runtime.reloadAll(_installedPlugins.value)
    }

    fun close() {
        scope.cancel()
        runtime.close()
    }

    private suspend fun verifiedRepositoryPlugin(repoUrl: String, requested: PluginInfo): PluginInfo? {
        val repository = _repositories.value.firstOrNull { sameCloudStreamRepositoryUrl(it.url, repoUrl) } ?: return null
        val result = repoParser.fetchRepository(repository.url, repository.publisherPublicKey)
        val manifest = (result as? RepositoryResult.Success)?.manifest ?: return null
        return manifest.plugins.firstOrNull { candidate ->
            candidate.internalName == requested.internalName &&
                candidate.version == requested.version &&
                candidate.url == requested.url &&
                candidate.sha256.equals(requested.sha256, ignoreCase = true)
        }
    }

    // ==================== Auto Update Functions ====================

    /**
     * Check all installed plugins for updates from their repositories.
     * Auto-updates plugins in background if newer version found.
     * Call this periodically (e.g., on app start or once per day).
     */
    internal suspend fun checkAndAutoUpdatePlugins(): PluginAutoUpdater.UpdateReport = mutex.withLock {
        PluginAutoUpdater(
            repositories = { _repositories.value },
            installedPlugins = { _installedPlugins.value },
            updatePlugin = ::updatePluginInternal
        ).checkAndAutoUpdate()
    }
}
