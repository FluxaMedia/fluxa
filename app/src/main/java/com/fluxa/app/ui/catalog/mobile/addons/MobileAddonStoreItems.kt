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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
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
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import java.net.URLEncoder
import java.nio.charset.StandardCharsets

@Composable
fun MobileInstalledAddonItem(
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
    var isFocused by remember { mutableStateOf(false) }
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(18.dp))
            .background(
                when {
                    !isEnabled -> Color.White.copy(alpha = 0.025f)
                    isFocused -> Color.White.copy(alpha = 0.08f)
                    else -> Color.White.copy(alpha = 0.04f)
                }
            )
            .border(1.dp, Color.White.copy(alpha = if (isFocused) 0.18f else 0.06f), RoundedCornerShape(18.dp))
            .onFocusChanged { isFocused = it.isFocused }
            .padding(horizontal = 18.dp, vertical = 16.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(end = 56.dp),
            verticalAlignment = Alignment.Top
        ) {
            Box(
                modifier = Modifier
                    .size(70.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(Color.White.copy(alpha = 0.08f)),
                contentAlignment = Alignment.Center
            ) {
                var logoFailed by remember(addon.logoUrl) { mutableStateOf(false) }
                if (!addon.logoUrl.isNullOrBlank() && !logoFailed) {
                    AsyncImage(
                        model = addon.logoUrl,
                        contentDescription = addon.name,
                        modifier = Modifier
                            .size(58.dp)
                            .clip(RoundedCornerShape(8.dp)),
                        contentScale = ContentScale.Fit,
                        onError = { logoFailed = true }
                    )
                } else {
                    Icon(addon.fallbackIcon, null, tint = Color.White.copy(alpha = 0.78f), modifier = Modifier.size(34.dp))
                }
            }
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
                AddonVersionBadge(version = addon.version, lang = lang)
                if (canRemove) {
                    Spacer(Modifier.height(8.dp))
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(2.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        if (onConfigure != null) {
                            IconButton(onClick = onConfigure) {
                                Icon(FluxaIcons.Settings, null, tint = Color.White.copy(alpha = 0.68f), modifier = Modifier.size(20.dp))
                            }
                        }
                        IconButton(onClick = onRefresh, enabled = !isRefreshing) {
                            if (isRefreshing) {
                                CircularProgressIndicator(color = Color.White.copy(alpha = 0.68f), strokeWidth = 2.dp, modifier = Modifier.size(18.dp))
                            } else {
                                Icon(FluxaIcons.Refresh, null, tint = Color.White.copy(alpha = 0.68f), modifier = Modifier.size(20.dp))
                            }
                        }
                        IconButton(onClick = onMoveUp, enabled = canMoveUp) {
                            Icon(FluxaIcons.ArrowUp, null, tint = Color.White.copy(alpha = if (canMoveUp) 0.68f else 0.22f), modifier = Modifier.size(24.dp))
                        }
                        IconButton(onClick = onMoveDown, enabled = canMoveDown) {
                            Icon(FluxaIcons.ArrowDown, null, tint = Color.White.copy(alpha = if (canMoveDown) 0.68f else 0.22f), modifier = Modifier.size(24.dp))
                        }
                        IconButton(onClick = onRemove) {
                            Icon(FluxaIcons.Delete, null, tint = Color.White.copy(alpha = 0.62f), modifier = Modifier.size(20.dp))
                        }
                    }
                } else {
                    Spacer(Modifier.height(4.dp))
                    Text(
                        text = AppStrings.t(lang, "auto.account"),
                        color = Color.White.copy(alpha = 0.52f),
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Bold
                    )
                }
                Spacer(Modifier.height(8.dp))
                Text(
                    text = addon.description,
                    color = Color.White.copy(alpha = 0.48f),
                    fontSize = 12.sp,
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis
                )
                Spacer(Modifier.height(10.dp))
                AddonManifestSummary(addon = addon, lang = lang, isEnabled = isEnabled)
            }
        }
        if (canRemove) {
            Switch(
                checked = isEnabled,
                onCheckedChange = onToggleEnabled,
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .height(40.dp),
                colors = SwitchDefaults.colors(
                    checkedThumbColor = Color.White,
                    checkedTrackColor = accentColor,
                    checkedBorderColor = Color.Transparent,
                    uncheckedThumbColor = Color.White.copy(alpha = 0.72f),
                    uncheckedTrackColor = Color.White.copy(alpha = 0.18f),
                    uncheckedBorderColor = Color.White.copy(alpha = 0.18f)
                )
            )
        }
    }
}

