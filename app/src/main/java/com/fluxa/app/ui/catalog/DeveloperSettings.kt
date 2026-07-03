@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.Alignment
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.player.LastMediaDebugInfoStore

@Composable
internal fun DeveloperSettings(lang: String) {
    val context = LocalContext.current
    val clipboard = remember(context) {
        context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    }
    LaunchedEffect(context) {
        LastMediaDebugInfoStore.initialize(context)
    }
    val debugInfo by LastMediaDebugInfoStore.state.collectAsState()
    val updatedAt = LastMediaDebugInfoStore.formattedUpdatedAt(debugInfo.updatedAtMs)
    val sections = debugInfo.technicalInfo.debugSections()
    SettingsSection(
        title = AppStrings.t(lang, "settings.developer"),
        subtitle = AppStrings.t(lang, "settings.developer_desc")
    ) {
        SettingsInfoTile(
            title = AppStrings.t(lang, "settings.last_media_probe_updated"),
            value = updatedAt.ifBlank { AppStrings.t(lang, "settings.no_media_probe") },
            icon = FluxaIcons.Memory
        )
        if (debugInfo.title.isNotBlank()) {
            SettingsInfoTile(
                title = AppStrings.t(lang, "settings.last_media_probe_title"),
                value = debugInfo.title,
                icon = FluxaIcons.Storage
            )
        }
        if (debugInfo.url.isNotBlank()) {
            SettingsDebugTextBlock(
                title = AppStrings.t(lang, "settings.last_media_probe_url"),
                value = debugInfo.url
            )
        }
        if (debugInfo.technicalInfo.isBlank()) {
            SettingsDebugTextBlock(
                title = AppStrings.t(lang, "settings.last_media_probe_report"),
                value = AppStrings.t(lang, "settings.no_media_probe")
            )
        } else {
            SettingsProbeReportCard(
                title = AppStrings.t(lang, "settings.media_file_data"),
                value = sections.fileData
            )
            SettingsProbeReportCard(
                title = AppStrings.t(lang, "settings.player_runtime_data"),
                value = sections.playerData
            )
            if (sections.dvProxyData.isNotBlank()) {
                SettingsProbeReportCard(
                    title = AppStrings.t(lang, "settings.dv_proxy_debug"),
                    value = sections.dvProxyData
                )
            }
        }
        if (debugInfo.hasInfo) {
            SettingsActionTile(
                title = AppStrings.t(lang, "settings.copy_debug_info"),
                subtitle = AppStrings.t(lang, "settings.copy_debug_info_desc"),
                icon = FluxaIcons.Copy,
                onClick = {
                    clipboard.setPrimaryClip(
                        ClipData.newPlainText(
                            AppStrings.t(lang, "settings.copy_debug_info"),
                            debugInfo.technicalInfo
                        )
                    )
                }
            )
        }
    }
}

@Composable
private fun SettingsProbeReportCard(title: String, value: String) {
    SettingsPanel {
        ProbeReportCard(
            title = title,
            value = value,
            modifier = Modifier.fillMaxWidth(),
            background = Color.White.copy(alpha = 0.075f),
            border = Color.White.copy(alpha = 0.12f),
            header = Color.White.copy(alpha = 0.72f),
            label = Color.White.copy(alpha = 0.54f),
            text = Color.White.copy(alpha = 0.90f),
            valueHighlight = Color(0xFF9BFFD1)
        )
    }
}

@Composable
private fun SettingsDebugTextBlock(title: String, value: String) {
    SettingsPanel {
        Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 16.sp)
            SelectionContainer {
                Text(
                    text = value,
                    color = Color.White.copy(alpha = 0.74f),
                    fontSize = 12.sp,
                    lineHeight = 16.sp,
                    fontFamily = FontFamily.Monospace,
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(max = 420.dp)
                        .verticalScroll(rememberScrollState())
                        .clip(RoundedCornerShape(12.dp))
                        .background(Color.Black.copy(alpha = 0.28f))
                        .border(1.dp, Color.White.copy(alpha = 0.06f), RoundedCornerShape(12.dp))
                        .padding(12.dp)
                )
            }
        }
    }
}

