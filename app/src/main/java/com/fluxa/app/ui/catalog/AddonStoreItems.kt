@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.plugins.cloudstream.PluginInfo
import com.fluxa.app.domain.discovery.StremioAddonUrls
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.util.Locale
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.Icon
import androidx.tv.material3.Surface

@Composable
fun InstalledAddonItem(
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
    onRemove: () -> Unit,
    accentColor: Color = Color(0xFF4CAF50)
) {
    if (LocalDeviceType.current == DeviceType.Mobile) {
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
            onRemove = onRemove,
            accentColor = accentColor
        )
    } else {
        TvInstalledAddonItem(
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
}

@Composable
fun AddonStoreItem(
    addon: CommunityAddon,
    lang: String,
    isInstalled: Boolean,
    onInstall: () -> Unit
) {
    if (LocalDeviceType.current == DeviceType.Mobile) {
        MobileAddonStoreItem(
            addon = addon,
            lang = lang,
            isInstalled = isInstalled,
            onInstall = onInstall
        )
    } else {
        TvAddonStoreItem(
            addon = addon,
            lang = lang,
            isInstalled = isInstalled,
            onInstall = onInstall
        )
    }
}

internal fun addonNameFromUrl(url: String): String {
    val host = runCatching { java.net.URI(url).host.orEmpty() }.getOrDefault("")
        .removePrefix("www.")
        .substringBefore(".")
    return when {
        url.contains("torrentio", ignoreCase = true) -> "Torrentio"
        url.contains("opensubtitles", ignoreCase = true) -> "OpenSubtitles"
        url.contains("cizgivedizi", ignoreCase = true) -> "Cizgi ve Dizi"
        host.isNotBlank() -> host.replace('-', ' ').replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
        else -> "Stremio Addon"
    }
}

internal fun addonConfigUrl(url: String): String? {
    return StremioAddonUrls.normalizeManifestUrl(url)
        .substringBefore("/manifest.json")
        .takeIf { it.startsWith("http://") || it.startsWith("https://") }
}


internal fun normalizeAddonUrlForProfile(rawUrl: String): String {
    return FluxaCoreNative.normalizeManifestUrl(rawUrl)
}

internal fun addonUrlIdentity(rawUrl: String): String {
    return FluxaCoreNative.identity(rawUrl)
}

@Composable
fun CS3RepoItem(
    repoName: String,
    repoUrl: String,
    repoIconUrl: String? = null,
    lang: String,
    onRemove: () -> Unit,
    onClick: () -> Unit
) {
    if (LocalDeviceType.current == DeviceType.Mobile) {
        MobileCS3RepoItem(
            repoName = repoName,
            repoUrl = repoUrl,
            repoIconUrl = repoIconUrl,
            lang = lang,
            onRemove = onRemove,
            onClick = onClick
        )
    } else {
        TvCS3RepoItem(
            repoName = repoName,
            repoUrl = repoUrl,
            lang = lang,
            onRemove = onRemove,
            onClick = onClick
        )
    }
}

@Composable
fun CS3PluginItem(
    plugin: PluginInfo,
    lang: String,
    isInstalling: Boolean,
    isInstalled: Boolean = false,
    onInstall: () -> Unit
) {
    if (LocalDeviceType.current == DeviceType.Mobile) {
        MobileCS3PluginItem(
            plugin = plugin,
            lang = lang,
            isInstalling = isInstalling,
            isInstalled = isInstalled,
            onInstall = onInstall
        )
    } else {
        TvCS3PluginItem(
            plugin = plugin,
            lang = lang,
            isInstalling = isInstalling,
            isInstalled = isInstalled,
            onInstall = onInstall
        )
    }
}

@Composable
fun CS3InstalledPluginItem(
    plugin: com.fluxa.app.plugins.cloudstream.InstalledPlugin,
    lang: String,
    onRemove: () -> Unit
) {
    if (LocalDeviceType.current == DeviceType.Mobile) {
        MobileCS3InstalledPluginItem(
            plugin = plugin,
            lang = lang,
            onRemove = onRemove
        )
    } else {
        TvCS3InstalledPluginItem(
            plugin = plugin,
            lang = lang,
            onRemove = onRemove
        )
    }
}

