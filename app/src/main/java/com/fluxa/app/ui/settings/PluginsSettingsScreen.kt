package com.fluxa.app.ui.settings

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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
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
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import com.fluxa.app.common.AppStrings
import com.fluxa.app.plugins.PluginRepositoryUiModel
import com.fluxa.app.plugins.PluginScraperUiModel
import com.fluxa.app.plugins.PluginsUiState
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun PluginsSettingsRoute(
    language: String?,
    onBackRequested: () -> Unit,
    viewModel: PluginsSettingsViewModel = hiltViewModel()
) {
    val state by viewModel.state.collectAsState()
    PluginsSettingsScreen(
        state = state,
        language = language,
        onAddRepository = viewModel::addRepository,
        onRemoveRepository = viewModel::removeRepository,
        onRefreshRepository = viewModel::refreshRepository,
        onToggleScraper = viewModel::toggleScraper,
        onOpenScraperSettings = viewModel::openScraperSettings,
        onBackRequested = onBackRequested
    )

    val settingsSheet = viewModel.settingsSheet
    if (settingsSheet != null) {
        PluginScraperSettingsSheet(
            scraperName = settingsSheet.scraper.name,
            language = language,
            loading = settingsSheet.loading,
            fields = settingsSheet.fields,
            initialValues = settingsSheet.scraper.settings,
            onDismiss = viewModel::dismissScraperSettings,
            onSave = { values -> viewModel.saveScraperSettings(settingsSheet.scraper.id, values) }
        )
    }
}

@Composable
fun PluginsSettingsScreen(
    state: PluginsUiState,
    language: String?,
    onAddRepository: (String) -> Unit,
    onRemoveRepository: (String) -> Unit,
    onRefreshRepository: (String) -> Unit,
    onToggleScraper: (String, Boolean) -> Unit,
    onOpenScraperSettings: (PluginScraperUiModel) -> Unit,
    onBackRequested: () -> Unit,
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
                PluginRepositoryInput(language = language, onSubmit = onAddRepository)
            }

            if (state.error != null) {
                item {
                    Text(
                        text = AppStrings.format(language, "settings.plugins.error", state.error),
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

            item {
                Text(
                    text = AppStrings.t(language, "settings.plugins.installed_repositories"),
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 16.sp
                )
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
                items(state.repositories, key = { "repo:${it.manifestUrl}" }) { repository ->
                    PluginRepositoryGroup(
                        repository = repository,
                        scrapers = scrapersByRepository[repository.manifestUrl].orEmpty(),
                        language = language,
                        isRefreshing = state.addingRepositoryUrl == repository.manifestUrl,
                        onRemove = { onRemoveRepository(repository.manifestUrl) },
                        onRefresh = { onRefreshRepository(repository.manifestUrl) },
                        onToggleScraper = onToggleScraper,
                        onOpenScraperSettings = onOpenScraperSettings
                    )
                }
            }

            item { Spacer(Modifier.height(80.dp)) }
        }
    }
}

@Composable
private fun PluginRepositoryInput(
    language: String?,
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
                    AppStrings.t(language, "settings.plugins.repository_url_placeholder"),
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
    onOpenScraperSettings: (PluginScraperUiModel) -> Unit
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
                PluginIconButton(icon = Icons.Filled.Refresh, onClick = onRefresh)
            }
            PluginIconButton(icon = Icons.Filled.Close, tint = Color.White.copy(alpha = 0.5f), onClick = onRemove)
        }

        if (scrapers.isNotEmpty()) {
            HorizontalDivider(color = Color.White.copy(alpha = 0.06f), thickness = 0.5.dp)
            scrapers.forEachIndexed { index, scraper ->
                PluginScraperRow(
                    scraper = scraper,
                    language = language,
                    onToggle = { enabled -> onToggleScraper(scraper.id, enabled) },
                    onOpenSettings = { onOpenScraperSettings(scraper) }
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
            PluginIconButton(
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
private fun PluginIconButton(
    icon: ImageVector,
    contentDescription: String? = null,
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
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(
            icon,
            contentDescription = contentDescription,
            tint = if (focused) Color.Black else tint,
            modifier = Modifier.size(18.dp)
        )
    }
}
