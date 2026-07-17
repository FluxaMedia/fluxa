package com.fluxa.app.shared.feature.player

import com.fluxa.app.common.AppStrings
import com.fluxa.app.player.MediaTrack
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaIcons

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import kotlin.math.roundToInt
import kotlinx.coroutines.delay

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
    onInlineDelayAdjust: (Int) -> Unit = {},
    deviceType: DeviceType,
    lang: String = "en",
    languageDisplayName: (String) -> String = { it },
    onClose: () -> Unit
) {
    if (activeTab == 2) {
        SpeedPopup(playbackSpeed, lang, onSpeedChange, onClose)
        return
    }

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val twoColumn = deviceType != DeviceType.Mobile || maxWidth > maxHeight
        var tab by remember(activeTab) { mutableStateOf(activeTab.coerceIn(0, 1)) }
        var showAdjust by remember(activeTab) { mutableStateOf(false) }
        val listMaxHeight = if (deviceType == DeviceType.TV) 400.dp else 440.dp

        val audioList: @Composable () -> Unit = {
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
                        title = AppStrings.t(lang, "player.audio_delay"),
                        isSelected = false,
                        onClick = { onInlineDelayAdjust(0) },
                        subtitle = formatDelayMs(audioDelayMs),
                        deviceType = deviceType,
                        trailingIcon = FluxaIcons.ChevronRight
                    )
                }
            }
        }
        val subtitleList: @Composable () -> Unit = {
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
                        trailingIcon = FluxaIcons.ChevronRight
                    )
                }
            }
        }
        val subtitleAdjust: @Composable () -> Unit = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
                TrackItem(
                    title = AppStrings.t(lang, "player.subtitle_delay"),
                    isSelected = false,
                    onClick = { onInlineDelayAdjust(1) },
                    subtitle = formatDelayMs(subtitleDelayMs),
                    deviceType = deviceType,
                    trailingIcon = FluxaIcons.ChevronRight
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

        if (twoColumn) {
            PlayerSidebarShell(
                title = if (showAdjust) AppStrings.t(lang, "player.adjust") else "",
                deviceType = deviceType,
                onClose = onClose,
                onBack = if (showAdjust) { { showAdjust = false } } else null,
                cardWidth = if (showAdjust) 380.dp else 580.dp
            ) {
                if (showAdjust) {
                    subtitleAdjust()
                } else {
                    Row(horizontalArrangement = Arrangement.spacedBy(20.dp), modifier = Modifier.fillMaxWidth()) {
                        Column(modifier = Modifier.weight(1f)) {
                            ColumnLabel(AppStrings.t(lang, "player.audio_title"))
                            audioList()
                        }
                        Column(modifier = Modifier.weight(1f)) {
                            ColumnLabel(AppStrings.t(lang, "player.subtitles_title"))
                            subtitleList()
                        }
                    }
                }
            }
        } else {
            PlayerSidebarShell(
                title = if (showAdjust) AppStrings.t(lang, "player.adjust") else "",
                deviceType = deviceType,
                onClose = onClose,
                onBack = if (showAdjust) { { showAdjust = false } } else null
            ) {
                if (showAdjust) {
                    subtitleAdjust()
                } else {
                    SegmentedTabRow(
                        options = listOf(
                            AppStrings.t(lang, "player.audio_title") to 0,
                            AppStrings.t(lang, "player.subtitles_title") to 1
                        ),
                        selected = tab,
                        onSelect = { tab = it }
                    )
                    Spacer(Modifier.height(14.dp))
                    if (tab == 0) audioList() else subtitleList()
                }
            }
        }
    }
}

