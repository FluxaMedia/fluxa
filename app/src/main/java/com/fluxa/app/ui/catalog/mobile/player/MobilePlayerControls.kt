@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.player.AudioCodecBadge
import com.fluxa.app.player.VideoFormatBadge

import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
@Composable
internal fun MobilePlayerUIContent(
    title: String,
    content: PlayerContentUiModel,
    lang: String,
    duration: Long,
    position: Long,
    bufferedFraction: Float,
    isPlaying: Boolean,
    isBuffering: Boolean,
    hasStartedPlaying: Boolean,
    onPlayPause: () -> Unit,
    onSeek: (Long) -> Unit,
    playbackSpeed: Float,
    subtitlesEnabled: Boolean,
    supportsTrackSettings: Boolean,
    technicalInfo: String?,
    episodeMetaLine: String?,
    streamDetailLine: String?,
    seekForwardMs: Long,
    seekBackwardMs: Long,
    hasPreviousEpisode: Boolean,
    hasNextEpisode: Boolean,
    showSourcesButton: Boolean,
    showEpisodesButton: Boolean,
    onPlayPrevious: () -> Unit,
    onPlayNext: () -> Unit,
    onCast: () -> Unit,
    onOpenInExternalPlayer: () -> Unit,
    onPictureInPicture: () -> Unit,
    onToggleAspect: () -> Unit,
    onShowSettings: (Int) -> Unit,
    onClose: () -> Unit,
    isScrubbing: Boolean = false,
    scrubPosition: Long = 0L,
    onScrubbingChange: (Boolean, Long) -> Unit = { _, _ -> },
    accentColor: Color = Color(0xFFE53935),
    audioCodecBadge: AudioCodecBadge? = null,
    videoFormatBadge: VideoFormatBadge? = null
) {
    val topFade = Brush.verticalGradient(
        listOf(Color.Black.copy(alpha = 0.72f), Color.Transparent)
    )
    val bottomFade = Brush.verticalGradient(
        listOf(Color.Transparent, Color.Black.copy(alpha = 0.88f))
    )

    Box(modifier = Modifier.fillMaxSize()) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(156.dp)
                .background(topFade)
        )
        Box(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .height(230.dp)
                .background(bottomFade)
        )

        // Top bar
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .align(Alignment.TopCenter)
                .padding(horizontal = 18.dp)
                .padding(top = 14.dp)
        ) {
            PlayerTopIconButton(
                icon = FluxaIcons.ArrowBack,
                onClick = onClose,
                modifier = Modifier.align(Alignment.TopStart)
            )
            Column(
                modifier = Modifier
                    .align(Alignment.TopCenter)
                    .fillMaxWidth()
                    .padding(horizontal = 112.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                val topLine = streamDetailLine ?: episodeMetaLine ?: content.releaseInfo ?: content.runtime ?: ""
                val secondaryLine = if (!streamDetailLine.isNullOrBlank()) {
                    episodeMetaLine ?: content.releaseInfo ?: content.runtime ?: ""
                } else {
                    ""
                }
                Text(
                    text = title,
                    color = Color.White,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Medium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    textAlign = TextAlign.Center
                )
                if (topLine.isNotBlank()) {
                    Text(
                        text = topLine,
                        color = Color.White.copy(alpha = 0.82f),
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Medium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        textAlign = TextAlign.Center
                    )
                }
                if (secondaryLine.isNotBlank()) {
                    Text(
                        text = secondaryLine,
                        color = Color.White.copy(alpha = 0.68f),
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Medium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        textAlign = TextAlign.Center
                    )
                }
            }
            Column(
                modifier = Modifier.align(Alignment.TopEnd),
                horizontalAlignment = Alignment.End,
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    PlayerTopIconButton(FluxaIcons.Cast, onCast)
                    PlayerTopIconButton(FluxaIcons.OpenInNew, onOpenInExternalPlayer)
                    PlayerTopIconButton(FluxaIcons.PictureInPictureAlt, onPictureInPicture)
                }
                audioCodecBadge?.let { AudioCodecBadgeView(it) }
                videoFormatBadge?.let { VideoFormatBadgeView(it) }
            }
        }

        // Transport controls — centered in the screen
        Row(
            modifier = Modifier.align(Alignment.Center),
            horizontalArrangement = Arrangement.spacedBy(14.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            MobileTransportButton(FluxaIcons.SkipPrevious, enabled = hasPreviousEpisode) { onPlayPrevious() }
            Box(
                modifier = Modifier
                    .size(64.dp)
                    .clip(CircleShape)
                    .background(Color.Black.copy(alpha = 0.46f))
                    .border(1.dp, Color.White.copy(alpha = 0.12f), CircleShape)
                    .clickable { onPlayPause() },
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = if (isPlaying) FluxaIcons.Pause else FluxaIcons.PlayArrow,
                    contentDescription = null,
                    tint = Color.White,
                    modifier = Modifier.size(32.dp)
                )
            }
            MobileTransportButton(FluxaIcons.SkipNext, enabled = hasNextEpisode) { onPlayNext() }
        }

        // Seekbar + bottom actions — pinned to bottom
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .align(Alignment.BottomCenter)
                .padding(horizontal = 22.dp)
                .padding(bottom = 10.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = formatTime(if (isScrubbing) scrubPosition else position),
                    color = Color.White,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Medium
                )
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .padding(horizontal = 10.dp)
                ) {
                    val seekPreview = rememberSeekThumbnail(LocalSeekSurfaceView.current, scrubPosition, isScrubbing)
                    MobilePlayerSeekbar(
                        position = position,
                        duration = duration,
                        bufferedFraction = bufferedFraction,
                        onSeek = onSeek,
                        accentColor = Color(0xFFE50914),
                        onScrubbingChange = onScrubbingChange,
                        seekPreviewBitmap = seekPreview
                    )
                }
                Text(
                    text = formatTime(duration),
                    color = Color.White.copy(alpha = 0.84f),
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Medium
                )
            }

            Spacer(modifier = Modifier.height(6.dp))

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(2.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                MobileBottomAction(
                    icon = FluxaIcons.Speed,
                    label = "${playerText(lang, "speed")} (${playbackSpeed}x)",
                    onClick = { onShowSettings(2) },
                    modifier = Modifier.weight(1f)
                )
                if (supportsTrackSettings) {
                    MobileBottomAction(
                        icon = if (subtitlesEnabled) FluxaIcons.Subtitles else FluxaIcons.AudioTrack,
                        label = playerText(lang, "audio_and_subtitles"),
                        onClick = { onShowSettings(0) },
                        modifier = Modifier.weight(1f)
                    )
                }
                if (showEpisodesButton) {
                    MobileBottomAction(
                        icon = FluxaIcons.List,
                        label = playerText(lang, "episodes"),
                        onClick = { onShowSettings(3) },
                        modifier = Modifier.weight(1f)
                    )
                }
                if (showSourcesButton) {
                    MobileBottomAction(
                        icon = FluxaIcons.Storage,
                        label = playerText(lang, "source"),
                        onClick = { onShowSettings(4) },
                        modifier = Modifier.weight(1f)
                    )
                }
            }
        }
    }
}
