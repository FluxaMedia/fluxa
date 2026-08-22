package com.fluxa.app.shared.feature.watchtogether

/** Platform persistence boundary for Watch Together configuration. */
interface WatchTogetherConfigStore {
    fun load(defaultDisplayName: String = "Guest"): WatchTogetherConfig
    fun save(config: WatchTogetherConfig)
}

/** Shared orchestration prevents every platform UI from re-implementing sanitize/save/configure. */
fun WatchTogetherConfigStore.loadIntoManager(defaultDisplayName: String = "Guest"): WatchTogetherConfig {
    val config = load(defaultDisplayName)
    WatchTogetherManager.configure(config.serverUrl, config.serverSecret, config.displayName)
    return WatchTogetherManager.config.value
}

fun WatchTogetherConfigStore.saveAndConfigure(
    serverUrl: String,
    serverSecret: String,
    displayName: String,
): WatchTogetherConfig {
    val config = WatchTogetherAddress.sanitizeConfig(serverUrl, serverSecret, displayName)
    save(config)
    WatchTogetherManager.configure(config.serverUrl, config.serverSecret, config.displayName)
    return config
}
