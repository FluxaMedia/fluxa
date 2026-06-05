package com.fluxa.app.plugins

import android.util.Log
import com.fluxa.app.BuildConfig
import com.fluxa.app.plugins.cloudstream.ExternalRepoParser
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import com.fluxa.app.plugins.cloudstream.PluginInfo
import com.fluxa.app.plugins.cloudstream.PluginRepositoryEntry
import com.fluxa.app.plugins.cloudstream.RepositoryResult

private const val TAG = "PluginAutoUpdater"

internal class PluginAutoUpdater(
    private val repositories: () -> List<PluginRepositoryEntry>,
    private val installedPlugins: () -> List<InstalledPlugin>,
    private val updatePlugin: suspend (String, PluginInfo) -> Result<InstalledPlugin>
) {
    suspend fun checkAndAutoUpdate(): List<String> {
        val updatedPlugins = mutableListOf<String>()
        val repos = repositories()
        val installed = installedPlugins()

        if (repos.isEmpty() || installed.isEmpty()) {
            logDebug { "[AutoUpdate] No repos or plugins to check" }
            return updatedPlugins
        }

        logDebug { "[AutoUpdate] Checking ${installed.size} plugins for updates..." }

        installed.groupBy { it.repositoryUrl }.forEach { (repoUrl, plugins) ->
            if (repoUrl == null) {
                logDebug { "[AutoUpdate] Skipping ${plugins.size} plugins with no repository URL" }
                return@forEach
            }

            try {
                val repoData = when (val repoResult = ExternalRepoParser().fetchRepository(repoUrl)) {
                    is RepositoryResult.Success -> repoResult.manifest
                    is RepositoryResult.Error -> {
                        Log.w(TAG, "[AutoUpdate] Failed to fetch repo: $repoUrl - ${repoResult.message}")
                        return@forEach
                    }
                }

                val remotePlugins = repoData.plugins.associateBy { it.internalName }
                plugins.forEach { localPlugin ->
                    val remotePlugin = remotePlugins[localPlugin.internalName] ?: run {
                        logDebug { "[AutoUpdate] Plugin ${localPlugin.internalName} not found in repo" }
                        return@forEach
                    }

                    if (remotePlugin.version > localPlugin.version) {
                        updateSinglePlugin(localPlugin, remotePlugin, updatedPlugins)
                    } else {
                        logDebug { "[AutoUpdate] ${localPlugin.name} is up to date (v${localPlugin.version})" }
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "[AutoUpdate] Error processing repo: $repoUrl", e)
            }
        }

        logDebug { "[AutoUpdate] Completed. Updated ${updatedPlugins.size} plugins: $updatedPlugins" }
        return updatedPlugins
    }

    private suspend fun updateSinglePlugin(
        localPlugin: InstalledPlugin,
        remotePlugin: PluginInfo,
        updatedPlugins: MutableList<String>
    ) {
        logDebug {
            "[AutoUpdate] Update available: ${localPlugin.name} " +
                "(${localPlugin.version} -> ${remotePlugin.version})"
        }

        try {
            val updateResult = updatePlugin(localPlugin.internalName, remotePlugin)
            if (updateResult.isSuccess) {
                logDebug { "[AutoUpdate] Successfully updated: ${localPlugin.name} to v${remotePlugin.version}" }
                updatedPlugins.add(localPlugin.name)
            } else {
                Log.e(TAG, "[AutoUpdate] Failed to update: ${localPlugin.name}", updateResult.exceptionOrNull())
            }
        } catch (e: Exception) {
            Log.e(TAG, "[AutoUpdate] Error updating: ${localPlugin.name}", e)
        }
    }

    private inline fun logDebug(message: () -> String) {
        if (BuildConfig.DEBUG) Log.d(TAG, message())
    }
}
