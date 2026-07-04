@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.util.Log
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.plugins.cloudstream.PluginInfo
import com.fluxa.app.plugins.cloudstream.RepositoryManifest
import com.fluxa.app.data.repository.HttpRequestSecurity
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import java.io.IOException
import java.util.Locale
import java.util.concurrent.ConcurrentHashMap

@Composable
fun AddonStoreScreen(
    activeProfile: UserProfile?,
    onBack: () -> Unit,
    onInstallAddon: (String) -> Unit,
    onRemoveAddon: (String) -> Unit,
    onMoveAddon: (String, Int) -> Unit,
    onToggleAddon: (String, Boolean) -> Unit
) {
    val lang = activeProfile?.language ?: "en"
    val deviceType = LocalDeviceType.current
    val horizontalPadding = if (deviceType == DeviceType.TV) 58.dp else 16.dp
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    val pluginManager = remember { AppContainer.pluginManager }
    val repository = remember { AppContainer.repository }
    
    var smartInput by remember { mutableStateOf(SmartInputState()) }
    var showRepoDialog by remember { mutableStateOf<RepositoryManifest?>(null) }
    var selectedRepoUrl by remember { mutableStateOf<String?>(null) }
    var selectedRepoPlugins by remember { mutableStateOf<List<PluginInfo>>(emptyList()) }
    var isLoadingRepoPlugins by remember { mutableStateOf(false) }
    var installedPlugins by remember { mutableStateOf(pluginManager.installedPlugins.value) }
    var cs3Repos by remember { mutableStateOf(pluginManager.repositories.value) }
    var installedUserAddons by remember { mutableStateOf<List<AddonDescriptor>>(emptyList()) }
    var installedUserAddonsLoaded by remember { mutableStateOf(false) }
    var refreshingAddonUrl by remember { mutableStateOf<String?>(null) }
    
    LaunchedEffect(Unit) {
        pluginManager.installedPlugins.collect { installedPlugins = it }
    }
    LaunchedEffect(Unit) {
        pluginManager.repositories.collect { cs3Repos = it }
    }

    LaunchedEffect(activeProfile?.id, activeProfile?.authKey, activeProfile?.safeInstalledLocalAddons) {
        installedUserAddonsLoaded = false
        val profile = activeProfile
        if (profile == null) {
            installedUserAddons = emptyList()
            installedUserAddonsLoaded = true
            return@LaunchedEffect
        }
        installedUserAddons = runCatching {
            repository.getUserAddons(
                authKey = profile.authKey,
                localAddons = profile.safeInstalledLocalAddons,
                forceRefresh = false
            )
        }.getOrDefault(emptyList())
        installedUserAddonsLoaded = true
    }
    
    fun detectInputType(text: String): DetectedType {
        return when (FluxaCoreNative.addonStoreInputType(text)) {
            "stremio_manifest" -> DetectedType.STREMIO_MANIFEST
            "cloudstream_repo" -> DetectedType.CLOUDSTREAM_REPO
            "search_query" -> DetectedType.SEARCH_QUERY
            else -> DetectedType.UNKNOWN
        }
    }
    
    fun handleSmartInput() {
        val text = smartInput.text.trim()
        if (text.isEmpty()) return
        
        val type = detectInputType(text)
        
        when (type) {
            DetectedType.STREMIO_MANIFEST -> {
                onInstallAddon(text)
                smartInput = smartInput.copy(text = "", detectedType = DetectedType.UNKNOWN)
            }
            DetectedType.CLOUDSTREAM_REPO -> {
                smartInput = smartInput.copy(isLoading = true, error = null)
                scope.launch {
                    val normalizedUrl = FluxaCoreNative.normalizeCloudstreamRepoUrl(text)
                    
                    val result = pluginManager.addRepository(normalizedUrl)
                    smartInput = if (result.isSuccess) {
                        smartInput.copy(
                            text = "",
                            isLoading = false,
                            detectedType = DetectedType.UNKNOWN
                        )
                    } else {
                        smartInput.copy(
                            isLoading = false,
                            error = result.exceptionOrNull()?.message ?: AppStrings.t(lang, "addons.repository_add_failed")
                        )
                    }
                }
            }
            else -> {
            }
        }
    }
    
    LaunchedEffect(smartInput.text) {
        if (smartInput.text.isBlank()) {
            smartInput = smartInput.copy(detectedType = DetectedType.UNKNOWN)
        } else {
            smartInput = smartInput.copy(detectedType = detectInputType(smartInput.text))
        }
    }

    val normalizedInstalledAddonUrls = remember(activeProfile?.safeInstalledLocalAddons) {
        activeProfile?.safeInstalledLocalAddons.orEmpty().map(::normalizeAddonUrlForProfile)
    }
    val normalizedInstalledAddonIdentities = remember(normalizedInstalledAddonUrls) {
        normalizedInstalledAddonUrls.map(::addonUrlIdentity)
    }
    val disabledLocalAddonIdentities = remember(activeProfile?.disabledLocalAddons) {
        activeProfile?.disabledLocalAddons.orEmpty().map(::addonUrlIdentity).toSet()
    }

    val installedStremioAddons = remember(installedUserAddons, installedUserAddonsLoaded, activeProfile?.authKey, activeProfile?.safeInstalledLocalAddons) {
        val fromRepository = installedUserAddons.map { addon ->
            CommunityAddon(
                name = addon.manifest.name.takeIf { it.isNotBlank() } ?: addonNameFromUrl(addon.transportUrl),
                description = addon.manifest.description?.takeIf { it.isNotBlank() }.orEmpty(),
                url = addon.transportUrl,
                logoUrl = addon.manifest.logo,
                version = addon.manifest.version,
                manifest = addon.manifest,
                configUrl = addonConfigUrl(addon.transportUrl),
                configurable = addon.manifest.configurable == true,
                fallbackIcon = FluxaIcons.Extension
            )
        }
        val repositoryUrls = fromRepository.map { addonUrlIdentity(it.url) }.toSet()
        val localFallback = if (installedUserAddonsLoaded) activeProfile?.safeInstalledLocalAddons.orEmpty().filterNot { addonUrlIdentity(it) in repositoryUrls }.map { url ->
            val normalizedUrl = normalizeAddonUrlForProfile(url)
            CommunityAddon(
                name = addonNameFromUrl(normalizedUrl),
                description = "",
                url = normalizedUrl,
                logoUrl = null,
                configUrl = addonConfigUrl(normalizedUrl),
                fallbackIcon = FluxaIcons.Extension
            )
        } else emptyList()
        (fromRepository + localFallback)
            .distinctBy { addonUrlIdentity(it.url) }
            .sortedWith(compareBy<CommunityAddon> {
                normalizedInstalledAddonIdentities.indexOf(addonUrlIdentity(it.url)).takeIf { index -> index >= 0 } ?: Int.MAX_VALUE
            }.thenBy { it.name.lowercase() })
    }
    
    val accentColor = androidx.compose.ui.graphics.Color(activeProfile?.safeAccentColorArgb ?: 0xFF4CAF50.toInt())

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFF090B0F))
    ) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(
                start = horizontalPadding,
                end = horizontalPadding,
                top = if (deviceType == DeviceType.TV) 34.dp else 20.dp,
                bottom = if (deviceType == DeviceType.Mobile) 132.dp else 60.dp
            ),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            item {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(18.dp)
                ) {
                    Box(
                        modifier = Modifier
                            .size(48.dp)
                            .clip(CircleShape)
                            .background(Color.White.copy(alpha = 0.05f))
                            .clickable { onBack() },
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(FluxaIcons.ArrowBack, null, tint = Color.White)
                    }
                    Column {
                        Text(
                            text = AppStrings.t(lang, "auto.addons"),
                            color = Color.White,
                            fontWeight = FontWeight.Black,
                            fontSize = if (deviceType == DeviceType.TV) 30.sp else 26.sp
                        )
                    }
                }
            }

            addAddonSmartInputSection(
                lang = lang,
                smartInput = smartInput,
                onTextChange = { smartInput = smartInput.copy(text = it, error = null) },
                onSubmit = { handleSmartInput() }
            )
            
            addCloudstreamAddonSections(
                lang = lang,
                cs3Repos = cs3Repos,
                pluginManager = pluginManager,
                scope = scope,
                isInstalling = smartInput.isLoading,
                onRepoPluginsLoading = { repoUrl ->
                    selectedRepoUrl = repoUrl
                    isLoadingRepoPlugins = true
                    selectedRepoPlugins = emptyList()
                    scope.launch {
                        selectedRepoPlugins = pluginManager.getPluginsFromRepository(repoUrl)
                        isLoadingRepoPlugins = false
                    }
                },
                onError = { smartInput = smartInput.copy(error = it) }
            )

            if (installedStremioAddons.isNotEmpty()) {
                item {
                    Spacer(Modifier.height(16.dp))
                    Text(
                        text = AppStrings.t(lang, "auto.installed_stremio_addons"),
                        color = Color.White,
                        fontWeight = FontWeight.Bold,
                        fontSize = 16.sp
                    )
                }

                itemsIndexed(
                    items = installedStremioAddons,
                    key = { index, addon -> "${addonUrlIdentity(addon.url)}:$index" }
                ) { _, addon ->
                    val normalizedUrl = normalizeAddonUrlForProfile(addon.url)
                    val addonIdentity = addonUrlIdentity(normalizedUrl)
                    val localIndex = normalizedInstalledAddonIdentities.indexOf(addonIdentity)
                    val canRemove = localIndex >= 0
                    InstalledAddonItem(
                        addon = addon,
                        lang = lang,
                        canRemove = canRemove,
                        isEnabled = addonIdentity !in disabledLocalAddonIdentities,
                        canMoveUp = canRemove && localIndex > 0,
                        canMoveDown = canRemove && localIndex < normalizedInstalledAddonUrls.lastIndex,
                        onConfigure = addon.configUrl?.let { configUrl ->
                            {
                                context.startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW, android.net.Uri.parse(configUrl)))
                            }
                        },
                        onMoveUp = { onMoveAddon(normalizedUrl, -1) },
                        onMoveDown = { onMoveAddon(normalizedUrl, 1) },
                        onRefresh = {
                            refreshingAddonUrl = normalizedUrl
                            scope.launch {
                                val refreshed = withContext(Dispatchers.IO) {
                                    repository.getAddonManifest(normalizedUrl, forceRefresh = true)
                                }
                                if (refreshed != null) {
                                    installedUserAddons = installedUserAddons
                                        .filterNot { addonUrlIdentity(it.transportUrl) == addonUrlIdentity(normalizedUrl) } + refreshed
                                } else {
                                    installedUserAddons = withContext(Dispatchers.IO) {
                                        activeProfile?.let { profile ->
                                            repository.getUserAddons(profile.authKey, profile.safeInstalledLocalAddons, forceRefresh = true)
                                        }.orEmpty()
                                    }
                                }
                                refreshingAddonUrl = null
                            }
                        },
                        isRefreshing = addonUrlIdentity(refreshingAddonUrl.orEmpty()) == addonUrlIdentity(normalizedUrl),
                        onToggleEnabled = { enabled -> onToggleAddon(normalizedUrl, enabled) },
                        onRemove = { onRemoveAddon(normalizedUrl) },
                        accentColor = accentColor
                    )
                }
            }
        }
        
        val selectedRepo = selectedRepoUrl?.let { url -> cs3Repos.find { it.url == url } }
        AddonRepoPluginsDialog(
            selectedRepoUrl = selectedRepoUrl,
            selectedRepoName = selectedRepo?.name?.takeIf { it.isNotBlank() },
            selectedRepoPlugins = selectedRepoPlugins,
            isLoadingRepoPlugins = isLoadingRepoPlugins,
            pluginManager = pluginManager,
            lang = lang,
            isInstalling = smartInput.isLoading,
            onDismiss = {
                selectedRepoUrl = null
                selectedRepoPlugins = emptyList()
            },
            onError = { smartInput = smartInput.copy(error = it) },
            onInstalledPluginsChanged = { installedPlugins = pluginManager.installedPlugins.value }
        )
    }
}