@Composable
fun MobileAddonStoreItem(
    addon: CommunityAddon,
    lang: String,
    isInstalled: Boolean,
    onInstall: () -> Unit
) {
    var isFocused by remember { mutableStateOf(false) }
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(18.dp))
            .background(if (isFocused) Color.White.copy(alpha = 0.08f) else Color.White.copy(alpha = 0.04f))
            .border(1.dp, Color.White.copy(alpha = if (isFocused) 0.18f else 0.06f), RoundedCornerShape(18.dp))
            .onFocusChanged { isFocused = it.isFocused }
            .clickable { if (!isInstalled) onInstall() }
            .heightIn(min = 176.dp)
            .padding(horizontal = 18.dp, vertical = 16.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.Top
        ) {
            Box(
                modifier = Modifier
                    .size(84.dp)
                    .clip(RoundedCornerShape(16.dp))
                    .background(Color.White.copy(alpha = 0.08f)),
                contentAlignment = Alignment.Center
            ) {
                var logoFailed by remember(addon.logoUrl) { mutableStateOf(false) }
                if (!addon.logoUrl.isNullOrBlank() && !logoFailed) {
                    AsyncImage(
                        model = addon.logoUrl,
                        contentDescription = addon.name,
                        modifier = Modifier
                            .size(70.dp)
                            .clip(RoundedCornerShape(12.dp)),
                        contentScale = ContentScale.Fit,
                        onError = { logoFailed = true }
                    )
                } else {
                    Icon(addon.fallbackIcon, null, tint = Color.White, modifier = Modifier.size(36.dp))
                }
            }

            Spacer(Modifier.width(14.dp))

            Column(modifier = Modifier.weight(1f)) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        text = addon.name,
                        color = Color.White,
                        fontWeight = FontWeight.Bold,
                        fontSize = 16.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f)
                    )
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(999.dp))
                            .background(if (isInstalled) Color(0xFF1B5E20).copy(alpha = 0.22f) else Color.White.copy(alpha = 0.10f))
                            .padding(horizontal = 12.dp, vertical = 6.dp)
                    ) {
                        Text(
                            text = if (isInstalled) AppStrings.t(lang, "auto.installed") else AppStrings.t(lang, "auto.install"),
                            color = if (isInstalled) Color(0xFF7DFF9B) else Color.White,
                            fontWeight = FontWeight.Bold,
                            fontSize = 12.sp
                        )
                    }
                }
                AddonVersionBadge(version = addon.version, lang = lang)
                Spacer(Modifier.height(4.dp))
                Text(
                    text = addon.description,
                    color = Color.White.copy(alpha = 0.58f),
                    fontSize = 13.sp,
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis
                )
                Spacer(Modifier.height(10.dp))
                AddonManifestSummary(addon = addon, lang = lang)
            }
        }
    }
}

@Composable
fun MobileCS3RepoItem(
    repoName: String,
    repoUrl: String,
    repoIconUrl: String? = null,
    lang: String,
    onRemove: () -> Unit,
    onClick: () -> Unit
) {
    var isFocused by remember { mutableStateOf(false) }
    var iconFailed by remember(repoIconUrl) { mutableStateOf(false) }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .border(1.dp, Color.White.copy(alpha = 0.08f), RoundedCornerShape(14.dp))
            .onFocusChanged { isFocused = it.isFocused }
            .clickable { onClick() }
            .padding(horizontal = 14.dp, vertical = 12.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(Color.White.copy(alpha = 0.07f)),
                contentAlignment = Alignment.Center
            ) {
                if (!repoIconUrl.isNullOrBlank() && !iconFailed) {
                    AsyncImage(
                        model = repoIconUrl,
                        contentDescription = repoName,
                        modifier = Modifier
                            .size(28.dp)
                            .clip(RoundedCornerShape(6.dp)),
                        contentScale = ContentScale.Fit,
                        onError = { iconFailed = true }
                    )
                } else {
                    Icon(
                        FluxaIcons.Cloud,
                        null,
                        tint = Color.White.copy(alpha = 0.5f),
                        modifier = Modifier.size(18.dp)
                    )
                }
            }

            Spacer(Modifier.width(12.dp))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = repoName.takeIf { it.isNotBlank() } ?: AppStrings.t(lang, "addons.cloudstream_repo"),
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 14.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = repoUrl,
                    color = Color.White.copy(alpha = 0.5f),
                    fontSize = 11.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }

            IconButton(onClick = onRemove) {
                Icon(
                    FluxaIcons.Delete,
                    null,
                    tint = Color.White.copy(alpha = 0.5f),
                    modifier = Modifier.size(18.dp)
                )
            }
        }
    }
}

