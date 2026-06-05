@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.Crossfade
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
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
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.*
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import coil3.compose.AsyncImage
import com.fluxa.app.player.MediaTrack
import java.util.Locale

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
fun EpisodeSidebar(
    meta: Meta,
    currentId: String,
    deviceType: DeviceType,
    viewModel: HomeViewModel,
    activeProfile: UserProfile?,
    onSelect: (String, String?) -> Unit,
    onClose: (() -> Unit)? = null
) {
    var episodes by remember { mutableStateOf<List<Video>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var showSeasonMenu by remember { mutableStateOf(false) }
    var hasExtras by remember(meta.id) { mutableStateOf(false) }
    val currentSeasonFromId = remember(currentId) {
        val parts = currentId.split(":")
        if (parts.size >= 3) parts[parts.size - 2].toIntOrNull() ?: 1 else 1
    }
    var selectedSeason by remember(currentId) { mutableIntStateOf(currentSeasonFromId) }
    val availableSeasons = remember(meta.seasonsCount, hasExtras, currentSeasonFromId) {
        buildList {
            val count = maxOf(meta.seasonsCount ?: 1, currentSeasonFromId + 2)
            addAll(1..count.coerceAtLeast(1))
            if (hasExtras || currentSeasonFromId == 0) add(0)
        }.distinct().sortedWith(compareBy<Int> { if (it == 0) 1 else 0 }.thenBy { it })
    }

    LaunchedEffect(meta.id) {
        hasExtras = runCatching {
            viewModel.getSeasonEpisodes(meta.id, 0, activeProfile?.safeLanguage ?: "en").isNotEmpty()
        }.getOrDefault(false)
    }

    LaunchedEffect(currentId, selectedSeason) {
        isLoading = true
        episodes = viewModel.getSeasonEpisodes(meta.id, selectedSeason, activeProfile?.safeLanguage ?: "en")
        isLoading = false
    }

    PlayerSidebarShell(
        title = AppStrings.t(activeProfile?.safeLanguage ?: "en", "auto.episodes"),
        subtitle = "",
        deviceType = deviceType,
        onClose = onClose,
        sideSheetOnMobile = false
    ) {
            Box {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(14.dp))
                        .background(Color.White.copy(alpha = 0.06f))
                        .clickable { showSeasonMenu = true }
                        .padding(horizontal = 14.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = if (selectedSeason == 0) AppStrings.t(activeProfile?.safeLanguage ?: "en", "auto.specials") else AppStrings.format(activeProfile?.safeLanguage ?: "en", "format.season_number", selectedSeason),
                        color = Color.White,
                        fontSize = 13.sp,
                        fontWeight = FontWeight.SemiBold,
                        modifier = Modifier.weight(1f)
                    )
                    Icon(FluxaIcons.KeyboardArrowDown, null, tint = Color.White, modifier = Modifier.size(18.dp))
                }
                DropdownMenu(
                    expanded = showSeasonMenu,
                    onDismissRequest = { showSeasonMenu = false },
                    modifier = Modifier.background(Color(0xFF121922))
                ) {
                    availableSeasons.forEach { season ->
                        DropdownMenuItem(
                            text = {
                                Text(
                                    if (season == 0) AppStrings.t(activeProfile?.safeLanguage ?: "en", "auto.specials") else AppStrings.format(activeProfile?.safeLanguage ?: "en", "format.season_number", season),
                                    color = Color.White,
                                    fontSize = 13.sp
                                )
                            },
                            onClick = {
                                selectedSeason = season
                                showSeasonMenu = false
                            }
                        )
                    }
                }
            }
            Spacer(Modifier.height(14.dp))
            if (isLoading) {
                Text(AppStrings.t(activeProfile?.safeLanguage ?: "en", "auto.loading"), color = Color.Gray)
            } else {
                LazyColumn(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    items(episodes, key = { it.id }) { ep ->
                        if (deviceType == DeviceType.Mobile) {
                            MobilePlayerEpisodeRow(
                                episode = ep,
                                isSelected = ep.id == currentId,
                                onClick = {
                                    val episodeLabel = buildString {
                                        append("S")
                                        append(ep.season ?: selectedSeason)
                                        append(" E")
                                        append(ep.number ?: 0)
                                        ep.name?.takeIf { it.isNotBlank() }?.let {
                                            append(" ")
                                            append(it.uppercase())
                                        }
                                    }
                                    onSelect(ep.id, episodeLabel)
                                }
                            )
                        } else {
                            TrackItem(
                                modifier = Modifier.animateItem(),
                                title = "${ep.number}. ${ep.name ?: AppStrings.format(activeProfile?.safeLanguage ?: "en", "format.episode_number", ep.number ?: 0)}",
                                isSelected = ep.id == currentId,
                                onClick = {
                                    val episodeLabel = buildString {
                                        append("S")
                                        append(ep.season ?: selectedSeason)
                                        append(" E")
                                        append(ep.number ?: 0)
                                        ep.name?.takeIf { it.isNotBlank() }?.let {
                                            append(" ")
                                            append(it.uppercase())
                                        }
                                    }
                                    onSelect(ep.id, episodeLabel)
                                },
                                subtitle = ep.overview?.takeIf { it.isNotBlank() },
                                deviceType = deviceType,
                                leadingIcon = FluxaIcons.Tv
                            )
                        }
                    }
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
    val panelAlpha by animateFloatAsState(if (shown) 1f else 0f, animationSpec = tween(180), label = "sidebarAlpha")
    val panelOffset by animateFloatAsState(if (shown) 0f else 44f, animationSpec = tween(220, easing = FastOutSlowInEasing), label = "sidebarOffset")
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
                    interactionSource = remember { androidx.compose.foundation.interaction.MutableInteractionSource() },
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
                .then(if (isMobile && !sideSheetOnMobile) Modifier.navigationBarsPadding() else Modifier.navigationBarsPadding())
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
            isFocused  -> Color.White.copy(alpha = 0.18f)
            else       -> Color.White.copy(alpha = 0.04f)
        },
        animationSpec = tween(150),
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
        animationSpec = tween(170, easing = FastOutSlowInEasing),
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
            .animateContentSize(animationSpec = tween(220, easing = FastOutSlowInEasing))
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
