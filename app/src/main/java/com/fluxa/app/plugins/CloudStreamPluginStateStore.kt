package com.fluxa.app.plugins

import android.content.Context
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import com.fluxa.app.plugins.cloudstream.PluginRepositoryEntry

private const val PREFS_NAME = "fluxa_plugin_manager"
private const val KEY_INSTALLED_PLUGINS = "installed_plugins"
private const val KEY_REPOSITORIES = "repositories"
private const val KEY_AUTOMATIC_UPDATES_ENABLED = "automatic_updates_enabled"

internal data class StoredCloudStreamPluginState(
    val installedPlugins: List<InstalledPlugin>,
    val repositories: List<PluginRepositoryEntry>,
    val automaticUpdatesEnabled: Boolean,
)

/** SharedPreferences serialization boundary for CloudStream extensions. */
internal class CloudStreamPluginStateStore(context: Context) {
    private val preferences = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    fun load(): StoredCloudStreamPluginState = StoredCloudStreamPluginState(
        installedPlugins = PluginStateCodec.parseInstalledPlugins(
            preferences.getString(KEY_INSTALLED_PLUGINS, "[]") ?: "[]",
        ),
        repositories = PluginStateCodec.parseRepositories(
            preferences.getString(KEY_REPOSITORIES, "[]") ?: "[]",
        ),
        automaticUpdatesEnabled = preferences.getBoolean(KEY_AUTOMATIC_UPDATES_ENABLED, false),
    )

    fun save(
        installedPlugins: List<InstalledPlugin>,
        repositories: List<PluginRepositoryEntry>,
    ) {
        preferences.edit()
            .putString(KEY_INSTALLED_PLUGINS, PluginStateCodec.installedPluginsToJson(installedPlugins))
            .putString(KEY_REPOSITORIES, PluginStateCodec.repositoriesToJson(repositories))
            .apply()
    }

    fun setAutomaticUpdatesEnabled(enabled: Boolean) {
        preferences.edit().putBoolean(KEY_AUTOMATIC_UPDATES_ENABLED, enabled).apply()
    }
}