@Composable
fun MobileCS3PluginItem(
    plugin: PluginInfo,
    lang: String,
    isInstalling: Boolean,
    isInstalled: Boolean = false,
    onInstall: () -> Unit
) {
    val accentColor = if (isInstalled) Color(0xFF4CAF50) else Color.White.copy(alpha = 0.6f)
    var iconLoadFailed by remember(plugin.iconUrl) { mutableStateOf(false) }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = !isInstalling) { onInstall() }
            .padding(horizontal = 4.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .size(42.dp)
                .clip(RoundedCornerShape(10.dp))
                .background(Color.White.copy(alpha = 0.07f)),
            contentAlignment = Alignment.Center
        ) {
            if (!plugin.iconUrl.isNullOrBlank() && !iconLoadFailed) {
                AsyncImage(
                    model = plugin.iconUrl,
                    contentDescription = plugin.name,
                    modifier = Modifier
                        .size(34.dp)
                        .clip(RoundedCornerShape(8.dp)),
                    contentScale = ContentScale.Fit,
                    onError = { iconLoadFailed = true }
                )
            } else {
                Icon(
                    FluxaIcons.Extension,
                    null,
                    tint = accentColor,
                    modifier = Modifier.size(20.dp)
                )
            }
        }

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
                if (plugin.tvTypes.isNotEmpty()) {
                    if (isNotEmpty()) append("  ·  ")
                    append(plugin.getDisplayTypes())
                }
            }
            if (subtitle.isNotBlank()) {
                Text(
                    text = subtitle,
                    color = Color.White.copy(alpha = 0.45f),
                    fontSize = 11.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }

        Spacer(Modifier.width(10.dp))

        if (isInstalling) {
            CircularProgressIndicator(
                color = accentColor,
                strokeWidth = 2.dp,
                modifier = Modifier.size(20.dp)
            )
        } else {
            Icon(
                imageVector = if (isInstalled) FluxaIcons.Delete else FluxaIcons.Download,
                contentDescription = null,
                tint = if (isInstalled) Color(0xFFFF6B6B) else Color.White.copy(alpha = 0.5f),
                modifier = Modifier.size(22.dp)
            )
        }
    }
}

@Composable
fun MobileCS3InstalledPluginItem(
    plugin: com.fluxa.app.plugins.cloudstream.InstalledPlugin,
    lang: String,
    onRemove: () -> Unit
) {
    var isFocused by remember { mutableStateOf(false) }
    
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(Color(0xFF4CAF50).copy(alpha = 0.1f))
            .border(1.dp, Color(0xFF4CAF50).copy(alpha = 0.2f), RoundedCornerShape(14.dp))
            .onFocusChanged { isFocused = it.isFocused }
            .padding(horizontal = 14.dp, vertical = 12.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(RoundedCornerShape(10.dp))
                    .background(Color(0xFF4CAF50).copy(alpha = 0.2f)),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    FluxaIcons.Extension,
                    null,
                    tint = Color(0xFF4CAF50),
                    modifier = Modifier.size(20.dp)
                )
            }
            
            Spacer(Modifier.width(12.dp))
            
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = plugin.name,
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 14.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = "v${plugin.version}  ${plugin.internalName}",
                    color = Color.White.copy(alpha = 0.5f),
                    fontSize = 11.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
            
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(Color(0xFF4CAF50).copy(alpha = 0.2f))
                    .padding(horizontal = 10.dp, vertical = 6.dp)
            ) {
                Text(
                    text = AppStrings.t(lang, "auto.installed"),
                    color = Color(0xFF7DFF9B),
                    fontWeight = FontWeight.Bold,
                    fontSize = 11.sp
                )
            }
            
            Spacer(Modifier.width(8.dp))
            
            IconButton(onClick = onRemove) {
                Icon(
                    FluxaIcons.Delete,
                    null,
                    tint = Color.White.copy(alpha = 0.5f),
                    modifier = Modifier.size(18.dp)
                )
            }
        }
    }
}
