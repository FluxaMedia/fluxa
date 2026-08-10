package com.fluxa.app.plugins

import android.util.Log
import com.fluxa.app.BuildConfig
import com.fluxa.app.compat.cloudstream3.APIHolder
import com.fluxa.app.plugins.cloudstream.ExternalExtensionLoader
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import com.lagradost.cloudstream3.MainAPI
import java.io.File
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

internal class CloudStreamPluginRuntime(
    private val loader: ExternalExtensionLoader,
) {
    private val _loadedApis = MutableStateFlow<List<MainAPI>>(emptyList())
    val loadedApis: StateFlow<List<MainAPI>> = _loadedApis

    suspend fun load(plugin: InstalledPlugin): Boolean = try {
        debug { "Loading ${plugin.internalName}" }
        val file = File(plugin.filePath)
        if (!file.exists()) {
            Log.e(TAG, "Plugin file not found: ${plugin.filePath}")
            false
        } else {
            val apis = loader.loadApisFromFile(plugin.cloudStreamInstallKey(), file)
            if (apis.isEmpty()) {
                Log.e(TAG, "No APIs loaded for ${plugin.name}")
                false
            } else {
                _loadedApis.value = (_loadedApis.value + apis)
                    .distinctBy { "${it.sourcePlugin}:${it.name}" }
                debug { "Loaded ${plugin.name} with ${apis.size} APIs" }
                true
            }
        }
    } catch (error: Throwable) {
        Log.e(TAG, "Failed to load ${plugin.internalName}", error)
        false
    }

    suspend fun loadAll(plugins: List<InstalledPlugin>) {
        coroutineScope {
            plugins.map { plugin -> async { load(plugin) } }.awaitAll()
        }
        debug { "Loaded ${_loadedApis.value.size} APIs" }
    }

    fun unload(plugin: InstalledPlugin) {
        val installId = plugin.cloudStreamInstallKey()
        loader.unloadExtension(installId)
        _loadedApis.value = _loadedApis.value.filterNot { api ->
            api.sourcePlugin?.contains(installId) == true
        }
        debug { "Unloaded ${plugin.internalName}" }
    }

    suspend fun reloadAll(plugins: List<InstalledPlugin>) {
        _loadedApis.value = emptyList()
        APIHolder.clearProviders()
        loader.clearCaches()
        loadAll(plugins)
    }

    fun close() = loader.clearCaches()

    private inline fun debug(message: () -> String) {
        if (BuildConfig.DEBUG) Log.d(TAG, message())
    }

    private companion object {
        const val TAG = "CloudStreamPluginRuntime"
    }
}
