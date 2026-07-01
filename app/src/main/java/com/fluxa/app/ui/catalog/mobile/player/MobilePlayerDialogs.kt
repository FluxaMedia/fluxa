@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import androidx.activity.compose.BackHandler
import androidx.compose.animation.Crossfade
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
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
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
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import androidx.compose.foundation.interaction.MutableInteractionSource
import com.fluxa.app.data.remote.Video
import com.fluxa.app.player.MediaTrack

@Composable
internal fun MobilePlayerEpisodeRow(
    episode: Video,
    isSelected: Boolean,
    lang: String,
    accentColor: Color,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(if (isSelected) Color.White.copy(alpha = 0.08f) else Color.Transparent)
            .clickable { onClick() }
            .padding(8.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.Top
    ) {
        Box(
            modifier = Modifier
                .width(128.dp)
                .height(72.dp)
                .clip(RoundedCornerShape(10.dp))
                .then(
                    if (isSelected) Modifier.border(2.dp, accentColor, RoundedCornerShape(10.dp)) else Modifier
                )
        ) {
            AsyncImage(
                model = episode.thumbnail,
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
            if (isSelected) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(Color.Black.copy(alpha = 0.35f)),
                    contentAlignment = Alignment.Center
                ) {
                    Box(
                        modifier = Modifier
                            .size(30.dp)
                            .clip(CircleShape)
                            .background(Color.Black.copy(alpha = 0.55f)),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            imageVector = FluxaIcons.PlayArrow,
                            contentDescription = null,
                            tint = Color.White,
                            modifier = Modifier.size(16.dp)
                        )
                    }
                }
            }
            episode.episodeRuntime?.takeIf { it > 0 }?.let { runtime ->
                Box(
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .padding(4.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(Color.Black.copy(alpha = 0.65f))
                        .padding(horizontal = 5.dp, vertical = 2.dp)
                ) {
                    Text(
                        text = AppStrings.runtimeMinutes(lang, runtime),
                        color = Color.White,
                        fontSize = 10.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                }
            }
        }

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = "${episode.number ?: 0}. ${episode.name.orEmpty()}",
                color = if (isSelected) accentColor else Color.White,
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            episode.overview?.takeIf { it.isNotBlank() }?.let { overview ->
                Spacer(Modifier.height(4.dp))
                Text(
                    text = overview,
                    color = Color.White.copy(alpha = 0.55f),
                    fontSize = 12.sp,
                    lineHeight = 16.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}

internal data class PlayerChoiceOption(val value: String, val label: String)

@Composable
internal fun MobilePlayerChoiceDialog(
    title: String,
    options: List<PlayerChoiceOption>,
    selected: String,
    footerContent: @Composable ColumnScope.() -> Unit = {},
    onDismiss: () -> Unit,
    onSelect: (String) -> Unit
) {
    androidx.compose.ui.window.Dialog(onDismissRequest = onDismiss) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .clickable(
                    onClick = onDismiss,
                    indication = null,
                    interactionSource = remember { androidx.compose.foundation.interaction.MutableInteractionSource() }
                ),
            contentAlignment = Alignment.Center
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth(0.84f)
                    .widthIn(max = 360.dp)
                    .clip(RoundedCornerShape(24.dp))
                    .background(FluxaColors.surface)
                    .border(1.dp, Color.White.copy(alpha = 0.10f), RoundedCornerShape(24.dp))
                    .padding(22.dp)
            ) {
                Text(
                    text = title,
                    color = Color.White,
                    fontSize = 19.sp,
                    fontWeight = FontWeight.Black,
                    modifier = Modifier.padding(bottom = 18.dp)
                )

                LazyColumn(
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier.heightIn(max = 420.dp)
                ) {
                    items(options, key = { it.value }) { option ->
                        val isSelected = option.value == selected
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(14.dp))
                                .background(if (isSelected) Color.White else Color.White.copy(alpha = 0.04f))
                                .clickable { onSelect(option.value) }
                                .padding(horizontal = 16.dp, vertical = 15.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.SpaceBetween
                        ) {
                            Text(
                                text = option.label,
                                color = if (isSelected) Color.Black else Color.White,
                                fontSize = 14.sp,
                                fontWeight = if (isSelected) FontWeight.Black else FontWeight.Medium,
                                modifier = Modifier.weight(1f)
                            )
                            if (isSelected) {
                                Icon(
                                    imageVector = FluxaIcons.Check,
                                    contentDescription = null,
                                    tint = Color.Black,
                                    modifier = Modifier.size(18.dp)
                                )
                            }
                        }
                    }
                }
                footerContent()
            }
        }
    }
}

