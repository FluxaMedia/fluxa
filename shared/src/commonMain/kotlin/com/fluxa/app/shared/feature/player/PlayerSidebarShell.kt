package com.fluxa.app.shared.feature.player

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.ui.catalog.LocalDeviceType
import com.fluxa.app.ui.catalog.streamRawBody
import com.fluxa.app.ui.catalog.streamSourceHeader

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
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
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import kotlinx.coroutines.delay

@Composable
fun SourceSidebar(streams: List<Stream>, currentUrl: String, deviceType: DeviceType, lang: String = "en", onSelect: (String) -> Unit, onClose: (() -> Unit)? = null) {
    PlayerSidebarShell(
        title = AppStrings.t(lang, "player.source_selection_title"),
        deviceType = deviceType,
        onClose = onClose,
        cardWidth = 420.dp
    ) {
        LazyColumn(verticalArrangement = Arrangement.spacedBy(2.dp)) {
            items(streams, key = { it.playableUrl ?: (it.title.orEmpty() + it.name.orEmpty()) }) { stream ->
                val playableUrl = stream.playableUrl
                TrackItem(
                    modifier = Modifier.animateItem(),
                    title = stream.streamSourceHeader(),
                    isSelected = stream.playableUrl == currentUrl,
                    onClick = { playableUrl?.let(onSelect) },
                    subtitle = stream.streamRawBody(),
                    deviceType = deviceType,
                    leadingIcon = FluxaIcons.PlayArrow
                )
            }
        }
    }
}

@Composable
fun PlayerSidebarShell(
    title: String,
    deviceType: DeviceType,
    onClose: (() -> Unit)? = null,
    onBack: (() -> Unit)? = null,
    cardWidth: Dp? = null,
    anchorTop: Boolean = false,
    scrimAlpha: Float = 0.42f,
    dimmed: Boolean = false,
    content: @Composable ColumnScope.() -> Unit
) {
    var shown by remember { mutableStateOf(false) }
    var closing by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { shown = true }
    val visible = shown && !closing
    val progress by animateFloatAsState(
        targetValue = if (visible) 1f else 0f,
        animationSpec = if (visible) tween(300, easing = FastOutSlowInEasing) else tween(200, easing = FastOutSlowInEasing),
        label = "sidebarProgress"
    )
    fun requestClose() {
        if (onClose != null && !closing) closing = true
    }
    LaunchedEffect(closing) {
        if (closing) {
            delay(210L)
            onClose?.invoke()
        }
    }
    val isMobile = deviceType == DeviceType.Mobile

    val dim by animateFloatAsState(
        targetValue = if (dimmed) 0.15f else 1f,
        animationSpec = tween(150),
        label = "sidebarDim"
    )

    BoxWithConstraints(
        modifier = Modifier
            .fillMaxSize()
            .zIndex(100f)
            .pointerInput(Unit) {
                detectDragGestures { change, _ -> change.consume() }
            }
    ) {
        val isLandscape = maxWidth > maxHeight
        val isCard = !isMobile || isLandscape || anchorTop

        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Color.Black.copy(alpha = scrimAlpha * progress * dim))
                .clickable(
                    interactionSource = remember { MutableInteractionSource() },
                    indication = null
                ) { requestClose() }
        )

        val panelShape = if (isCard) {
            RoundedCornerShape(24.dp)
        } else {
            RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp, bottomStart = 0.dp, bottomEnd = 0.dp)
        }
        val panelSizeModifier = if (isCard) {
            Modifier
                .padding(top = if (anchorTop) 16.dp else 0.dp, end = if (anchorTop) 16.dp else 0.dp)
                .width(minOf(cardWidth ?: if (isMobile) 340.dp else 360.dp, maxWidth - 48.dp))
                .wrapContentHeight()
                .heightIn(max = maxHeight - 48.dp)
        } else {
            Modifier
                .fillMaxWidth(0.92f)
                .widthIn(max = 430.dp)
                .wrapContentHeight()
                .heightIn(min = 180.dp, max = 560.dp)
        }

        val density = LocalDensity.current
        var dragOffsetPx by remember { mutableStateOf(0f) }
        val dismissThresholdPx = with(density) { 110.dp.toPx() }

        Column(
            modifier = Modifier
                .align(
                    when {
                        anchorTop -> Alignment.TopEnd
                        isCard -> Alignment.Center
                        else -> Alignment.BottomCenter
                    }
                )
                .then(panelSizeModifier)
                .animateContentSize(tween(220, easing = FastOutSlowInEasing))
                .graphicsLayer {
                    alpha = (if (isCard) progress else 1f) * dim
                    translationY = if (isCard) {
                        (1f - progress) * (if (anchorTop) (-16).dp.toPx() else 16.dp.toPx())
                    } else {
                        (1f - progress) * size.height + dragOffsetPx
                    }
                }
                .background(Color(0xE6101418), shape = panelShape)
                .clip(panelShape)
                .then(if (!isCard) Modifier.windowInsetsPadding(WindowInsets.navigationBars) else Modifier)
                .padding(16.dp)
        ) {
            if (!isCard) {
                Box(
                    modifier = Modifier
                        .align(Alignment.CenterHorizontally)
                        .padding(bottom = 14.dp)
                        .width(52.dp)
                        .height(5.dp)
                        .clip(CircleShape)
                        .background(Color.White.copy(alpha = 0.16f))
                        .pointerInput(onClose) {
                            detectVerticalDragGestures(
                                onDragEnd = {
                                    if (dragOffsetPx > dismissThresholdPx) {
                                        requestClose()
                                    } else {
                                        dragOffsetPx = 0f
                                    }
                                },
                                onVerticalDrag = { change, dragAmount ->
                                    change.consume()
                                    dragOffsetPx = (dragOffsetPx + dragAmount).coerceAtLeast(0f)
                                }
                            )
                        }
                )
            }
            if (title.isNotBlank() || onBack != null) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(start = 8.dp, bottom = 12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    if (onBack != null) {
                        Box(
                            modifier = Modifier
                                .size(if (deviceType == DeviceType.TV) 34.dp else 30.dp)
                                .clip(CircleShape)
                                .background(Color.White.copy(alpha = 0.06f))
                                .clickable { onBack() },
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(FluxaIcons.ChevronLeft, null, tint = Color.White, modifier = Modifier.size(if (deviceType == DeviceType.TV) 20.dp else 16.dp))
                        }
                        Spacer(modifier = Modifier.width(10.dp))
                    }
                    Text(
                        text = title,
                        color = Color.White,
                        fontSize = if (deviceType == DeviceType.TV) 16.sp else 15.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                }
            }
            content()
        }
    }
}

