@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.foundation.ExperimentalFoundationApi::class, androidx.compose.ui.ExperimentalComposeUiApi::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Intent
import android.graphics.drawable.BitmapDrawable
import android.net.Uri
import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerDefaults
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.*
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.key.*
import androidx.compose.ui.layout.*
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import androidx.compose.ui.viewinterop.AndroidView
import androidx.palette.graphics.Palette
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items as mobileGridItems
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import coil3.imageLoader
import coil3.request.ImageRequest
import coil3.request.crossfade
import coil3.size.Size
import androidx.compose.ui.res.painterResource
import com.fluxa.app.R
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.distinctUntilChanged
import androidx.compose.runtime.snapshotFlow
import java.text.SimpleDateFormat
import java.util.*
import kotlin.math.abs

private val BILLBOARD_H_GRADIENT = Brush.horizontalGradient(
    0.0f to Color.Black.copy(alpha = 0.82f),
    0.34f to Color.Black.copy(alpha = 0.46f),
    0.8f to Color.Transparent
)
private val BILLBOARD_V_GRADIENT = Brush.verticalGradient(
    0.0f to Color.Transparent,
    0.45f to Color.Transparent,
    0.65f to Color.Black.copy(alpha = 0.33f),
    0.85f to Color.Black.copy(alpha = 0.75f),
    1.0f to Color.Black
)
private val BILLBOARD_BOTTOM_GRADIENT = Brush.verticalGradient(
    0.0f to Color.Transparent,
    0.25f to Color.Black.copy(alpha = 0.18f),
    0.55f to Color.Black.copy(alpha = 0.52f),
    0.82f to Color.Black.copy(alpha = 0.82f),
    1.0f to Color.Black.copy(alpha = 0.96f)
)