@Composable
internal fun MobileDelayAdjustmentRow(
    title: String,
    valueMs: Long,
    onDecrease: () -> Unit,
    onIncrease: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 14.dp)
            .clip(RoundedCornerShape(16.dp))
            .background(Color.White.copy(alpha = 0.05f))
            .border(1.dp, Color.White.copy(alpha = 0.08f), RoundedCornerShape(16.dp))
            .padding(horizontal = 12.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Text(
            text = title,
            color = Color.White.copy(alpha = 0.82f),
            fontSize = 13.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.weight(1f)
        )
        MobileDelayButton(FluxaIcons.Remove, onDecrease)
        Text(
            text = formatDelayMs(valueMs),
            color = Color.White,
            fontSize = 13.sp,
            fontWeight = FontWeight.Black,
            textAlign = TextAlign.Center,
            modifier = Modifier.width(58.dp)
        )
        MobileDelayButton(FluxaIcons.Add, onIncrease)
    }
}

@Composable
private fun MobileDelayButton(icon: ImageVector, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(34.dp)
            .clip(CircleShape)
            .background(Color.Black.copy(alpha = 0.38f))
            .border(1.dp, Color.White.copy(alpha = 0.10f), CircleShape)
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(icon, null, tint = Color.White, modifier = Modifier.size(18.dp))
    }
}

@Composable
private fun MobileOpacityRow(title: String, value: Float, onChange: (Float) -> Unit) {
    Column(modifier = Modifier.fillMaxWidth().padding(top = 12.dp)) {
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = title,
                color = Color.White.copy(alpha = 0.62f),
                fontSize = 11.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.weight(1f)
            )
            Text(
                text = "${(value.coerceIn(0f, 1f) * 100).toInt()}%",
                color = Color.White,
                fontSize = 12.sp,
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
            ),
            modifier = Modifier.fillMaxWidth()
        )
    }
}

