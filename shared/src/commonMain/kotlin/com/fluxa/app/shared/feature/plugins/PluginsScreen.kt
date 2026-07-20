package com.fluxa.app.shared.feature.plugins

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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun PluginsScreen(
    state: PluginsUiState,
    language: String?,
    onAction: (PluginsAction) -> Unit,
    onBackRequested: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    val scrapersByRepository = state.scrapers.groupBy { it.repositoryUrl }

    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 20.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            item {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(18.dp)) {
                    var backFocused by remember { mutableStateOf(false) }
                    Box(
                        modifier = Modifier
                            .size(40.dp)
                            .clip(CircleShape)
                            .onFocusChanged { backFocused = it.isFocused }
                            .background(if (backFocused) Color.White else Color.White.copy(alpha = 0.05f))
                            .clickable(onClick = onBackRequested),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = null,
                            tint = if (backFocused) Color.Black else Color.White,
                            modifier = Modifier.size(20.dp)
                        )
                    }
                    Text(
                        text = AppStrings.t(language, "settings.plugins.title"),
                        color = Color.White,
                        fontWeight = FontWeight.Black,
                        fontSize = 26.sp
                    )
                }
            }

            item {
                Text(
                    text = AppStrings.t(language, "settings.plugins.installed_repositories"),
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 16.sp
                )
            }

            item {
                PluginRepositoryInput(
                    language = language,
                    placeholderKey = "settings.plugins.repository_url_placeholder",
                    onSubmit = { url -> onAction(PluginsAction.RepositoryAdded(url)) }
                )
            }

            if (state.repositoryError != null) {
                item {
                    Text(
                        text = AppStrings.format(language, "settings.plugins.error", state.repositoryError),
                        color = FluxaColors.errorRed,
                        fontSize = 13.sp
                    )
                }
            }

            val addingNewRepositoryUrl = state.addingRepositoryUrl
                ?.takeIf { url -> state.repositories.none { it.manifestUrl == url } }
            if (addingNewRepositoryUrl != null) {
                item {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        CircularProgressIndicator(color = Color.White.copy(alpha = 0.68f), strokeWidth = 2.dp, modifier = Modifier.size(16.dp))
                        Text(
                            text = AppStrings.t(language, "settings.plugins.adding"),
                            color = Color.White.copy(alpha = 0.6f),
                            fontSize = 13.sp
                        )
                    }
                }
            }

            if (state.repositories.isEmpty()) {
                item {
                    Text(
                        text = AppStrings.t(language, "settings.plugins.no_repositories"),
                        color = Color.White.copy(alpha = 0.5f),
                        fontSize = 13.sp,
                        modifier = Modifier.padding(vertical = 8.dp)
                    )
                }
            } else {
                items(state.repositories, key = { "plugin-repo:${it.manifestUrl}" }) { repository ->
                    PluginRepositoryGroup(
                        repository = repository,
                        scrapers = scrapersByRepository[repository.manifestUrl].orEmpty(),
                        language = language,
                        isRefreshing = state.addingRepositoryUrl == repository.manifestUrl,
                        onRemove = { onAction(PluginsAction.RepositoryRemoved(repository.manifestUrl)) },
                        onRefresh = { onAction(PluginsAction.RepositoryRefreshed(repository.manifestUrl)) },
                        onToggleScraper = { scraperId, enabled -> onAction(PluginsAction.ScraperToggled(scraperId, enabled)) },
                        onOpenScraperSettings = { scraperId -> onAction(PluginsAction.ScraperSettingsRequested(scraperId)) }
                    )
                }
            }

            item {
                Spacer(Modifier.height(4.dp))
                Text(
                    text = AppStrings.t(language, "auto.cloudstream3_repositories"),
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 16.sp
                )
            }

            item {
                PluginRepositoryInput(
                    language = language,
                    placeholderKey = "addons.cloudstream_repo",
                    onSubmit = { url -> onAction(PluginsAction.CloudstreamRepoAdded(url)) }
                )
            }

            if (state.cloudstreamRepoError != null) {
                item {
                    Text(
                        text = AppStrings.format(language, "settings.plugins.error", state.cloudstreamRepoError),
                        color = FluxaColors.errorRed,
                        fontSize = 13.sp
                    )
                }
            }

            if (state.isAddingCloudstreamRepo) {
                item {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        CircularProgressIndicator(color = Color.White.copy(alpha = 0.68f), strokeWidth = 2.dp, modifier = Modifier.size(16.dp))
                        Text(
                            text = AppStrings.t(language, "settings.plugins.adding"),
                            color = Color.White.copy(alpha = 0.6f),
                            fontSize = 13.sp
                        )
                    }
                }
            }

            if (state.cloudstreamRepos.isNotEmpty()) {
                items(state.cloudstreamRepos, key = { "cs-repo:${it.url}" }) { repo ->
                    CloudstreamRepoItem(
                        repo = repo,
                        language = language,
                        onRemove = { onAction(PluginsAction.RepoRemoved(repo.url)) },
                        onClick = { onAction(PluginsAction.RepoOpened(repo.url)) }
                    )
                }
            }

            item { Spacer(Modifier.height(80.dp)) }
        }

        if (state.openRepoUrl != null) {
            CloudstreamRepoPluginsDialog(
                state = state,
                language = language,
                onDismiss = { onAction(PluginsAction.RepoDialogDismissed) },
                onTogglePlugin = { internalName ->
                    onAction(PluginsAction.RepoPluginToggled(state.openRepoUrl, internalName))
                }
            )
        }
    }

    val settingsSheet = state.scraperSettingsSheet
    if (settingsSheet != null) {
        PluginScraperSettingsSheet(
            scraperName = settingsSheet.scraper.name,
            language = language,
            loading = settingsSheet.loading,
            fields = settingsSheet.fields,
            initialValues = settingsSheet.scraper.settings,
            onDismiss = { onAction(PluginsAction.ScraperSettingsDismissed) },
            onSave = { values -> onAction(PluginsAction.ScraperSettingsSaved(settingsSheet.scraper.id, values)) }
        )
    }
}

