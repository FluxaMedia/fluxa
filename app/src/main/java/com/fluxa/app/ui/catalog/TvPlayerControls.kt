package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.shared.feature.player.PlayerContentUiModel

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.*
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.key.*
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun TVSeekbar(
    position: Long,
    duration: Long,
    bufferedFraction: Float,
    onSeek: (Long) -> Unit,
    focusRequester: FocusRequester,
    playPauseFocusRequester: FocusRequester,
    onScrubbing: (Boolean, Long) -> Unit,
    seekForwardMs: Long = 10_000L,
    seekBackwardMs: Long = 10_000L,
    chapters: List<com.fluxa.app.player.Chapter> = emptyList(),
    isPlaying: Boolean = false,
    onScrubSeek: (Long) -> Unit = {}
) {
    var isFocused by remember { mutableStateOf(false) }
    var internalPos by remember { mutableFloatStateOf(position.toFloat()) }
    var consecutivePresses by remember { mutableIntStateOf(0) }
    var lastPressDirection by remember { mutableIntStateOf(0) }
    val seekbarAccent = FluxaColors.accent
    val seekPreview = rememberSeekThumbnail(LocalSeekSurfaceView.current, internalPos.toLong(), isFocused, position, isPlaying, onScrubSeek)

    LaunchedEffect(position) { if(!isFocused) internalPos = position.toFloat() }

    val chapterBoundaries = remember(chapters, duration) {
        if (chapters.size >= 2 && duration > 0L) {
            (chapters.map { it.startMs.toFloat() / duration } + 1f).sorted()
        } else {
            emptyList()
        }
    }
    val focusedChapterTitle = remember(chapters, internalPos, isFocused) {
        if (!isFocused || chapters.isEmpty()) null
        else chapters.lastOrNull { it.startMs <= internalPos }?.title?.takeIf { it.isNotBlank() }
    }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(48.dp)
            .focusRequester(focusRequester)
            .onFocusChanged { focusState ->
                isFocused = focusState.isFocused
                if (!focusState.isFocused) { consecutivePresses = 0; lastPressDirection = 0 }
                onScrubbing(focusState.isFocused, internalPos.toLong())
            }
            .focusProperties { up = playPauseFocusRequester }
            .focusable()
            .onKeyEvent {
                if (it.type == KeyEventType.KeyDown) {
                    when (it.key) {
                        Key.DirectionLeft -> {
                            if (lastPressDirection != -1) { consecutivePresses = 0; lastPressDirection = -1 }
                            consecutivePresses++
                            val multiplier = when { consecutivePresses >= 8 -> 6L; consecutivePresses >= 3 -> 3L; else -> 1L }
                            internalPos = (internalPos - seekBackwardMs * multiplier).coerceAtLeast(0f)
                            onScrubbing(true, internalPos.toLong())
                            true
                        }
                        Key.DirectionRight -> {
                            if (lastPressDirection != 1) { consecutivePresses = 0; lastPressDirection = 1 }
                            consecutivePresses++
                            val multiplier = when { consecutivePresses >= 8 -> 6L; consecutivePresses >= 3 -> 3L; else -> 1L }
                            internalPos = (internalPos + seekForwardMs * multiplier).coerceAtMost(duration.toFloat())
                            onScrubbing(true, internalPos.toLong())
                            true
                        }
                        Key.Enter, Key.DirectionCenter -> {
                            consecutivePresses = 0; lastPressDirection = 0
                            onSeek(internalPos.toLong())
                            onScrubbing(false, internalPos.toLong())
                            true
                        }
                        else -> false
                    }
                } else false
            }
            .pointerInput(Unit) {
               detectTapGestures { offset ->
                   val newPos = (offset.x / size.width) * duration
                   internalPos = newPos.coerceIn(0f, duration.toFloat())
                   onSeek(internalPos.toLong())
               }
            }
            .pointerInput(Unit) {
                detectDragGestures(
                    onDragStart = { onScrubbing(true, internalPos.toLong()) },
                    onDragEnd = { onSeek(internalPos.toLong()); onScrubbing(false, internalPos.toLong()) },
                    onDrag = { change, _ ->
                        val newPos = (change.position.x / size.width) * duration
                        internalPos = newPos.coerceIn(0f, duration.toFloat())
                        onScrubbing(true, internalPos.toLong())
                    }
                )
            }
    ) {
        if (isFocused && seekPreview != null) {
            val progress = if (duration > 0) internalPos / duration else 0f
            BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
                val cardWidth = 240.dp
                val rawLeft = maxWidth * progress.coerceIn(0f, 1f) - cardWidth / 2
                val clampedLeft = rawLeft.coerceIn(0.dp, maxOf(0.dp, maxWidth - cardWidth))
                androidx.compose.foundation.Image(
                    bitmap = seekPreview,
                    contentDescription = null,
                    modifier = Modifier
                        .offset(x = clampedLeft, y = (-160).dp)
                        .width(cardWidth)
                        .aspectRatio(16f / 9f)
                        .clip(RoundedCornerShape(8.dp))
                        .border(1.dp, Color.White.copy(alpha = 0.18f), RoundedCornerShape(8.dp)),
                    contentScale = androidx.compose.ui.layout.ContentScale.Crop
                )
            }
        }
        if (focusedChapterTitle != null) {
            val progress = if (duration > 0) internalPos / duration else 0f
            BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
                val labelWidth = 220.dp
                val rawLeft = maxWidth * progress.coerceIn(0f, 1f) - labelWidth / 2
                val clampedLeft = rawLeft.coerceIn(0.dp, maxOf(0.dp, maxWidth - labelWidth))
                Text(
                    text = focusedChapterTitle,
                    color = Color.White.copy(alpha = 0.85f),
                    fontSize = 13.sp,
                    fontWeight = FontWeight.SemiBold,
                    textAlign = TextAlign.Center,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier
                        .offset(x = clampedLeft, y = (-4).dp)
                        .width(labelWidth)
                )
            }
        }
        Canvas(modifier = Modifier.fillMaxSize().padding(vertical = 22.dp)) {
            val trackHeight = FluxaDimensions.PlayerChrome.seekTrackHeight.toPx()
            val thumbRadius = if (isFocused) 6.dp.toPx() else 4.dp.toPx()
            val progress = if (duration > 0) internalPos / duration else 0f
            val visibleBufferedFraction = bufferedFraction.coerceIn(0f, 1f).coerceAtLeast(progress)
            val cornerRadius = androidx.compose.ui.geometry.CornerRadius(trackHeight / 2)

            if (chapterBoundaries.isEmpty()) {
                drawRoundRect(
                    color = FluxaColors.seekTrack,
                    size = androidx.compose.ui.geometry.Size(size.width, trackHeight),
                    cornerRadius = cornerRadius
                )

                drawRoundRect(
                    color = FluxaColors.seekBuffer,
                    size = androidx.compose.ui.geometry.Size(size.width * visibleBufferedFraction, trackHeight),
                    cornerRadius = cornerRadius
                )

                drawRoundRect(
                    color = seekbarAccent,
                    size = androidx.compose.ui.geometry.Size(size.width * progress, trackHeight),
                    cornerRadius = cornerRadius
                )
            } else {
                val gapPx = 3.dp.toPx()
                var start = 0f
                for (end in chapterBoundaries) {
                    val left = size.width * start + gapPx / 2f
                    val right = (size.width * end - gapPx / 2f).coerceAtLeast(left)
                    val segWidth = right - left
                    if (segWidth > 0f) {
                        drawRoundRect(
                            color = FluxaColors.seekTrack,
                            topLeft = androidx.compose.ui.geometry.Offset(left, 0f),
                            size = androidx.compose.ui.geometry.Size(segWidth, trackHeight),
                            cornerRadius = cornerRadius
                        )
                        val bufferedRight = (size.width * visibleBufferedFraction).coerceIn(left, right)
                        if (bufferedRight > left) {
                            drawRoundRect(
                                color = FluxaColors.seekBuffer,
                                topLeft = androidx.compose.ui.geometry.Offset(left, 0f),
                                size = androidx.compose.ui.geometry.Size(bufferedRight - left, trackHeight),
                                cornerRadius = cornerRadius
                            )
                        }
                        val activeRight = (size.width * progress).coerceIn(left, right)
                        if (activeRight > left) {
                            drawRoundRect(
                                color = seekbarAccent,
                                topLeft = androidx.compose.ui.geometry.Offset(left, 0f),
                                size = androidx.compose.ui.geometry.Size(activeRight - left, trackHeight),
                                cornerRadius = cornerRadius
                            )
                        }
                    }
                    start = end
                }
            }

            drawCircle(
                color = seekbarAccent,
                radius = thumbRadius,
                center = androidx.compose.ui.geometry.Offset(size.width * progress, trackHeight / 2)
            )
        }
    }
}

