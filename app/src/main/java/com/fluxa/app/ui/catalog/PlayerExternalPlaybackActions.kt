package com.fluxa.app.ui.catalog

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.widget.Toast
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.remote.Meta

internal class PlayerExternalPlaybackActions(
    private val context: Context,
    private val state: PlayerScreenState,
    private val useMpvBackend: Boolean,
    private val exoPlayer: ExoPlayer,
    private val meta: Meta,
    private val videoId: String?,
    private val activeProfile: UserProfile?,
    private val viewModel: HomeViewModel,
    private val launchExternalIntent: (Intent) -> Unit,
) {
    fun openExternalPlayer() {
        val chosenStream = state.currentStreams.getOrNull(state.currentStreamIndex)
        val rawUrl = state.resolvedUrl
            ?: chosenStream?.playableUrl
            ?: state.currentUrl
        if (rawUrl.isNullOrEmpty()) return

        val currentPosition = currentPosition()
        state.externalPlayerStartedPosition = currentPosition
        val intent = AndroidExternalPlayerLauncher.createLaunchIntent(
            context = context,
            url = rawUrl,
            title = playbackNotificationTitle(meta, state.currentVideoId ?: videoId),
            positionMs = currentPosition,
            packageName = activeProfile?.externalPlayerTarget,
            headers = chosenStream?.resolveHeaders().orEmpty(),
            subtitles = state.currentExternalSubtitleTracks,
        ) ?: run {
            showToast("toast.external_player_not_found")
            return
        }

        PlayerPipSuppression.suppressAutoEnter = true
        runCatching { launchExternalIntent(intent) }
            .onSuccess {
                viewModel.startExternalPlaybackTracking(
                    meta = meta,
                    videoId = state.currentVideoId ?: videoId,
                    initialPositionMs = currentPosition,
                    initialDurationMs = state.engine.timeline.duration.coerceAtLeast(0L),
                    streamIndex = state.currentStreamIndex,
                    episodeName = state.currentEpisodeMetaLine,
                    streamUrl = rawUrl,
                    streamTitle = chosenStream?.title,
                    targetPackage = activeProfile?.externalPlayerTarget,
                )
            }
            .onFailure {
                PlayerPipSuppression.suppressAutoEnter = false
                showToast("toast.external_player_not_found")
            }
    }

    fun cast(url: String? = state.resolvedUrl) {
        if (url.isNullOrEmpty()) return
        val intent = Intent(Intent.ACTION_VIEW).apply {
            setDataAndType(Uri.parse(PlayerCastNetwork.reachableUrl(url)), VIDEO_MIME_TYPE)
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            putExtra(TITLE_EXTRA, meta.name)
            putExtra(POSTER_EXTRA, meta.poster)
        }
        runCatching {
            context.startActivity(
                Intent.createChooser(
                    intent,
                    AppStrings.t(activeProfile?.safeLanguage, "auto.choose_player"),
                ),
            )
        }.onFailure {
            showToast("toast.cast_app_not_found")
        }
    }

    private fun currentPosition(): Long =
        if (useMpvBackend) state.engine.timeline.position else exoPlayer.currentPosition

    private fun showToast(key: String) {
        Toast.makeText(
            context,
            AppStrings.t(activeProfile?.safeLanguage, key),
            Toast.LENGTH_SHORT,
        ).show()
    }

    private companion object {
        const val VIDEO_MIME_TYPE = "video/*"
        const val TITLE_EXTRA = "title"
        const val POSTER_EXTRA = "poster"
    }
}

@Composable
internal fun rememberPlayerExternalPlaybackActions(
    context: Context,
    state: PlayerScreenState,
    useMpvBackend: Boolean,
    exoPlayer: ExoPlayer,
    meta: Meta,
    videoId: String?,
    activeProfile: UserProfile?,
    viewModel: HomeViewModel,
): PlayerExternalPlaybackActions {
    val externalPlayerLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.StartActivityForResult(),
    ) { result ->
        val returnedPosition = result.data?.getLongExtra("position", -1L)?.takeIf { it >= 0L }
            ?: result.data?.getLongExtra("position_ms", -1L)?.takeIf { it >= 0L }
        val returnedDuration = result.data?.getLongExtra("duration", -1L)?.takeIf { it > 0L }
            ?: result.data?.getLongExtra("duration_ms", -1L)?.takeIf { it > 0L }
        viewModel.finishExternalPlaybackTracking(returnedPosition, returnedDuration)
        state.externalPlayerStartedPosition = -1L
        PlayerPipSuppression.suppressAutoEnter = false
    }

    return PlayerExternalPlaybackActions(
        context = context,
        state = state,
        useMpvBackend = useMpvBackend,
        exoPlayer = exoPlayer,
        meta = meta,
        videoId = videoId,
        activeProfile = activeProfile,
        viewModel = viewModel,
        launchExternalIntent = externalPlayerLauncher::launch,
    )
}
