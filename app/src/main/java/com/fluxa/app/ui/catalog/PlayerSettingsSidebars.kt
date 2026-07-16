@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.shared.feature.player.SeekIconButton

import androidx.compose.animation.Crossfade
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.*
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import coil3.compose.AsyncImage
import com.fluxa.app.player.MediaTrack
import java.util.Locale

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
    onClose: () -> Unit
) {
    val title = when (activeTab) {
        0 -> AppStrings.t(lang, "player.audio_title")
        1 -> AppStrings.t(lang, "player.subtitles_title")
        else -> AppStrings.t(lang, "player.speed_title")
    }
    val subtitle = when (activeTab) {
        0 -> AppStrings.t(lang, "player.audio_subtitle")
        1 -> AppStrings.t(lang, "player.subtitles_subtitle")
        else -> AppStrings.t(lang, "player.speed_subtitle")
    }
    PlayerSidebarShell(
        title = title,
        subtitle = subtitle,
        deviceType = deviceType,
        onClose = onClose,
        sideSheetOnMobile = activeTab == 0 || activeTab == 1,
        compactCenterOnMobile = activeTab == 2
    ) {
        val listMaxHeight = if (activeTab == 0 || activeTab == 1) 720.dp else if (deviceType == DeviceType.TV) 420.dp else 300.dp
        Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.TopCenter) {
            Crossfade(
                targetState = activeTab,
                
            ) { tab ->
                when (tab) {
                    0 -> {
                        LazyColumn(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth().heightIn(max = listMaxHeight)) {
                            item {
                                DelayAdjustmentItem(
                                    title = AppStrings.t(lang, "player.audio_delay"),
                                    valueMs = audioDelayMs,
                                    deviceType = deviceType,
                                    onDecrease = { onAudioDelayChange(audioDelayMs - 250L) },
                                    onIncrease = { onAudioDelayChange(audioDelayMs + 250L) }
                                )
                            }
                            items(audioTracks, key = { it.id }) { track ->
                                val title = if (track.language != null) {
                                    nativeLanguageName(track.language!!)
                                } else {
                                    track.label
                                }
                                val channelLabel = track.audioChannelLabel
                                val subtitle = when {
                                    !track.isSupported -> AppStrings.t(lang, "player.unsupported")
                                    channelLabel.isNotBlank() -> channelLabel
                                    else -> null
                                }
                                TrackItem(
                                    title = title,
                                    isSelected = track == currentAudio,
                                    onClick = { onSelectAudio(track) },
                                    subtitle = subtitle,
                                    deviceType = deviceType,
                                    leadingIcon = FluxaIcons.AudioTrack
                                )
                            }
                        }
                    }
                    1 -> {
                        LazyColumn(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth().heightIn(max = listMaxHeight)) {
                            item {
                                DelayAdjustmentItem(
                                    title = AppStrings.t(lang, "player.subtitle_delay"),
                                    valueMs = subtitleDelayMs,
                                    deviceType = deviceType,
                                    onDecrease = { onSubtitleDelayChange(subtitleDelayMs - 250L) },
                                    onIncrease = { onSubtitleDelayChange(subtitleDelayMs + 250L) }
                                )
                            }
                            item {
                                OpacityAdjustmentItem(
                                    title = AppStrings.t(lang, "settings.subtitle_text"),
                                    value = subtitleTextOpacity,
                                    deviceType = deviceType,
                                    onChange = onSubtitleTextOpacityChange
                                )
                            }
                            item {
                                OpacityAdjustmentItem(
                                    title = AppStrings.t(lang, "settings.subtitle_background"),
                                    value = subtitleBackgroundOpacity,
                                    deviceType = deviceType,
                                    onChange = onSubtitleBackgroundOpacityChange
                                )
                            }
                            item {
                                OpacityAdjustmentItem(
                                    title = AppStrings.t(lang, "settings.subtitle_outline"),
                                    value = subtitleOutlineOpacity,
                                    deviceType = deviceType,
                                    onChange = onSubtitleOutlineOpacityChange
                                )
                            }
                            item {
                                TrackItem(
                                    title = AppStrings.t(lang, "player.off"),
                                    isSelected = currentSubtitle == null,
                                    onClick = { onDisableSubtitle() },
                                    subtitle = AppStrings.t(lang, "player.hide_subtitles"),
                                    deviceType = deviceType,
                                    leadingIcon = FluxaIcons.SubtitlesOff
                                )
                            }
                            items(subtitleTracks, key = { it.id }) { track ->
                                TrackItem(
                                    title = track.label,
                                    isSelected = track == currentSubtitle,
                                    onClick = { onSelectSubtitle(track) },
                                    subtitle = track.language?.let { nativeLanguageName(it) } ?: AppStrings.t(lang, "player.embedded_subtitle"),
                                    deviceType = deviceType,
                                    leadingIcon = FluxaIcons.Subtitles
                                )
                            }
                        }
                    }
                    2 -> {
                        val speeds = listOf(0.5f, 0.75f, 1.0f, 1.25f, 1.5f, 2.0f)
                        if (deviceType == DeviceType.Mobile) {
                            Column(
                                verticalArrangement = Arrangement.spacedBy(10.dp),
                                modifier = Modifier.fillMaxWidth()
                            ) {
                                speeds.chunked(3).forEach { row ->
                                    Row(
                                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                                        modifier = Modifier.fillMaxWidth()
                                    ) {
                                        row.forEach { speed ->
                                            SpeedChip(
                                                label = formatSpeedLabel(speed),
                                                isSelected = playbackSpeed == speed,
                                                onClick = { onSpeedChange(speed) },
                                                modifier = Modifier.weight(1f)
                                            )
                                        }
                                    }
                                }
                            }
                        } else {
                            LazyColumn(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth().heightIn(max = listMaxHeight)) {
                                items(speeds, key = { it }) { speed ->
                                    TrackItem(
                                        title = formatSpeedLabel(speed),
                                        isSelected = playbackSpeed == speed,
                                        onClick = { onSpeedChange(speed) },
                                        subtitle = when {
                                            speed == 1.0f -> AppStrings.t(lang, "player.speed_standard")
                                            speed < 1.0f -> AppStrings.t(lang, "player.speed_slower")
                                            else -> AppStrings.t(lang, "player.speed_faster")
                                        },
                                        deviceType = deviceType,
                                        leadingIcon = FluxaIcons.Speed
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

private fun formatSpeedLabel(speed: Float): String {
    val rounded = Math.round(speed * 100) / 100f
    val text = if (rounded == rounded.toInt().toFloat()) rounded.toInt().toString() else rounded.toString()
    return "${text}x"
}

@Composable
private fun SpeedChip(
    label: String,
    isSelected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Box(
        modifier = modifier
            .height(46.dp)
            .clip(RoundedCornerShape(10.dp))
            .background(if (isSelected) Color.White else Color.White.copy(alpha = 0.08f))
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = label,
            color = if (isSelected) Color.Black else Color.White.copy(alpha = 0.85f),
            fontSize = 14.sp,
            fontWeight = FontWeight.Bold
        )
    }
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
            .clip(RoundedCornerShape(20.dp))
            .background(Color.White.copy(alpha = 0.05f))
            .border(1.dp, Color.White.copy(alpha = 0.09f), RoundedCornerShape(20.dp))
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
                fontWeight = FontWeight.Bold,
                modifier = Modifier.weight(1f)
            )
            SeekIconButton(FluxaIcons.Remove, deviceType, onClick = onDecrease)
            Text(
                text = formatDelayMs(valueMs),
                color = Color.White,
                fontSize = if (deviceType == DeviceType.TV) 17.sp else 14.sp,
                fontWeight = FontWeight.Black,
                textAlign = TextAlign.Center,
                modifier = Modifier.width(if (deviceType == DeviceType.TV) 76.dp else 62.dp)
            )
            SeekIconButton(FluxaIcons.Add, deviceType, onClick = onIncrease)
        }
    }
}

private fun formatDelayMs(valueMs: Long): String {
    val seconds = valueMs / 1000.0
    return "${if (valueMs >= 0) "+" else ""}${String.format(Locale.US, "%.2f", seconds)}s"
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
            .clip(RoundedCornerShape(20.dp))
            .background(Color.White.copy(alpha = 0.05f))
            .border(1.dp, Color.White.copy(alpha = 0.09f), RoundedCornerShape(20.dp))
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
                    fontWeight = FontWeight.Bold,
                    modifier = Modifier.weight(1f)
                )
                Text(
                    text = "${(value.coerceIn(0f, 1f) * 100).toInt()}%",
                    color = Color.White,
                    fontSize = if (deviceType == DeviceType.TV) 15.sp else 13.sp,
                    fontWeight = FontWeight.Black
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