@Composable
fun TrackItem(
    modifier: Modifier = Modifier,
    title: String,
    isSelected: Boolean,
    onClick: () -> Unit,
    subtitle: String? = null,
    badge: String? = null,
    formatBadge: (@Composable () -> Unit)? = null,
    deviceType: DeviceType? = null,
    leadingIcon: ImageVector? = null,
    trailingIcon: ImageVector? = null
) {
    val resolvedDeviceType = deviceType ?: LocalDeviceType.current
    val contentAlpha = if (isSelected) 1f else 0.7f
    val secondaryLine = listOfNotNull(subtitle?.takeIf { it.isNotBlank() }, badge?.takeIf { it.isNotBlank() })
        .joinToString(" · ")

    Row(
        modifier = modifier
            .fillMaxWidth()
            .heightIn(min = if (resolvedDeviceType == DeviceType.TV) 52.dp else 46.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(if (isSelected) Color.White.copy(alpha = 0.08f) else Color.Transparent)
            .clickable { onClick() }
            .padding(horizontal = 12.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        if (leadingIcon != null) {
            Icon(
                leadingIcon,
                null,
                tint = Color.White.copy(alpha = contentAlpha),
                modifier = Modifier.size(20.dp)
            )
        }

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                color = Color.White.copy(alpha = contentAlpha),
                fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal,
                fontSize = if (resolvedDeviceType == DeviceType.TV) 15.sp else 14.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            if (secondaryLine.isNotBlank() || formatBadge != null) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    if (secondaryLine.isNotBlank()) {
                        Text(
                            text = secondaryLine,
                            color = Color.White.copy(alpha = 0.5f),
                            fontSize = 11.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                    formatBadge?.invoke()
                }
            }
        }

        when {
            isSelected -> Icon(FluxaIcons.Check, null, tint = Color.White, modifier = Modifier.size(18.dp))
            trailingIcon != null -> Icon(trailingIcon, null, tint = Color.White.copy(alpha = 0.5f), modifier = Modifier.size(18.dp))
        }
    }
}

fun Meta.withCurrentEpisodeArtwork(artwork: String?): Meta {
    val episodeArtwork = artwork?.takeIf { it.isNotBlank() } ?: return this
    if (type != "series") return this
    return copy(continueWatchingPoster = episodeArtwork, continueWatchingBackground = episodeArtwork)
}
