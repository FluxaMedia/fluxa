@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.shared.feature.player

import com.fluxa.app.common.AppStrings
import com.fluxa.app.player.Chapter
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaDimensions
import com.fluxa.app.ui.catalog.FluxaIcons

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.Image
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
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
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.key.*
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.layout
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
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
    chapters: List<Chapter> = emptyList(),
    isPlaying: Boolean = false,
    seekPreviewBitmap: ImageBitmap? = null
) {
    var isFocused by remember { mutableStateOf(false) }
    var internalPos by remember { mutableFloatStateOf(position.toFloat()) }
    var consecutivePresses by remember { mutableIntStateOf(0) }
    var lastPressDirection by remember { mutableIntStateOf(0) }
    val seekbarAccent = FluxaColors.accent

    LaunchedEffect(position) { if (!isFocused) internalPos = position.toFloat() }

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
        if (isFocused && seekPreviewBitmap != null) {
            val progress = if (duration > 0) internalPos / duration else 0f
            BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
                val cardWidth = 240.dp
                val rawLeft = maxWidth * progress.coerceIn(0f, 1f) - cardWidth / 2
                val clampedLeft = rawLeft.coerceIn(0.dp, maxOf(0.dp, maxWidth - cardWidth))
                Image(
                    bitmap = seekPreviewBitmap,
                    contentDescription = null,
                    modifier = Modifier
                        .offset(x = clampedLeft, y = (-160).dp)
                        .width(cardWidth)
                        .aspectRatio(16f / 9f)
                        .clip(RoundedCornerShape(8.dp))
                        .border(1.dp, Color.White.copy(alpha = 0.18f), RoundedCornerShape(8.dp)),
                    contentScale = ContentScale.Crop
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
            val cornerRadius = CornerRadius(trackHeight / 2)

            if (chapterBoundaries.isEmpty()) {
                drawRoundRect(
                    color = FluxaColors.seekTrack,
                    size = Size(size.width, trackHeight),
                    cornerRadius = cornerRadius
                )
                drawRoundRect(
                    color = FluxaColors.seekBuffer,
                    size = Size(size.width * visibleBufferedFraction, trackHeight),
                    cornerRadius = cornerRadius
                )
                drawRoundRect(
                    color = seekbarAccent,
                    size = Size(size.width * progress, trackHeight),
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
                            topLeft = Offset(left, 0f),
                            size = Size(segWidth, trackHeight),
                            cornerRadius = cornerRadius
                        )
                        val bufferedRight = (size.width * visibleBufferedFraction).coerceIn(left, right)
                        if (bufferedRight > left) {
                            drawRoundRect(
                                color = FluxaColors.seekBuffer,
                                topLeft = Offset(left, 0f),
                                size = Size(bufferedRight - left, trackHeight),
                                cornerRadius = cornerRadius
                            )
                        }
                        val activeRight = (size.width * progress).coerceIn(left, right)
                        if (activeRight > left) {
                            drawRoundRect(
                                color = seekbarAccent,
                                topLeft = Offset(left, 0f),
                                size = Size(activeRight - left, trackHeight),
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
                center = Offset(size.width * progress, trackHeight / 2)
            )
        }
    }
}

@Composable
fun PlayerControlBtn(icon: ImageVector, deviceType: DeviceType, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(42.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .clickable { onClick() }
            .then(if (deviceType == DeviceType.TV) Modifier.focusable() else Modifier),
        contentAlignment = Alignment.Center
    ) {
        Icon(icon, null, modifier = Modifier.size(20.dp), tint = Color.White)
    }
}

@Composable
fun SeekIconButton(icon: ImageVector, deviceType: DeviceType, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(52.dp)
            .clip(CircleShape)
            .background(Color.White.copy(alpha = 0.06f))
            .clickable { onClick() }
            .then(if (deviceType == DeviceType.TV) Modifier.focusable() else Modifier),
        contentAlignment = Alignment.Center
    ) {
        Icon(icon, null, modifier = Modifier.size(24.dp), tint = Color.White)
    }
}

@Composable
fun TvPlayerUIContent(
    title: String, content: PlayerContentUiModel, lang: String, duration: Long, position: Long, bufferedFraction: Float, chapters: List<Chapter> = emptyList(), isPlaying: Boolean, isBuffering: Boolean, hasStartedPlaying: Boolean, deviceType: DeviceType,
    onPlayPause: () -> Unit, onSeek: (Long) -> Unit, onToggleSubtitles: () -> Unit, onToggleAspect: () -> Unit, onSpeedChange: (Float) -> Unit, playbackSpeed: Float, playPauseFocusRequester: FocusRequester, seekbarFocusRequester: FocusRequester,
    isScrubbing: Boolean, scrubPosition: Long, onScrubbingChange: (Boolean, Long) -> Unit, seekPreviewBitmap: ImageBitmap? = null,
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
                        fontSize = if (deviceType == DeviceType.TV) 24.sp else 18.sp,
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
                    .padding(horizontal = if (deviceType == DeviceType.TV) 18.dp else 12.dp, vertical = if (deviceType == DeviceType.TV) 14.dp else 10.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(if (deviceType == DeviceType.TV) 20.dp else 14.dp)
            ) {
                SeekIconButton(FluxaIcons.SkipPrevious, deviceType) {
                    if (hasPreviousEpisode) onPlayPrevious()
                }
                Box(
                    modifier = Modifier
                        .size(if (deviceType == DeviceType.TV) 78.dp else 60.dp)
                        .clip(CircleShape)
                        .background(Color.Black.copy(alpha = 0.46f))
                        .border(1.dp, Color.White.copy(alpha = 0.12f), CircleShape)
                        .clickable { onPlayPause() }
                        .then(if (deviceType == DeviceType.TV) Modifier.focusRequester(playPauseFocusRequester).focusProperties { down = seekbarFocusRequester }.focusable() else Modifier),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(if (isPlaying) FluxaIcons.Pause else FluxaIcons.PlayArrow, null, tint = Color.White, modifier = Modifier.size(if (deviceType == DeviceType.TV) 38.dp else 28.dp))
                }
                SeekIconButton(FluxaIcons.SkipNext, deviceType) {
                    if (hasNextEpisode) onPlayNext()
                }
            }

            Column(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .padding(horizontal = if (deviceType == DeviceType.TV) 4.dp else 0.dp)
                    .clip(RoundedCornerShape(18.dp))
                    .background(panelColor)
                    .padding(horizontal = if (deviceType == DeviceType.TV) 18.dp else 14.dp, vertical = if (deviceType == DeviceType.TV) 14.dp else 12.dp)
            ) {
                Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
                    Text(
                        text = formatPlayerTime(if (isScrubbing) scrubPosition else position),
                        color = Color.White,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Medium,
                        modifier = Modifier.width(if (duration >= 3600000) 88.dp else 70.dp)
                    )

                    Box(modifier = Modifier.weight(1f)) {
                        TVSeekbar(position, duration, bufferedFraction, onSeek, seekbarFocusRequester, playPauseFocusRequester, onScrubbingChange, seekForwardMs, seekBackwardMs, chapters, isPlaying, seekPreviewBitmap)
                    }

                    Column(
                        horizontalAlignment = Alignment.End,
                        modifier = Modifier.width(if (duration >= 3600000) 120.dp else 100.dp)
                    ) {
                        Text(
                            text = when {
                                isSwitchingAudioSource -> AppStrings.t(lang, "player.english_source")
                                hasStartedPlaying && duration > 0 -> formatPlayerTime(duration)
                                else -> if (detailedStatus.startsWith("player.")) AppStrings.t(lang, detailedStatus) else detailedStatus
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

@Composable
fun PlayerTopIconButton(icon: ImageVector, onClick: () -> Unit, modifier: Modifier = Modifier, contentDescription: String? = null) {
    Box(
        modifier = modifier
            .size(34.dp)
            .clip(CircleShape)
            .background(Color.Black.copy(alpha = 0.38f))
            .border(1.dp, Color.White.copy(alpha = 0.10f), CircleShape)
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(icon, contentDescription, tint = Color.White, modifier = Modifier.size(18.dp))
    }
}

@Composable
fun MobileTransportButton(icon: ImageVector, enabled: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(48.dp)
            .clip(CircleShape)
            .background(Color.Black.copy(alpha = if (enabled) 0.40f else 0.18f))
            .border(1.dp, Color.White.copy(alpha = if (enabled) 0.10f else 0.04f), CircleShape)
            .clickable(enabled = enabled) { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(
            icon,
            null,
            tint = Color.White.copy(alpha = if (enabled) 0.92f else 0.26f),
            modifier = Modifier.size(24.dp)
        )
    }
}

@Composable
fun MobileBottomAction(
    icon: ImageVector,
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier
            .clip(RoundedCornerShape(16.dp))
            .clickable { onClick() }
            .padding(horizontal = 4.dp, vertical = 6.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier.size(24.dp)
        )
        Spacer(modifier = Modifier.width(6.dp))
        Text(
            text = label,
            color = Color.White,
            fontSize = 13.sp,
            fontWeight = FontWeight.Medium,
            textAlign = TextAlign.Start,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
    }
}

private fun Modifier.overlayAboveBottom(gap: Dp): Modifier = layout { measurable, constraints ->
    val placeable = measurable.measure(constraints)
    val gapPx = gap.roundToPx()
    layout(0, 0) {
        placeable.place(0, -gapPx - placeable.height)
    }
}

@Composable
fun MobilePlayerSeekbar(
    position: Long,
    duration: Long,
    bufferedFraction: Float,
    onSeek: (Long) -> Unit,
    accentColor: Color = FluxaColors.accent,
    onScrubbingChange: (Boolean, Long) -> Unit = { _, _ -> },
    seekPreviewBitmap: ImageBitmap? = null,
    chapters: List<Chapter> = emptyList()
) {
    var sliderPosition by remember(duration) { mutableFloatStateOf(position.toFloat()) }
    var isDragging by remember { mutableStateOf(false) }

    LaunchedEffect(position, duration) {
        if (!isDragging) {
            sliderPosition = position.coerceIn(0L, duration.coerceAtLeast(0L)).toFloat()
        }
    }

    val chapterBoundaries = remember(chapters, duration) {
        if (chapters.size >= 2 && duration > 0L) {
            (chapters.map { it.startMs.toFloat() / duration } + 1f).sorted()
        } else {
            emptyList()
        }
    }
    val previewChapterTitle = remember(chapters, sliderPosition) {
        if (chapters.isEmpty()) null
        else chapters.lastOrNull { it.startMs <= sliderPosition }?.title?.takeIf { it.isNotBlank() }
    }

    BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
        if (isDragging && (seekPreviewBitmap != null || previewChapterTitle != null)) {
            val fraction = if (duration > 0L) (sliderPosition / duration.toFloat()).coerceIn(0f, 1f) else 0f
            val cardWidth = 200.dp
            val rawLeft = maxWidth * fraction - cardWidth / 2
            val clampedLeft = rawLeft.coerceIn(0.dp, maxOf(0.dp, maxWidth - cardWidth))

            Column(
                modifier = Modifier
                    .overlayAboveBottom(gap = 14.dp)
                    .offset(x = clampedLeft)
                    .width(cardWidth),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                if (seekPreviewBitmap != null) {
                    Image(
                        bitmap = seekPreviewBitmap,
                        contentDescription = null,
                        modifier = Modifier
                            .fillMaxWidth()
                            .aspectRatio(16f / 9f)
                            .clip(RoundedCornerShape(6.dp))
                            .border(1.dp, Color.White.copy(alpha = 0.18f), RoundedCornerShape(6.dp)),
                        contentScale = ContentScale.Crop
                    )
                }
                if (previewChapterTitle != null) {
                    Text(
                        text = previewChapterTitle,
                        color = Color.White.copy(alpha = 0.85f),
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        style = TextStyle(
                            shadow = Shadow(
                                color = Color.Black.copy(alpha = 0.7f),
                                blurRadius = 6f
                            )
                        ),
                        modifier = Modifier.padding(top = 2.dp)
                    )
                }
                Text(
                    text = formatPlayerTime(sliderPosition.toLong()),
                    color = Color.White,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Medium,
                    style = TextStyle(
                        shadow = Shadow(
                            color = Color.Black.copy(alpha = 0.7f),
                            blurRadius = 6f
                        )
                    ),
                    modifier = Modifier.padding(vertical = 3.dp)
                )
            }
        }

        Slider(
            value = sliderPosition,
            onValueChange = {
                if (!isDragging) {
                    isDragging = true
                }
                sliderPosition = it
                onScrubbingChange(true, it.toLong())
            },
            onValueChangeFinished = {
                isDragging = false
                onScrubbingChange(false, sliderPosition.toLong())
                onSeek(sliderPosition.toLong())
            },
            valueRange = 0f..duration.coerceAtLeast(1L).toFloat(),
            colors = SliderDefaults.colors(
                thumbColor = accentColor,
                activeTrackColor = accentColor,
                inactiveTrackColor = Color.Black
            ),
            modifier = Modifier.fillMaxWidth(),
            thumb = {
                Box(
                    modifier = Modifier
                        .size(12.dp)
                        .clip(CircleShape)
                        .background(accentColor)
                )
            },
            track = { sliderState ->
                Canvas(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(4.dp)
                ) {
                    val trackHeight = 4.dp.toPx()
                    val radius = CornerRadius(trackHeight / 2f)
                    val activeFraction = if (duration > 0L) {
                        (sliderState.value / duration.toFloat()).coerceIn(0f, 1f)
                    } else {
                        0f
                    }
                    val visibleBufferedFraction = bufferedFraction.coerceIn(0f, 1f).coerceAtLeast(activeFraction)

                    if (chapterBoundaries.isEmpty()) {
                        drawRoundRect(
                            color = FluxaColors.seekTrack,
                            size = Size(size.width, trackHeight),
                            cornerRadius = radius
                        )
                        drawRoundRect(
                            color = FluxaColors.seekBuffer,
                            size = Size(size.width * visibleBufferedFraction, trackHeight),
                            cornerRadius = radius
                        )
                        drawRoundRect(
                            color = accentColor,
                            size = Size(size.width * activeFraction, trackHeight),
                            cornerRadius = radius
                        )
                    } else {
                        val gapPx = 3.dp.toPx()
                        val segRadius = CornerRadius(trackHeight / 3f)
                        var start = 0f
                        for (end in chapterBoundaries) {
                            val left = size.width * start + gapPx / 2f
                            val right = (size.width * end - gapPx / 2f).coerceAtLeast(left)
                            val segWidth = right - left
                            if (segWidth > 0f) {
                                drawRoundRect(
                                    color = Color.Black,
                                    topLeft = Offset(left, 0f),
                                    size = Size(segWidth, trackHeight),
                                    cornerRadius = segRadius
                                )
                                val bufferedRight = (size.width * visibleBufferedFraction).coerceIn(left, right)
                                if (bufferedRight > left) {
                                    drawRoundRect(
                                        color = FluxaColors.seekBuffer,
                                        topLeft = Offset(left, 0f),
                                        size = Size(bufferedRight - left, trackHeight),
                                        cornerRadius = segRadius
                                    )
                                }
                                val activeRight = (size.width * activeFraction).coerceIn(left, right)
                                if (activeRight > left) {
                                    drawRoundRect(
                                        color = accentColor,
                                        topLeft = Offset(left, 0f),
                                        size = Size(activeRight - left, trackHeight),
                                        cornerRadius = segRadius
                                    )
                                }
                            }
                            start = end
                        }
                    }
                }
            }
        )
    }
}

@Composable
fun MobilePlayerUIContent(
    title: String,
    content: PlayerContentUiModel,
    lang: String,
    duration: Long,
    position: Long,
    bufferedFraction: Float,
    chapters: List<Chapter> = emptyList(),
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
    introDbMarkingEnabled: Boolean = false,
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
    seekPreviewBitmap: ImageBitmap? = null,
    accentColor: Color = FluxaColors.accent
) {
    val topFade = Brush.verticalGradient(
        listOf(Color.Black.copy(alpha = FluxaDimensions.PlayerChrome.topScrimAlpha), Color.Transparent)
    )
    val bottomFade = Brush.verticalGradient(
        listOf(Color.Transparent, Color.Black.copy(alpha = FluxaDimensions.PlayerChrome.bottomScrimAlpha))
    )

    Box(modifier = Modifier.fillMaxSize()) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(FluxaDimensions.PlayerChrome.topScrimHeight)
                .background(topFade)
        )
        Box(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .height(FluxaDimensions.PlayerChrome.bottomScrimHeight)
                .background(bottomFade)
        )

        Box(modifier = Modifier.fillMaxSize().windowInsetsPadding(WindowInsets.safeDrawing)) {

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
                    modifier = Modifier.align(Alignment.TopStart),
                    contentDescription = AppStrings.t(lang, "common.back")
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
                Row(
                    modifier = Modifier.align(Alignment.TopEnd),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    PlayerTopIconButton(FluxaIcons.Cast, onCast, contentDescription = AppStrings.t(lang, "auto.cast"))
                    PlayerTopIconButton(FluxaIcons.OpenInNew, onOpenInExternalPlayer, contentDescription = AppStrings.t(lang, "common.external_player"))
                    PlayerTopIconButton(FluxaIcons.PictureInPictureAlt, onPictureInPicture, contentDescription = AppStrings.t(lang, "common.picture_in_picture"))
                }
            }

            AnimatedVisibility(
                visible = !isScrubbing,
                modifier = Modifier.align(Alignment.Center),
                enter = fadeIn(),
                exit = fadeOut()
            ) {
                Row(
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
            }

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
                        text = formatPlayerTime(if (isScrubbing) scrubPosition else position),
                        color = Color.White,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Medium
                    )
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .padding(horizontal = 10.dp)
                    ) {
                        MobilePlayerSeekbar(
                            position = position,
                            duration = duration,
                            bufferedFraction = bufferedFraction,
                            onSeek = onSeek,
                            accentColor = FluxaColors.accent,
                            onScrubbingChange = onScrubbingChange,
                            seekPreviewBitmap = seekPreviewBitmap,
                            chapters = chapters
                        )
                    }
                    Text(
                        text = formatPlayerTime(duration),
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
                        label = "${AppStrings.t(lang, "player.speed")} (${playbackSpeed}x)",
                        onClick = { onShowSettings(2) },
                        modifier = Modifier.weight(1f)
                    )
                    if (supportsTrackSettings) {
                        MobileBottomAction(
                            icon = if (subtitlesEnabled) FluxaIcons.Subtitles else FluxaIcons.AudioTrack,
                            label = AppStrings.t(lang, "player.audio_and_subtitles"),
                            onClick = { onShowSettings(0) },
                            modifier = Modifier.weight(1f)
                        )
                    }
                    if (showEpisodesButton) {
                        MobileBottomAction(
                            icon = FluxaIcons.List,
                            label = AppStrings.t(lang, "player.episodes"),
                            onClick = { onShowSettings(3) },
                            modifier = Modifier.weight(1f)
                        )
                    }
                    if (showSourcesButton) {
                        MobileBottomAction(
                            icon = FluxaIcons.Storage,
                            label = AppStrings.t(lang, "player.source"),
                            onClick = { onShowSettings(4) },
                            modifier = Modifier.weight(1f)
                        )
                    }
                    if (introDbMarkingEnabled) {
                        MobileBottomAction(
                            icon = FluxaIcons.BookmarkBorder,
                            label = AppStrings.t(lang, "player.mark_segment"),
                            onClick = { onShowSettings(5) },
                            modifier = Modifier.weight(1f)
                        )
                    }
                }
            }
        }
    }
}
