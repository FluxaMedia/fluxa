package com.fluxa.app.shared.feature.addonstore

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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun AddonStoreScreen(
    state: AddonStoreUiState,
    language: String?,
    onAction: (AddonStoreAction) -> Unit,
    onConfigureRequested: (String) -> Unit,
    onBackRequested: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    val accentColor = Color(state.accentColorArgb.toInt())

    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 20.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            item {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(18.dp)) {
                    Box(
                        modifier = Modifier
                            .size(40.dp)
                            .clip(CircleShape)
                            .background(Color.White.copy(alpha = 0.05f))
                            .clickable(onClick = onBackRequested),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = null, tint = Color.White, modifier = Modifier.size(20.dp))
                    }
                    Text(
                        text = AppStrings.t(language, "auto.addons"),
                        color = Color.White,
                        fontWeight = FontWeight.Black,
                        fontSize = 26.sp
                    )
                }
            }

            item {
                AddonSmartInput(
                    state = state,
                    language = language,
                    onTextChange = { onAction(AddonStoreAction.InputChanged(it)) },
                    onSubmit = { onAction(AddonStoreAction.SubmitInput) }
                )
            }

            if (state.cloudstreamRepos.isNotEmpty()) {
                item {
                    Text(
                        text = AppStrings.t(language, "auto.cloudstream3_repositories"),
                        color = Color.White,
                        fontWeight = FontWeight.Bold,
                        fontSize = 16.sp
                    )
                }
                items(state.cloudstreamRepos, key = { "repo:${it.url}" }) { repo ->
                    CloudstreamRepoItem(
                        repo = repo,
                        language = language,
                        onRemove = { onAction(AddonStoreAction.RepoRemoved(repo.url)) },
                        onClick = { onAction(AddonStoreAction.RepoOpened(repo.url)) }
                    )
                }
            }

            if (state.installedAddons.isNotEmpty()) {
                item {
                    Spacer(Modifier.height(4.dp))
                    Text(
                        text = AppStrings.t(language, "auto.installed_stremio_addons"),
                        color = Color.White,
                        fontWeight = FontWeight.Bold,
                        fontSize = 16.sp
                    )
                }
                itemsIndexed(
                    items = state.installedAddons,
                    key = { index, addon -> "${addon.url}:$index" }
                ) { _, addon ->
                    InstalledAddonItem(
                        addon = addon,
                        language = language,
                        accentColor = accentColor,
                        onConfigure = addon.configUrl?.let { url -> { onConfigureRequested(url) } },
                        onMoveUp = { onAction(AddonStoreAction.AddonMoved(addon.url, -1)) },
                        onMoveDown = { onAction(AddonStoreAction.AddonMoved(addon.url, 1)) },
                        onRefresh = { onAction(AddonStoreAction.AddonRefreshed(addon.url)) },
                        onToggleEnabled = { enabled -> onAction(AddonStoreAction.AddonToggled(addon.url, enabled)) },
                        onRemove = { onAction(AddonStoreAction.AddonRemoved(addon.url)) }
                    )
                }
            }
        }

        if (state.openRepoUrl != null) {
            AddonRepoPluginsDialog(
                state = state,
                language = language,
                onDismiss = { onAction(AddonStoreAction.RepoDialogDismissed) },
                onTogglePlugin = { internalName ->
                    onAction(AddonStoreAction.RepoPluginToggled(state.openRepoUrl, internalName))
                }
            )
        }
    }
}

@Composable
private fun AddonSmartInput(
    state: AddonStoreUiState,
    language: String?,
    onTextChange: (String) -> Unit,
    onSubmit: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(18.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        OutlinedTextField(
            value = state.inputText,
            onValueChange = onTextChange,
            modifier = Modifier.fillMaxWidth().height(52.dp),
            singleLine = true,
            placeholder = {
                Text(
                    AppStrings.t(language, "addons.paste_manifest_url"),
                    color = Color.White.copy(alpha = 0.3f),
                    fontSize = 13.sp
                )
            },
            trailingIcon = {
                if (state.isSubmittingInput) {
                    CircularProgressIndicator(
                        color = Color.White.copy(alpha = 0.72f),
                        strokeWidth = 2.dp,
                        modifier = Modifier.size(18.dp)
                    )
                } else if (state.inputDetectedType != AddonStoreInputType.UNKNOWN &&
                    state.inputDetectedType != AddonStoreInputType.SEARCH_QUERY
                ) {
                    IconButton(enabled = state.inputText.isNotBlank(), onClick = onSubmit) {
                        Icon(Icons.Filled.Add, contentDescription = null, tint = Color.White)
                    }
                }
            },
            colors = OutlinedTextFieldDefaults.colors(
                unfocusedBorderColor = Color.White.copy(alpha = 0.1f),
                focusedTextColor = Color.White,
                unfocusedTextColor = Color.White,
                cursorColor = Color.White,
                focusedContainerColor = Color.Black.copy(alpha = 0.2f),
                unfocusedContainerColor = Color.Black.copy(alpha = 0.2f)
            ),
            shape = RoundedCornerShape(12.dp),
            isError = state.inputError != null
        )
        val error = state.inputError
        if (error != null) {
            Text(text = error, color = FluxaColors.errorRed, fontSize = 12.sp)
        }
    }
}

