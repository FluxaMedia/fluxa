@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.ui.AspectRatioFrameLayout
import androidx.media3.ui.PlayerView
import com.fluxa.app.shared.feature.player.TrailerCue
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive

@Composable
internal fun HeroTrailerVideoSurface(
    url: String,
    cues: List<TrailerCue>,
    onActiveSubtitleChanged: (String) -> Unit,
    modifier: Modifier
) {
    val context = LocalContext.current
    val exoPlayer = remember(url) {
        ExoPlayer.Builder(context)
            .setMediaSourceFactory(DefaultMediaSourceFactory(TrailerResolver.mediaDataSourceFactory()))
            .build().apply {
                setMediaItem(MediaItem.fromUri(url))
                volume = 0f
                repeatMode = Player.REPEAT_MODE_ONE
                playWhenReady = true
                prepare()
            }
    }
    DisposableEffect(exoPlayer) {
        onDispose { exoPlayer.release() }
    }

    LaunchedEffect(exoPlayer, cues) {
        while (isActive) {
            val positionSeconds = exoPlayer.currentPosition / 1000.0
            onActiveSubtitleChanged(cues.firstOrNull { positionSeconds in it.start..it.end }?.text.orEmpty())
            delay(200)
        }
    }
    DisposableEffect(Unit) {
        onDispose { onActiveSubtitleChanged("") }
    }

    AndroidView(
        factory = { ctx ->
            PlayerView(ctx).apply {
                player = exoPlayer
                useController = false
                resizeMode = AspectRatioFrameLayout.RESIZE_MODE_ZOOM
                setBackgroundColor(android.graphics.Color.TRANSPARENT)
            }
        },
        modifier = modifier
    )
}
