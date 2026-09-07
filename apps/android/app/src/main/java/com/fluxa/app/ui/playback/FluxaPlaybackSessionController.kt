package com.fluxa.app.ui.playback

import android.content.Context
import android.content.Intent
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.player.MpvEmbeddedPlayer

internal data class FluxaPlaybackSessionState(
    val exoPlayer: ExoPlayer? = null,
    val mpvPlayer: MpvEmbeddedPlayer? = null,
    val title: String = "Fluxa",
    val subtitle: String? = null,
    val artworkUri: String? = null,
    val language: String? = null,
)

internal object FluxaPlaybackSessionRegistry {
    @Volatile
    var state: FluxaPlaybackSessionState = FluxaPlaybackSessionState()
        private set

    fun attach(next: FluxaPlaybackSessionState) {
        state = next
    }

    fun updateMetadata(title: String, subtitle: String?, artworkUri: String?, language: String?) {
        state = state.copy(title = title, subtitle = subtitle, artworkUri = artworkUri, language = language)
    }

    fun clear(exoPlayer: ExoPlayer?, mpvPlayer: MpvEmbeddedPlayer?) {
        if (state.exoPlayer === exoPlayer && state.mpvPlayer === mpvPlayer) {
            state = FluxaPlaybackSessionState()
        }
    }
}

internal object FluxaPlaybackSessionController {
    fun attach(
        context: Context,
        exoPlayer: ExoPlayer?,
        mpvPlayer: MpvEmbeddedPlayer?,
        title: String,
        subtitle: String?,
        artworkUri: String?,
        language: String?,
    ) {
        FluxaPlaybackSessionRegistry.attach(
            FluxaPlaybackSessionState(
                exoPlayer = exoPlayer,
                mpvPlayer = mpvPlayer,
                title = title,
                subtitle = subtitle,
                artworkUri = artworkUri,
                language = language,
            )
        )
        context.startService(Intent(context, FluxaMediaSessionService::class.java))
    }

    fun updateMetadata(title: String, subtitle: String?, artworkUri: String?, language: String?) {
        FluxaPlaybackSessionRegistry.updateMetadata(title, subtitle, artworkUri, language)
        FluxaMediaSessionService.instance?.refreshMpvNotification()
    }

    fun detach(context: Context, exoPlayer: ExoPlayer?, mpvPlayer: MpvEmbeddedPlayer?) {
        FluxaPlaybackSessionRegistry.clear(exoPlayer, mpvPlayer)
        context.stopService(Intent(context, FluxaMediaSessionService::class.java))
    }
}
