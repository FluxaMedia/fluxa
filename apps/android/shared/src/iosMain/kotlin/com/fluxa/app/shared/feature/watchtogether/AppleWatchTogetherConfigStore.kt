package com.fluxa.app.shared.feature.watchtogether

import platform.Foundation.NSUserDefaults

internal class AppleWatchTogetherConfigStore(
    private val defaults: NSUserDefaults = NSUserDefaults.standardUserDefaults,
) : WatchTogetherConfigStore {
    override fun load(defaultDisplayName: String): WatchTogetherConfig = WatchTogetherConfig(
        serverUrl = defaults.stringForKey(KEY_SERVER_URL).orEmpty(),
        serverSecret = defaults.stringForKey(KEY_SERVER_SECRET).orEmpty(),
        displayName = defaults.stringForKey(KEY_DISPLAY_NAME)
            ?.takeIf { it.isNotBlank() }
            ?: defaultDisplayName.ifBlank { "Guest" },
    )

    override fun save(config: WatchTogetherConfig) {
        defaults.setObject(config.serverUrl, KEY_SERVER_URL)
        defaults.setObject(config.serverSecret, KEY_SERVER_SECRET)
        defaults.setObject(config.displayName, KEY_DISPLAY_NAME)
    }

    private companion object {
        const val KEY_SERVER_URL = "fluxa.watch.server"
        const val KEY_SERVER_SECRET = "fluxa.watch.secret"
        const val KEY_DISPLAY_NAME = "fluxa.watch.name"
    }
}
