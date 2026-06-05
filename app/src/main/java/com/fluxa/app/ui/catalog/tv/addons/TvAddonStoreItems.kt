package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Composable
import com.fluxa.app.plugins.cloudstream.InstalledPlugin
import com.fluxa.app.plugins.cloudstream.PluginInfo

@Composable
fun TvInstalledAddonItem(
    addon: CommunityAddon,
    lang: String,
    canRemove: Boolean,
    isEnabled: Boolean,
    canMoveUp: Boolean,
    canMoveDown: Boolean,
    onConfigure: (() -> Unit)?,
    onMoveUp: () -> Unit,
    onMoveDown: () -> Unit,
    onRefresh: () -> Unit,
    isRefreshing: Boolean,
    onToggleEnabled: (Boolean) -> Unit,
    onRemove: () -> Unit
) {
    MobileInstalledAddonItem(
        addon = addon,
        lang = lang,
        canRemove = canRemove,
        isEnabled = isEnabled,
        canMoveUp = canMoveUp,
        canMoveDown = canMoveDown,
        onConfigure = onConfigure,
        onMoveUp = onMoveUp,
        onMoveDown = onMoveDown,
        onRefresh = onRefresh,
        isRefreshing = isRefreshing,
        onToggleEnabled = onToggleEnabled,
        onRemove = onRemove
    )
}

@Composable
fun TvAddonStoreItem(
    addon: CommunityAddon,
    lang: String,
    isInstalled: Boolean,
    onInstall: () -> Unit
) {
    MobileAddonStoreItem(
        addon = addon,
        lang = lang,
        isInstalled = isInstalled,
        onInstall = onInstall
    )
}

@Composable
fun TvCS3RepoItem(
    repoName: String,
    repoUrl: String,
    lang: String,
    onRemove: () -> Unit,
    onClick: () -> Unit
) {
    MobileCS3RepoItem(
        repoName = repoName,
        repoUrl = repoUrl,
        lang = lang,
        onRemove = onRemove,
        onClick = onClick
    )
}

@Composable
fun TvCS3PluginItem(
    plugin: PluginInfo,
    lang: String,
    isInstalling: Boolean,
    isInstalled: Boolean = false,
    onInstall: () -> Unit
) {
    MobileCS3PluginItem(
        plugin = plugin,
        lang = lang,
        isInstalling = isInstalling,
        isInstalled = isInstalled,
        onInstall = onInstall
    )
}

@Composable
fun TvCS3InstalledPluginItem(
    plugin: InstalledPlugin,
    lang: String,
    onRemove: () -> Unit
) {
    MobileCS3InstalledPluginItem(
        plugin = plugin,
        lang = lang,
        onRemove = onRemove
    )
}
