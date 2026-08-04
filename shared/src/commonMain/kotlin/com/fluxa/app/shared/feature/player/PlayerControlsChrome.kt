@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.shared.feature.player

import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaDimensions
import com.fluxa.app.ui.catalog.FluxaIcons

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
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
import androidx.compose.material3.CircularProgressIndicator
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
                var playPauseFocused by remember { mutableStateOf(false) }
                Box(
                    modifier = Modifier
                        .size(if (deviceType == DeviceType.TV) 78.dp else 60.dp)
                        .clip(CircleShape)
                        .then(if (deviceType == DeviceType.TV) Modifier.onFocusChanged { playPauseFocused = it.isFocused } else Modifier)
                        .background(if (playPauseFocused) Color.White else Color.Black.copy(alpha = 0.46f))
                        .border(if (playPauseFocused) 0.dp else 1.dp, Color.White.copy(alpha = 0.12f), CircleShape)
                        .clickable { onPlayPause() }
                        .then(if (deviceType == DeviceType.TV) Modifier.focusRequester(playPauseFocusRequester).focusProperties { down = seekbarFocusRequester }.focusable() else Modifier),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        if (isPlaying) FluxaIcons.Pause else FluxaIcons.PlayArrow,
                        null,
                        tint = if (playPauseFocused) Color.Black else Color.White,
                        modifier = Modifier.size(if (deviceType == DeviceType.TV) 38.dp else 28.dp)
                    )
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
                    .overlayAboveBottom(gap = 6.dp)
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

        val thumbSize by animateDpAsState(targetValue = if (isDragging) 18.dp else 12.dp, label = "seekThumbSize")
        val trackThickness by animateDpAsState(
            targetValue = if (isDragging) FluxaDimensions.PlayerChrome.seekTrackHeightDragging else FluxaDimensions.PlayerChrome.seekTrackHeight,
            label = "seekTrackThickness"
        )

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
                        .size(thumbSize)
                        .clip(CircleShape)
                        .background(accentColor)
                )
            },
            track = { sliderState ->
                Canvas(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(trackThickness)
                ) {
                    val trackHeight = trackThickness.toPx()
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
    controls: MobilePlayerControlsUiModel,
    callbacks: MobilePlayerControlsCallbacks,
    chapters: List<Chapter> = emptyList(),
    isScrubbing: Boolean = false,
    scrubPosition: Long = 0L,
    onScrubbingChange: (Boolean, Long) -> Unit = { _, _ -> },
    seekPreviewBitmap: ImageBitmap? = null,
    accentColor: Color = FluxaColors.accent
) {
    val duration = controls.duration
    val position = controls.position
    var showRemainingTime by remember { mutableStateOf(false) }
    var showOverflowMenu by remember { mutableStateOf(false) }
    val chromeVisible = !isScrubbing
    val edgeMargin = FluxaDimensions.PlayerChrome.edgeMargin
    val dimAlpha by animateFloatAsState(
        targetValue = if (chromeVisible) FluxaDimensions.PlayerChrome.chromeDimAlpha else 0f,
        label = "chromeDim"
    )
    val chromeEnter = fadeIn() + slideInVertically { it / 6 }
    val chromeExit = fadeOut() + slideOutVertically { it / 6 }

    Box(modifier = Modifier.fillMaxSize()) {
        Box(modifier = Modifier.fillMaxSize().background(Color.Black.copy(alpha = dimAlpha)))

        Box(
            modifier = Modifier
                .fillMaxSize()
                .windowInsetsPadding(WindowInsets.safeDrawing)
                .padding(horizontal = edgeMargin)
        ) {
            AnimatedVisibility(
                visible = chromeVisible,
                modifier = Modifier.align(Alignment.TopCenter).fillMaxWidth(),
                enter = fadeIn() + slideInVertically { -it / 4 },
                exit = fadeOut() + slideOutVertically { -it / 4 }
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(top = 14.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    PlayerFlatIconButton(
                        icon = FluxaIcons.ArrowBack,
                        onClick = callbacks.onClose,
                        contentDescription = AppStrings.t(lang, "common.back")
                    )
                    Spacer(modifier = Modifier.width(10.dp))
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = title,
                            color = Color.White,
                            fontSize = FluxaDimensions.PlayerChrome.titleTextSize,
                            fontWeight = FontWeight.SemiBold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                        val metaLine = controls.episodeMetaLine?.takeIf { it.isNotBlank() }
                            ?: listOfNotNull(content.releaseInfo, content.runtime).joinToString("   ").takeIf { it.isNotBlank() }
                        if (!metaLine.isNullOrBlank()) {
                            Text(
                                text = metaLine,
                                color = Color.White.copy(alpha = FluxaDimensions.PlayerChrome.textAlphaSecondary),
                                fontSize = FluxaDimensions.PlayerChrome.metaTextSize,
                                fontWeight = FontWeight.Medium,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                modifier = Modifier.padding(top = 2.dp)
                            )
                        }
                    }
                    PlayerOverflowMenuButton(
                        lang = lang,
                        showOverflowMenu = showOverflowMenu,
                        onToggle = { showOverflowMenu = !showOverflowMenu },
                        onDismiss = { showOverflowMenu = false },
                        onPictureInPicture = callbacks.onPictureInPicture,
                        onCast = callbacks.onCast,
                        onOpenInExternalPlayer = callbacks.onOpenInExternalPlayer
                    )
                }
            }

            AnimatedVisibility(
                visible = chromeVisible,
                modifier = Modifier.align(Alignment.Center),
                enter = chromeEnter,
                exit = chromeExit
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(30.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    PlayerFlatIconButton(
                        icon = FluxaIcons.Replay10,
                        onClick = { callbacks.onSeek((position - controls.seekBackwardMs).coerceAtLeast(0L)) },
                        contentDescription = AppStrings.t(lang, "player.rewind_10"),
                        size = 26.dp,
                        touchSize = 48.dp,
                        pressScale = true
                    )
                    Box(modifier = Modifier.size(56.dp), contentAlignment = Alignment.Center) {
                        if (controls.isBuffering && controls.hasStartedPlaying) {
                            CircularProgressIndicator(
                                color = Color.White,
                                strokeWidth = 2.5.dp,
                                modifier = Modifier.size(40.dp)
                            )
                        } else {
                            PlayerFlatIconButton(
                                icon = if (controls.isPlaying) FluxaIcons.Pause else FluxaIcons.PlayArrow,
                                onClick = callbacks.onPlayPause,
                                contentDescription = if (controls.isPlaying) AppStrings.t(lang, "player.pause") else AppStrings.t(lang, "player.play"),
                                size = 40.dp,
                                touchSize = 56.dp,
                                pressScale = true
                            )
                        }
                    }
                    PlayerFlatIconButton(
                        icon = FluxaIcons.Forward10,
                        onClick = {
                            val maxDuration = duration.takeIf { it > 0L } ?: Long.MAX_VALUE
                            callbacks.onSeek((position + controls.seekForwardMs).coerceAtMost(maxDuration))
                        },
                        contentDescription = AppStrings.t(lang, "player.forward_10"),
                        size = 26.dp,
                        touchSize = 48.dp,
                        pressScale = true
                    )
                }
            }

            Column(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .padding(bottom = 14.dp)
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = formatPlayerTime(if (isScrubbing) scrubPosition else position),
                        color = Color.White,
                        fontSize = FluxaDimensions.PlayerChrome.timeTextSize,
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
                            bufferedFraction = controls.bufferedFraction,
                            onSeek = callbacks.onSeek,
                            accentColor = accentColor,
                            onScrubbingChange = onScrubbingChange,
                            seekPreviewBitmap = seekPreviewBitmap,
                            chapters = chapters
                        )
                    }
                    Text(
                        text = if (showRemainingTime) {
                            "-" + formatPlayerTime((duration - (if (isScrubbing) scrubPosition else position)).coerceAtLeast(0L))
                        } else {
                            formatPlayerTime(duration)
                        },
                        color = Color.White.copy(alpha = FluxaDimensions.PlayerChrome.textAlphaSecondary),
                        fontSize = FluxaDimensions.PlayerChrome.timeTextSize,
                        fontWeight = FontWeight.Medium,
                        modifier = Modifier.clickable { showRemainingTime = !showRemainingTime }
                    )
                }

                AnimatedVisibility(
                    visible = chromeVisible,
                    enter = fadeIn() + expandVertically(expandFrom = Alignment.Top),
                    exit = fadeOut() + shrinkVertically(shrinkTowards = Alignment.Top)
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth().padding(top = 10.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                            MobileBottomAction(
                                icon = FluxaIcons.Settings,
                                label = AppStrings.t(lang, "nav.settings"),
                                onClick = { callbacks.onShowSettings(-1) },
                                iconOnly = true
                            )
                        }
                        Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                            if (controls.hasNextEpisode) {
                                MobileBottomAction(
                                    icon = FluxaIcons.SkipNext,
                                    label = AppStrings.t(lang, "player.next_ep_short"),
                                    onClick = callbacks.onPlayNext
                                )
                            }
                            if (controls.showSourcesButton) {
                                MobileBottomAction(
                                    icon = FluxaIcons.Storage,
                                    label = AppStrings.t(lang, "player.source"),
                                    onClick = { callbacks.onShowSettings(4) }
                                )
                            }
                            if (controls.introDbMarkingEnabled) {
                                MobileBottomAction(
                                    icon = FluxaIcons.BookmarkBorder,
                                    label = AppStrings.t(lang, "player.mark_segment"),
                                    onClick = { callbacks.onShowSettings(5) },
                                    iconOnly = true
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
