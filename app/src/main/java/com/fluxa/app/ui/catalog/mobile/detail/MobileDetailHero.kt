@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Intent
import android.net.Uri
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.datasource.DefaultDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.MergingMediaSource
import androidx.media3.exoplayer.source.ProgressiveMediaSource
import androidx.media3.ui.AspectRatioFrameLayout
import androidx.media3.ui.PlayerView
import coil3.compose.AsyncImage
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

private sealed interface HeroTrailerState {
    data object Idle : HeroTrailerState
    data object Loading : HeroTrailerState
    data class Ready(val result: TrailerResult) : HeroTrailerState
    data class GeoBlocked(val youtubeUrl: String) : HeroTrailerState
    data object Error : HeroTrailerState
}

@Composable
internal fun MobileDetailHero(
    heroImage: String?,
    onBack: () -> Unit,
    lang: String,
    activeTrailer: DetailTrailer? = null,
    featuredTrailer: DetailTrailer? = null,
    onStopTrailer: () -> Unit = {},
    onSelectTrailer: ((DetailTrailer) -> Unit)? = null,
    accentColor: Color = Color.White
) {
    val context = LocalContext.current
    val configuration = LocalConfiguration.current
    val scope = rememberCoroutineScope()

    val effectiveTrailer = activeTrailer ?: featuredTrailer

    var trailerState by remember { mutableStateOf<HeroTrailerState>(HeroTrailerState.Idle) }
    var isPlaying by remember { mutableStateOf(true) }
    var showControls by remember { mutableStateOf(false) }
    var showSubtitlePicker by remember { mutableStateOf(false) }
    var selectedSubtitleTag by remember { mutableStateOf<String?>(null) }
    var isFullscreen by remember { mutableStateOf(false) }
    var exoPlayer by remember { mutableStateOf<ExoPlayer?>(null) }
    var retryCount by remember(activeTrailer?.id, activeTrailer?.url) { mutableStateOf(0) }
    DisposableEffect(Unit) {
        onDispose {
            exoPlayer?.stop()
            exoPlayer?.release()
        }
    }

    LaunchedEffect(showControls, isPlaying) {
        if (showControls && isPlaying) { delay(2500); showControls = false }
    }

    LaunchedEffect(selectedSubtitleTag, exoPlayer) {
        val player = exoPlayer ?: return@LaunchedEffect
        val params = player.trackSelectionParameters.buildUpon()
        if (selectedSubtitleTag == null) {
            params.setTrackTypeDisabled(C.TRACK_TYPE_TEXT, true)
        } else {
            params.setTrackTypeDisabled(C.TRACK_TYPE_TEXT, false)
                .setPreferredTextLanguage(selectedSubtitleTag)
        }
        player.trackSelectionParameters = params.build()
    }

    LaunchedEffect(activeTrailer?.id, activeTrailer?.url, retryCount) {
        val oldPlayer = exoPlayer
        exoPlayer = null
        oldPlayer?.stop()
        oldPlayer?.clearMediaItems()
        oldPlayer?.release()
        trailerState = HeroTrailerState.Idle
        showControls = false; showSubtitlePicker = false; isFullscreen = false

        val url = activeTrailer?.url?.takeIf { it.isNotBlank() } ?: return@LaunchedEffect
        trailerState = HeroTrailerState.Loading
        when (val r = TrailerResolver.resolve(url)) {
            is TrailerResolveResult.Ok -> {
                val subs = r.data.subtitles.map { sub ->
                    MediaItem.SubtitleConfiguration.Builder(Uri.parse(sub.url))
                        .setMimeType(sub.mimeType).setLanguage(sub.languageTag).build()
                }
                val player = ExoPlayer.Builder(context).build().also { exoPlayer = it }
                player.trackSelectionParameters = player.trackSelectionParameters
                    .buildUpon()
                    .setMaxVideoSize(1920, 1080)
                    .setForceHighestSupportedBitrate(true)
                    .build()
                val videoItem = MediaItem.Builder().setUri(r.data.streamUrl)
                    .setSubtitleConfigurations(subs).build()
                val audioUrl = r.data.audioUrl
                if (audioUrl == null) {
                    player.setMediaItem(videoItem)
                } else {
                    val mediaSourceFactory = ProgressiveMediaSource.Factory(DefaultDataSource.Factory(context))
                    val videoSource = mediaSourceFactory.createMediaSource(videoItem)
                    val audioSource = mediaSourceFactory.createMediaSource(MediaItem.fromUri(audioUrl))
                    player.setMediaSource(MergingMediaSource(videoSource, audioSource))
                }
                player.prepare()
                player.playWhenReady = true
                isPlaying = true
                trailerState = HeroTrailerState.Ready(r.data)
            }
            is TrailerResolveResult.GeoBlocked -> trailerState = HeroTrailerState.GeoBlocked(url)
            is TrailerResolveResult.Failed -> trailerState = HeroTrailerState.Error
        }
    }

    val displayImage = effectiveTrailer?.thumbnail?.takeIf { it.isNotBlank() } ?: heroImage
    val accentLuma = accentColor.red * 0.299f + accentColor.green * 0.587f + accentColor.blue * 0.114f
    val onAccent = if (accentLuma > 0.68f) Color.Black else Color.White
    val readyState = trailerState as? HeroTrailerState.Ready
    val availableSubtitles = readyState?.result?.subtitles ?: emptyList()

    if (isFullscreen && trailerState is HeroTrailerState.Ready && exoPlayer != null) {
        val player = exoPlayer
        Dialog(
            onDismissRequest = { isFullscreen = false },
            properties = DialogProperties(usePlatformDefaultWidth = false, dismissOnBackPress = true)
        ) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Color.Black)
            ) {
                AndroidView(
                    factory = { ctx ->
                        PlayerView(ctx).apply {
                            useController = false
                            resizeMode = AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                        }
                    },
                    update = {
                        it.player = player
                        it.resizeMode = AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                    },
                    modifier = Modifier.fillMaxSize()
                )

                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) {
                            showSubtitlePicker = false
                            if (isPlaying) { player?.pause(); isPlaying = false; showControls = true }
                            else showControls = !showControls
                        }
                ) {
                    AnimatedVisibility(
                        visible = showControls || !isPlaying,
                        enter = fadeIn(), exit = fadeOut(),
                        modifier = Modifier.align(Alignment.Center)
                    ) {
                        Box(
                            modifier = Modifier.size(64.dp).clip(CircleShape)
                                .background(Color.Black.copy(alpha = 0.55f))
                                .clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) {
                                    if (isPlaying) { player?.pause(); isPlaying = false }
                                    else { player?.play(); isPlaying = true; scope.launch { delay(2500); showControls = false } }
                                },
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                if (isPlaying) FluxaIcons.Pause else FluxaIcons.PlayArrow,
                                null, tint = Color.White, modifier = Modifier.size(32.dp)
                            )
                        }
                    }
                }

                AnimatedVisibility(
                    visible = showControls || !isPlaying,
                    enter = fadeIn(), exit = fadeOut(),
                    modifier = Modifier.align(Alignment.TopStart).fillMaxWidth()
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth()
                            .background(Brush.verticalGradient(listOf(Color.Black.copy(0.6f), Color.Transparent)))
                            .statusBarsPadding().padding(14.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        MobileDetailTopIcon(
                            FluxaIcons.FullscreenExit,
                            AppStrings.t(lang, "common.close")
                        ) { isFullscreen = false }
                        if (availableSubtitles.isNotEmpty()) {
                            Box(
                                modifier = Modifier
                                    .clip(RoundedCornerShape(6.dp))
                                    .background(if (selectedSubtitleTag != null) accentColor.copy(0.88f) else Color.Black.copy(0.55f))
                                    .clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) { showSubtitlePicker = !showSubtitlePicker }
                                    .padding(horizontal = 8.dp, vertical = 5.dp)
                            ) {
                                Icon(
                                    if (selectedSubtitleTag != null) FluxaIcons.Subtitles else FluxaIcons.SubtitlesOff,
                                    null, tint = if (selectedSubtitleTag != null) onAccent else Color.White,
                                    modifier = Modifier.size(18.dp)
                                )
                            }
                        }
                    }
                }

                AnimatedVisibility(
                    visible = showSubtitlePicker,
                    enter = fadeIn(), exit = fadeOut(),
                    modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth()
                ) {
                    Box(modifier = Modifier.fillMaxWidth().background(Color.Black.copy(0.82f)).padding(12.dp)) {
                        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            item {
                                SubtitleChip("Off", selectedSubtitleTag == null, accentColor, onAccent) {
                                    selectedSubtitleTag = null; showSubtitlePicker = false
                                }
                            }
                            items(availableSubtitles, key = { it.languageTag }) { sub ->
                                SubtitleChip(sub.label, selectedSubtitleTag == sub.languageTag, accentColor, onAccent) {
                                    selectedSubtitleTag = sub.languageTag; showSubtitlePicker = false
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    val heroHeight = (configuration.screenWidthDp.dp * 0.62f).coerceIn(236.dp, 340.dp)
    Box(modifier = Modifier.fillMaxWidth().height(heroHeight)) {
        when (val state = trailerState) {
            is HeroTrailerState.Ready -> {
                val player = exoPlayer
                if (player != null) {
                    AndroidView(
                        factory = { ctx ->
                            PlayerView(ctx).apply {
                                useController = false
                                resizeMode = AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                            }
                        },
                        update = {
                            it.player = if (isFullscreen) null else player
                            it.resizeMode = AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                        },
                        modifier = Modifier.fillMaxSize()
                    )
                } else {
                    AsyncImage(model = displayImage, contentDescription = null, modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
                    Box(modifier = Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.45f)))
                    CircularProgressIndicator(color = Color.White, strokeWidth = 2.dp, modifier = Modifier.align(Alignment.Center).size(26.dp))
                }

                Box(
                    modifier = Modifier.fillMaxSize()
                        .clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) {
                            showSubtitlePicker = false
                            if (isPlaying) { player?.pause(); isPlaying = false; showControls = true }
                            else showControls = !showControls
                        }
                ) {
                    AnimatedVisibility(
                        visible = showControls || !isPlaying,
                        enter = fadeIn(), exit = fadeOut(),
                        modifier = Modifier.align(Alignment.Center)
                    ) {
                        Box(
                            modifier = Modifier.size(56.dp).clip(CircleShape)
                                .background(Color.Black.copy(alpha = 0.55f))
                                .clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) {
                                    if (isPlaying) { player?.pause(); isPlaying = false }
                                    else { player?.play(); isPlaying = true; scope.launch { delay(2500); showControls = false } }
                                },
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                if (isPlaying) FluxaIcons.Pause else FluxaIcons.PlayArrow,
                                null, tint = Color.White, modifier = Modifier.size(28.dp)
                            )
                        }
                    }

                    AnimatedVisibility(
                        visible = showControls || !isPlaying,
                        enter = fadeIn(), exit = fadeOut(),
                        modifier = Modifier.align(Alignment.BottomEnd).padding(10.dp)
                    ) {
                        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                            if (availableSubtitles.isNotEmpty()) {
                                Box(
                                    modifier = Modifier.clip(RoundedCornerShape(6.dp))
                                        .background(if (selectedSubtitleTag != null) accentColor.copy(0.88f) else Color.Black.copy(0.55f))
                                        .clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) { showSubtitlePicker = !showSubtitlePicker }
                                        .padding(horizontal = 8.dp, vertical = 5.dp)
                                ) {
                                    Icon(
                                        if (selectedSubtitleTag != null) FluxaIcons.Subtitles else FluxaIcons.SubtitlesOff,
                                        null, tint = if (selectedSubtitleTag != null) onAccent else Color.White,
                                        modifier = Modifier.size(18.dp)
                                    )
                                }
                            }
                            Box(
                                modifier = Modifier.clip(RoundedCornerShape(6.dp))
                                    .background(Color.Black.copy(0.55f))
                                    .clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) { isFullscreen = true }
                                    .padding(horizontal = 8.dp, vertical = 5.dp)
                            ) {
                                Icon(FluxaIcons.Fullscreen, null, tint = Color.White, modifier = Modifier.size(18.dp))
                            }
                        }
                    }
                }

                AnimatedVisibility(
                    visible = showSubtitlePicker,
                    enter = fadeIn(), exit = fadeOut(),
                    modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth()
                ) {
                    Box(modifier = Modifier.fillMaxWidth().background(Color.Black.copy(0.82f)).padding(12.dp)) {
                        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            item {
                                SubtitleChip("Off", selectedSubtitleTag == null, accentColor, onAccent) {
                                    selectedSubtitleTag = null; showSubtitlePicker = false
                                }
                            }
                            items(availableSubtitles, key = { it.languageTag }) { sub ->
                                SubtitleChip(sub.label, selectedSubtitleTag == sub.languageTag, accentColor, onAccent) {
                                    selectedSubtitleTag = sub.languageTag; showSubtitlePicker = false
                                }
                            }
                        }
                    }
                }
            }

            is HeroTrailerState.GeoBlocked -> {
                AsyncImage(model = displayImage, contentDescription = null, modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
                Box(modifier = Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.55f)))
                Column(
                    modifier = Modifier.align(Alignment.Center).padding(horizontal = 32.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Text(
                        text = AppStrings.t(lang, "error.geo_blocked"),
                        color = Color.White.copy(alpha = 0.85f),
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Medium,
                        textAlign = TextAlign.Center
                    )
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(10.dp))
                            .background(accentColor.copy(alpha = 0.92f))
                            .clickable {
                                runCatching { context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(state.youtubeUrl))) }
                            }
                            .padding(horizontal = 16.dp, vertical = 8.dp)
                    ) {
                        Text(AppStrings.t(lang, "action.open_on_youtube"), color = onAccent, fontSize = 13.sp, fontWeight = FontWeight.Bold)
                    }
                }
            }

            else -> {
                AsyncImage(model = displayImage, contentDescription = null, modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
                Box(modifier = Modifier.fillMaxSize().background(
                    Brush.verticalGradient(listOf(Color.Black.copy(0.25f), Color.Transparent, Color.Black.copy(0.75f)))
                ))
                Box(modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(84.dp).background(
                    Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(0.18f), Color.Black.copy(0.50f)))
                ))
                if (effectiveTrailer != null) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.Center)
                            .size(56.dp)
                            .clip(CircleShape)
                            .background(if (state is HeroTrailerState.Loading) Color.Black.copy(0.60f) else accentColor.copy(0.92f))
                            .then(if (state !is HeroTrailerState.Loading) Modifier.clickable {
                                if (activeTrailer == null) onSelectTrailer?.invoke(effectiveTrailer)
                                else retryCount++
                            } else Modifier),
                        contentAlignment = Alignment.Center
                    ) {
                        if (state is HeroTrailerState.Loading) {
                            CircularProgressIndicator(color = Color.White, strokeWidth = 2.dp, modifier = Modifier.size(26.dp))
                        } else {
                            Icon(FluxaIcons.PlayArrow, null, tint = onAccent, modifier = Modifier.size(30.dp))
                        }
                    }
                }
            }
        }

        Row(
            modifier = Modifier.fillMaxWidth().statusBarsPadding().padding(14.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            MobileDetailTopIcon(FluxaIcons.ArrowBack, AppStrings.t(lang, "common.back"), onBack)
            if (trailerState is HeroTrailerState.Ready || trailerState is HeroTrailerState.GeoBlocked) {
                MobileDetailTopIcon(FluxaIcons.Close, AppStrings.t(lang, "common.close")) {
                    exoPlayer?.stop(); exoPlayer?.clearMediaItems(); exoPlayer?.release(); exoPlayer = null
                    trailerState = HeroTrailerState.Idle
                    onStopTrailer()
                }
            }
        }
    }
}

@Composable
private fun SubtitleChip(label: String, selected: Boolean, accentColor: Color, onAccent: Color, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(999.dp))
            .background(if (selected) accentColor else Color.White.copy(alpha = 0.12f))
            .clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) { onClick() }
            .padding(horizontal = 12.dp, vertical = 6.dp)
    ) {
        Text(
            text = label,
            color = if (selected) onAccent else Color.White,
            fontSize = 12.sp,
            fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal
        )
    }
}
