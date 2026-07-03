@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import android.app.PendingIntent
import android.app.PictureInPictureParams
import android.app.RemoteAction
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.graphics.drawable.Icon
import android.util.Rational
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.core.content.ContextCompat
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.player.PlayerEngine

private const val PIP_ACTION_PLAY_PAUSE = "com.fluxa.app.PIP_PLAY_PAUSE"
private const val PIP_ACTION_NEXT = "com.fluxa.app.PIP_NEXT"

@Composable
internal fun PlayerEpisodeNavigationEffect(
    meta: Meta,
    currentVideoId: String?,
    viewModel: HomeViewModel,
    language: String,
    setPreviousEpisode: (Video?) -> Unit,
    setNextEpisode: (Video?) -> Unit
) {
    LaunchedEffect(currentVideoId, meta.id) {
        setNextEpisode(null)
        setPreviousEpisode(null)
        if (meta.type != "series") return@LaunchedEffect

        val parts = currentVideoId?.split(":") ?: return@LaunchedEffect
        val season = parts.getOrNull(parts.size - 2)?.toIntOrNull() ?: 1
        val episode = parts.getOrNull(parts.size - 1)?.toIntOrNull() ?: 1
        val episodes = viewModel.getSeasonEpisodes(meta.id, season, language)
        val currentIndex = episodes.indexOfFirst { it.id == currentVideoId }

        if (currentIndex > 0) {
            setPreviousEpisode(episodes[currentIndex - 1])
        } else if (currentIndex == 0 && season > 1) {
            setPreviousEpisode(viewModel.getSeasonEpisodes(meta.id, season - 1, language).lastOrNull())
        }

        if (currentIndex != -1 && currentIndex < episodes.size - 1) {
            setNextEpisode(episodes[currentIndex + 1])
        } else if (currentIndex != -1 && currentIndex == episodes.size - 1) {
            setNextEpisode(viewModel.getSeasonEpisodes(meta.id, season + 1, language).firstOrNull())
        }
    }
}

@Composable
internal fun PlayerPipEffect(
    context: Context,
    lang: String,
    isPlaying: Boolean,
    hasNextEpisode: Boolean,
    activeEngine: PlayerEngine?,
    playNext: () -> Unit
) {
    DisposableEffect(context, isPlaying, hasNextEpisode, activeEngine) {
        val appContext = context
        updatePlayerPipParams(appContext, lang, isPlaying, hasNextEpisode)
        val receiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, intent: Intent?) {
                when (intent?.action) {
                    PIP_ACTION_PLAY_PAUSE -> {
                        activeEngine?.setPaused(isPlaying)
                        updatePlayerPipParams(appContext, lang, isPlaying, hasNextEpisode)
                    }
                    PIP_ACTION_NEXT -> {
                        playNext()
                        updatePlayerPipParams(appContext, lang, isPlaying, hasNextEpisode)
                    }
                }
            }
        }
        val filter = IntentFilter().apply {
            addAction(PIP_ACTION_PLAY_PAUSE)
            addAction(PIP_ACTION_NEXT)
        }
        ContextCompat.registerReceiver(appContext, receiver, filter, ContextCompat.RECEIVER_NOT_EXPORTED)
        onDispose { runCatching { appContext.unregisterReceiver(receiver) } }
    }
}

internal fun enterPlayerPipMode(
    context: Context,
    lang: String,
    isPlaying: Boolean,
    hasNextEpisode: Boolean
) {
    val activity = context.findActivity() ?: return
    val params = buildPlayerPipParams(context, lang, isPlaying, hasNextEpisode) ?: return
    runCatching { activity.enterPictureInPictureMode(params) }
}

private fun updatePlayerPipParams(
    context: Context,
    lang: String,
    isPlaying: Boolean,
    hasNextEpisode: Boolean
) {
    val activity = context.findActivity() ?: return
    buildPlayerPipParams(context, lang, isPlaying, hasNextEpisode)?.let { params ->
        runCatching { activity.setPictureInPictureParams(params) }
    }
}

private fun buildPlayerPipParams(
    context: Context,
    lang: String,
    isPlaying: Boolean,
    hasNextEpisode: Boolean
): PictureInPictureParams? {
    val playPauseTitle = AppStrings.t(lang, if (isPlaying) "player.pause" else "player.play")
    val actions = buildList {
        add(
            RemoteAction(
                Icon.createWithResource(context, if (isPlaying) android.R.drawable.ic_media_pause else android.R.drawable.ic_media_play),
                playPauseTitle,
                playPauseTitle,
                pipPendingIntent(context, PIP_ACTION_PLAY_PAUSE, 4101)
            )
        )
        if (hasNextEpisode) {
            val nextTitle = AppStrings.t(lang, "auto.next_episode")
            add(
                RemoteAction(
                    Icon.createWithResource(context, android.R.drawable.ic_media_next),
                    nextTitle,
                    nextTitle,
                    pipPendingIntent(context, PIP_ACTION_NEXT, 4102)
                )
            )
        }
    }
    return PictureInPictureParams.Builder()
        .setAspectRatio(Rational(16, 9))
        .setActions(actions)
        .build()
}

private fun pipPendingIntent(context: Context, action: String, requestCode: Int): PendingIntent {
    return PendingIntent.getBroadcast(
        context,
        requestCode,
        Intent(action).setPackage(context.packageName),
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
    )
}
