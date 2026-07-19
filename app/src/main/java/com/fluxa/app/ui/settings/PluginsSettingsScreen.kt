package com.fluxa.app.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
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
        onToggleScraper = viewModel::toggleScraper,
        onBackRequested = onBackRequested
    )
}

@Composable
fun PluginsSettingsScreen(
    state: PluginsUiState,
    language: String?,
    onAddRepository: (String) -> Unit,
    onRemoveRepository: (String) -> Unit,
    onToggleScraper: (String, Boolean) -> Unit,
    onBackRequested: () -> Unit,
    modifier: Modifier = Modifier
) {
    var showAddDialog by remember { mutableStateOf(false) }

    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        Column(modifier = Modifier.fillMaxSize().padding(horizontal = 20.dp)) {
            PluginsTopBar(title = AppStrings.t(language, "settings.plugins.title"), onBack = onBackRequested)
            Column(modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                PluginsSectionHeader(AppStrings.t(language, "settings.plugins.repositories"))
                PluginsActionRow(AppStrings.t(language, "settings.plugins.add_repository")) { showAddDialog = true }

                if (state.addingRepositoryUrl != null) {
                    Text(
                        AppStrings.t(language, "settings.plugins.adding"),
                        color = Color.White.copy(alpha = 0.6f),
                        fontSize = 13.sp,
                        modifier = Modifier.padding(vertical = 8.dp)
                    )
                }
                state.error?.let { error ->
                    Text(
                        AppStrings.format(language, "settings.plugins.error", error),
                        color = FluxaColors.errorRed,
                        fontSize = 13.sp,
                        modifier = Modifier.padding(vertical = 8.dp)
                    )
                }

                if (state.repositories.isEmpty()) {
                    Text(
                        AppStrings.t(language, "settings.plugins.no_repositories"),
                        color = Color.White.copy(alpha = 0.5f),
                        modifier = Modifier.padding(vertical = 12.dp)
                    )
                } else {
                    state.repositories.forEach { repository ->
                        PluginRepositoryRow(
                            repository = repository,
                            language = language,
                            onRemove = { onRemoveRepository(repository.manifestUrl) }
                        )
                    }
                }

                Spacer(Modifier.height(20.dp))
                PluginsSectionHeader(AppStrings.t(language, "settings.plugins.scrapers"))
                if (state.scrapers.isEmpty()) {
                    Text(
                        AppStrings.t(language, "settings.plugins.no_scrapers"),
                        color = Color.White.copy(alpha = 0.5f),
                        modifier = Modifier.padding(vertical = 12.dp)
                    )
                } else {
                    state.scrapers.forEach { scraper ->
                        PluginScraperRow(scraper = scraper, onToggle = { enabled -> onToggleScraper(scraper.id, enabled) })
                    }
                }
                Spacer(Modifier.height(120.dp))
            }
        }
    }

    if (showAddDialog) {
        PluginAddRepositoryDialog(
            language = language,
            onDismiss = { showAddDialog = false },
            onConfirm = { url ->
                onAddRepository(url)
                showAddDialog = false
            }
        )
    }
}

@Composable
private fun PluginsTopBar(title: String, onBack: () -> Unit) {
    Row(modifier = Modifier.fillMaxWidth().padding(vertical = 16.dp), verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier.size(40.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.05f)).clickable(onClick = onBack),
            contentAlignment = Alignment.Center
        ) {
            Text("←", color = Color.White)
        }
        Spacer(Modifier.width(12.dp))
        Text(title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 22.sp)
    }
}

@Composable
private fun PluginsSectionHeader(title: String) {
    Text(
        title.uppercase(),
        color = Color.White.copy(alpha = 0.4f),
        fontSize = 12.sp,
        fontWeight = FontWeight.Bold,
        modifier = Modifier.padding(top = 16.dp, bottom = 8.dp)
    )
}

@Composable
private fun PluginsActionRow(label: String, onClick: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().clickable(onClick = onClick).padding(vertical = 12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(label, color = Color.White)
        Text("+", color = Color.White.copy(alpha = 0.6f), fontSize = 18.sp)
    }
}

@Composable
private fun PluginRepositoryRow(
    repository: PluginRepositoryUiModel,
    language: String?,
    onRemove: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 6.dp)
            .clip(RoundedCornerShape(10.dp))
            .background(Color.White.copy(alpha = 0.05f))
            .padding(horizontal = 14.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(repository.name.ifBlank { repository.manifestUrl }, color = Color.White, fontWeight = FontWeight.Bold)
            Text(
                AppStrings.format(language, "settings.plugins.scraper_count", repository.scraperCount),
                color = Color.White.copy(alpha = 0.5f),
                fontSize = 12.sp
            )
        }
        Text(
            AppStrings.t(language, "settings.plugins.remove"),
            color = FluxaColors.errorRed,
            fontSize = 13.sp,
            modifier = Modifier.clickable(onClick = onRemove).padding(8.dp)
        )
    }
}

@Composable
private fun PluginScraperRow(scraper: PluginScraperUiModel, onToggle: (Boolean) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(scraper.name, color = Color.White)
            Text(scraper.supportedTypes.joinToString(", "), color = Color.White.copy(alpha = 0.4f), fontSize = 11.sp)
        }
        Switch(
            checked = scraper.enabled,
            onCheckedChange = onToggle,
            colors = SwitchDefaults.colors(
                checkedTrackColor = Color.White,
                checkedThumbColor = Color.Black,
                uncheckedTrackColor = Color.White.copy(alpha = 0.15f),
                uncheckedThumbColor = Color.White.copy(alpha = 0.7f)
            )
        )
    }
}

@Composable
private fun PluginAddRepositoryDialog(
    language: String?,
    onDismiss: () -> Unit,
    onConfirm: (String) -> Unit
) {
    var url by remember { mutableStateOf("") }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(AppStrings.t(language, "settings.plugins.add_repository")) },
        text = {
            TextField(
                value = url,
                onValueChange = { url = it },
                placeholder = { Text(AppStrings.t(language, "settings.plugins.repository_url_placeholder")) },
                label = { Text(AppStrings.t(language, "settings.plugins.repository_url")) },
                singleLine = true,
                colors = TextFieldDefaults.colors(
                    focusedContainerColor = Color.White.copy(alpha = 0.05f),
                    unfocusedContainerColor = Color.White.copy(alpha = 0.05f),
                    focusedTextColor = Color.White,
                    unfocusedTextColor = Color.White
                )
            )
        },
        confirmButton = {
            TextButton(onClick = { onConfirm(url) }, enabled = url.isNotBlank()) {
                Text(AppStrings.t(language, "settings.plugins.add"))
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(AppStrings.t(language, "settings.plugins.cancel"))
            }
        }
    )
}
