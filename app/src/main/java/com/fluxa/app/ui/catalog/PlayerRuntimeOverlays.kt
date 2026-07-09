@file:OptIn(androidx.compose.animation.ExperimentalAnimationApi::class)

package com.fluxa.app.ui.catalog

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Video

@Composable
internal fun PlayerSkipSegmentOverlay(
    currentPosition: Long,
    skipSegments: List<IntroTimestamps>,
    dismissedSkipSegments: Set<String>,
    hasStartedPlaying: Boolean,
    showControls: Boolean,
    deviceType: DeviceType,
    nextEpisode: Video?,
    nextEpisodeThresholdReached: Boolean,
    autoSkipSegments: Boolean,
    autoPlayCountdownSeconds: Int? = null,
    lang: String,
    onSkipSegment: (IntroTimestamps) -> Unit,
    onPlayNextEpisode: () -> Unit,
    onDismissSegment: (IntroTimestamps) -> Unit,
    onNextEpisodeCardShown: () -> Unit,
    modifier: Modifier = Modifier
) {
    val activeSkipSegment = remember(currentPosition, skipSegments, dismissedSkipSegments, hasStartedPlaying, autoSkipSegments) {
        if (!hasStartedPlaying || autoSkipSegments) null
        else skipSegments.find { segment ->
            currentPosition in segment.startTime until segment.endTime &&
                segment.dismissKey() !in dismissedSkipSegments
        }
    }
    val showNextEpisodeCard = activeSkipSegment == null && nextEpisodeThresholdReached

    LaunchedEffect(showNextEpisodeCard) {
        if (showNextEpisodeCard) onNextEpisodeCardShown()
    }

    AnimatedVisibility(
        visible = activeSkipSegment != null || showNextEpisodeCard,
        enter = fadeIn(animationSpec = tween(FluxaDimensions.AnimDuration.scaleAlpha)) + slideInVertically(animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand, easing = FastOutSlowInEasing)) { it / 2 },
        exit = fadeOut(animationSpec = tween(FluxaDimensions.AnimDuration.quick)) + slideOutVertically(animationSpec = tween(FluxaDimensions.AnimDuration.scaleAlpha, easing = FastOutSlowInEasing)) { it / 2 },
        modifier = modifier
            .padding(
                start = 18.dp,
                end = 14.dp,
                bottom = if (deviceType == DeviceType.Mobile) {
                    if (showControls) 132.dp else 76.dp
                } else {
                    92.dp
                }
            )
            .zIndex(280f)
    ) {
        val episode = nextEpisode
        if (showNextEpisodeCard && episode != null) {
            SkipSegmentCard(
                deviceType = deviceType,
                type = "outro",
                nextEpisode = episode,
                lang = lang,
                autoAdvanceSeconds = autoPlayCountdownSeconds,
                onSkip = onPlayNextEpisode,
                onDismiss = {}
            )
        } else activeSkipSegment?.let { segment ->
            SkipSegmentCard(
                deviceType = deviceType,
                type = segment.type,
                nextEpisode = nextEpisode,
                lang = lang,
                onSkip = if (segment.type == "outro" && nextEpisode != null) onPlayNextEpisode else { { onSkipSegment(segment) } },
                onDismiss = { onDismissSegment(segment) }
            )
        }
    }
}

internal enum class ZoomOverlayMode { Original, Fit, Zoom }

