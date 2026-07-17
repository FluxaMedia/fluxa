package com.fluxa.app.shared.feature.player

import com.fluxa.app.common.AppStrings
import com.fluxa.app.player.MediaTrack
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaIcons

import androidx.compose.animation.Crossfade
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.roundToInt

@Composable
fun UniversalSettingsSidebar(
    activeTab: Int,
    audioTracks: List<MediaTrack>,
    currentAudio: MediaTrack?,
    subtitleTracks: List<MediaTrack>,
    currentSubtitle: MediaTrack?,
    playbackSpeed: Float,
    audioDelayMs: Long,
    subtitleDelayMs: Long,
    subtitleTextOpacity: Float,
    subtitleBackgroundOpacity: Float,
    subtitleOutlineOpacity: Float,
    onSelectAudio: (MediaTrack) -> Unit,
    onSelectSubtitle: (MediaTrack) -> Unit,
    onDisableSubtitle: () -> Unit,
    onSpeedChange: (Float) -> Unit,
    onAudioDelayChange: (Long) -> Unit,
    onSubtitleDelayChange: (Long) -> Unit,
    onSubtitleTextOpacityChange: (Float) -> Unit,
    onSubtitleBackgroundOpacityChange: (Float) -> Unit,
    onSubtitleOutlineOpacityChange: (Float) -> Unit,
    deviceType: DeviceType,
    lang: String = "en",
    languageDisplayName: (String) -> String = { it },
    onClose: () -> Unit
) {
    var tab by remember(activeTab) { mutableStateOf(activeTab.coerceIn(0, 2)) }
    var showAdjust by remember(activeTab) { mutableStateOf(false) }

    val title = when {
        showAdjust -> AppStrings.t(lang, "player.adjust")
        tab == 0 -> AppStrings.t(lang, "player.audio_title")
        tab == 1 -> AppStrings.t(lang, "player.subtitles_title")
        else -> AppStrings.t(lang, "player.speed_title")
    }

    PlayerSidebarShell(
        title = title,
        subtitle = "",
        deviceType = deviceType,
        onClose = onClose,
        onBack = if (showAdjust) { { showAdjust = false } } else null
    ) {
        if (!showAdjust && (tab == 0 || tab == 1)) {
            SegmentedTabRow(
                options = listOf(
                    AppStrings.t(lang, "player.audio_title") to 0,
                    AppStrings.t(lang, "player.subtitles_title") to 1
                ),
                selected = tab,
                onSelect = { tab = it }
            )
            Spacer(Modifier.height(14.dp))
        }

        val listMaxHeight = if (deviceType == DeviceType.TV) 420.dp else 480.dp
        Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.TopCenter) {
            Crossfade(targetState = Triple(tab, showAdjust, deviceType)) { (currentTab, adjusting, _) ->
                when {
                    currentTab == 0 && adjusting -> {
                        Column(modifier = Modifier.fillMaxWidth()) {
                            DelayAdjustmentItem(
                                title = AppStrings.t(lang, "player.audio_delay"),
                                valueMs = audioDelayMs,
                                deviceType = deviceType,
                                onDecrease = { onAudioDelayChange(audioDelayMs - 250L) },
                                onIncrease = { onAudioDelayChange(audioDelayMs + 250L) }
                            )
                        }
                    }
                    currentTab == 0 -> {
                        LazyColumn(verticalArrangement = Arrangement.spacedBy(2.dp), modifier = Modifier.fillMaxWidth().heightIn(max = listMaxHeight)) {
                            items(audioTracks, key = { it.id }) { track ->
                                val trackTitle = if (track.language != null) {
                                    languageDisplayName(track.language!!)
                                } else {
                                    track.label
                                }
                                val channelLabel = track.audioChannelLabel
                                val trackSubtitle = when {
                                    !track.isSupported -> AppStrings.t(lang, "player.unsupported")
                                    channelLabel.isNotBlank() -> channelLabel
                                    else -> null
                                }
                                TrackItem(
                                    title = trackTitle,
                                    isSelected = track == currentAudio,
                                    onClick = { onSelectAudio(track) },
                                    subtitle = trackSubtitle,
                                    deviceType = deviceType
                                )
                            }
                            item {
                                TrackItem(
                                    title = AppStrings.t(lang, "player.adjust"),
                                    isSelected = false,
                                    onClick = { showAdjust = true },
                                    deviceType = deviceType,
                                    leadingIcon = FluxaIcons.Settings,
                                    trailingIcon = FluxaIcons.ChevronRight
                                )
                            }
                        }
                    }
                    currentTab == 1 && adjusting -> {
                        Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
                            DelayAdjustmentItem(
                                title = AppStrings.t(lang, "player.subtitle_delay"),
                                valueMs = subtitleDelayMs,
                                deviceType = deviceType,
                                onDecrease = { onSubtitleDelayChange(subtitleDelayMs - 250L) },
                                onIncrease = { onSubtitleDelayChange(subtitleDelayMs + 250L) }
                            )
                            OpacityAdjustmentItem(
                                title = AppStrings.t(lang, "settings.subtitle_text"),
                                value = subtitleTextOpacity,
                                deviceType = deviceType,
                                onChange = onSubtitleTextOpacityChange
                            )
                            OpacityAdjustmentItem(
                                title = AppStrings.t(lang, "settings.subtitle_background"),
                                value = subtitleBackgroundOpacity,
                                deviceType = deviceType,
                                onChange = onSubtitleBackgroundOpacityChange
                            )
                            OpacityAdjustmentItem(
                                title = AppStrings.t(lang, "settings.subtitle_outline"),
                                value = subtitleOutlineOpacity,
                                deviceType = deviceType,
                                onChange = onSubtitleOutlineOpacityChange
                            )
                        }
                    }
                    currentTab == 1 -> {
                        LazyColumn(verticalArrangement = Arrangement.spacedBy(2.dp), modifier = Modifier.fillMaxWidth().heightIn(max = listMaxHeight)) {
                            item {
                                TrackItem(
                                    title = AppStrings.t(lang, "player.off"),
                                    isSelected = currentSubtitle == null,
                                    onClick = { onDisableSubtitle() },
                                    deviceType = deviceType
                                )
                            }
                            items(subtitleTracks, key = { it.id }) { track ->
                                TrackItem(
                                    title = track.label,
                                    isSelected = track == currentSubtitle,
                                    onClick = { onSelectSubtitle(track) },
                                    subtitle = track.language?.let { languageDisplayName(it) } ?: AppStrings.t(lang, "player.embedded_subtitle"),
                                    deviceType = deviceType
                                )
                            }
                            item {
                                TrackItem(
                                    title = AppStrings.t(lang, "player.adjust"),
                                    isSelected = false,
                                    onClick = { showAdjust = true },
                                    deviceType = deviceType,
                                    leadingIcon = FluxaIcons.Settings,
                                    trailingIcon = FluxaIcons.ChevronRight
                                )
                            }
                        }
                    }
                    else -> {
                        val speeds = listOf(0.5f, 0.75f, 1.0f, 1.25f, 1.5f, 2.0f)
                        LazyColumn(verticalArrangement = Arrangement.spacedBy(2.dp), modifier = Modifier.fillMaxWidth().heightIn(max = listMaxHeight)) {
                            items(speeds, key = { it }) { speed ->
                                TrackItem(
                                    title = formatSpeedLabel(speed),
                                    isSelected = playbackSpeed == speed,
                                    onClick = { onSpeedChange(speed) },
                                    subtitle = if (speed == 1.0f) AppStrings.t(lang, "player.speed_standard") else null,
                                    deviceType = deviceType
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun SegmentedTabRow(options: List<Pair<String, Int>>, selected: Int, onSelect: (Int) -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(Color.White.copy(alpha = 0.06f))
            .padding(3.dp),
        horizontalArrangement = Arrangement.spacedBy(3.dp)
    ) {
        options.forEach { (label, value) ->
            val isSelected = value == selected
            Box(
                modifier = Modifier
                    .weight(1f)
                    .clip(RoundedCornerShape(9.dp))
                    .background(if (isSelected) Color.White.copy(alpha = 0.12f) else Color.Transparent)
                    .clickable { onSelect(value) }
                    .padding(vertical = 9.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = label,
                    color = Color.White.copy(alpha = if (isSelected) 1f else 0.55f),
                    fontSize = 12.sp,
                    fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Medium
                )
            }
        }
    }
}

private fun formatSpeedLabel(speed: Float): String {
    val rounded = (speed * 100).roundToInt() / 100f
    val text = if (rounded == rounded.toInt().toFloat()) rounded.toInt().toString() else rounded.toString()
    return "${text}x"
}

@Composable
private fun DelayAdjustmentItem(
    title: String,
    valueMs: Long,
    deviceType: DeviceType,
    onDecrease: () -> Unit,
    onIncrease: () -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(Color.White.copy(alpha = 0.05f))
            .padding(horizontal = 14.dp, vertical = 12.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                text = title,
                color = Color.White.copy(alpha = 0.82f),
                fontSize = if (deviceType == DeviceType.TV) 16.sp else 14.sp,
                fontWeight = FontWeight.Medium,
                modifier = Modifier.weight(1f)
            )
            SeekIconButton(FluxaIcons.Remove, deviceType, onClick = onDecrease)
            Text(
                text = formatDelayMs(valueMs),
                color = Color.White,
                fontSize = if (deviceType == DeviceType.TV) 17.sp else 14.sp,
                fontWeight = FontWeight.SemiBold,
                textAlign = TextAlign.Center,
                modifier = Modifier.width(if (deviceType == DeviceType.TV) 76.dp else 62.dp)
            )
            SeekIconButton(FluxaIcons.Add, deviceType, onClick = onIncrease)
        }
    }
}

private fun formatDelayMs(valueMs: Long): String {
    val hundredths = (kotlin.math.abs(valueMs) / 10) % 100
    val seconds = kotlin.math.abs(valueMs) / 1000
    val sign = if (valueMs >= 0) "+" else "-"
    return "$sign$seconds.${hundredths.toString().padStart(2, '0')}s"
}

@Composable
private fun OpacityAdjustmentItem(
    title: String,
    value: Float,
    deviceType: DeviceType,
    onChange: (Float) -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(Color.White.copy(alpha = 0.05f))
            .padding(horizontal = 14.dp, vertical = 12.dp)
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = title,
                    color = Color.White.copy(alpha = 0.82f),
                    fontSize = if (deviceType == DeviceType.TV) 16.sp else 14.sp,
                    fontWeight = FontWeight.Medium,
                    modifier = Modifier.weight(1f)
                )
                Text(
                    text = "${(value.coerceIn(0f, 1f) * 100).toInt()}%",
                    color = Color.White,
                    fontSize = if (deviceType == DeviceType.TV) 15.sp else 13.sp,
                    fontWeight = FontWeight.SemiBold
                )
            }
            Slider(
                value = value.coerceIn(0f, 1f),
                onValueChange = { onChange(it.coerceIn(0f, 1f)) },
                valueRange = 0f..1f,
                colors = SliderDefaults.colors(
                    thumbColor = Color.White,
                    activeTrackColor = Color.White,
                    inactiveTrackColor = Color.White.copy(alpha = 0.18f)
                )
            )
        }
    }
}
