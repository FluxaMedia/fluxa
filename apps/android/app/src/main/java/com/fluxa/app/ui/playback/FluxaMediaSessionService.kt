@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.ui.playback

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.graphics.Color
import android.graphics.drawable.Icon
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import androidx.media3.session.MediaSessionService
import com.fluxa.app.R
import com.fluxa.app.common.AppStrings
import com.fluxa.app.player.MpvEmbeddedPlayer
import com.fluxa.app.ui.MainActivity

internal class FluxaMediaSessionService : MediaSessionService() {
    private var exoSession: androidx.media3.session.MediaSession? = null
    private var mpvSession: MediaSession? = null
    private val handler = Handler(Looper.getMainLooper())
    private val mpvStateTicker = object : Runnable {
        override fun run() {
            updateMpvPlaybackState()
            if (mpvSession != null) handler.postDelayed(this, MPV_STATE_UPDATE_MS)
        }
    }

    override fun onCreate() {
        super.onCreate()
        instance = this
        val state = FluxaPlaybackSessionRegistry.state
        state.exoPlayer?.let { player ->
            exoSession = androidx.media3.session.MediaSession.Builder(this, player)
                .setId(MEDIA_SESSION_ID)
                .setSessionActivity(mainActivityPendingIntent())
                .build()
        }
        state.mpvPlayer?.let { player ->
            createMpvSession(player)
            startForeground(MPV_NOTIFICATION_ID, buildMpvNotification())
            handler.post(mpvStateTicker)
        }
    }

    override fun onGetSession(controllerInfo: androidx.media3.session.MediaSession.ControllerInfo): androidx.media3.session.MediaSession? = exoSession

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_PLAY -> FluxaPlaybackSessionRegistry.state.mpvPlayer?.setPaused(false)
            ACTION_PAUSE -> FluxaPlaybackSessionRegistry.state.mpvPlayer?.setPaused(true)
        }
        if (mpvSession != null) {
            updateMpvPlaybackState()
            refreshMpvNotification()
        }
        return Service.START_STICKY
    }

    fun refreshMpvNotification() {
        if (mpvSession == null) return
        updateMpvPlaybackState()
        getSystemService(NotificationManager::class.java)?.notify(MPV_NOTIFICATION_ID, buildMpvNotification())
    }

    override fun onDestroy() {
        handler.removeCallbacksAndMessages(null)
        mpvSession?.isActive = false
        mpvSession?.release()
        mpvSession = null
        exoSession?.release()
        exoSession = null
        if (android.os.Build.VERSION.SDK_INT >= 24) stopForeground(STOP_FOREGROUND_REMOVE)
        instance = null
        super.onDestroy()
    }

    private fun createMpvSession(player: MpvEmbeddedPlayer) {
        mpvSession = MediaSession(this, MEDIA_SESSION_ID).apply {
            setFlags(
                MediaSession.FLAG_HANDLES_MEDIA_BUTTONS or
                    MediaSession.FLAG_HANDLES_TRANSPORT_CONTROLS
            )
            setCallback(object : MediaSession.Callback() {
                override fun onPlay() = player.setPaused(false)
                override fun onPause() = player.setPaused(true)
                override fun onSeekTo(pos: Long) = player.seekTo(pos)
            }, handler)
            isActive = true
        }
    }

    private fun updateMpvPlaybackState() {
        val session = mpvSession ?: return
        val player = FluxaPlaybackSessionRegistry.state.mpvPlayer ?: return
        val state = player.state.value
        val playbackState = if (state.isPlaying) PlaybackState.STATE_PLAYING else PlaybackState.STATE_PAUSED
        session.setPlaybackState(
            PlaybackState.Builder()
                .setActions(
                    PlaybackState.ACTION_PLAY or
                        PlaybackState.ACTION_PAUSE or
                        PlaybackState.ACTION_PLAY_PAUSE or
                        PlaybackState.ACTION_SEEK_TO
                )
                .setState(playbackState, state.positionMs.coerceAtLeast(0L), 1f)
                .build()
        )
    }

    private fun buildMpvNotification(): Notification {
        val language = FluxaPlaybackSessionRegistry.state.language
        ensureNotificationChannel(language)
        val state = FluxaPlaybackSessionRegistry.state
        val isPlaying = state.mpvPlayer?.state?.value?.isPlaying == true
        val builder = Notification.Builder(this, MPV_NOTIFICATION_CHANNEL)
            .setSmallIcon(R.drawable.ic_launcher)
            .setContentTitle(state.title)
            .setContentText(state.subtitle ?: "Fluxa")
            .setContentIntent(mainActivityPendingIntent())
            .setOngoing(isPlaying)
            .setVisibility(Notification.VISIBILITY_PUBLIC)
            .setStyle(Notification.MediaStyle().setMediaSession(mpvSession?.sessionToken))
            .addAction(
                notificationAction(
                    if (isPlaying) ACTION_PAUSE else ACTION_PLAY,
                    AppStrings.t(language, if (isPlaying) "player.pause" else "player.play"),
                )
            )
        return builder.build()
    }

    private fun notificationAction(action: String, title: String): Notification.Action {
        val intent = Intent(this, FluxaMediaSessionService::class.java).setAction(action)
        val pendingIntent = PendingIntent.getService(
            this,
            action.hashCode(),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        return Notification.Action.Builder(Icon.createWithResource(this, R.drawable.ic_launcher), title, pendingIntent).build()
    }

    private fun mainActivityPendingIntent(): PendingIntent = PendingIntent.getActivity(
        this,
        0,
        Intent(this, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
    )

    private fun ensureNotificationChannel(language: String?) {
        if (android.os.Build.VERSION.SDK_INT >= 26) {
            getSystemService(NotificationManager::class.java)?.createNotificationChannel(
                NotificationChannel(
                    MPV_NOTIFICATION_CHANNEL,
                    AppStrings.t(language, "player.playback_channel"),
                    NotificationManager.IMPORTANCE_LOW,
                ).apply { lightColor = Color.WHITE }
            )
        }
    }

    companion object {
        private const val MEDIA_SESSION_ID = "fluxa-player"
        private const val MPV_NOTIFICATION_CHANNEL = "fluxa_playback"
        private const val MPV_NOTIFICATION_ID = 4101
        private const val MPV_STATE_UPDATE_MS = 1000L
        private const val ACTION_PLAY = "com.fluxa.app.action.PLAY"
        private const val ACTION_PAUSE = "com.fluxa.app.action.PAUSE"

        @Volatile
        var instance: FluxaMediaSessionService? = null
            private set
    }
}
