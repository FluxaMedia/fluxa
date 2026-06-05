@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.ui.catalog

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
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
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.fluxa.app.data.local.OfflineDownloadItem
import com.fluxa.app.data.local.UserProfile
import java.util.Locale

@Composable
internal fun MobileSettingsSection(
    icon: ImageVector,
    title: String,
    content: @Composable ColumnScope.() -> Unit
) {
    Column(verticalArrangement = Arrangement.spacedBy(18.dp)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Icon(icon, null, tint = Color.White.copy(alpha = 0.48f), modifier = Modifier.size(20.dp))
            Text(
                text = title,
                color = Color.White.copy(alpha = 0.64f),
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold
            )
        }
        Column(verticalArrangement = Arrangement.spacedBy(20.dp), content = content)
    }
}

@Composable
internal fun MobileSettingsGroup(
    title: String?,
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    Column(
        modifier = modifier.animateContentSize(animationSpec = tween(240)),
        verticalArrangement = Arrangement.spacedBy(7.dp)
    ) {
        title?.let {
            Text(
                text = it,
                color = colors.mutedText,
                fontSize = 10.sp,
                fontWeight = FontWeight.Black,
                modifier = Modifier.padding(start = 4.dp)
            )
        }
        MobileSettingsCard(content = content)
    }
}

