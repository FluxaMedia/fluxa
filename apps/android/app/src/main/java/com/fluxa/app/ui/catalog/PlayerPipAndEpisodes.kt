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
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.player.PlayerEngine

internal object PlayerPipSuppression {
    @Volatile
    var suppressAutoEnter = false
}

private const val PIP_ACTION_PLAY_PAUSE = "com.fluxa.app.PIP_PLAY_PAUSE"
private const val PIP_ACTION_REWIND = "com.fluxa.app.PIP_REWIND"
private const val PIP_ACTION_FORWARD = "com.fluxa.app.PIP_FORWARD"

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

        val locator = currentVideoId?.let(FluxaCoreNative::parseEpisodeLocator) ?: return@LaunchedEffect
        val season = locator.season
        val episode = locator.episode
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
    activeEngine: PlayerEngine?,
    seekBackward: () -> Unit,
    seekForward: () -> Unit
) {
    DisposableEffect(context, isPlaying, activeEngine) {
        val appContext = context
        updatePlayerPipParams(appContext, lang, isPlaying)
        val receiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, intent: Intent?) {
                when (intent?.action) {
                    PIP_ACTION_PLAY_PAUSE -> {
                        activeEngine?.setPaused(isPlaying)
                        updatePlayerPipParams(appContext, lang, !isPlaying)
                    }
                    PIP_ACTION_REWIND -> {
                        seekBackward()
                    }
                    PIP_ACTION_FORWARD -> {
                        seekForward()
                    }
                }
            }
        }
        val filter = IntentFilter().apply {
            addAction(PIP_ACTION_PLAY_PAUSE)
            addAction(PIP_ACTION_REWIND)
            addAction(PIP_ACTION_FORWARD)
        }
        ContextCompat.registerReceiver(appContext, receiver, filter, ContextCompat.RECEIVER_NOT_EXPORTED)
        onDispose { runCatching { appContext.unregisterReceiver(receiver) } }
    }
}

internal fun enterPlayerPipMode(
    context: Context,
    lang: String,
    isPlaying: Boolean
) {
    val activity = context.findActivity() ?: return
    val params = buildPlayerPipParams(context, lang, isPlaying) ?: return
    runCatching { activity.enterPictureInPictureMode(params) }
}

private fun updatePlayerPipParams(
    context: Context,
    lang: String,
    isPlaying: Boolean
) {
    val activity = context.findActivity() ?: return
    buildPlayerPipParams(context, lang, isPlaying)?.let { params ->
        runCatching { activity.setPictureInPictureParams(params) }
    }
}

private fun buildPlayerPipParams(
    context: Context,
    lang: String,
    isPlaying: Boolean
): PictureInPictureParams? {
    val rewindTitle = AppStrings.t(lang, "player.seek_back")
    val playPauseTitle = AppStrings.t(lang, if (isPlaying) "player.pause" else "player.play")
    val forwardTitle = AppStrings.t(lang, "player.seek_forward")
    val actions = buildList {
        // Android launchers commonly show at most three PiP actions.
        add(pipRemoteAction(context, android.R.drawable.ic_media_rew, rewindTitle, PIP_ACTION_REWIND, 4103))
        add(
            pipRemoteAction(
                context,
                if (isPlaying) android.R.drawable.ic_media_pause else android.R.drawable.ic_media_play,
                playPauseTitle,
                PIP_ACTION_PLAY_PAUSE,
                4101
            )
        )
        add(pipRemoteAction(context, android.R.drawable.ic_media_ff, forwardTitle, PIP_ACTION_FORWARD, 4105))
    }
    return PictureInPictureParams.Builder()
        .setAspectRatio(Rational(16, 9))
        .setActions(actions)
        .build()
}

private fun pipRemoteAction(
    context: Context,
    iconRes: Int,
    title: String,
    action: String,
    requestCode: Int
): RemoteAction = RemoteAction(
    Icon.createWithResource(context, iconRes),
    title,
    title,
    pipPendingIntent(context, action, requestCode)
)

private fun pipPendingIntent(context: Context, action: String, requestCode: Int): PendingIntent {
    return PendingIntent.getBroadcast(
        context,
        requestCode,
        Intent(action).setPackage(context.packageName),
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
    )
}
