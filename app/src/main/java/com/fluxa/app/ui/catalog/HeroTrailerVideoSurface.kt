@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.AspectRatioFrameLayout
import androidx.media3.ui.PlayerView
import com.fluxa.app.shared.feature.player.TrailerCue
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive

private const val HERO_TRAILER_CROP_SCALE = 1.32f

@Composable
internal fun HeroTrailerVideoSurface(
    url: String,
    cues: List<TrailerCue>,
    onActiveSubtitleChanged: (String) -> Unit,
    modifier: Modifier
) {
    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current
    val exoPlayer = remember(url) {
        DetailTrailerPreloader.takeOrCreate(context, url).apply {
            volume = 0f
            repeatMode = Player.REPEAT_MODE_ONE
            playWhenReady = true
        }
    }
    DisposableEffect(exoPlayer) {
        onDispose { exoPlayer.release() }
    }
    DisposableEffect(lifecycleOwner, exoPlayer) {
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_PAUSE -> exoPlayer.pause()
                Lifecycle.Event.ON_RESUME -> {
                    exoPlayer.playWhenReady = true
                    exoPlayer.play()
                }
                else -> Unit
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose { lifecycleOwner.lifecycle.removeObserver(observer) }
    }
    LaunchedEffect(exoPlayer) {
        exoPlayer.playWhenReady = true
        exoPlayer.play()
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
        update = { playerView -> playerView.player = exoPlayer },
        modifier = modifier.scale(HERO_TRAILER_CROP_SCALE)
    )
}