@Composable
internal fun MobileDeveloperSettings(lang: String) {
    val context = LocalContext.current
    val clipboard = remember(context) {
        context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    }
    LaunchedEffect(context) {
        LastMediaDebugInfoStore.initialize(context)
    }
    val debugInfo by LastMediaDebugInfoStore.state.collectAsState()
    val updatedAt = LastMediaDebugInfoStore.formattedUpdatedAt(debugInfo.updatedAtMs)
    val sections = debugInfo.technicalInfo.debugSections()
    Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
        MobileSettingsGroup(AppStrings.t(lang, "settings.last_media_probe")) {
            MobileDebugInfoRow(
                title = AppStrings.t(lang, "settings.last_media_probe_updated"),
                value = updatedAt.ifBlank { AppStrings.t(lang, "settings.no_media_probe") }
            )
            if (debugInfo.title.isNotBlank()) {
                MobileDebugInfoRow(AppStrings.t(lang, "settings.last_media_probe_title"), debugInfo.title)
            }
            if (debugInfo.url.isNotBlank()) {
                MobileDebugTextBlock(AppStrings.t(lang, "settings.last_media_probe_url"), debugInfo.url)
            }
        }
        MobileSettingsGroup(AppStrings.t(lang, "settings.last_media_probe_report")) {
            if (debugInfo.technicalInfo.isBlank()) {
                MobileDebugTextBlock(
                    title = AppStrings.t(lang, "settings.last_media_probe_report"),
                    value = AppStrings.t(lang, "settings.no_media_probe")
                )
            } else {
                MobileProbeReportCard(
                    title = AppStrings.t(lang, "settings.media_file_data"),
                    value = sections.fileData
                )
                MobileProbeReportCard(
                    title = AppStrings.t(lang, "settings.player_runtime_data"),
                    value = sections.playerData
                )
                if (sections.dvProxyData.isNotBlank()) {
                    MobileProbeReportCard(
                        title = AppStrings.t(lang, "settings.dv_proxy_debug"),
                        value = sections.dvProxyData
                    )
                }
            }
            if (debugInfo.hasInfo) {
                MobileActionRow(
                    title = AppStrings.t(lang, "settings.copy_debug_info"),
                    value = AppStrings.t(lang, "settings.copy_debug_info_desc"),
                    icon = FluxaIcons.Copy,
                    prominent = true,
                    onClick = {
                        clipboard.setPrimaryClip(
                            ClipData.newPlainText(
                                AppStrings.t(lang, "settings.copy_debug_info"),
                                debugInfo.technicalInfo
                            )
                        )
                    }
                )
            }
        }
    }
}

private data class DebugRow(
    val label: String,
    val value: String
)

private data class DebugSections(
    val fileData: String,
    val playerData: String,
    val dvProxyData: String
)

private fun String.debugSections(): DebugSections {
    if (isBlank()) return DebugSections("", "", "")
    val playerMarker = "\nplayer_data\n"
    val dvMarker = "\ndv_proxy_data\n"
    val playerIdx = indexOf(playerMarker)
    val dvIdx = indexOf(dvMarker)
    return when {
        playerIdx >= 0 && dvIdx > playerIdx -> DebugSections(
            fileData = take(playerIdx).trim(),
            playerData = substring(playerIdx + playerMarker.length, dvIdx).trim(),
            dvProxyData = substring(dvIdx + dvMarker.length).trim()
        )
        playerIdx >= 0 -> DebugSections(
            fileData = take(playerIdx).trim(),
            playerData = substring(playerIdx + playerMarker.length).trim(),
            dvProxyData = ""
        )
        else -> DebugSections(
            fileData = lineSequence().takeWhile { it != "player_data" }.joinToString("\n").trim(),
            playerData = lineSequence().dropWhile { it != "player_data" }.drop(1).joinToString("\n").trim().ifBlank { this },
            dvProxyData = ""
        )
    }
}

private fun String.debugRows(): List<DebugRow> {
    return lineSequence()
        .mapNotNull { line ->
            val colonIndex = line.indexOf(':').takeIf { it > 0 } ?: Int.MAX_VALUE
            val equalsIndex = line.indexOf('=').takeIf { it > 0 } ?: Int.MAX_VALUE
            val index = minOf(colonIndex, equalsIndex)
            if (index == Int.MAX_VALUE) return@mapNotNull null
            if (index <= 0) return@mapNotNull null
            val rawLabel = line.take(index).trim()
            DebugRow(
                label = rawLabel.readableDebugLabel(),
                value = line.drop(index + 1).trim().ifBlank { "-" }
            )
        }
        .toList()
}