@Composable
internal fun OfflineDownloadRow(
    item: OfflineDownloadItem,
    lang: String,
    onCancel: () -> Unit,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    var showDeleteConfirm by remember(item.id) { mutableStateOf(false) }
    val statusText = when (item.status) {
        "downloaded" -> AppStrings.t(lang, "downloads.status_downloaded")
        "failed" -> AppStrings.t(lang, "downloads.status_failed")
        "paused" -> AppStrings.t(lang, "downloads.status_paused")
        "downloading" -> AppStrings.format(lang, "downloads.status_downloading", item.progress)
        else -> AppStrings.t(lang, "downloads.status_queued")
    }
    val transferText = if (item.status == "downloading") item.speedEtaLabel(lang) else null
    val artwork = if (item.metaType == "series") item.background ?: item.poster else item.poster ?: item.background
    val cardAlpha by animateFloatAsState(
        targetValue = if (item.status == "failed") 0.72f else 1f,
        animationSpec = tween(220),
        label = "downloadAlpha"
    )
    Column(
        modifier = modifier
            .fillMaxWidth()
            .graphicsLayer { alpha = cardAlpha }
            .animateContentSize(animationSpec = tween(260))
            .clip(RoundedCornerShape(14.dp))
            .background(Color.White.copy(alpha = 0.07f))
            .clickable(enabled = item.isPlayable) { onClick() }
            .padding(14.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Row(
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .width(if (item.metaType == "series") 92.dp else 58.dp)
                    .height(82.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(Color.White.copy(alpha = 0.08f)),
                contentAlignment = Alignment.Center
            ) {
                if (!artwork.isNullOrBlank()) {
                    AsyncImage(
                        model = artwork,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop
                    )
                } else {
                    Icon(FluxaIcons.Download, null, tint = Color.White.copy(alpha = 0.38f), modifier = Modifier.size(24.dp))
                }
            }
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(7.dp)
            ) {
                Text(
                    text = listOfNotNull(item.title, item.episodeTitle).joinToString(" - "),
                    color = Color.White,
                    fontSize = 15.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = listOfNotNull(statusText, item.downloadSizeLabel(), transferText).joinToString("  "),
                    color = Color.White.copy(alpha = 0.62f),
                    fontSize = 12.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                item.streamTitle?.takeIf { it.isNotBlank() }?.let { source ->
                    Text(
                        text = AppStrings.format(lang, "downloads.video_source", source),
                        color = Color.White.copy(alpha = 0.52f),
                        fontSize = 11.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
                item.subtitleLabel?.let { label ->
                    Text(
                        text = AppStrings.format(lang, "downloads.subtitle_label", label),
                        color = Color.White.copy(alpha = 0.45f),
                        fontSize = 11.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
                if (item.status != "downloaded") {
                    LinearProgressIndicator(
                        progress = { item.progress.coerceIn(0, 100) / 100f },
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(4.dp)
                            .clip(CircleShape),
                        color = Color.White,
                        trackColor = Color.White.copy(alpha = 0.12f)
                    )
                }
            }
        }
        Row(
            modifier = Modifier.align(Alignment.End),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            if (item.status != "downloaded") {
                TextButton(onClick = onCancel) {
                    Text(AppStrings.t(lang, "downloads.cancel"), color = Color.White)
                }
            }
            TextButton(onClick = { showDeleteConfirm = true }) {
                Text(AppStrings.t(lang, "downloads.delete"), color = Color.White)
            }
        }
    }

    if (showDeleteConfirm) {
        AlertDialog(
            onDismissRequest = { showDeleteConfirm = false },
            title = { Text(AppStrings.t(lang, "downloads.delete_confirm_title")) },
            text = { Text(AppStrings.t(lang, "downloads.delete_confirm_message")) },
            confirmButton = {
                TextButton(
                    onClick = {
                        showDeleteConfirm = false
                        onCancel()
                    }
                ) {
                    Text(AppStrings.t(lang, "downloads.delete"))
                }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteConfirm = false }) {
                    Text(AppStrings.t(lang, "auto.cancel"))
                }
            }
        )
    }
}

private fun OfflineDownloadItem.speedEtaLabel(lang: String): String? {
    val speed = speedBytesPerSecond.takeIf { it > 0L }?.let { "${it.formatBytes()}/s" }
    val eta = etaSeconds.takeIf { it >= 0L }?.let {
        val minutes = (it / 60L + 1L).coerceAtLeast(1L)
        AppStrings.format(lang, "format.remaining_minutes", minutes)
    }
    return listOfNotNull(speed, eta).joinToString("  ").takeIf { it.isNotBlank() }
}

private fun OfflineDownloadItem.downloadSizeLabel(): String? {
    return when {
        totalBytes > 0L -> "${downloadedBytes.formatBytes()} / ${totalBytes.formatBytes()}"
        downloadedBytes > 0L -> downloadedBytes.formatBytes()
        else -> null
    }
}

private fun Long.formatBytes(): String {
    val value = this.toDouble()
    return when {
        this >= 1024L * 1024L * 1024L -> String.format(Locale.US, "%.1f GB", value / (1024.0 * 1024.0 * 1024.0))
        this >= 1024L * 1024L -> String.format(Locale.US, "%.0f MB", value / (1024.0 * 1024.0))
        this >= 1024L -> String.format(Locale.US, "%.0f KB", value / 1024.0)
        else -> "$this B"
    }
}

@Composable
internal fun MobileCollapsibleSettingsGroup(
    title: String,
    count: Int,
    expanded: Boolean,
    onToggleExpanded: () -> Unit,
    content: @Composable ColumnScope.() -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    Column(verticalArrangement = Arrangement.spacedBy(7.dp)) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(8.dp))
                .clickable { onToggleExpanded() }
                .padding(start = 4.dp, end = 4.dp, top = 2.dp, bottom = 2.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = title,
                color = colors.mutedText,
                fontSize = 10.sp,
                fontWeight = FontWeight.Black,
                modifier = Modifier.weight(1f)
            )
            Text(
                text = count.toString(),
                color = colors.mutedText.copy(alpha = 0.62f),
                fontSize = 10.sp,
                fontWeight = FontWeight.Black
            )
            Icon(
                if (expanded) FluxaIcons.KeyboardArrowUp else FluxaIcons.KeyboardArrowDown,
                null,
                tint = colors.mutedText.copy(alpha = 0.72f),
                modifier = Modifier.size(18.dp)
            )
        }
        AnimatedVisibility(visible = expanded) {
            MobileSettingsCard(content = content)
        }
    }
}