@Composable
fun PlayerControlBtn(icon: ImageVector, deviceType: DeviceType, onClick: () -> Unit) {
    Box(modifier = Modifier.size(42.dp).clip(RoundedCornerShape(12.dp)).background(Color.White.copy(alpha = 0.04f)).clickable { onClick() }.then(if(deviceType == DeviceType.TV) Modifier.focusable() else Modifier), contentAlignment = Alignment.Center) {
        Icon(icon, null, modifier = Modifier.size(20.dp), tint = Color.White)
    }
}

@Composable
fun SeekIconButton(icon: ImageVector, deviceType: DeviceType, onClick: () -> Unit) {
    Box(modifier = Modifier.size(52.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.06f)).clickable { onClick() }.then(if(deviceType == DeviceType.TV) Modifier.focusable() else Modifier), contentAlignment = Alignment.Center) {
        Icon(icon, null, modifier = Modifier.size(24.dp), tint = Color.White)
    }
}

@Composable
internal fun TvPlayerUIContent(
    title: String, content: PlayerContentUiModel, lang: String, duration: Long, position: Long, bufferedFraction: Float, chapters: List<com.fluxa.app.player.Chapter> = emptyList(), isPlaying: Boolean, isBuffering: Boolean, hasStartedPlaying: Boolean, deviceType: DeviceType,
    onPlayPause: () -> Unit, onSeek: (Long) -> Unit, onToggleSubtitles: () -> Unit, onToggleAspect: () -> Unit, onSpeedChange: (Float) -> Unit, playbackSpeed: Float, playPauseFocusRequester: FocusRequester, seekbarFocusRequester: FocusRequester,
    isScrubbing: Boolean, scrubPosition: Long, onScrubbingChange: (Boolean, Long) -> Unit, onScrubSeek: (Long) -> Unit = {},
    isSwitchingAudioSource: Boolean = false, detailedStatus: String = "", episodeMetaLine: String? = null, streamDetailLine: String? = null, subtitlesEnabled: Boolean = false, technicalInfo: String? = null,
    supportsTrackSettings: Boolean = true,
    seekForwardMs: Long = 10_000L, seekBackwardMs: Long = 10_000L,
    hasPreviousEpisode: Boolean = false,
    hasNextEpisode: Boolean = false,
    showSourcesButton: Boolean = false,
    showEpisodesButton: Boolean = false,
    introDbMarkingEnabled: Boolean = false,
    onPlayPrevious: () -> Unit = {},
    onPlayNext: () -> Unit = {},
    onCast: () -> Unit = {},
    onOpenInExternalPlayer: () -> Unit = {},
    onPictureInPicture: () -> Unit = {},
    onShowSettings: (Int) -> Unit,
    onClose: () -> Unit
) {
    val panelColor = Color(0x8010141A)
    Box(modifier = Modifier.fillMaxSize()) {
        Box(modifier = Modifier.fillMaxWidth().height(FluxaDimensions.PlayerChrome.topScrimHeight).background(Brush.verticalGradient(listOf(Color.Black.copy(alpha = FluxaDimensions.PlayerChrome.topScrimAlpha), Color.Transparent))))
        Box(modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(FluxaDimensions.PlayerChrome.bottomScrimHeight).background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = FluxaDimensions.PlayerChrome.bottomScrimAlpha)))))
        Box(
            modifier = Modifier
                .fillMaxSize()
                .windowInsetsPadding(WindowInsets.safeDrawing)
                .padding(if (deviceType == DeviceType.TV) 28.dp else 16.dp)
        ) {
            Row(
                modifier = Modifier
                    .align(Alignment.TopCenter)
                    .fillMaxWidth()
                    .padding(horizontal = if (deviceType == DeviceType.TV) 4.dp else 0.dp, vertical = 4.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = title,
                        color = Color.White,
                        fontSize = if(deviceType == DeviceType.TV) 24.sp else 18.sp,
                        fontWeight = FontWeight.Black,
                        maxLines = 1
                    )
                    streamDetailLine?.takeIf { it.isNotBlank() }?.let { detail ->
                        Spacer(modifier = Modifier.height(3.dp))
                        Text(
                            text = detail,
                            color = Color.White.copy(alpha = 0.82f),
                            fontSize = 13.sp,
                            fontWeight = FontWeight.Medium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        text = episodeMetaLine ?: buildString {
                            if (!content.releaseInfo.isNullOrBlank()) append(content.releaseInfo)
                            if (!content.runtime.isNullOrBlank()) {
                                if (isNotEmpty()) append("    ")
                                append(content.runtime)
                            }
                        },
                        color = Color.White.copy(alpha = 0.68f),
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Medium,
                        maxLines = 1
                    )
                }

                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    PlayerControlBtn(FluxaIcons.OpenInNew, deviceType) { onOpenInExternalPlayer() }
                    PlayerControlBtn(FluxaIcons.AspectRatio, deviceType) { onToggleAspect() }
                    if (introDbMarkingEnabled) {
                        PlayerControlBtn(FluxaIcons.BookmarkBorder, deviceType) { onShowSettings(5) }
                    }
                }
            }

            Row(
                modifier = Modifier
                    .align(Alignment.Center)
                    .clip(RoundedCornerShape(999.dp))
                    .background(Color.Black.copy(alpha = 0.22f))
                    .padding(horizontal = if(deviceType == DeviceType.TV) 18.dp else 12.dp, vertical = if(deviceType == DeviceType.TV) 14.dp else 10.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(if(deviceType == DeviceType.TV) 20.dp else 14.dp)
            ) {
                SeekIconButton(FluxaIcons.SkipPrevious, deviceType) {
                    if (hasPreviousEpisode) onPlayPrevious()
                }
                Box(
                    modifier = Modifier
                        .size(if(deviceType == DeviceType.TV) 78.dp else 60.dp)
                        .clip(CircleShape)
                        .background(Color.Black.copy(alpha = 0.46f))
                        .border(1.dp, Color.White.copy(alpha = 0.12f), CircleShape)
                        .clickable { onPlayPause() }
                        .then(if(deviceType == DeviceType.TV) Modifier.focusRequester(playPauseFocusRequester).focusProperties { down = seekbarFocusRequester }.focusable() else Modifier),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(if (isPlaying) FluxaIcons.Pause else FluxaIcons.PlayArrow, null, tint = Color.White, modifier = Modifier.size(if(deviceType == DeviceType.TV) 38.dp else 28.dp))
                }
                SeekIconButton(FluxaIcons.SkipNext, deviceType) {
                    if (hasNextEpisode) onPlayNext()
                }
            }

            Column(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .padding(horizontal = if(deviceType == DeviceType.TV) 4.dp else 0.dp)
                    .clip(RoundedCornerShape(18.dp))
                    .background(panelColor)
                    .padding(horizontal = if(deviceType == DeviceType.TV) 18.dp else 14.dp, vertical = if(deviceType == DeviceType.TV) 14.dp else 12.dp)
            ) {
                
                Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
                    Text(
                        text = formatTime(if (isScrubbing) scrubPosition else position),
                        color = Color.White,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Medium,
                        modifier = Modifier.width(if (duration >= 3600000) 88.dp else 70.dp)
                    )

                    Box(modifier = Modifier.weight(1f)) {
                        TVSeekbar(position, duration, bufferedFraction, onSeek, seekbarFocusRequester, playPauseFocusRequester, onScrubbingChange, seekForwardMs, seekBackwardMs, chapters, isPlaying, onScrubSeek)
                    }

                    Column(
                        horizontalAlignment = Alignment.End,
                        modifier = Modifier.width(if (duration >= 3600000) 120.dp else 100.dp)
                    ) {
                        Text(
                            text = when {
                                isSwitchingAudioSource -> playerText(lang, "english_source")
                                hasStartedPlaying && duration > 0 -> formatTime(duration)
                                else -> playerStatusText(lang, detailedStatus)
                            },
                            color = Color.White,
                            fontSize = 14.sp,
                            fontWeight = FontWeight.Bold,
                            textAlign = TextAlign.End
                        )
                    }
                }

                Spacer(modifier = Modifier.height(8.dp))

                Row(
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    if (supportsTrackSettings) {
                        PlayerControlBtn(FluxaIcons.AudioTrack, deviceType) { onShowSettings(0) }
                        PlayerControlBtn(if (subtitlesEnabled) FluxaIcons.Subtitles else FluxaIcons.SubtitlesOff, deviceType) { onShowSettings(1) }
                    }
                    PlayerControlBtn(FluxaIcons.Speed, deviceType) { onShowSettings(2) }
                    if (showSourcesButton) {
                        PlayerControlBtn(FluxaIcons.Storage, deviceType) { onShowSettings(4) }
                    }
                }
            }
        }
    }

}