@Composable
private fun SpeedPopup(playbackSpeed: Float, lang: String, onSpeedChange: (Float) -> Unit, onClose: () -> Unit) {
    var shown by remember { mutableStateOf(false) }
    var closing by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { shown = true }
    val progress by animateFloatAsState(
        targetValue = if (shown && !closing) 1f else 0f,
        animationSpec = tween(180, easing = FastOutSlowInEasing),
        label = "speedPopup"
    )
    LaunchedEffect(closing) {
        if (closing) {
            delay(190L)
            onClose()
        }
    }
    Box(modifier = Modifier.fillMaxSize().zIndex(100f)) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Color.Black.copy(alpha = 0.2f * progress))
                .clickable(
                    interactionSource = remember { MutableInteractionSource() },
                    indication = null
                ) { closing = true }
        )
        Column(
            modifier = Modifier
                .align(Alignment.BottomEnd)
                .padding(end = 24.dp, bottom = 104.dp)
                .width(220.dp)
                .graphicsLayer {
                    transformOrigin = TransformOrigin(1f, 1f)
                    alpha = progress
                    scaleX = 0.92f + 0.08f * progress
                    scaleY = 0.92f + 0.08f * progress
                }
                .background(Color(0xE6101418), RoundedCornerShape(18.dp))
                .clip(RoundedCornerShape(18.dp))
                .padding(6.dp)
        ) {
            listOf(0.5f, 0.75f, 1.0f, 1.25f, 1.5f, 2.0f).forEach { speed ->
                val isSelected = playbackSpeed == speed
                val label = if (speed == 1.0f) {
                    "${formatSpeedLabel(speed)} · ${AppStrings.t(lang, "player.speed_standard")}"
                } else {
                    formatSpeedLabel(speed)
                }
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(12.dp))
                        .background(if (isSelected) Color.White.copy(alpha = 0.08f) else Color.Transparent)
                        .clickable { onSpeedChange(speed) }
                        .padding(horizontal = 12.dp, vertical = 10.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = label,
                        color = Color.White.copy(alpha = if (isSelected) 1f else 0.75f),
                        fontSize = 14.sp,
                        fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal,
                        modifier = Modifier.weight(1f)
                    )
                    if (isSelected) {
                        Icon(FluxaIcons.Check, null, tint = Color.White, modifier = Modifier.size(16.dp))
                    }
                }
            }
        }
    }
}

@Composable
fun DelayAdjustPill(
    label: String,
    valueMs: Long,
    onDecrease: () -> Unit,
    onIncrease: () -> Unit,
    onDone: () -> Unit
) {
    Box(modifier = Modifier.fillMaxSize().zIndex(90f)) {
        Row(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .padding(bottom = 112.dp)
                .clip(RoundedCornerShape(999.dp))
                .background(Color(0xE6101418))
                .padding(horizontal = 10.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Text(
                text = label,
                color = Color.White.copy(alpha = 0.7f),
                fontSize = 13.sp,
                fontWeight = FontWeight.Medium,
                modifier = Modifier.padding(start = 8.dp)
            )
            PillIconButton(FluxaIcons.Remove, onDecrease)
            Text(
                text = formatDelayMs(valueMs),
                color = Color.White,
                fontSize = 14.sp,
                fontWeight = FontWeight.SemiBold,
                textAlign = TextAlign.Center,
                modifier = Modifier.width(64.dp)
            )
            PillIconButton(FluxaIcons.Add, onIncrease)
            PillIconButton(FluxaIcons.Check, onDone, emphasized = true)
        }
    }
}

@Composable
private fun PillIconButton(icon: ImageVector, onClick: () -> Unit, emphasized: Boolean = false) {
    Box(
        modifier = Modifier
            .size(34.dp)
            .clip(CircleShape)
            .background(Color.White.copy(alpha = if (emphasized) 0.14f else 0.07f))
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(icon, null, tint = Color.White, modifier = Modifier.size(16.dp))
    }
}

@Composable
private fun ColumnLabel(text: String) {
    Text(
        text = text,
        color = Color.White.copy(alpha = 0.55f),
        fontSize = 12.sp,
        fontWeight = FontWeight.SemiBold,
        modifier = Modifier.padding(start = 12.dp, bottom = 6.dp)
    )
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