@Composable
internal fun BoxScope.PlayerTransientOverlays(
    showSegmentSkipFeedback: Boolean,
    holdSpeedVisible: Boolean,
    activeProfile: UserProfile?,
    deviceType: DeviceType,
    showVolumeBar: Boolean,
    currentVolume: Int,
    maxVolume: Int,
    showSeekFeedback: Boolean,
    seekDirection: Int,
    seekFeedbackMs: Long,
    seekForwardMs: Long,
    seekBackwardMs: Long,
    showZoomOverlay: Boolean = false,
    zoomOverlayMode: ZoomOverlayMode = ZoomOverlayMode.Original,
    zoomLabelText: String = ""
) {
    AnimatedVisibility(
        visible = showSegmentSkipFeedback,
        enter = fadeIn(animationSpec = tween(FluxaDimensions.AnimDuration.blink)) + scaleIn(animationSpec = tween(FluxaDimensions.AnimDuration.quick), initialScale = 0.82f),
        exit = fadeOut(animationSpec = tween(FluxaDimensions.AnimDuration.settingsExpandAlt)) + scaleOut(animationSpec = tween(FluxaDimensions.AnimDuration.settingsExpandAlt), targetScale = 1.08f),
        modifier = Modifier
            .align(Alignment.Center)
            .zIndex(320f)
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            SegmentSkipChevronFeedback()
        }
    }

    AnimatedVisibility(
        visible = showZoomOverlay,
        enter = fadeIn(animationSpec = tween(FluxaDimensions.AnimDuration.routeExit)) + scaleIn(animationSpec = tween(FluxaDimensions.AnimDuration.routeExit, easing = FastOutSlowInEasing), initialScale = 0.80f),
        exit = fadeOut(animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand)) + scaleOut(animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand), targetScale = 1.06f),
        modifier = Modifier
            .align(Alignment.TopCenter)
            .padding(top = if (deviceType == DeviceType.Mobile) 34.dp else 54.dp)
            .zIndex(295f)
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
            Row(
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(Color.Black.copy(alpha = 0.74f))
                    .border(1.dp, Color.White.copy(alpha = 0.14f), RoundedCornerShape(999.dp))
                    .padding(horizontal = 14.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                val icon = when (zoomOverlayMode) {
                    ZoomOverlayMode.Original -> FluxaIcons.ZoomFitScreen
                    ZoomOverlayMode.Fit -> FluxaIcons.ZoomOutMap
                    ZoomOverlayMode.Zoom -> FluxaIcons.ZoomIn
                }
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    tint = Color.White,
                    modifier = Modifier.size(15.dp)
                )
                Text(
                    text = zoomLabelText,
                    color = Color.White,
                    fontSize = if (deviceType == DeviceType.Mobile) 15.sp else 18.sp,
                    fontWeight = FontWeight.SemiBold
                )
            }
        }
    }

    AnimatedVisibility(
        visible = holdSpeedVisible,
        enter = fadeIn(animationSpec = tween(FluxaDimensions.AnimDuration.heightAnim)) + scaleIn(animationSpec = tween(FluxaDimensions.AnimDuration.scaleAlpha, easing = FastOutSlowInEasing), initialScale = 0.86f),
        exit = fadeOut(animationSpec = tween(FluxaDimensions.AnimDuration.quick)) + scaleOut(animationSpec = tween(FluxaDimensions.AnimDuration.quick), targetScale = 0.92f),
        modifier = Modifier
            .align(Alignment.TopCenter)
            .padding(top = if (deviceType == DeviceType.Mobile) 34.dp else 54.dp)
            .zIndex(310f)
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(Color.Black.copy(alpha = 0.74f))
                    .border(1.dp, Color.White.copy(alpha = 0.14f), RoundedCornerShape(999.dp))
                    .padding(horizontal = 18.dp, vertical = 8.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "${activeProfile?.safeHoldSpeed ?: 2f}x",
                    color = Color.White,
                    fontSize = if (deviceType == DeviceType.Mobile) 18.sp else 22.sp,
                    fontWeight = FontWeight.Black
                )
            }
        }
    }

    AnimatedVisibility(
        visible = showVolumeBar,
        enter = fadeIn() + slideInVertically(),
        exit = fadeOut() + slideOutVertically(),
        modifier = Modifier
            .align(Alignment.TopCenter)
            .padding(top = 40.dp)
            .zIndex(300f)
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
            VolumeBar(currentVolume, maxVolume)
        }
    }

    AnimatedVisibility(
        visible = showSeekFeedback,
        enter = if (seekDirection > 0) slideInHorizontally { it } + fadeIn() else slideInHorizontally { -it } + fadeIn(),
        exit = if (seekDirection > 0) slideOutHorizontally { it } + fadeOut() else slideOutHorizontally { -it } + fadeOut(),
        modifier = Modifier
            .align(if (seekDirection > 0) Alignment.CenterEnd else Alignment.CenterStart)
            .fillMaxHeight()
            .width(if (deviceType == DeviceType.TV) 300.dp else 180.dp)
            .background(
                brush = Brush.horizontalGradient(
                    colors = if (seekDirection > 0)
                        listOf(Color.Transparent, Color.White.copy(alpha = 0.2f))
                    else
                        listOf(Color.White.copy(alpha = 0.2f), Color.Transparent)
                )
            )
            .zIndex(300f)
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            SeekFeedback(seekDirection, seekFeedbackMs.takeIf { it > 0L } ?: if (seekDirection > 0) seekForwardMs else seekBackwardMs)
        }
    }
}

internal fun IntroTimestamps.dismissKey(): String {
    return "$type:$startTime:$endTime"
}