@Composable
private fun PluginRepositoryInput(
    language: String?,
    placeholderKey: String,
    onSubmit: (String) -> Unit
) {
    var url by remember { mutableStateOf("") }
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(18.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        OutlinedTextField(
            value = url,
            onValueChange = { url = it },
            modifier = Modifier.fillMaxWidth().height(52.dp),
            singleLine = true,
            placeholder = {
                Text(
                    AppStrings.t(language, placeholderKey),
                    color = Color.White.copy(alpha = 0.3f),
                    fontSize = 13.sp
                )
            },
            colors = OutlinedTextFieldDefaults.colors(
                unfocusedBorderColor = Color.White.copy(alpha = 0.1f),
                focusedTextColor = Color.White,
                unfocusedTextColor = Color.White,
                cursorColor = Color.White,
                focusedContainerColor = Color.Black.copy(alpha = 0.2f),
                unfocusedContainerColor = Color.Black.copy(alpha = 0.2f)
            ),
            shape = RoundedCornerShape(12.dp)
        )
        androidx.compose.material3.Button(
            onClick = {
                val trimmed = url.trim()
                if (trimmed.isNotEmpty()) {
                    onSubmit(trimmed)
                    url = ""
                }
            },
            enabled = url.isNotBlank(),
            modifier = Modifier.fillMaxWidth().height(44.dp),
            shape = RoundedCornerShape(12.dp),
            colors = androidx.compose.material3.ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black)
        ) {
            Icon(Icons.Filled.Add, contentDescription = null, modifier = Modifier.size(18.dp))
            Spacer(Modifier.width(6.dp))
            Text(AppStrings.t(language, "settings.plugins.add_repository"), fontWeight = FontWeight.Bold, fontSize = 14.sp)
        }
    }
}

@Composable
private fun PluginRepositoryGroup(
    repository: PluginRepositoryUiModel,
    scrapers: List<PluginScraperUiModel>,
    language: String?,
    isRefreshing: Boolean,
    onRemove: () -> Unit,
    onRefresh: () -> Unit,
    onToggleScraper: (String, Boolean) -> Unit,
    onOpenScraperSettings: (String) -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(18.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .border(1.dp, Color.White.copy(alpha = 0.06f), RoundedCornerShape(18.dp))
            .padding(vertical = 4.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = repository.name.ifBlank { repository.manifestUrl },
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 15.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = AppStrings.format(language, "settings.plugins.scraper_count", repository.scraperCount),
                    color = Color.White.copy(alpha = 0.5f),
                    fontSize = 12.sp
                )
            }
            if (isRefreshing) {
                Box(modifier = Modifier.size(36.dp), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator(color = Color.White.copy(alpha = 0.68f), strokeWidth = 2.dp, modifier = Modifier.size(18.dp))
                }
            } else {
                PluginsIconButton(icon = Icons.Filled.Refresh, onClick = onRefresh)
            }
            PluginsIconButton(icon = Icons.Filled.Close, tint = Color.White.copy(alpha = 0.5f), onClick = onRemove)
        }

        if (scrapers.isNotEmpty()) {
            HorizontalDivider(color = Color.White.copy(alpha = 0.06f), thickness = 0.5.dp)
            scrapers.forEachIndexed { index, scraper ->
                PluginScraperRow(
                    scraper = scraper,
                    language = language,
                    onToggle = { enabled -> onToggleScraper(scraper.id, enabled) },
                    onOpenSettings = { onOpenScraperSettings(scraper.id) }
                )
                if (index < scrapers.lastIndex) {
                    HorizontalDivider(color = Color.White.copy(alpha = 0.06f), thickness = 0.5.dp, modifier = Modifier.padding(horizontal = 16.dp))
                }
            }
        }
    }
}

