package com.fluxa.app.ui.catalog

import android.content.Context
import com.fluxa.app.shared.feature.watchtogether.WatchTogetherConfig
import com.fluxa.app.shared.feature.watchtogether.WatchTogetherConfigStore

internal class AndroidWatchTogetherConfigStore(context: Context) : WatchTogetherConfigStore {
    private val preferences = context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    override fun load(defaultDisplayName: String): WatchTogetherConfig = WatchTogetherConfig(
        serverUrl = preferences.getString(KEY_SERVER_URL, "").orEmpty(),
        serverSecret = preferences.getString(KEY_SERVER_SECRET, "").orEmpty(),
        displayName = preferences.getString(KEY_DISPLAY_NAME, defaultDisplayName)
            ?.takeIf(String::isNotBlank)
            ?: defaultDisplayName.ifBlank { "Guest" },
    )

    override fun save(config: WatchTogetherConfig) {
        preferences.edit()
            .putString(KEY_SERVER_URL, config.serverUrl)
            .putString(KEY_SERVER_SECRET, config.serverSecret)
            .putString(KEY_DISPLAY_NAME, config.displayName)
            .apply()
    }

    private companion object {
        const val PREFS_NAME = "fluxa_watch_together"
        const val KEY_SERVER_URL = "serverUrl"
        const val KEY_SERVER_SECRET = "serverSecret"
        const val KEY_DISPLAY_NAME = "displayName"
    }
}