@Composable
internal fun MobileTrackPickerOverlay(
    lang: String,
    availableAudios: List<MediaTrack>,
    currentAudio: MediaTrack?,
    availableSubtitles: List<MediaTrack>,
    currentSubtitle: MediaTrack?,
    audioDelayMs: Long,
    subtitleDelayMs: Long,
    subtitleTextOpacity: Float,
    subtitleBackgroundOpacity: Float,
    subtitleOutlineOpacity: Float,
    onApply: (audio: MediaTrack?, subtitle: MediaTrack?) -> Unit,
    onAudioDelayChange: (Long) -> Unit,
    onSubtitleDelayChange: (Long) -> Unit,
    onSubtitleTextOpacityChange: (Float) -> Unit,
    onSubtitleBackgroundOpacityChange: (Float) -> Unit,
    onSubtitleOutlineOpacityChange: (Float) -> Unit,
    onDismiss: () -> Unit
) {
    var pendingAudio by remember { mutableStateOf(currentAudio) }
    var pendingSubtitle by remember { mutableStateOf(currentSubtitle) }
    var settingsMode by remember { mutableStateOf(false) }

    BackHandler(enabled = settingsMode) { settingsMode = false }

    Box(modifier = Modifier.fillMaxSize()) {
        // Base scrim — dims the video without fully killing it
        Box(modifier = Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.76f)))
        // Cool-tinted overlay: shifts remaining video colors toward cold dark blue (simulates dimmed/desaturated look)
        Box(modifier = Modifier.fillMaxSize().background(Color(0xFF060B14).copy(alpha = 0.42f)))
        // Radial vignette: edges collapse to near-black, center stays open
        Box(
            modifier = Modifier.fillMaxSize().background(
                Brush.radialGradient(
                    colorStops = arrayOf(
                        0.0f to Color.Transparent,
                        0.60f to Color.Black.copy(alpha = 0.10f),
                        1.0f to Color.Black.copy(alpha = 0.46f)
                    )
                )
            )
        )
        // Top edge darkening
        Box(
            modifier = Modifier.fillMaxSize().background(
                Brush.verticalGradient(
                    listOf(Color.Black.copy(alpha = 0.38f), Color.Transparent),
                    endY = 220f
                )
            )
        )
        // Bottom edge darkening
        Box(
            modifier = Modifier.fillMaxSize().background(
                Brush.verticalGradient(
                    listOf(Color.Transparent, Color.Black.copy(alpha = 0.30f))
                )
            )
        )

        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 24.dp, vertical = 20.dp)
        ) {
            // Header row
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = AppStrings.t(lang, "player.audio_and_subtitles"),
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold,
                    modifier = Modifier.weight(1f)
                )
                Icon(
                    imageVector = if (settingsMode) FluxaIcons.ArrowBack else FluxaIcons.Settings,
                    contentDescription = null,
                    tint = Color.White.copy(alpha = if (settingsMode) 0.92f else 0.64f),
                    modifier = Modifier
                        .size(22.dp)
                        .clickable(
                            indication = null,
                            interactionSource = remember { MutableInteractionSource() }
                        ) { settingsMode = !settingsMode }
                )
                Spacer(modifier = Modifier.width(18.dp))
                Icon(
                    imageVector = FluxaIcons.Close,
                    contentDescription = null,
                    tint = Color.White,
                    modifier = Modifier
                        .size(22.dp)
                        .clickable(
                            indication = null,
                            interactionSource = remember { MutableInteractionSource() }
                        ) { onDismiss() }
                )
            }

            Spacer(modifier = Modifier.height(24.dp))

            Crossfade(
                targetState = settingsMode,
                animationSpec = tween(180),
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f),
                label = "trackPickerMode"
            ) { inSettings ->
                if (inSettings) {
                    // Settings mode: audio settings left | subtitle settings right
                    Row(
                        modifier = Modifier.fillMaxSize(),
                        horizontalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        Column(
                            modifier = Modifier
                                .weight(1f)
                                .verticalScroll(rememberScrollState())
                        ) {
                            Text(
                                text = AppStrings.t(lang, "player.audio_title"),
                                color = Color.White.copy(alpha = 0.48f),
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(bottom = 2.dp, start = 4.dp)
                            )
                            MobileDelayAdjustmentRow(
                                title = AppStrings.t(lang, "player.audio_delay"),
                                valueMs = audioDelayMs,
                                onDecrease = { onAudioDelayChange(audioDelayMs - 250L) },
                                onIncrease = { onAudioDelayChange(audioDelayMs + 250L) }
                            )
                        }

                        Box(
                            modifier = Modifier
                                .width(1.dp)
                                .fillMaxHeight()
                                .background(Color.White.copy(alpha = 0.12f))
                        )

                        Column(
                            modifier = Modifier
                                .weight(1f)
                                .verticalScroll(rememberScrollState())
                        ) {
                            Text(
                                text = AppStrings.t(lang, "player.subtitles_title"),
                                color = Color.White.copy(alpha = 0.48f),
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(bottom = 2.dp, start = 4.dp)
                            )
                            MobileDelayAdjustmentRow(
                                title = AppStrings.t(lang, "player.subtitle_delay"),
                                valueMs = subtitleDelayMs,
                                onDecrease = { onSubtitleDelayChange(subtitleDelayMs - 250L) },
                                onIncrease = { onSubtitleDelayChange(subtitleDelayMs + 250L) }
                            )
                            MobileOpacityRow(
                                title = AppStrings.t(lang, "settings.subtitle_text"),
                                value = subtitleTextOpacity,
                                onChange = onSubtitleTextOpacityChange
                            )
                            MobileOpacityRow(
                                title = AppStrings.t(lang, "settings.subtitle_background"),
                                value = subtitleBackgroundOpacity,
                                onChange = onSubtitleBackgroundOpacityChange
                            )
                            MobileOpacityRow(
                                title = AppStrings.t(lang, "settings.subtitle_outline"),
                                value = subtitleOutlineOpacity,
                                onChange = onSubtitleOutlineOpacityChange
                            )
                        }
                    }
                } else {
                    // Tracks mode: audio tracks left | subtitle tracks right
                    Row(
                        modifier = Modifier.fillMaxSize(),
                        horizontalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = AppStrings.t(lang, "player.audio_title"),
                                color = Color.White.copy(alpha = 0.48f),
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(bottom = 10.dp, start = 4.dp)
                            )
                            LazyColumn(verticalArrangement = Arrangement.spacedBy(0.dp)) {
                                items(availableAudios, key = { it.id }) { track ->
                                    val selected = track == pendingAudio
                                    Row(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .clip(RoundedCornerShape(6.dp))
                                            .clickable { pendingAudio = track }
                                            .padding(horizontal = 4.dp, vertical = 11.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(10.dp)
                                    ) {
                                        Text(
                                            text = track.label,
                                            color = if (selected) Color.White else Color.White.copy(alpha = 0.55f),
                                            fontSize = 15.sp,
                                            fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal,
                                            modifier = Modifier.weight(1f),
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis
                                        )
                                        if (selected) {
                                            Icon(FluxaIcons.Check, null, tint = Color.White, modifier = Modifier.size(18.dp))
                                        }
                                    }
                                }
                            }
                        }

                        Box(
                            modifier = Modifier
                                .width(1.dp)
                                .fillMaxHeight()
                                .background(Color.White.copy(alpha = 0.12f))
                        )

                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = AppStrings.t(lang, "player.subtitles_title"),
                                color = Color.White.copy(alpha = 0.48f),
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(bottom = 10.dp, start = 4.dp)
                            )
                            LazyColumn(verticalArrangement = Arrangement.spacedBy(0.dp)) {
                                item(key = "off") {
                                    val selected = pendingSubtitle == null
                                    Row(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .clip(RoundedCornerShape(6.dp))
                                            .clickable { pendingSubtitle = null }
                                            .padding(horizontal = 4.dp, vertical = 11.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(10.dp)
                                    ) {
                                        Text(
                                            text = AppStrings.t(lang, "common.off"),
                                            color = if (selected) Color.White else Color.White.copy(alpha = 0.55f),
                                            fontSize = 15.sp,
                                            fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal,
                                            modifier = Modifier.weight(1f)
                                        )
                                        if (selected) {
                                            Icon(FluxaIcons.Check, null, tint = Color.White, modifier = Modifier.size(18.dp))
                                        }
                                    }
                                }
                                items(availableSubtitles, key = { it.id }) { track ->
                                    val selected = track == pendingSubtitle
                                    Row(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .clip(RoundedCornerShape(6.dp))
                                            .clickable { pendingSubtitle = track }
                                            .padding(horizontal = 4.dp, vertical = 11.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(10.dp)
                                    ) {
                                        Text(
                                            text = track.label,
                                            color = if (selected) Color.White else Color.White.copy(alpha = 0.55f),
                                            fontSize = 15.sp,
                                            fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal,
                                            modifier = Modifier.weight(1f),
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis
                                        )
                                        if (selected) {
                                            Icon(FluxaIcons.Check, null, tint = Color.White, modifier = Modifier.size(18.dp))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if (!settingsMode) {
                Spacer(modifier = Modifier.height(20.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(10.dp))
                            .clickable { onDismiss() }
                            .padding(horizontal = 18.dp, vertical = 10.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = AppStrings.t(lang, "common.cancel"),
                            color = Color.White.copy(alpha = 0.60f),
                            fontSize = 14.sp,
                            fontWeight = FontWeight.Medium
                        )
                    }
                    Spacer(modifier = Modifier.width(6.dp))
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(10.dp))
                            .background(Color.White)
                            .clickable { onApply(pendingAudio, pendingSubtitle) }
                            .padding(horizontal = 20.dp, vertical = 10.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = AppStrings.t(lang, "common.apply"),
                            color = Color.Black,
                            fontSize = 14.sp,
                            fontWeight = FontWeight.SemiBold
                        )
                    }
                }
            }
        }
    }
}

private fun formatDelayMs(valueMs: Long): String {
    val seconds = valueMs / 1000.0
    return "${if (valueMs >= 0) "+" else ""}${String.format(java.util.Locale.US, "%.2f", seconds)}s"
}
