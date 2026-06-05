@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.domain.discovery.*

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
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.*
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import coil3.compose.AsyncImage
import com.fluxa.app.player.AudioCodecBadge
import com.fluxa.app.player.MediaTrack
import com.fluxa.app.player.VideoFormatBadge
import java.util.Locale

private fun String?.isTorrentPlaybackUrlForOverlay(): Boolean {
    return FluxaCoreNative.isTorrentPlaybackUrl(this)
}

private fun isEnglishUi(lang: String?): Boolean {
    return lang?.substringBefore('-')?.substringBefore('_')?.equals("en", ignoreCase = true) == true
}

internal fun playerText(lang: String?, key: String): String {
    return AppStrings.t(lang, "player.$key")
}

internal fun playerStatusText(lang: String?, value: String): String {
    return if (value.startsWith("player.")) AppStrings.t(lang, value) else value
}

internal suspend fun resolveIntroImdbId(
    viewModel: HomeViewModel,
    meta: Meta,
    videoId: String?,
    language: String
): String? {
    return viewModel.resolvePlaybackIntroImdbId(meta, videoId, language)
}

internal fun extractSeasonEpisode(videoId: String?): Pair<Int, Int>? {
    return FluxaCoreNative.parseEpisodeLocator(videoId)?.let { it.season to it.episode }
}

