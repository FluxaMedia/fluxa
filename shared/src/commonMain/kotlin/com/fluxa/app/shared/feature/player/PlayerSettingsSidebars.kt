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
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
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
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
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
        var liveTextOpacity by remember(subtitleTextOpacity) { mutableStateOf(subtitleTextOpacity) }
        var liveBackgroundOpacity by remember(subtitleBackgroundOpacity) { mutableStateOf(subtitleBackgroundOpacity) }
        var liveOutlineOpacity by remember(subtitleOutlineOpacity) { mutableStateOf(subtitleOutlineOpacity) }
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
            }
        }
        val subtitleAdjust: @Composable () -> Unit = {
            Column(
                verticalArrangement = Arrangement.spacedBy(8.dp),
                modifier = Modifier.fillMaxWidth().verticalScroll(rememberScrollState())
            ) {
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
                    value = liveTextOpacity,
                    deviceType = deviceType,
                    onChange = { liveTextOpacity = it },
                    onCommit = { onSubtitleTextOpacityChange(liveTextOpacity) }
                )
                OpacityAdjustmentItem(
                    title = AppStrings.t(lang, "settings.subtitle_background"),
                    value = liveBackgroundOpacity,
                    deviceType = deviceType,
                    onChange = { liveBackgroundOpacity = it },
                    onCommit = { onSubtitleBackgroundOpacityChange(liveBackgroundOpacity) }
                )
                OpacityAdjustmentItem(
                    title = AppStrings.t(lang, "settings.subtitle_outline"),
                    value = liveOutlineOpacity,
                    deviceType = deviceType,
                    onChange = { liveOutlineOpacity = it },
                    onCommit = { onSubtitleOutlineOpacityChange(liveOutlineOpacity) }
                )
            }
        }

        if (showAdjust) {
            SubtitlePreviewCue(
                lang = lang,
                textOpacity = liveTextOpacity,
                backgroundOpacity = liveBackgroundOpacity,
                outlineOpacity = liveOutlineOpacity
            )
            PlayerSidebarShell(
                title = AppStrings.t(lang, "player.adjust"),
                deviceType = deviceType,
                onClose = onClose,
                onBack = { showAdjust = false },
                cardWidth = 380.dp,
                anchorTop = true,
                scrimAlpha = 0.06f
            ) {
                subtitleAdjust()
            }
        } else if (twoColumn) {
            PlayerSidebarShell(
                title = "",
                deviceType = deviceType,
                onClose = onClose,
                cardWidth = 580.dp
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(20.dp), modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.weight(1f)) {
                        ColumnHeader(AppStrings.t(lang, "player.audio_title")) { onInlineDelayAdjust(0) }
                        audioList()
                    }
                    Column(modifier = Modifier.weight(1f)) {
                        ColumnHeader(AppStrings.t(lang, "player.subtitles_title")) { showAdjust = true }
                        subtitleList()
                    }
                }
            }
        } else {
            PlayerSidebarShell(
                title = "",
                deviceType = deviceType,
                onClose = onClose
            ) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Box(modifier = Modifier.weight(1f)) {
                        SegmentedTabRow(
                            options = listOf(
                                AppStrings.t(lang, "player.audio_title") to 0,
                                AppStrings.t(lang, "player.subtitles_title") to 1
                            ),
                            selected = tab,
                            onSelect = { tab = it }
                        )
                    }
                    AdjustIconButton {
                        if (tab == 0) onInlineDelayAdjust(0) else showAdjust = true
                    }
                }
                Spacer(Modifier.height(14.dp))
                if (tab == 0) audioList() else subtitleList()
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
    Box(
        modifier = Modifier
            .fillMaxSize()
            .zIndex(100f)
            .pointerInput(Unit) {
                detectDragGestures { change, _ -> change.consume() }
            }
    ) {
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
private fun ColumnHeader(text: String, onAdjust: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(start = 12.dp, end = 4.dp, bottom = 6.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = text,
            color = Color.White.copy(alpha = 0.55f),
            fontSize = 12.sp,
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.weight(1f)
        )
        AdjustIconButton(onAdjust)
    }
}

@Composable
private fun AdjustIconButton(onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(28.dp)
            .clip(CircleShape)
            .background(Color.White.copy(alpha = 0.06f))
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(FluxaIcons.Tune, null, tint = Color.White.copy(alpha = 0.8f), modifier = Modifier.size(14.dp))
    }
}

@Composable
private fun SubtitlePreviewCue(
    lang: String,
    textOpacity: Float,
    backgroundOpacity: Float,
    outlineOpacity: Float
) {
    Box(modifier = Modifier.fillMaxSize().zIndex(60f)) {
        Box(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .padding(bottom = 20.dp)
                .clip(RoundedCornerShape(6.dp))
                .background(Color.Black.copy(alpha = backgroundOpacity.coerceIn(0f, 1f)))
                .padding(horizontal = 12.dp, vertical = 6.dp)
        ) {
            Text(
                text = AppStrings.t(lang, "player.subtitle_sample"),
                color = Color.White.copy(alpha = textOpacity.coerceIn(0f, 1f)),
                fontSize = 18.sp,
                fontWeight = FontWeight.Medium,
                style = TextStyle(
                    shadow = Shadow(
                        color = Color.Black.copy(alpha = outlineOpacity.coerceIn(0f, 1f)),
                        offset = Offset.Zero,
                        blurRadius = 4f
                    )
                )
            )
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
    onChange: (Float) -> Unit,
    onCommit: () -> Unit = {}
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(Color.White.copy(alpha = 0.05f))
            .padding(horizontal = 12.dp, vertical = 2.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Text(
            text = title,
            color = Color.White.copy(alpha = 0.82f),
            fontSize = if (deviceType == DeviceType.TV) 14.sp else 13.sp,
            fontWeight = FontWeight.Medium,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(0.42f)
        )
        Slider(
            value = value.coerceIn(0f, 1f),
            onValueChange = { onChange(it.coerceIn(0f, 1f)) },
            onValueChangeFinished = onCommit,
            valueRange = 0f..1f,
            colors = SliderDefaults.colors(
                thumbColor = Color.White,
                activeTrackColor = Color.White,
                inactiveTrackColor = Color.White.copy(alpha = 0.18f)
            ),
            modifier = Modifier.weight(0.58f)
        )
        Text(
            text = "${(value.coerceIn(0f, 1f) * 100).toInt()}%",
            color = Color.White,
            fontSize = 12.sp,
            fontWeight = FontWeight.SemiBold,
            textAlign = TextAlign.End,
            modifier = Modifier.width(36.dp)
        )
    }
}