@Composable
private fun PluginScraperRow(
    scraper: PluginScraperUiModel,
    language: String?,
    onToggle: (Boolean) -> Unit,
    onOpenSettings: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = scraper.name,
                color = Color.White,
                fontWeight = FontWeight.SemiBold,
                fontSize = 14.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            Text(
                text = scraper.supportedTypes.joinToString(", "),
                color = Color.White.copy(alpha = 0.45f),
                fontSize = 11.sp
            )
        }
        if (scraper.hasSettings) {
            PluginsIconButton(
                icon = Icons.Filled.Settings,
                contentDescription = AppStrings.t(language, "settings.plugins.settings"),
                onClick = onOpenSettings
            )
            Spacer(Modifier.width(4.dp))
        }
        Switch(
            checked = scraper.enabled,
            onCheckedChange = onToggle,
            colors = SwitchDefaults.colors(
                checkedTrackColor = Color.White,
                checkedThumbColor = Color.Black,
                uncheckedTrackColor = Color.White.copy(alpha = 0.18f),
                uncheckedThumbColor = Color.White.copy(alpha = 0.72f)
            )
        )
    }
}

@Composable
private fun CloudstreamRepoItem(
    repo: CloudstreamRepoUiModel,
    language: String?,
    onRemove: () -> Unit,
    onClick: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .onFocusChanged { focused = it.isFocused }
            .background(if (focused) Color.White.copy(alpha = 0.14f) else Color.White.copy(alpha = 0.04f))
            .border(if (focused) 2.dp else 1.dp, if (focused) Color.White else Color.White.copy(alpha = 0.08f), RoundedCornerShape(14.dp))
            .clickable(onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 12.dp)
    ) {
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            PluginsLogo(url = repo.iconUrl, size = 36.dp, contentDescription = repo.name)
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
            PluginsIconButton(icon = Icons.Filled.Close, tint = Color.White.copy(alpha = 0.5f), onClick = onRemove)
        }
    }
}

@Composable
private fun CloudstreamRepoPluginsDialog(
    state: PluginsUiState,
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
    var focused by remember { mutableStateOf(false) }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .onFocusChanged { focused = it.isFocused }
            .background(if (focused) Color.White.copy(alpha = 0.14f) else Color.Transparent)
            .clickable(enabled = !isInstalling, onClick = onClick)
            .padding(horizontal = 4.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        PluginsLogo(url = plugin.iconUrl, size = 42.dp, contentDescription = plugin.name)
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
private fun PluginsIconButton(
    icon: ImageVector,
    contentDescription: String? = null,
    enabled: Boolean = true,
    tint: Color = Color.White.copy(alpha = 0.68f),
    onClick: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    Box(
        modifier = Modifier
            .size(36.dp)
            .clip(CircleShape)
            .onFocusChanged { focused = it.isFocused }
            .background(if (focused) Color.White else Color.Transparent)
            .clickable(enabled = enabled, onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(icon, contentDescription = contentDescription, tint = if (focused) Color.Black else tint, modifier = Modifier.size(18.dp))
    }
}

@Composable
private fun PluginsLogo(url: String?, size: Dp, contentDescription: String?) {
    Box(
        modifier = Modifier.size(size).clip(RoundedCornerShape(12.dp)).background(Color.White.copy(alpha = 0.08f)),
        contentAlignment = Alignment.Center
    ) {
        if (!url.isNullOrBlank()) {
            FluxaRemoteImage(
                imageUrl = url,
                cacheKey = "plugin-logo:$url",
                contentDescription = contentDescription,
                modifier = Modifier.size(size * 0.82f).clip(RoundedCornerShape(8.dp)),
                contentScale = androidx.compose.ui.layout.ContentScale.Fit
            )
        }
    }
}
