package com.fluxa.app.plugins

import android.content.Context
import android.util.Log
import com.fluxa.app.BuildConfig
import com.fluxa.app.compat.cloudstream3.APIHolder
import com.fluxa.app.core.rust.FluxaCoreNative
import com.lagradost.cloudstream3.MainAPI
import com.lagradost.cloudstream3.AcraApplication
import com.fluxa.app.plugins.cloudstream.ExternalExtensionLoader
import com.fluxa.app.plugins.cloudstream.ExternalRepoParser
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import com.fluxa.app.plugins.cloudstream.PluginInfo
import com.fluxa.app.plugins.cloudstream.PluginRepositoryEntry
import com.fluxa.app.plugins.cloudstream.RepositoryManifest
import com.fluxa.app.plugins.cloudstream.RepositoryResult
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.cancel
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import dagger.hilt.android.qualifiers.ApplicationContext
import java.io.File
import java.security.MessageDigest
import javax.inject.Inject
import javax.inject.Singleton

private const val TAG = "PluginManager"
private const val PREFS_NAME = "fluxa_plugin_manager"
private const val KEY_INSTALLED_PLUGINS = "installed_plugins"
private const val KEY_REPOSITORIES = "repositories"

private inline fun logDebug(message: () -> String) {
    if (BuildConfig.DEBUG) Log.d(TAG, message())
}

@Singleton
class PluginManager @Inject constructor(@param:ApplicationContext private val context: Context) {
    
    private val loader = ExternalExtensionLoader(context)
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
    
    private val _loadedApis = MutableStateFlow<List<MainAPI>>(emptyList())
    val loadedApis: StateFlow<List<MainAPI>> = _loadedApis
    
    init {
        // Initialize AcraApplication context for plugins
        AcraApplication.init(context.applicationContext as android.app.Application)
        
        // Load saved state
        loadSavedState()
    }
    
    // ==================== State Persistence ====================
    
    private fun loadSavedState() {
        val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        
        // Load installed plugins
        val pluginsJson = prefs.getString(KEY_INSTALLED_PLUGINS, "[]") ?: "[]"
        _installedPlugins.value = PluginStateCodec.parseInstalledPlugins(pluginsJson)
        
        // Load repositories
        val reposJson = prefs.getString(KEY_REPOSITORIES, "[]") ?: "[]"
        _repositories.value = PluginStateCodec.parseRepositories(reposJson)
        
        scope.launch {
            loadAllInstalledPlugins()
        }
    }
    
    private fun saveState() {
        val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        
        prefs.edit().apply {
            putString(KEY_INSTALLED_PLUGINS, PluginStateCodec.installedPluginsToJson(_installedPlugins.value))
            putString(KEY_REPOSITORIES, PluginStateCodec.repositoriesToJson(_repositories.value))
            apply()
        }
    }
    
    // ==================== Repository Management ====================
    