@Composable
private fun CloudstreamRepoItem(
    repo: CloudstreamRepoUiModel,
    language: String?,
    onRemove: () -> Unit,
    onClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .border(1.dp, Color.White.copy(alpha = 0.08f), RoundedCornerShape(14.dp))
            .clickable(onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 12.dp)
    ) {
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            AddonLogo(url = repo.iconUrl, size = 36.dp, contentDescription = repo.name)
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = repo.name.ifBlank { AppStrings.t(language, "addons.cloudstream_repo") },
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 14.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = repo.url,
                    color = Color.White.copy(alpha = 0.5f),
                    fontSize = 11.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
            IconButton(onClick = onRemove) {
                Icon(Icons.Filled.Close, contentDescription = null, tint = Color.White.copy(alpha = 0.5f), modifier = Modifier.size(18.dp))
            }
        }
    }
}

@Composable
private fun InstalledAddonItem(
    addon: InstalledAddonUiModel,
    language: String?,
    accentColor: Color,
    onConfigure: (() -> Unit)?,
    onMoveUp: () -> Unit,
    onMoveDown: () -> Unit,
    onRefresh: () -> Unit,
    onToggleEnabled: (Boolean) -> Unit,
    onRemove: () -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(18.dp))
            .background(
                if (addon.isEnabled) Color.White.copy(alpha = 0.04f) else Color.White.copy(alpha = 0.025f)
            )
            .border(1.dp, Color.White.copy(alpha = 0.06f), RoundedCornerShape(18.dp))
            .padding(horizontal = 18.dp, vertical = 16.dp)
    ) {
        Row(modifier = Modifier.fillMaxWidth().padding(end = 56.dp), verticalAlignment = Alignment.Top) {
            AddonLogo(url = addon.logoUrl, size = 70.dp, contentDescription = addon.name)
            Spacer(Modifier.width(14.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = addon.name,
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 16.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
                if (addon.canRemove) {
                    Spacer(Modifier.height(8.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(2.dp), verticalAlignment = Alignment.CenterVertically) {
                        if (onConfigure != null) {
                            IconButton(onClick = onConfigure) {
                                Icon(Icons.Filled.Settings, contentDescription = null, tint = Color.White.copy(alpha = 0.68f), modifier = Modifier.size(18.dp))
                            }
                        }
                        IconButton(onClick = onRefresh, enabled = !addon.isRefreshing) {
                            if (addon.isRefreshing) {
                                CircularProgressIndicator(color = Color.White.copy(alpha = 0.68f), strokeWidth = 2.dp, modifier = Modifier.size(18.dp))
                            } else {
                                Icon(Icons.Filled.Refresh, contentDescription = null, tint = Color.White.copy(alpha = 0.68f), modifier = Modifier.size(18.dp))
                            }
                        }
                        IconButton(onClick = onMoveUp, enabled = addon.canMoveUp) {
                            Icon(Icons.Filled.KeyboardArrowUp, contentDescription = null, tint = Color.White.copy(alpha = if (addon.canMoveUp) 0.68f else 0.22f))
                        }
                        IconButton(onClick = onMoveDown, enabled = addon.canMoveDown) {
                            Icon(Icons.Filled.KeyboardArrowDown, contentDescription = null, tint = Color.White.copy(alpha = if (addon.canMoveDown) 0.68f else 0.22f))
                        }
                        IconButton(onClick = onRemove) {
                            Icon(Icons.Filled.Close, contentDescription = null, tint = Color.White.copy(alpha = 0.62f), modifier = Modifier.size(18.dp))
                        }
                    }
                }
                Spacer(Modifier.height(8.dp))
                Text(
                    text = addon.description,
                    color = Color.White.copy(alpha = 0.48f),
                    fontSize = 12.sp,
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
        if (addon.canRemove) {
            Switch(
                checked = addon.isEnabled,
                onCheckedChange = onToggleEnabled,
                modifier = Modifier.align(Alignment.TopEnd).height(40.dp),
                colors = SwitchDefaults.colors(
                    checkedTrackColor = accentColor,
                    uncheckedThumbColor = Color.White.copy(alpha = 0.72f),
                    uncheckedTrackColor = Color.White.copy(alpha = 0.18f)
                )
            )
        }
    }
}

@Composable
private fun AddonRepoPluginsDialog(
    state: AddonStoreUiState,
    language: String?,
    onDismiss: () -> Unit,
    onTogglePlugin: (String) -> Unit
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Text(
                text = state.openRepoName ?: AppStrings.t(language, "auto.repository_plugins"),
                color = Color.White,
                fontWeight = FontWeight.Bold
            )
        },
        text = {
            LazyColumn(modifier = Modifier.height(400.dp)) {
                if (state.isLoadingRepoPlugins) {
                    item {
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) {
                            CircularProgressIndicator(color = Color.White, modifier = Modifier.size(32.dp))
                        }
                    }
                } else if (state.openRepoPlugins.isEmpty()) {
                    item {
                        Text(
                            text = AppStrings.t(language, "auto.no_plugins_found_in_this_repository"),
                            color = Color.White.copy(alpha = 0.6f)
                        )
                    }
                } else {
                    val error = state.repoDialogError
                    if (error != null) {
                        item {
                            Text(
                                text = error,
                                color = FluxaColors.errorRed,
                                fontSize = 13.sp,
                                modifier = Modifier.fillMaxWidth().padding(bottom = 8.dp)
                            )
                        }
                    }
                    itemsIndexed(state.openRepoPlugins, key = { _, p -> p.internalName }) { index, plugin ->
                        val installing = plugin.internalName in state.installingPluginKeys
                        CloudstreamPluginItem(
                            plugin = plugin,
                            isInstalling = installing,
                            onClick = { if (!installing) onTogglePlugin(plugin.internalName) }
                        )
                        if (index < state.openRepoPlugins.lastIndex) {
                            HorizontalDivider(color = Color.White.copy(alpha = 0.07f), thickness = 0.5.dp)
                        }
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) {
                Text(text = AppStrings.t(language, "auto.close"), color = Color.White.copy(alpha = 0.7f))
            }
        },
        containerColor = Color(0xFF1A1D26)
    )
}

@Composable
private fun CloudstreamPluginItem(
    plugin: CloudstreamPluginUiModel,
    isInstalling: Boolean,
    onClick: () -> Unit
) {
    val accentColor = if (plugin.isInstalled) FluxaColors.successGreen else Color.White.copy(alpha = 0.6f)
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = !isInstalling, onClick = onClick)
            .padding(horizontal = 4.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        AddonLogo(url = plugin.iconUrl, size = 42.dp, contentDescription = plugin.name)
        Spacer(Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = plugin.name,
                color = Color.White,
                fontWeight = FontWeight.SemiBold,
                fontSize = 14.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            val subtitle = buildString {
                if (plugin.description.isNotBlank()) append(plugin.description.take(60))
                if (plugin.typesLabel.isNotBlank()) {
                    if (isNotEmpty()) append("  ·  ")
                    append(plugin.typesLabel)
                }
            }
            if (subtitle.isNotBlank()) {
                Text(text = subtitle, color = Color.White.copy(alpha = 0.45f), fontSize = 11.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
        Spacer(Modifier.width(10.dp))
        if (isInstalling) {
            CircularProgressIndicator(color = accentColor, strokeWidth = 2.dp, modifier = Modifier.size(20.dp))
        } else {
            Icon(
                imageVector = if (plugin.isInstalled) Icons.Filled.Close else Icons.Filled.Download,
                contentDescription = null,
                tint = if (plugin.isInstalled) FluxaColors.errorRed else Color.White.copy(alpha = 0.5f),
                modifier = Modifier.size(20.dp)
            )
        }
    }
}

@Composable
private fun AddonLogo(url: String?, size: androidx.compose.ui.unit.Dp, contentDescription: String?) {
    Box(
        modifier = Modifier.size(size).clip(RoundedCornerShape(12.dp)).background(Color.White.copy(alpha = 0.08f)),
        contentAlignment = Alignment.Center
    ) {
        if (!url.isNullOrBlank()) {
            FluxaRemoteImage(
                imageUrl = url,
                cacheKey = "addon-logo:$url",
                contentDescription = contentDescription,
                modifier = Modifier.size(size * 0.82f).clip(RoundedCornerShape(8.dp)),
                contentScale = androidx.compose.ui.layout.ContentScale.Fit
            )
        }
    }
}