@Composable
internal fun MobileThemeSelector(
    profile: UserProfile,
    lang: String,
    onUpdateProfile: (UserProfile) -> Unit
) {
    val accent = Color(profile.safeAccentColorArgb)
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        listOf(
            Triple("system", AppStrings.t(lang, "auto.system"), Color(0xFF151923)),
            Triple("light", AppStrings.t(lang, "auto.light"), Color(0xFFE9EDF3)),
            Triple("dark", AppStrings.t(lang, "auto.dark"), Color(0xFF050608))
        ).forEach { (value, label, color) ->
            val selected = profile.safeAppTheme == value
            Column(
                modifier = Modifier
                    .weight(1f)
                    .height(82.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .clickable { onUpdateProfile(profile.copy(appTheme = value)) }
                    .background(Color.White.copy(alpha = 0.04f))
                    .border(
                        width = if (selected) 3.dp else 1.dp,
                        color = if (selected) accent else Color.White.copy(alpha = 0.06f),
                        shape = RoundedCornerShape(8.dp)
                    )
                    .padding(5.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.SpaceBetween
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .weight(1f)
                        .clip(RoundedCornerShape(6.dp))
                        .background(color)
                ) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.TopEnd)
                            .padding(6.dp)
                            .size(14.dp)
                            .clip(RoundedCornerShape(3.dp))
                            .background(if (color == Color(0xFFE9EDF3)) Color.White else Color.White.copy(alpha = 0.16f))
                    )
                    if (selected) {
                        Box(
                            modifier = Modifier
                                .align(Alignment.BottomCenter)
                                .fillMaxWidth()
                                .height(5.dp)
                                .background(accent)
                        )
                        Icon(
                            FluxaIcons.Check,
                            contentDescription = null,
                            tint = if (accent == Color.White) Color.Black else Color.White,
                            modifier = Modifier
                                .align(Alignment.Center)
                                .size(22.dp)
                                .clip(CircleShape)
                                .background(accent)
                                .padding(4.dp)
                        )
                    }
                }
                Text(
                    label,
                    color = if (selected) accent else Color.White.copy(alpha = 0.86f),
                    fontSize = 9.sp,
                    fontWeight = FontWeight.Black
                )
            }
        }
    }
}

@Composable
internal fun MobileAccentDots(profile: UserProfile, onUpdateProfile: (UserProfile) -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 14.dp),
        horizontalArrangement = Arrangement.spacedBy(13.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        listOf(
            Color.White,
            Color(0xFF3F7CFF),
            Color(0xFF35C2A0),
            Color(0xFFFF9D42),
            Color(0xFFFF5D5D),
            Color(0xFFFF4DA0)
        ).forEach { color ->
            val colorArgb = color.toArgb()
            val selected = profile.safeAccentColorArgb == colorArgb
            Box(
                modifier = Modifier
                    .size(if (selected) 32.dp else 24.dp)
                    .clip(CircleShape)
                    .clickable { onUpdateProfile(profile.copy(accentColorArgb = colorArgb)) }
                    .border(
                        width = if (selected) 3.dp else 0.dp,
                        color = if (color == Color.White) Color(0xFF8F5CFF) else Color.White.copy(alpha = 0.88f),
                        shape = CircleShape
                    )
                    .padding(if (selected) 4.dp else 0.dp)
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .clip(CircleShape)
                        .background(color)
                )
                if (selected) {
                    Icon(
                        FluxaIcons.Check,
                        contentDescription = null,
                        tint = if (color == Color.White) Color.Black else Color.White,
                        modifier = Modifier
                            .align(Alignment.Center)
                            .size(15.dp)
                    )
                }
            }
        }
    }
}

@Composable
internal fun MobileAppearancePreview(profile: UserProfile, lang: String) {
    val accent = Color(profile.safeAccentColorArgb)
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(12.dp)
            .height(100.dp)
            .clip(RoundedCornerShape(mobileCornerRadius(profile.safeCardCornerPreset)))
            .background(
                Brush.linearGradient(
                    listOf(Color(0xFF222B1F), Color(0xFF171B14), Color(0xFF0B0D0A))
                )
            )
            .border(1.dp, Color.White.copy(alpha = 0.07f), RoundedCornerShape(mobileCornerRadius(profile.safeCardCornerPreset)))
            .padding(12.dp),
        verticalArrangement = Arrangement.Bottom
    ) {
        Text(AppStrings.t(lang, "preview.sample_title"), color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.Black)
        Text(AppStrings.t(lang, "auto.season_1_episode_1"), color = Color.White.copy(alpha = 0.56f), fontSize = 9.sp)
        Spacer(Modifier.height(10.dp))
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(3.dp)
                .clip(CircleShape)
                .background(Color.White.copy(alpha = 0.18f))
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth(0.72f)
                    .height(3.dp)
                    .clip(CircleShape)
                    .background(accent)
            )
        }
    }
}

internal fun toggleTopTenFeed(selectedKeys: Set<String>, key: String): List<String> {
    return if (selectedKeys.contains(key)) {
        selectedKeys - key
    } else {
        selectedKeys + key
    }.toList()
}