    /**
     * Add a new repository URL
     */
    suspend fun addRepository(url: String): Result<RepositoryManifest> = mutex.withLock {
        _isLoading.value = true
        
        return try {
            val trustedUrl = normalizeRepositoryUrl(expandShortcode(url))
            if (!isSecureRemoteUrl(trustedUrl)) {
                return Result.failure(Exception("Repository URL must use HTTPS"))
            }
            // Check if already exists
            if (_repositories.value.any { sameUrl(it.url, trustedUrl) }) {
                return Result.failure(Exception("Repository already exists"))
            }
            
            // Fetch and parse
            when (val result = repoParser.fetchRepository(trustedUrl)) {
                is RepositoryResult.Success -> {
                    val entry = PluginRepositoryEntry(
                        url = trustedUrl,
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
        val pluginsFromRepo = _installedPlugins.value.filter { it.repositoryUrl != null && sameUrl(it.repositoryUrl, url) }
        pluginsFromRepo.forEach { plugin ->
            unloadPlugin(plugin.internalName)
            loader.deleteExtensionFile(plugin.internalName)
        }
        _installedPlugins.value = _installedPlugins.value.filter { it.repositoryUrl == null || !sameUrl(it.repositoryUrl, url) }
        _repositories.value = _repositories.value.filter { !sameUrl(it.url, url) }
        saveState()
        refreshAvailablePlugins()
    }
    
    /**
     * Refresh available plugins from all repositories
     */
    suspend fun refreshAvailablePlugins() {
        _isLoading.value = true
        
        val allPlugins = mutableListOf<PluginInfo>()
        
        _repositories.value.forEach { repoEntry ->
            try {
                when (val result = repoParser.fetchRepository(repoEntry.url)) {
                    is RepositoryResult.Success -> {
                        allPlugins.addAll(result.manifest.plugins)
                    }
                    is RepositoryResult.Error -> {
                        Log.w(TAG, "Failed to fetch ${repoEntry.url}: ${result.message}")
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error fetching ${repoEntry.url}", e)
            }
        }
        
        // Filter out installed plugins
        val installedNames = _installedPlugins.value.map { it.internalName }.toSet()
        _availablePlugins.value = allPlugins.filter { it.internalName !in installedNames }
        
        _isLoading.value = false
    }
    
    /**
     * Get plugins from a specific repository URL (for dialog display)
     */
    suspend fun getPluginsFromRepository(repoUrl: String): List<PluginInfo> {
        return try {
            when (val result = repoParser.fetchRepository(repoUrl)) {
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
    
    // ==================== Plugin Installation ====================
    
    /**
     * Install a plugin from a PluginInfo
     */
    suspend fun installPlugin(pluginInfo: PluginInfo, repoUrl: String?): Result<InstalledPlugin> = mutex.withLock {
        _isLoading.value = true
        
        return try {
            if (repoUrl.isNullOrBlank() || !_repositories.value.any { sameUrl(it.url, repoUrl) }) {
                return Result.failure(Exception("Plugin must come from a trusted repository"))
            }
            if (!isSecureRemoteUrl(pluginInfo.url)) {
                return Result.failure(Exception("Plugin download URL must use HTTPS"))
            }
            // Check if already installed
            if (_installedPlugins.value.any { it.internalName == pluginInfo.internalName }) {
                return Result.failure(Exception("Plugin already installed"))
            }
            
            logDebug { "Installing plugin: ${pluginInfo.internalName} from ${pluginInfo.url}" }
            
            // Download the .cs3 file
            val file = loader.downloadExtension(
                scraperId = pluginInfo.internalName,
                downloadUrl = pluginInfo.url
            ) ?: run {
                Log.e(TAG, "Failed to download plugin ${pluginInfo.internalName}")
                return Result.failure(Exception("Failed to download plugin"))
            }
            logDebug { "Downloaded plugin: ${file.absolutePath}" }

            verifyPluginChecksum(file, pluginInfo.sha256)?.let { error ->
                file.delete()
                return Result.failure(Exception(error))
            }
            
            // Create installed plugin record
            val installed = InstalledPlugin(
                internalName = pluginInfo.internalName,
                name = pluginInfo.name,
                description = pluginInfo.description,
                version = pluginInfo.version,
                url = pluginInfo.url,
                filePath = file.absolutePath,
                repositoryUrl = repoUrl,
                sha256 = pluginInfo.sha256?.lowercase(),
                iconUrl = pluginInfo.iconUrl
            )
            
            // Add to installed list
            _installedPlugins.value = _installedPlugins.value + installed
            saveState()
            logDebug { "Saved plugin to installed list: ${pluginInfo.internalName}" }
            
            // Load the plugin
            logDebug { "Loading plugin: ${pluginInfo.internalName}" }
            loadPlugin(installed)
            logDebug { "Plugin loaded: ${pluginInfo.internalName}" }
            
            // Refresh available plugins (remove from available)
            _availablePlugins.value = _availablePlugins.value.filter { 
                it.internalName != pluginInfo.internalName 
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
            if (!isSecureRemoteUrl(url)) {
                return Result.failure(Exception("Plugin download URL must use HTTPS"))
            }
            // Generate internal name from URL
            val internalName = PluginStateCodec.sha256(url).take(16)
            
            // Check if already installed
            if (_installedPlugins.value.any { it.internalName == internalName }) {
                return Result.failure(Exception("Plugin already installed"))
            }
            
            // Download
            val file = loader.downloadExtension(internalName, url)
                ?: return Result.failure(Exception("Failed to download plugin"))
            
            val installed = InstalledPlugin(
                internalName = internalName,
                name = name,
                description = "Manual installation",
                version = 1,
                url = url,
                filePath = file.absolutePath,
                repositoryUrl = null,
                sha256 = sha256(file)
            )
            
            _installedPlugins.value = _installedPlugins.value + installed
            saveState()
            
            loadPlugin(installed)
            
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
        val plugin = _installedPlugins.value.find { it.internalName == internalName }
            ?: return@withLock
        
        // Unload from memory
        unloadPlugin(internalName)
        
        // Delete file
        loader.deleteExtensionFile(internalName)
        
        // Remove from list
        _installedPlugins.value = _installedPlugins.value.filter { it.internalName != internalName }
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
        updatePluginInternal(internalName, newPluginInfo)
    }
    
    /**
     * Internal update logic without lock - for use when already holding mutex
     */
    private suspend fun updatePluginInternal(
        internalName: String,
        newPluginInfo: PluginInfo
    ): Result<InstalledPlugin> {
        _isLoading.value = true
        
        return try {
            val oldPlugin = _installedPlugins.value.find { it.internalName == internalName }
                ?: return Result.failure(Exception("Plugin is not installed"))
            if (oldPlugin.repositoryUrl.isNullOrBlank() || _repositories.value.none { sameUrl(it.url, oldPlugin.repositoryUrl) }) {
                return Result.failure(Exception("Plugin update repository is not trusted"))
            }
            if (!isSecureRemoteUrl(newPluginInfo.url)) {
                return Result.failure(Exception("Plugin download URL must use HTTPS"))
            }
            // Unload current version
            unloadPlugin(internalName)
            
            // Delete old file
            loader.deleteExtensionFile(internalName)
            
            // Download new version
            val file = loader.downloadExtension(internalName, newPluginInfo.url)
                ?: return Result.failure(Exception("Failed to download update"))

            verifyPluginChecksum(file, newPluginInfo.sha256)?.let { error ->
                file.delete()
                return Result.failure(Exception(error))
            }
            
            // Update record
            val updated = InstalledPlugin(
                internalName = internalName,
                name = newPluginInfo.name,
                description = newPluginInfo.description,
                version = newPluginInfo.version,
                url = newPluginInfo.url,
                filePath = file.absolutePath,
                repositoryUrl = oldPlugin.repositoryUrl,
                sha256 = newPluginInfo.sha256?.lowercase()
            )
            
            _installedPlugins.value = _installedPlugins.value.map {
                if (it.internalName == internalName) updated else it
            }
            saveState()
            
            // Load new version
            loadPlugin(updated)
            
            Result.success(updated)
        } catch (t: Throwable) {
            Log.e(TAG, "Error updating plugin $internalName", t)
            Result.failure(if (t is Exception) t else Exception(t.message, t))
        } finally {
            _isLoading.value = false
        }
    }
    
    // ==================== Plugin Loading ====================
    
    /**
     * Load a single plugin
     */
    private suspend fun loadPlugin(installed: InstalledPlugin) {
        try {
            logDebug { "[loadPlugin] Starting to load: ${installed.internalName}" }
            
            val file = File(installed.filePath)
            if (!file.exists()) {
                Log.e(TAG, "[loadPlugin] Plugin file NOT FOUND: ${installed.filePath}")
                return
            }
            logDebug { "[loadPlugin] File exists: ${file.absolutePath}, size=${file.length()}" }
            
            val apis = loader.loadApisFromFile(installed.internalName, file)
            if (apis.isNotEmpty()) {
                _loadedApis.update { current -> (current + apis).distinctBy { it.name } }
                logDebug { "[loadPlugin] SUCCESS: Loaded ${installed.name} with ${apis.size} APIs" }
            } else {
                Log.e(TAG, "[loadPlugin] FAILED: ${installed.name} - No APIs loaded!")
            }
        } catch (t: Throwable) {
            Log.e(TAG, "[loadPlugin] EXCEPTION loading ${installed.internalName}", t)
        }
    }
    
    /**
     * Load all installed plugins
     */
    private suspend fun loadAllInstalledPlugins() {
        coroutineScope {
            val deferred = _installedPlugins.value.map { plugin ->
                async { loadPlugin(plugin) }
            }
            deferred.awaitAll()
        }
        
        logDebug { "Loaded ${_loadedApis.value.size} plugins" }
    }
    
    /**
     * Unload a plugin from memory
     */
    private fun unloadPlugin(internalName: String) {
        loader.unloadExtension(internalName)
        _loadedApis.value = _loadedApis.value.filter { 
            it.sourcePlugin?.contains(internalName) != true 
        }
        logDebug { "Unloaded plugin $internalName" }
    }
    
    /**
     * Reload all plugins
     */
    suspend fun reloadAllPlugins() {
        _loadedApis.value = emptyList()
        APIHolder.clearProviders()
        loader.clearCaches()
        loadAllInstalledPlugins()
    }
    
    fun close() {
        scope.cancel()
        loader.clearCaches()
    }

    private fun isSecureRemoteUrl(url: String): Boolean {
        return FluxaCoreNative.pluginIsSecureRemoteUrl(url)
    }

    private fun expandShortcode(input: String): String {
        val trimmed = input.trim()
        // Already a full URL
        if (trimmed.contains("://")) return trimmed
        // github.com/owner/repo — prepend https://
        if (trimmed.startsWith("github.com/") || trimmed.startsWith("www.github.com/")) {
            return "https://$trimmed"
        }
        // owner/repo — GitHub shorthand → raw.githubusercontent.com builds branch
        val githubShorthand = Regex("^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$")
        if (trimmed.matches(githubShorthand)) {
            return "https://raw.githubusercontent.com/$trimmed/builds"
        }
        return trimmed
    }

    private fun normalizeRepositoryUrl(url: String): String {
        return FluxaCoreNative.normalizePluginRepositoryUrl(url)
    }

    private fun sameUrl(left: String, right: String): Boolean {
        return FluxaCoreNative.pluginSameRepositoryUrl(left, right)
    }

    private fun verifyPluginChecksum(file: File, expectedSha256: String?): String? {
        val expected = expectedSha256?.trim()?.lowercase()?.takeIf { it.isNotBlank() } ?: run {
            Log.w(TAG, "Plugin ${file.name} has no manifest checksum")
            return null
        }
        if (!expected.matches(Regex("^[a-f0-9]{64}$"))) {
            return "Invalid plugin checksum"
        }
        val actual = sha256(file)
        return if (actual == expected) null else "Plugin checksum verification failed"
    }

    private fun sha256(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input ->
            val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
            while (true) {
                val read = input.read(buffer)
                if (read <= 0) break
                digest.update(buffer, 0, read)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }

    // ==================== Auto Update Functions ====================
    
    /**
     * Check all installed plugins for updates from their repositories.
     * Auto-updates plugins in background if newer version found.
     * Call this periodically (e.g., on app start or once per day).
     */
    suspend fun checkAndAutoUpdatePlugins(): List<String> = mutex.withLock {
        PluginAutoUpdater(
            repositories = { _repositories.value },
            installedPlugins = { _installedPlugins.value },
            updatePlugin = ::updatePluginInternal
        ).checkAndAutoUpdate()
    }
}
