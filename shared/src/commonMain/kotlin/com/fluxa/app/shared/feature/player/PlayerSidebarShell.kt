package com.fluxa.app.shared.feature.player

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaDimensions
import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.ui.catalog.LocalDeviceType
import com.fluxa.app.ui.catalog.streamRawBody
import com.fluxa.app.ui.catalog.streamSourceHeader

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
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
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex

@Composable
fun SourceSidebar(streams: List<Stream>, currentUrl: String, deviceType: DeviceType, lang: String = "en", onSelect: (String) -> Unit, onClose: (() -> Unit)? = null) {
    PlayerSidebarShell(
        title = AppStrings.t(lang, "player.source_selection_title"),
        subtitle = AppStrings.t(lang, "player.source_selection_subtitle"),
        deviceType = deviceType,
        onClose = onClose,
        sideSheetOnMobile = false
    ) {
        LazyColumn(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            items(streams, key = { it.playableUrl ?: (it.title.orEmpty() + it.name.orEmpty()) }) { stream ->
                val playableUrl = stream.playableUrl
                TrackItem(
                    modifier = Modifier.animateItem(),
                    title = stream.streamSourceHeader(),
                    isSelected = stream.playableUrl == currentUrl,
                    onClick = { playableUrl?.let(onSelect) },
                    subtitle = stream.streamRawBody(),
                    badge = null,
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
    subtitle: String,
    deviceType: DeviceType,
    onClose: (() -> Unit)? = null,
    sideSheetOnMobile: Boolean = false,
    compactCenterOnMobile: Boolean = false,
    content: @Composable ColumnScope.() -> Unit
) {
    var shown by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { shown = true }
    val panelAlpha by animateFloatAsState(if (shown) 1f else 0f, animationSpec = tween(FluxaDimensions.AnimDuration.scaleAlpha), label = "sidebarAlpha")
    val panelOffset by animateFloatAsState(if (shown) 0f else 44f, animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand, easing = FastOutSlowInEasing), label = "sidebarOffset")
    val isMobile = deviceType == DeviceType.Mobile
    val panelShape = if (isMobile) {
        if (compactCenterOnMobile) {
            RoundedCornerShape(24.dp)
        } else if (sideSheetOnMobile) {
            RoundedCornerShape(topStart = 24.dp, bottomStart = 24.dp, topEnd = 0.dp, bottomEnd = 0.dp)
        } else {
            RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp, bottomStart = 0.dp, bottomEnd = 0.dp)
        }
    } else {
        RoundedCornerShape(topStart = 24.dp, bottomStart = 24.dp, topEnd = 0.dp, bottomEnd = 0.dp)
    }
    val panelSizeModifier = when {
        isMobile && compactCenterOnMobile -> Modifier
            .fillMaxWidth(0.74f)
            .widthIn(max = 340.dp)
            .wrapContentHeight()
            .heightIn(min = 160.dp, max = 390.dp)
        isMobile && sideSheetOnMobile -> Modifier
            .fillMaxHeight()
            .fillMaxWidth(0.46f)
            .widthIn(min = 300.dp, max = 520.dp)
        isMobile -> Modifier
            .fillMaxWidth(0.92f)
            .widthIn(max = 430.dp)
            .wrapContentHeight()
            .heightIn(min = 180.dp, max = 520.dp)
        else -> Modifier
            .widthIn(min = 300.dp, max = 420.dp)
            .wrapContentHeight()
            .heightIn(max = 620.dp)
    }

    Box(modifier = Modifier.fillMaxSize().zIndex(100f)) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Color.Black.copy(alpha = 0.48f))
                .clickable(
                    interactionSource = remember { MutableInteractionSource() },
                    indication = null
                ) { onClose?.invoke() }
        )

        Column(
            modifier = Modifier
                .align(
                    if (isMobile) {
                        if (compactCenterOnMobile) Alignment.Center else if (sideSheetOnMobile) Alignment.CenterEnd else Alignment.BottomCenter
                    } else {
                        Alignment.CenterEnd
                    }
                )
                .then(panelSizeModifier)
                .graphicsLayer {
                    alpha = panelAlpha
                    translationY = if (isMobile && !sideSheetOnMobile) panelOffset else 0f
                    translationX = if (!isMobile || sideSheetOnMobile) panelOffset else 0f
                }
                .background(
                    brush = if (isMobile && !sideSheetOnMobile) {
                        Brush.verticalGradient(
                            listOf(Color(0xFF151A22).copy(alpha = 0.99f), Color(0xFF0D1218).copy(alpha = 0.99f))
                        )
                    } else {
                        Brush.verticalGradient(
                            listOf(Color(0xFF121922).copy(alpha = 0.98f), Color(0xFF0A0F15).copy(alpha = 0.98f))
                        )
                    },
                    shape = panelShape
                )
                .border(
                    BorderStroke(1.dp, Brush.horizontalGradient(listOf(Color.White.copy(alpha = 0.14f), Color.Transparent))),
                    shape = panelShape
                )
                .clip(panelShape)
                .windowInsetsPadding(WindowInsets.navigationBars)
                .padding(if (deviceType == DeviceType.TV) 16.dp else if (isMobile && !sideSheetOnMobile) 18.dp else 16.dp)
        ) {
            if (isMobile && sideSheetOnMobile) {
                Box(
                    modifier = Modifier
                        .align(Alignment.CenterHorizontally)
                        .padding(bottom = 14.dp)
                        .width(52.dp)
                        .height(5.dp)
                        .clip(CircleShape)
                        .background(Color.White.copy(alpha = 0.16f))
                )
            }
            Row(
                modifier = Modifier.fillMaxWidth().padding(bottom = 14.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.Top
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = title,
                        color = Color.White,
                        fontSize = if (deviceType == DeviceType.TV) 18.sp else 17.sp,
                        fontWeight = FontWeight.Black,
                        letterSpacing = 0.8.sp
                    )
                    if (subtitle.isNotBlank()) {
                        Spacer(Modifier.height(8.dp))
                        Text(
                            text = subtitle,
                            color = Color.White.copy(alpha = 0.58f),
                            fontSize = if (deviceType == DeviceType.TV) 11.sp else 10.sp,
                            lineHeight = 14.sp
                        )
                    }
                }

                if (onClose != null) {
                    Box(
                        modifier = Modifier
                            .size(if (deviceType == DeviceType.TV) 38.dp else 34.dp)
                            .clip(CircleShape)
                            .background(Color.White.copy(alpha = 0.08f))
                            .clickable { onClose() },
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(FluxaIcons.Close, null, tint = Color.White, modifier = Modifier.size(if (deviceType == DeviceType.TV) 20.dp else 16.dp))
                    }
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
    leadingIcon: ImageVector? = null
) {
    val resolvedDeviceType = deviceType ?: LocalDeviceType.current
    var isFocused by remember { mutableStateOf(false) }
    val bgColor by animateColorAsState(
        targetValue = when {
            isSelected -> Color.White
            isFocused -> Color.White.copy(alpha = 0.18f)
            else -> Color.White.copy(alpha = 0.04f)
        },
        animationSpec = tween(FluxaDimensions.AnimDuration.heroSnap),
        label = "bg"
    )
    val textColor = if (isSelected) Color.Black else Color.White
    val secondaryTextColor = if (isSelected) Color.Black.copy(alpha = 0.58f) else Color.White.copy(alpha = 0.56f)
    val iconColor = if (isSelected) Color.Black else Color.White
    val iconBackgroundColor = if (isSelected) Color.Black.copy(alpha = 0.08f) else Color.White.copy(alpha = 0.07f)
    val badgeBackgroundColor = if (isSelected) Color.Black.copy(alpha = 0.08f) else Color.White.copy(alpha = 0.1f)
    val scale by animateFloatAsState(
        targetValue = when {
            isSelected -> 1.015f
            isFocused -> 1.01f
            else -> 1f
        },
        animationSpec = tween(FluxaDimensions.AnimDuration.scaleAlpha, easing = FastOutSlowInEasing),
        label = "trackScale"
    )
    val minHeight = if (subtitle.isNullOrBlank()) {
        if (resolvedDeviceType == DeviceType.TV) 70.dp else 68.dp
    } else {
        if (resolvedDeviceType == DeviceType.TV) 88.dp else 84.dp
    }

    Box(
        modifier = modifier
            .fillMaxWidth()
            .defaultMinSize(minHeight = minHeight)
            .graphicsLayer {
                scaleX = scale
                scaleY = scale
            }
            .animateContentSize(animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand, easing = FastOutSlowInEasing))
            .clip(RoundedCornerShape(if (resolvedDeviceType == DeviceType.TV) 18.dp else 16.dp))
            .background(bgColor)
            .clickable { onClick() }
            .onFocusChanged { isFocused = it.isFocused }
            .focusable(),
        contentAlignment = Alignment.CenterStart
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 18.dp, vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            if (leadingIcon != null) {
                Box(
                    modifier = Modifier
                        .size(if (resolvedDeviceType == DeviceType.TV) 36.dp else 34.dp)
                        .clip(RoundedCornerShape(12.dp))
                        .background(iconBackgroundColor),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(leadingIcon, null, tint = iconColor, modifier = Modifier.size(20.dp))
                }
                Spacer(modifier = Modifier.width(14.dp))
            }

            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        text = title,
                        modifier = Modifier.weight(1f, fill = false),
                        color = textColor,
                        fontWeight = if (isSelected) FontWeight.ExtraBold else FontWeight.Bold,
                        fontSize = if (resolvedDeviceType == DeviceType.TV) 15.sp else 14.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    if (!badge.isNullOrBlank()) {
                        Box(
                            modifier = Modifier
                                .clip(CircleShape)
                                .background(badgeBackgroundColor)
                                .padding(horizontal = 10.dp, vertical = 4.dp)
                        ) {
                            Text(
                                text = badge,
                                color = textColor,
                                fontSize = 10.sp,
                                fontWeight = FontWeight.Black,
                                maxLines = 1
                            )
                        }
                    }
                }
                if (!subtitle.isNullOrBlank() || formatBadge != null) {
                    Spacer(modifier = Modifier.height(5.dp))
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        if (!subtitle.isNullOrBlank()) {
                            Text(
                                text = subtitle,
                                color = secondaryTextColor,
                                fontWeight = FontWeight.Medium,
                                fontSize = 11.sp,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                lineHeight = 15.sp
                            )
                        }
                        if (formatBadge != null) {
                            formatBadge()
                        }
                    }
                }
            }

            if (isSelected) {
                Spacer(modifier = Modifier.width(12.dp))
                Icon(
                    imageVector = FluxaIcons.CheckCircle,
                    contentDescription = null,
                    tint = iconColor,
                    modifier = Modifier.size(20.dp)
                )
            }
        }
    }
}

fun Meta.withCurrentEpisodeArtwork(artwork: String?): Meta {
    val episodeArtwork = artwork?.takeIf { it.isNotBlank() } ?: return this
    if (type != "series") return this
    return copy(continueWatchingPoster = episodeArtwork, continueWatchingBackground = episodeArtwork)
}