@Composable
internal fun SkipSegmentCard(
    deviceType: DeviceType,
    type: String,
    nextEpisode: Video? = null,
    lang: String? = "en",
    onSkip: () -> Unit,
    onDismiss: () -> Unit
) {
    if (type == "outro" && nextEpisode != null) {
        NextEpisodeSkipCard(
            deviceType = deviceType,
            episode = nextEpisode,
            lang = lang,
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
    Box(
        modifier = Modifier
            .widthIn(min = if (deviceType == DeviceType.Mobile) 130.dp else 160.dp)
            .clip(RoundedCornerShape(10.dp))
            .background(Color.White)
            .clickable { onSkip() }
            .padding(horizontal = if (deviceType == DeviceType.Mobile) 20.dp else 28.dp, vertical = if (deviceType == DeviceType.Mobile) 10.dp else 13.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = label,
            color = Color.Black,
            fontWeight = FontWeight.SemiBold,
            fontSize = if (deviceType == DeviceType.Mobile) 14.sp else 16.sp,
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
    onSkip: () -> Unit
) {
    val thumbnailSize = if (deviceType == DeviceType.Mobile) 58.dp else 74.dp
    val cardWidth = if (deviceType == DeviceType.Mobile) 286.dp else 364.dp
    Row(
        modifier = Modifier
            .width(cardWidth)
            .clip(RoundedCornerShape(14.dp))
            .background(Color.Black.copy(alpha = 0.82f))
            .border(1.dp, Color.White.copy(alpha = 0.16f), RoundedCornerShape(14.dp))
            .clickable { onSkip() }
            .padding(8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        AsyncImage(
            model = episode.thumbnail,
            contentDescription = null,
            modifier = Modifier
                .size(thumbnailSize)
                .clip(RoundedCornerShape(10.dp))
                .background(Color.White.copy(alpha = 0.08f)),
            contentScale = ContentScale.Crop
        )
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            Text(
                text = AppStrings.t(lang, "auto.next_episode").uppercase(Locale.ROOT),
                color = Color.White,
                fontWeight = FontWeight.Black,
                fontSize = if (deviceType == DeviceType.Mobile) 12.sp else 14.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            Text(
                text = nextEpisodeSubtitle(lang, episode),
                color = Color.White.copy(alpha = 0.68f),
                fontWeight = FontWeight.Bold,
                fontSize = if (deviceType == DeviceType.Mobile) 11.sp else 13.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
        Icon(
            FluxaIcons.KeyboardArrowRight,
            null,
            tint = Color.White.copy(alpha = 0.92f),
            modifier = Modifier.size(if (deviceType == DeviceType.Mobile) 24.dp else 28.dp)
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
            animation = tween(560, easing = FastOutSlowInEasing),
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
internal fun ArtisticLoadingOverlay(bg: String, logo: String, title: String, status: com.fluxa.app.player.TorrentStreamStatus, deviceType: DeviceType, buffer: BufferSnapshot = BufferSnapshot(), error: String? = null, currentUrl: String?, isSwitchingAudioSource: Boolean = false, currentSourceIdx: Int = 0, totalSources: Int = 0, playback: PlaybackSnapshot = PlaybackSnapshot(), lang: String? = "en") {
    Box(modifier = Modifier.fillMaxSize().background(Color.Black)) {
        if (bg.isNotEmpty()) {
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
        val currentUrlStr = currentUrl ?: ""
        val isTorrent = currentUrlStr.isTorrentPlaybackUrlForOverlay() || currentUrlStr.contains(".torrent")
        val bufferingMedia = buffer.loadProgress > 0f ||
            buffer.bufferPercent > 0 ||
            status.bufferProgress > 0 ||
            status.downloadSpeed > 0.0 ||
            status.activePeers > 0 ||
            status.totalPeers > 0 ||
            status.detailedStatus.isNotBlank()

        val targetProgress = when {
            playback.hasStartedPlaying -> 1.0f
            buffer.loadProgress > 0f -> buffer.loadProgress.coerceIn(0f, 1f)
            status.bufferProgress > 0 -> (status.bufferProgress / 100f).coerceIn(0f, 1f)
            !isTorrent && buffer.bufferPercent > 0 -> (buffer.bufferPercent.toFloat() / 100f).coerceIn(0f, 1f)
            else -> 0f
        }
        val loadProgress by animateFloatAsState(
            targetValue = targetProgress,
            animationSpec = tween(520, easing = FastOutSlowInEasing),
            label = "logoLoadProgress"
        )
        val hasProgress = bufferingMedia || loadProgress > 0.015f || targetProgress > 0.015f
        val visibleLoadProgress = if (hasProgress) maxOf(loadProgress, 0.045f) else 0f
        val breatheTransition = rememberInfiniteTransition(label = "loadingLogoBreathe")
        val breatheAlpha by breatheTransition.animateFloat(
            initialValue = 0.42f,
            targetValue = 0.66f,
            animationSpec = infiniteRepeatable(
                animation = tween(1120, easing = FastOutSlowInEasing),
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
                        // Idle: logo breathes at low alpha
                        AsyncImage(
                            model = logo,
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize().alpha(breatheAlpha),
                            contentScale = ContentScale.Fit,
                            onError = { logoFailed = true }
                        )
                    }
                    else -> {
                        // Ghost outline underneath
                        AsyncImage(
                            model = logo,
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize().alpha(0.18f),
                            contentScale = ContentScale.Fit,
                            onError = { logoFailed = true }
                        )
                        // Left-to-right reveal: clip drawing to progress fraction of width
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
internal fun PlayerUIContent(
    content: PlayerContentUiModel, lang: String, duration: Long, position: Long, bufferedFraction: Float, isPlaying: Boolean, isBuffering: Boolean, hasStartedPlaying: Boolean, deviceType: DeviceType,
    onPlayPause: () -> Unit, onSeek: (Long) -> Unit, onToggleSubtitles: () -> Unit, onToggleAspect: () -> Unit, onSpeedChange: (Float) -> Unit, playbackSpeed: Float, playPauseFocusRequester: FocusRequester, seekbarFocusRequester: FocusRequester,
    isScrubbing: Boolean, scrubPosition: Long, onScrubbingChange: (Boolean, Long) -> Unit,
    isSwitchingAudioSource: Boolean = false, detailedStatus: String = "", episodeMetaLine: String? = null, streamDetailLine: String? = null, subtitlesEnabled: Boolean = false, technicalInfo: String? = null,
    supportsTrackSettings: Boolean = true,
    seekForwardMs: Long = 10_000L, seekBackwardMs: Long = 10_000L,
    hasPreviousEpisode: Boolean = false,
    hasNextEpisode: Boolean = false,
    showSourcesButton: Boolean = false,
    showEpisodesButton: Boolean = false,
    onPlayPrevious: () -> Unit = {},
    onPlayNext: () -> Unit = {},
    onCast: () -> Unit = {},
    onOpenInExternalPlayer: () -> Unit = {},
    onPictureInPicture: () -> Unit = {},
    onShowSettings: (Int) -> Unit,
    onClose: () -> Unit,
    accentColor: Color = Color(0xFFE53935),
    audioCodecBadge: AudioCodecBadge? = null,
    videoFormatBadge: VideoFormatBadge? = null
) {
    if (deviceType == DeviceType.Mobile) {
        MobilePlayerUIContent(
            title = content.title,
            content = content,
            lang = lang,
            duration = duration,
            position = position,
            bufferedFraction = bufferedFraction,
            isPlaying = isPlaying,
            isBuffering = isBuffering,
            hasStartedPlaying = hasStartedPlaying,
            onPlayPause = onPlayPause,
            onSeek = onSeek,
            playbackSpeed = playbackSpeed,
            subtitlesEnabled = subtitlesEnabled,
            supportsTrackSettings = supportsTrackSettings,
            technicalInfo = technicalInfo,
            episodeMetaLine = episodeMetaLine,
            streamDetailLine = streamDetailLine,
            seekForwardMs = seekForwardMs,
            seekBackwardMs = seekBackwardMs,
            hasPreviousEpisode = hasPreviousEpisode,
            hasNextEpisode = hasNextEpisode,
            showSourcesButton = showSourcesButton,
            showEpisodesButton = showEpisodesButton,
            onPlayPrevious = onPlayPrevious,
            onPlayNext = onPlayNext,
            onCast = onCast,
            onOpenInExternalPlayer = onOpenInExternalPlayer,
            onPictureInPicture = onPictureInPicture,
            onToggleAspect = onToggleAspect,
            onShowSettings = onShowSettings,
            onClose = onClose,
            isScrubbing = isScrubbing,
            scrubPosition = scrubPosition,
            onScrubbingChange = onScrubbingChange,
            accentColor = accentColor,
            audioCodecBadge = audioCodecBadge,
            videoFormatBadge = videoFormatBadge
        )
        return
    }

    TvPlayerUIContent(
        title = content.title,
        content = content,
        lang = lang,
        duration = duration,
        position = position,
        bufferedFraction = bufferedFraction,
        isPlaying = isPlaying,
        isBuffering = isBuffering,
        hasStartedPlaying = hasStartedPlaying,
        deviceType = deviceType,
        onPlayPause = onPlayPause,
        onSeek = onSeek,
        onToggleSubtitles = onToggleSubtitles,
        onToggleAspect = onToggleAspect,
        onSpeedChange = onSpeedChange,
        playbackSpeed = playbackSpeed,
        playPauseFocusRequester = playPauseFocusRequester,
        seekbarFocusRequester = seekbarFocusRequester,
        isScrubbing = isScrubbing,
        scrubPosition = scrubPosition,
        onScrubbingChange = onScrubbingChange,
        isSwitchingAudioSource = isSwitchingAudioSource,
        detailedStatus = detailedStatus,
        episodeMetaLine = episodeMetaLine,
        streamDetailLine = streamDetailLine,
        subtitlesEnabled = subtitlesEnabled,
        supportsTrackSettings = supportsTrackSettings,
        technicalInfo = technicalInfo,
        seekForwardMs = seekForwardMs,
        seekBackwardMs = seekBackwardMs,
        hasPreviousEpisode = hasPreviousEpisode,
        hasNextEpisode = hasNextEpisode,
        showSourcesButton = showSourcesButton,
        showEpisodesButton = showEpisodesButton,
        onPlayPrevious = onPlayPrevious,
        onPlayNext = onPlayNext,
        onCast = onCast,
        onOpenInExternalPlayer = onOpenInExternalPlayer,
        onPictureInPicture = onPictureInPicture,
        onShowSettings = onShowSettings,
        onClose = onClose,
        audioCodecBadge = audioCodecBadge,
        videoFormatBadge = videoFormatBadge
    )
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