@Composable
fun BillboardSection(
    movie: Meta?,
    index: Int,
    poolSize: Int,
    logoUrl: String?,
    errorText: String?,
    isWatchlisted: Boolean,
    trailerUrl: String?,
    sidebarFocusRequester: FocusRequester,
    isSidebarAttached: Boolean,
    billboardFocusRequester: FocusRequester,
    lang: String,
    isFocused: Boolean,
    onPlayClick: () -> Unit,
    onInfoClick: () -> Unit,
    onToggleWatchlist: () -> Unit,
    sharedPlayer: androidx.media3.exoplayer.ExoPlayer,
    activePreviewUrl: String?
) {
    val deviceType = LocalDeviceType.current
    val horizontalPadding = if (deviceType == DeviceType.TV) 64.dp else 20.dp
    if (movie == null) return
    val billboardUiModel = remember(movie) { movie.toBillboardUiModel() }
    Box(modifier = Modifier.fillMaxSize()) {
        AnimatedContent(
            targetState = billboardUiModel,
            transitionSpec = { fadeIn(animationSpec = tween(150)) togetherWith fadeOut(animationSpec = tween(150)) },
            label = "billboard"
        ) { targetMovie ->
            BillboardContent(
                targetMovie, logoUrl, lang, horizontalPadding, deviceType,
                isWatchlisted, trailerUrl, sidebarFocusRequester, isSidebarAttached,
                billboardFocusRequester, isFocused, onInfoClick, onPlayClick, onToggleWatchlist,
                sharedPlayer, activePreviewUrl
            )
        }

        if (deviceType == DeviceType.TV && !isFocused && poolSize > 1) {
            Row(
                modifier = Modifier.align(Alignment.BottomCenter).padding(bottom = 20.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                repeat(poolSize) { i ->
                    val active = i == index
                    val width by animateDpAsState(if (active) 20.dp else 8.dp, label = "dotWidth")
                    val alpha by animateFloatAsState(if (active) 1f else 0.3f, label = "dotAlpha")
                    Box(modifier = Modifier.width(width).height(8.dp).clip(CircleShape).background(Color.White.copy(alpha = alpha)))
                }
            }
        }
    }
}

@Composable
fun BillboardContent(
    movie: BillboardUiModel,
    logoUrl: String?,
    lang: String,
    horizontalPadding: androidx.compose.ui.unit.Dp,
    deviceType: DeviceType,
    isWatchlisted: Boolean,
    trailerUrl: String?,
    sidebarFocusRequester: FocusRequester,
    isSidebarAttached: Boolean,
    billboardFocusRequester: FocusRequester,
    isFocused: Boolean,
    onInfoClick: () -> Unit,
    onPlayClick: () -> Unit,
    onToggleWatchlist: () -> Unit,
    sharedPlayer: androidx.media3.exoplayer.ExoPlayer,
    activePreviewUrl: String?
) {
    Box(modifier = Modifier.fillMaxSize()) {
        val isPreviewActive = activePreviewUrl != null && !activePreviewUrl.contains("youtube.com") && !activePreviewUrl.contains("youtu.be")
        
        if (isPreviewActive) {
            AndroidView<androidx.media3.ui.PlayerView>(
                factory = { ctx ->
                    androidx.media3.ui.PlayerView(ctx).apply {
                        player = sharedPlayer
                        useController = false
                        resizeMode = androidx.media3.ui.AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                        subtitleView?.setFixedTextSize(android.util.TypedValue.COMPLEX_UNIT_SP, 20f)
                    }
                },
                modifier = Modifier.fillMaxSize()
            )
        } else {
            val backgroundCandidates = remember(movie.id, movie.backgroundUrl) {
                listOfNotNull(movie.backgroundUrl)
            }
            var backgroundIndex by remember(movie.id) { mutableIntStateOf(0) }
            LaunchedEffect(movie.id, backgroundCandidates) {
                backgroundIndex = 0
            }
            val backgroundUrl = backgroundCandidates.getOrNull(backgroundIndex)
            AsyncImage(
                model = backgroundUrl,
                contentDescription = null,
                onError = {
                    backgroundIndex = backgroundCandidates.size
                },
                modifier = Modifier.fillMaxSize().alpha(0.65f),
                contentScale = ContentScale.Crop,
                alignment = Alignment.CenterEnd //  Focus on the visible right side
            )
        }
        Box(modifier = Modifier.fillMaxSize().background(BILLBOARD_H_GRADIENT))
        Box(modifier = Modifier.fillMaxSize().background(BILLBOARD_V_GRADIENT))
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight(if (deviceType == DeviceType.TV) 0.62f else 0.7f)
                .align(Alignment.BottomCenter)
                .background(BILLBOARD_BOTTOM_GRADIENT)
        )
        Column(
            modifier = Modifier
                .align(if (isFocused) Alignment.BottomStart else Alignment.TopStart)
                .padding(
                    start = horizontalPadding,
                    end = horizontalPadding,
                    top = if (!isFocused && deviceType == DeviceType.TV) 24.dp else 0.dp, //  Breath from top in compact mode
                    bottom = if (deviceType == DeviceType.TV) 28.dp else 44.dp
                )
                .fillMaxWidth(if (deviceType == DeviceType.TV) 0.42f else 1f) //  Shrink width to keep synopsis on the left side
        ) {
            val logoCandidates = remember(movie.id, logoUrl, movie.logoFallbackUrl) {
                listOfNotNull(logoUrl, movie.logoFallbackUrl).distinct()
            }
            var logoIndex by remember(movie.id) { mutableIntStateOf(0) }
            val activeLogo = logoCandidates.getOrNull(logoIndex)
            var showTextFallback by remember(movie.id) { mutableStateOf(activeLogo.isNullOrEmpty()) }
            LaunchedEffect(movie.id, logoCandidates) {
                logoIndex = 0
                showTextFallback = logoCandidates.isEmpty()
            }
            
            LaunchedEffect(movie.id, activeLogo) {
                if (activeLogo.isNullOrEmpty()) {
                    delay(1800)
                    showTextFallback = true
                } else {
                    showTextFallback = false
                }
            }
            //  LOGO OR TEXT RENDERER (Simplified & Guaranteed)
            val billboardContext = LocalContext.current
            val billboardLogoRequest = remember(activeLogo) {
                activeLogo?.takeIf { it.isNotBlank() }?.let {
                    ImageRequest.Builder(billboardContext)
                        .data(it)
                        .crossfade(true)
                        .memoryCacheKey("billboard-logo:$it")
                        .diskCacheKey(it)
                        .build()
                }
            }
            key(movie.id, logoUrl, movie.logoFallbackUrl, activeLogo, logoIndex, showTextFallback) {
                if (!activeLogo.isNullOrEmpty() && !showTextFallback) {
                    val lHeight = if (!isFocused && deviceType == DeviceType.TV) 80.dp else if (deviceType == DeviceType.Mobile) 110.dp else 120.dp
                    val lWidth = if (!isFocused && deviceType == DeviceType.TV) 0.45f else if (deviceType == DeviceType.Mobile) 0.85f else 0.48f

                    Box(
                        modifier = Modifier
                            .height(lHeight)
                            .fillMaxWidth(lWidth)
                            .padding(vertical = 14.dp)
                    ) {
                        AsyncImage(
                            model = billboardLogoRequest,
                            contentDescription = null,
                            onSuccess = { showTextFallback = false },
                            onError = { showTextFallback = true },
                            modifier = Modifier.matchParentSize(),
                            contentScale = ContentScale.Fit,
                            alignment = Alignment.BottomStart
                        )
                    }
                } else {
                    Text(
                        text = movie.title,
                        style = if (!isFocused && deviceType == DeviceType.TV) MaterialTheme.typography.headlineSmall
                                else if(deviceType == DeviceType.Mobile) MaterialTheme.typography.displaySmall
                                else MaterialTheme.typography.displaySmall,
                        color = Color.White,
                        fontWeight = FontWeight.Black,
                        maxLines = if (!isFocused && deviceType == DeviceType.TV) 1 else 2,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(vertical = 4.dp)
                    )
                }
            }
            Spacer(Modifier.height(if(deviceType == DeviceType.Mobile) 12.dp else 8.dp))
            val metadata = remember(movie.id, movie.type, movie.releaseInfo, movie.genres, movie.runtime, movie.seasonsCount, movie.ageRating, lang) {
                buildList {
                    movie.releaseInfo?.let { add(it) }
                    movie.genres.take(1).let { addAll(it) }
                    if (movie.type == "movie") {
                        movie.runtime?.toBillboardRuntimeLabel(lang)?.let { add(it.replace(" ", "\u00A0")) }
                    } else {
                        val seasonLabel = AppStrings.t(lang, "auto.seasons")
                        movie.seasonsCount?.let { add("$it $seasonLabel") }
                        movie.runtime?.toBillboardRuntimeLabel(lang)?.let { add(it.replace(" ", "\u00A0")) }
                    }
                    movie.ageRating?.takeIf { it.isNotBlank() }?.let { add(it) }
                }.distinct()
            }
            if (metadata.isNotEmpty()) {
                FlowRow(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(10.dp, Alignment.CenterHorizontally),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                    maxItemsInEachRow = if (deviceType == DeviceType.Mobile) 3 else 4
                ) {
                    metadata.forEach { item ->
                        Text(
                            text = item,
                            color = Color.White.copy(alpha = 0.92f),
                            fontSize = if (deviceType == DeviceType.Mobile) 14.sp else 15.sp,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                }
            }
            if ((deviceType == DeviceType.TV && !isFocused) || deviceType == DeviceType.Mobile) {
                movie.description?.takeIf { it.isNotBlank() }?.let { description ->
                    Spacer(Modifier.height(if(deviceType == DeviceType.Mobile) 14.dp else 10.dp))
                    Text(
                        text = description,
                        style = if(deviceType == DeviceType.Mobile) MaterialTheme.typography.bodyLarge else MaterialTheme.typography.bodyLarge,
                        color = Color.White.copy(alpha = 0.85f),
                        maxLines = if(deviceType == DeviceType.Mobile) 4 else 4,
                        overflow = TextOverflow.Ellipsis,
                        lineHeight = if(deviceType == DeviceType.Mobile) 22.sp else 24.sp
                    )
                }
            }

            AnimatedVisibility(
                visible = isFocused && deviceType == DeviceType.TV,
                enter = fadeIn() + expandVertically(),
                exit = fadeOut() + shrinkVertically()
            ) {
                Column {
                    movie.description?.takeIf { it.isNotBlank() }?.let { description ->
                        Spacer(Modifier.height(14.dp))
                        Text(
                            text = description,
                            style = MaterialTheme.typography.bodyLarge,
                            color = Color.White.copy(alpha = 0.8f),
                            maxLines = 3,
                            overflow = TextOverflow.Ellipsis,
                            lineHeight = 24.sp
                        )
                    }
                    
                    Spacer(modifier = Modifier.height(24.dp))
                    
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                        val playText = AppStrings.t(lang, "auto.play")
                        val infoText = AppStrings.t(lang, "auto.details")
                        
                        Surface(
                            onClick = onPlayClick,
                            modifier = Modifier.height(48.dp).widthIn(min = 140.dp).focusRequester(billboardFocusRequester),
                            shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(8.dp)),
                            colors = ClickableSurfaceDefaults.colors(containerColor = Color.White, contentColor = Color.Black)
                        ) {
                            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Icon(FluxaIcons.PlayArrow, null, modifier = Modifier.size(24.dp))
                                    Spacer(Modifier.width(8.dp))
                                    Text(playText, fontWeight = FontWeight.Black, fontSize = 15.sp)
                                }
                            }
                        }

                        Surface(
                            onClick = onInfoClick,
                            modifier = Modifier.height(48.dp).widthIn(min = 140.dp),
                            shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(8.dp)),
                            colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(alpha = 0.2f), contentColor = Color.White)
                        ) {
                            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                Text(infoText, fontWeight = FontWeight.Black, fontSize = 15.sp)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun PremiumSidebar(
    currentFilter: String,
    onFilterChange: (String) -> Unit,
    onSearchClick: () -> Unit,
    onWatchlistClick: () -> Unit,
    onProfileClick: () -> Unit,
    onExploreClick: () -> Unit,
    lang: String,
    contentFocusRequester: FocusRequester,
    modifier: Modifier = Modifier
) {
    var isExpanded by remember { mutableStateOf(false) }
    val deviceType = LocalDeviceType.current
    val minWidth = if (deviceType == DeviceType.TV) 56.dp else 12.dp
    val width by animateDpAsState(targetValue = if (isExpanded) 240.dp else minWidth, label = "width")
    val contentAlpha by animateFloatAsState(targetValue = if (isExpanded) 1f else if (deviceType == DeviceType.TV) 0.6f else 0f, label = "alpha")

    Box(
        modifier = modifier
            .fillMaxHeight()
            .width(width)
            .onFocusChanged { isExpanded = it.hasFocus }
            .zIndex(200f)
            .background(Brush.horizontalGradient(colors = listOf(Color.Black.copy(alpha = 0.98f * contentAlpha), Color.Transparent)))
    ) {
        Column(
            modifier = Modifier
                .fillMaxHeight() // Fill height but keep child Column centered
                .padding(vertical = 32.dp, horizontal = 4.dp)
                .alpha(contentAlpha),
            verticalArrangement = Arrangement.Center, //  VERTICAL CENTERED
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Column(
                verticalArrangement = Arrangement.spacedBy(28.dp), 
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier.wrapContentHeight()
            ) {
                Box(modifier = Modifier.size(8.dp).clip(CircleShape).background(Color.White))
                Spacer(modifier = Modifier.height(20.dp))
                SidebarItem(FluxaIcons.Search, AppStrings.t(lang, "auto.search"), false, isExpanded, contentFocusRequester, onSearchClick)
                SidebarItem(FluxaIcons.Explore, AppStrings.t(lang, "auto.explore"), false, isExpanded, contentFocusRequester, onExploreClick)
                SidebarItem(FluxaIcons.Home, AppStrings.t(lang, "nav.home"), currentFilter == "all", isExpanded, contentFocusRequester) { onFilterChange("all") }
                SidebarItem(FluxaIcons.Movie, AppStrings.t(lang, "auto.movies"), currentFilter == "movie", isExpanded, contentFocusRequester) { onFilterChange("movie") }
                SidebarItem(FluxaIcons.Tv, AppStrings.t(lang, "auto.tv_shows"), currentFilter == "series", isExpanded, contentFocusRequester) { onFilterChange("series") }
                SidebarItem(FluxaIcons.Bookmark, AppStrings.t(lang, "auto.my_list"), false, isExpanded, contentFocusRequester, onWatchlistClick)
                SidebarItem(FluxaIcons.AccountCircle, AppStrings.t(lang, "auto.profile"), false, isExpanded, contentFocusRequester, onProfileClick)
            }
        }
    }
}

@Composable
fun SidebarItem(
    icon: ImageVector,
    label: String,
    isSelected: Boolean,
    isExpanded: Boolean,
    contentFocusRequester: FocusRequester,
    onClick: () -> Unit
) {
    var isFocused by remember { mutableStateOf(false) }
    val contentColor by animateColorAsState(targetValue = if (isSelected) Color.White else if (isFocused) Color.White.copy(alpha = 0.9f) else Color.White.copy(alpha = 0.35f), label = "color")

    Surface(
        onClick = onClick,
        modifier = Modifier
            .fillMaxWidth()
            .height(48.dp)
            .onFocusChanged { isFocused = it.isFocused }
            .focusProperties { 
                right = contentFocusRequester //  EXPLICIT ROUTE TO BILLBOARD
            },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(8.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = if (isFocused) Color.White.copy(alpha = 0.05f) else Color.Transparent
        ),
        scale = ClickableSurfaceDefaults.scale(focusedScale = 1.1f)
    ) {
        Row(
            modifier = Modifier.fillMaxSize(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Center
        ) {
            Box(contentAlignment = Alignment.Center, modifier = Modifier.size(48.dp)) {
                Icon(icon, null, tint = contentColor, modifier = Modifier.size(22.dp))
                if (isSelected) {
                    Box(modifier = Modifier.align(Alignment.BottomCenter).width(12.dp).height(2.dp).background(Color.White, RoundedCornerShape(2.dp)))
                }
            }
            if (isExpanded) {
                Spacer(Modifier.width(12.dp))
                Text(
                    text = label,
                    color = contentColor,
                    fontSize = 16.sp,
                    fontWeight = if (isSelected) FontWeight.Black else FontWeight.Bold,
                    modifier = Modifier.weight(1f)
                )
            }
        }
    }
}

// Delegates to the shared util (LocalDate + lexicographic compare) instead of per-card
// SimpleDateFormat parsing on the composition thread.
internal fun isUpcoming(dateStr: String?): Boolean =
    com.fluxa.app.common.ReleaseDateUtils.isUpcoming(dateStr)
