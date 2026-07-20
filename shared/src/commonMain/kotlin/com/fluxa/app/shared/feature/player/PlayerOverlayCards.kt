package com.fluxa.app.shared.feature.player

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.remote.Video
import com.fluxa.app.player.TorrentStreamStatus
import com.fluxa.app.ui.catalog.BufferSnapshot
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaDimensions
import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.ui.catalog.PlaybackSnapshot

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage

fun playerText(lang: String?, key: String): String {
    return AppStrings.t(lang, "player.$key")
}

fun playerStatusText(lang: String?, value: String): String {
    return if (value.startsWith("player.")) AppStrings.t(lang, value) else value
}

@Composable
fun SkipSegmentCard(
    deviceType: DeviceType,
    type: String,
    nextEpisode: Video? = null,
    lang: String? = "en",
    autoAdvanceSeconds: Int? = null,
    onSkip: () -> Unit,
    onDismiss: () -> Unit
) {
    if (type == "outro" && nextEpisode != null) {
        NextEpisodeSkipCard(
            deviceType = deviceType,
            episode = nextEpisode,
            lang = lang,
            autoAdvanceSeconds = autoAdvanceSeconds,
            onSkip = onSkip
        )
        return
    }
    val label = when (type) {
        "intro" -> playerText(lang, "skip_intro")
        "outro" -> playerText(lang, "finish_episode")
        "recap" -> playerText(lang, "skip_recap")
        else -> playerText(lang, "skip")
    }
    val focusRequester = remember { FocusRequester() }
    var isFocused by remember { mutableStateOf(false) }
    LaunchedEffect(deviceType) {
        if (deviceType == DeviceType.TV) focusRequester.requestFocus()
    }
    Box(
        modifier = Modifier
            .widthIn(min = if (deviceType == DeviceType.Mobile) 108.dp else 160.dp)
            .clip(RoundedCornerShape(10.dp))
            .background(Color.White)
            .then(
                if (deviceType == DeviceType.TV) {
                    Modifier.border(2.dp, if (isFocused) FluxaColors.accent else Color.Transparent, RoundedCornerShape(10.dp))
                } else {
                    Modifier
                }
            )
            .focusRequester(focusRequester)
            .onFocusChanged { isFocused = it.isFocused }
            .focusable()
            .clickable { onSkip() }
            .padding(horizontal = if (deviceType == DeviceType.Mobile) 16.dp else 28.dp, vertical = if (deviceType == DeviceType.Mobile) 8.dp else 13.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = label,
            color = Color.Black,
            fontWeight = FontWeight.SemiBold,
            fontSize = if (deviceType == DeviceType.Mobile) 13.sp else 16.sp,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
    }
}

@Composable
private fun NextEpisodeSkipCard(
    deviceType: DeviceType,
    episode: Video,
    lang: String?,
    autoAdvanceSeconds: Int? = null,
    onSkip: () -> Unit
) {
    var remainingSeconds by remember(episode.id, autoAdvanceSeconds) { mutableStateOf(autoAdvanceSeconds) }
    LaunchedEffect(episode.id, autoAdvanceSeconds) {
        var remaining = autoAdvanceSeconds ?: return@LaunchedEffect
        while (remaining > 0) {
            kotlinx.coroutines.delay(1000)
            remaining -= 1
            remainingSeconds = remaining
        }
        onSkip()
    }
    val thumbnailSize = if (deviceType == DeviceType.Mobile) 46.dp else 74.dp
    val cardWidth = if (deviceType == DeviceType.Mobile) 240.dp else 364.dp
    val focusRequester = remember { FocusRequester() }
    var isFocused by remember { mutableStateOf(false) }
    LaunchedEffect(deviceType) {
        if (deviceType == DeviceType.TV) focusRequester.requestFocus()
    }
    Row(
        modifier = Modifier
            .width(cardWidth)
            .clip(RoundedCornerShape(if (deviceType == DeviceType.Mobile) 12.dp else 14.dp))
            .background(Color.Black.copy(alpha = 0.82f))
            .border(
                1.dp,
                if (deviceType == DeviceType.TV && isFocused) FluxaColors.accent else Color.White.copy(alpha = 0.16f),
                RoundedCornerShape(if (deviceType == DeviceType.Mobile) 12.dp else 14.dp)
            )
            .focusRequester(focusRequester)
            .onFocusChanged { isFocused = it.isFocused }
            .focusable()
            .clickable { onSkip() }
            .padding(if (deviceType == DeviceType.Mobile) 7.dp else 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(if (deviceType == DeviceType.Mobile) 10.dp else 12.dp)
    ) {
        AsyncImage(
            model = episode.thumbnail,
            contentDescription = null,
            modifier = Modifier
                .size(thumbnailSize)
                .clip(RoundedCornerShape(if (deviceType == DeviceType.Mobile) 8.dp else 10.dp))
                .background(Color.White.copy(alpha = 0.08f)),
            contentScale = ContentScale.Crop
        )
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(
                text = remainingSeconds?.let { "${AppStrings.t(lang, "auto.next_episode").uppercase()} · ${it}s" }
                    ?: AppStrings.t(lang, "auto.next_episode").uppercase(),
                color = Color.White,
                fontWeight = FontWeight.Black,
                fontSize = if (deviceType == DeviceType.Mobile) 10.sp else 14.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            Text(
                text = nextEpisodeSubtitle(lang, episode),
                color = Color.White.copy(alpha = 0.68f),
                fontWeight = FontWeight.Bold,
                fontSize = if (deviceType == DeviceType.Mobile) 10.sp else 13.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
        Icon(
            FluxaIcons.KeyboardArrowRight,
            null,
            tint = Color.White.copy(alpha = 0.92f),
            modifier = Modifier.size(if (deviceType == DeviceType.Mobile) 20.dp else 28.dp)
        )
    }
}

private fun nextEpisodeSubtitle(lang: String?, episode: Video): String {
    val season = episode.season ?: 1
    val number = episode.number ?: 0
    val name = episode.name.orEmpty().trim()
    return if (name.isBlank()) {
        AppStrings.format(lang, "player.next_episode_number_format", season, number)
    } else {
        AppStrings.format(lang, "player.next_episode_detail_format", season, number, name)
    }
}

@Composable
internal fun SegmentSkipChevronFeedback() {
    val transition = rememberInfiniteTransition(label = "segmentSkip")
    val phase by transition.animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(FluxaDimensions.AnimDuration.nextEpisode, easing = FastOutSlowInEasing),
            repeatMode = RepeatMode.Restart
        ),
        label = "segmentSkipPhase"
    )
    Canvas(modifier = Modifier.size(150.dp, 96.dp)) {
        val stroke = size.minDimension * 0.09f
        val centerY = size.height / 2f
        val startX = size.width * 0.30f
        repeat(3) { index ->
            val local = ((phase + index * 0.22f) % 1f)
            val alpha = 0.18f + local * 0.48f
            val x = startX + index * size.width * 0.18f + local * size.width * 0.05f
            drawLine(
                color = Color.White.copy(alpha = alpha),
                start = Offset(x - size.width * 0.055f, centerY - size.height * 0.14f),
                end = Offset(x + size.width * 0.055f, centerY),
                strokeWidth = stroke,
                cap = StrokeCap.Round
            )
            drawLine(
                color = Color.White.copy(alpha = alpha),
                start = Offset(x + size.width * 0.055f, centerY),
                end = Offset(x - size.width * 0.055f, centerY + size.height * 0.14f),
                strokeWidth = stroke,
                cap = StrokeCap.Round
            )
        }
    }
}

@Composable
fun ArtisticLoadingOverlay(bg: String, logo: String, title: String, status: TorrentStreamStatus, deviceType: DeviceType, buffer: BufferSnapshot = BufferSnapshot(), error: String? = null, currentUrl: String?, isSwitchingAudioSource: Boolean = false, currentSourceIdx: Int = 0, totalSources: Int = 0, playback: PlaybackSnapshot = PlaybackSnapshot(), hasRenderedFirstFrame: Boolean = false, lang: String? = "en", isTorrentUrl: Boolean = false) {
    val startupLoading = !hasRenderedFirstFrame
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(if (startupLoading) Color.Black else Color.Transparent)
    ) {
        if (startupLoading && bg.isNotEmpty()) {
            AsyncImage(
                bg,
                null,
                modifier = Modifier
                    .fillMaxSize()
                    .alpha(if (deviceType == DeviceType.TV) 0.35f else 0.30f),
                contentScale = ContentScale.Crop
            )
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Color.Black.copy(alpha = 0.34f))
            )
        }
        val isTorrent = isTorrentUrl || (currentUrl ?: "").contains(".torrent")
        val byteProgress = buffer.loadProgress > 0.015f ||
            buffer.bufferPercent > 0 ||
            status.bufferProgress > 0 ||
            status.downloadSpeed > 0.0

        val rebufferProgress = when {
            status.bufferProgress > 0 -> (status.bufferProgress / 100f).coerceIn(0f, 1f)
            buffer.bufferPercent > 0 -> (buffer.bufferPercent.toFloat() / 100f).coerceIn(0f, 1f)
            buffer.seekbarBufferedProgress > 0f -> buffer.seekbarBufferedProgress.coerceIn(0f, 1f)
            else -> 0f
        }
        val activeRebuffer = hasRenderedFirstFrame && playback.isBuffering
        val rawTargetProgress = when {
            activeRebuffer -> rebufferProgress
            hasRenderedFirstFrame && playback.hasStartedPlaying && !playback.isBuffering -> 1.0f
            buffer.loadProgress > 0f -> buffer.loadProgress.coerceIn(0f, 1f)
            status.bufferProgress > 0 -> (status.bufferProgress / 100f).coerceIn(0f, 1f)
            !isTorrent && buffer.bufferPercent > 0 -> (buffer.bufferPercent.toFloat() / 100f).coerceIn(0f, 1f)
            else -> 0f
        }
        val targetProgress = when {
            startupLoading && rawTargetProgress > 0f -> rawTargetProgress.coerceAtMost(0.92f)
            activeRebuffer && rawTargetProgress > 0f -> rawTargetProgress.coerceAtMost(0.96f)
            else -> rawTargetProgress
        }
        val loadProgress by animateFloatAsState(
            targetValue = targetProgress,
            animationSpec = tween(FluxaDimensions.AnimDuration.progressRing, easing = FastOutSlowInEasing),
            label = "logoLoadProgress"
        )
        val useBreathe = startupLoading && !byteProgress && loadProgress <= 0.015f && targetProgress <= 0.015f
        val hasProgress = !useBreathe && (activeRebuffer || byteProgress || loadProgress > 0.015f || targetProgress > 0.015f)
        val visibleLoadProgress = if (hasProgress) maxOf(loadProgress, if (activeRebuffer) 0.08f else 0.045f) else 0f
        val breatheTransition = rememberInfiniteTransition(label = "loadingLogoBreathe")
        val breatheAlpha by breatheTransition.animateFloat(
            initialValue = 0.42f,
            targetValue = 0.66f,
            animationSpec = infiniteRepeatable(
                animation = tween(FluxaDimensions.AnimDuration.marquee, easing = FastOutSlowInEasing),
                repeatMode = RepeatMode.Reverse
            ),
            label = "loadingLogoAlpha"
        )
        val containerWidth = if (deviceType == DeviceType.TV) 500.dp else 280.dp
        Column(modifier = Modifier.align(Alignment.Center), horizontalAlignment = Alignment.CenterHorizontally) {
            Box(
                modifier = Modifier
                    .width(containerWidth)
                    .height(200.dp),
                contentAlignment = Alignment.Center
            ) {
                var logoFailed by remember { mutableStateOf(false) }

                when {
                    logo.isEmpty() || logoFailed -> {
                        val spinnerAlpha = if (hasProgress) 0.18f + 0.72f * loadProgress.coerceIn(0f, 1f) else breatheAlpha
                        CircularProgressIndicator(
                            color = Color.White.copy(alpha = spinnerAlpha),
                            strokeWidth = 3.dp,
                            modifier = Modifier.size(52.dp)
                        )
                    }
                    !hasProgress -> {
                        AsyncImage(
                            model = logo,
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize().alpha(breatheAlpha),
                            contentScale = ContentScale.Fit,
                            onError = { logoFailed = true }
                        )
                    }
                    else -> {
                        AsyncImage(
                            model = logo,
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize().alpha(0.18f),
                            contentScale = ContentScale.Fit,
                            onError = { logoFailed = true }
                        )
                        val revealProgress = visibleLoadProgress.coerceIn(0f, 1f)
                        AsyncImage(
                            model = logo,
                            contentDescription = null,
                            modifier = Modifier
                                .fillMaxSize()
                                .drawWithContent {
                                    clipRect(right = size.width * revealProgress) {
                                        this@drawWithContent.drawContent()
                                    }
                                },
                            contentScale = ContentScale.Fit,
                            onError = { logoFailed = true }
                        )
                    }
                }
            }
        }

        if (error != null) {
            Box(modifier = Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.8f)), contentAlignment = Alignment.Center) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(FluxaIcons.ErrorOutline, null, tint = Color.White, modifier = Modifier.size(64.dp).padding(bottom = 16.dp))
                    Text(text = error, color = Color.White, fontSize = 20.sp, fontWeight = FontWeight.Bold, textAlign = TextAlign.Center, modifier = Modifier.padding(horizontal = 64.dp))
                }
            }
        }
    }
}

@Composable
fun VolumeBar(current: Int, max: Int) {
    val progress = current.toFloat() / max.toFloat()
    Row(modifier = Modifier.background(Color(0xB010141A), RoundedCornerShape(18.dp)).padding(horizontal = 16.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Icon(imageVector = when { progress == 0f -> FluxaIcons.VolumeMute; progress < 0.5f -> FluxaIcons.VolumeDown; else -> FluxaIcons.VolumeUp }, contentDescription = null, tint = Color.White, modifier = Modifier.size(20.dp))
        Box(modifier = Modifier.width(150.dp).height(4.dp).background(Color.White.copy(alpha = 0.2f), CircleShape)) { Box(modifier = Modifier.fillMaxWidth(progress).fillMaxHeight().background(Color.White, CircleShape)) }
        Text(text = "${(progress * 100).toInt()}%", color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.Bold)
    }
}