private fun String.readableDebugLabel(): String {
    if (!contains('_')) return this
    return split('_')
        .filter { it.isNotBlank() }
        .joinToString(" ") { part ->
            part.replaceFirstChar { char ->
                if (char.isLowerCase()) char.titlecase() else char.toString()
            }
        }
}

@Composable
private fun ProbeReportCard(
    title: String,
    value: String,
    modifier: Modifier,
    background: Color,
    border: Color,
    header: Color,
    label: Color,
    text: Color,
    valueHighlight: Color
) {
    val rows = remember(value) { value.debugRows() }
    SelectionContainer {
        Column(
            modifier = modifier
                .clip(RoundedCornerShape(14.dp))
                .background(background)
                .border(1.dp, border, RoundedCornerShape(14.dp))
                .padding(horizontal = 16.dp, vertical = 14.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = title.uppercase(),
                    color = header,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Black,
                    letterSpacing = 0.sp,
                    modifier = Modifier.weight(1f)
                )
            }
            if (rows.isEmpty()) {
                Text(
                    text = value,
                    color = text.copy(alpha = 0.78f),
                    fontSize = 12.sp,
                    lineHeight = 16.sp,
                    fontFamily = FontFamily.Monospace
                )
            } else {
                Column(verticalArrangement = Arrangement.spacedBy(5.dp)) {
                    rows.forEach { row ->
                        ProbeReportRow(
                            row = row,
                            label = label,
                            text = text,
                            highlight = valueHighlight
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun ProbeReportRow(
    row: DebugRow,
    label: Color,
    text: Color,
    highlight: Color
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.Top
    ) {
        Text(
            text = row.label,
            color = label,
            fontSize = 13.sp,
            lineHeight = 17.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(0.42f)
        )
        Text(
            text = row.value,
            color = if (row.value.isProbePositiveValue()) highlight else text,
            fontSize = 13.sp,
            lineHeight = 17.sp,
            fontWeight = FontWeight.Bold,
            fontFamily = FontFamily.Monospace,
            modifier = Modifier.weight(0.58f)
        )
    }
}

private fun String.isProbePositiveValue(): Boolean {
    return equals("true", ignoreCase = true) ||
        equals("played", ignoreCase = true) ||
        equals("ready", ignoreCase = true) ||
        equals("dolby vision", ignoreCase = true)
}

@Composable
private fun MobileDebugInfoRow(title: String, value: String) {
    val colors = LocalMobileSettingsPalette.current
    Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp)) {
        Text(title, color = colors.text.copy(alpha = 0.55f), fontSize = 11.sp, fontWeight = FontWeight.SemiBold)
        Text(value, color = colors.text.copy(alpha = 0.9f), fontSize = 14.sp, lineHeight = 18.sp)
    }
}

@Composable
private fun MobileProbeReportCard(title: String, value: String) {
    val colors = LocalMobileSettingsPalette.current
    Box(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 10.dp)) {
        ProbeReportCard(
            title = title,
            value = value,
            modifier = Modifier.fillMaxWidth(),
            background = colors.text.copy(alpha = 0.065f),
            border = colors.text.copy(alpha = 0.11f),
            header = colors.text.copy(alpha = 0.72f),
            label = colors.text.copy(alpha = 0.58f),
            text = colors.text.copy(alpha = 0.88f),
            valueHighlight = Color(0xFF9BFFD1)
        )
    }
}

@Composable
private fun MobileDebugTextBlock(title: String, value: String) {
    val colors = LocalMobileSettingsPalette.current
    Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(title, color = colors.text.copy(alpha = 0.55f), fontSize = 11.sp, fontWeight = FontWeight.SemiBold)
        SelectionContainer {
            Text(
                text = value,
                color = colors.text.copy(alpha = 0.78f),
                fontSize = 11.sp,
                lineHeight = 15.sp,
                fontFamily = FontFamily.Monospace,
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(max = 360.dp)
                    .verticalScroll(rememberScrollState())
                    .clip(RoundedCornerShape(12.dp))
                    .background(Color.Black.copy(alpha = 0.22f))
                    .border(1.dp, colors.border, RoundedCornerShape(12.dp))
                    .padding(12.dp)
            )
        }
    }
}
