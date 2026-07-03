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
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.plugins.cloudstream.PluginInfo
import com.fluxa.app.plugins.cloudstream.RepositoryManifest
import com.fluxa.app.data.repository.HttpRequestSecurity
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import java.io.IOException
import java.net.URLEncoder
import java.nio.charset.StandardCharsets
import java.util.Locale
import java.util.concurrent.ConcurrentHashMap

@Composable
internal fun AddonRepoPluginsDialog(
    selectedRepoUrl: String?,
    selectedRepoName: String? = null,
    selectedRepoPlugins: List<PluginInfo>,
    isLoadingRepoPlugins: Boolean,
    pluginManager: PluginManager,
    lang: String,
    isInstalling: Boolean,
    onDismiss: () -> Unit,
    onError: (String?) -> Unit,
    onInstalledPluginsChanged: () -> Unit
) {
    val scope = rememberCoroutineScope()

    // Track installed state reactively inside the dialog
    var installedNames by remember(selectedRepoUrl) {
        mutableStateOf(pluginManager.installedPlugins.value.map { it.internalName }.toSet())
    }
    // Track which plugins are currently being installed
    var installingNames by remember(selectedRepoUrl) { mutableStateOf(setOf<String>()) }
    // Error shown inside the dialog
    var dialogError by remember(selectedRepoUrl) { mutableStateOf<String?>(null) }

    if (selectedRepoUrl != null) {
        AlertDialog(
            onDismissRequest = onDismiss,
            title = {
                Text(
                    text = selectedRepoName ?: AppStrings.t(lang, "auto.repository_plugins"),
                    color = Color.White,
                    fontWeight = FontWeight.Bold
                )
            },
            text = {
                LazyColumn(modifier = Modifier.height(400.dp)) {
                    if (isLoadingRepoPlugins) {
                        item {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.Center
                            ) {
                                CircularProgressIndicator(
                                    color = Color.White,
                                    modifier = Modifier.size(32.dp)
                                )
                            }
                        }
                    } else if (selectedRepoPlugins.isEmpty()) {
                        item {
                            Text(
                                text = AppStrings.t(lang, "auto.no_plugins_found_in_this_repository"),
                                color = Color.White.copy(alpha = 0.6f)
                            )
                        }
                    } else {
                        // In-dialog error message
                        if (dialogError != null) {
                            item {
                                Text(
                                    text = dialogError!!,
                                    color = Color(0xFFFF6B6B),
                                    fontSize = 13.sp,
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(bottom = 8.dp)
                                )
                            }
                        }

                        itemsIndexed(selectedRepoPlugins, key = { _, p -> p.internalName }) { index, plugin ->
                            val installed = plugin.internalName in installedNames
                            val installing = plugin.internalName in installingNames

                            CS3PluginItem(
                                plugin = plugin,
                                lang = lang,
                                isInstalling = installing,
                                isInstalled = installed,
                                onInstall = {
                                    if (installing) return@CS3PluginItem
                                    scope.launch {
                                        dialogError = null
                                        installingNames = installingNames + plugin.internalName
                                        if (installed) {
                                            pluginManager.uninstallPlugin(plugin.internalName)
                                            installedNames = installedNames - plugin.internalName
                                            onInstalledPluginsChanged()
                                        } else {
                                            val result = pluginManager.installPlugin(plugin, selectedRepoUrl)
                                            if (result.isSuccess) {
                                                installedNames = installedNames + plugin.internalName
                                                onInstalledPluginsChanged()
                                            } else {
                                                dialogError = result.exceptionOrNull()?.message
                                                    ?: AppStrings.t(lang, "auto.install_failed")
                                                onError(dialogError)
                                            }
                                        }
                                        installingNames = installingNames - plugin.internalName
                                    }
                                }
                            )
                            if (index < selectedRepoPlugins.lastIndex) {
                                androidx.compose.material3.HorizontalDivider(
                                    color = Color.White.copy(alpha = 0.07f),
                                    thickness = 0.5.dp
                                )
                            }
                        }
                    }
                }
            },
            confirmButton = {
                TextButton(onClick = onDismiss) {
                    Text(
                        text = AppStrings.t(lang, "auto.close"),
                        color = Color.White.copy(alpha = 0.7f)
                    )
                }
            },
            containerColor = Color(0xFF1A1D26)
        )
    }
}
