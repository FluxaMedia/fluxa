package com.fluxa.app.ui.rust

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager
import android.media.MediaMetadata
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.os.Build
import org.json.JSONObject

class PlaybackMediaSession(
    private val context: Context,
    private val send: (command: String, value: Double) -> Unit,
) {
    private val audio = context.getSystemService(Context.AUDIO_SERVICE) as AudioManager
    private var session: MediaSession? = null
    private var focus: AudioFocusRequest? = null
    private var pausedByFocus = false
    private var noisyRegistered = false

    private val noisy = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            if (intent.action == AudioManager.ACTION_AUDIO_BECOMING_NOISY) send("pause", 0.0)
        }
    }

    private val focusChange = AudioManager.OnAudioFocusChangeListener { change ->
        when (change) {
            AudioManager.AUDIOFOCUS_LOSS -> send("pause", 0.0)
            AudioManager.AUDIOFOCUS_LOSS_TRANSIENT -> {
                pausedByFocus = true
                send("pause", 0.0)
            }
            AudioManager.AUDIOFOCUS_GAIN -> if (pausedByFocus) {
                pausedByFocus = false
                send("play", 0.0)
            }
        }
    }

    private val callback = object : MediaSession.Callback() {
        override fun onPlay() = send("play", 0.0)
        override fun onPause() = send("pause", 0.0)
        override fun onStop() = send("stop", 0.0)
        override fun onSkipToNext() = send("next", 0.0)
        override fun onSkipToPrevious() = send("previous", 0.0)
        override fun onFastForward() = send("fastForward", 0.0)
        override fun onRewind() = send("rewind", 0.0)
        override fun onSeekTo(pos: Long) = send("seekTo", pos / 1000.0)
    }

    fun update(plan: JSONObject) {
        val state = plan.optString("state")
        val session = session ?: create()
        session.setMetadata(
            MediaMetadata.Builder()
                .putString(MediaMetadata.METADATA_KEY_TITLE, plan.optString("title"))
                .putString(MediaMetadata.METADATA_KEY_ARTIST, plan.optString("subtitle"))
                .putString(MediaMetadata.METADATA_KEY_ART_URI, plan.optString("poster"))
                .putLong(MediaMetadata.METADATA_KEY_DURATION, plan.optLong("durationMs"))
                .build(),
        )
        val playing = state == "playing"
        val code = when (state) {
            "playing" -> PlaybackState.STATE_PLAYING
            "buffering" -> PlaybackState.STATE_BUFFERING
            else -> PlaybackState.STATE_PAUSED
        }
        session.setPlaybackState(
            PlaybackState.Builder()
                .setActions(actions(plan))
                .setState(code, plan.optLong("positionMs"), plan.optDouble("speed", 1.0).toFloat())
                .build(),
        )
        if (playing) requestFocus()
    }

    fun release() {
        abandonFocus()
        if (noisyRegistered) {
            runCatching { context.unregisterReceiver(noisy) }
            noisyRegistered = false
        }
        session?.isActive = false
        session?.release()
        session = null
        pausedByFocus = false
    }

    private fun create(): MediaSession {
        val created = MediaSession(context, "fluxa-playback")
        @Suppress("DEPRECATION")
        created.setFlags(MediaSession.FLAG_HANDLES_MEDIA_BUTTONS or MediaSession.FLAG_HANDLES_TRANSPORT_CONTROLS)
        created.setCallback(callback)
        created.isActive = true
        session = created
        context.registerReceiver(noisy, IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY))
        noisyRegistered = true
        return created
    }

    private fun actions(plan: JSONObject): Long {
        val names = plan.optJSONArray("actions") ?: return 0L
        var mask = 0L
        for (index in 0 until names.length()) {
            mask = mask or when (names.optString(index)) {
                "play" -> PlaybackState.ACTION_PLAY or PlaybackState.ACTION_PLAY_PAUSE
                "pause" -> PlaybackState.ACTION_PAUSE
                "stop" -> PlaybackState.ACTION_STOP
                "next" -> PlaybackState.ACTION_SKIP_TO_NEXT
                "previous" -> PlaybackState.ACTION_SKIP_TO_PREVIOUS
                "seekTo" -> PlaybackState.ACTION_SEEK_TO
                "seekForward" -> PlaybackState.ACTION_FAST_FORWARD
                "seekBackward" -> PlaybackState.ACTION_REWIND
                else -> 0L
            }
        }
        return mask
    }

    private fun requestFocus() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            if (focus != null) return
            val request = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
                .setAudioAttributes(
                    AudioAttributes.Builder()
                        .setUsage(AudioAttributes.USAGE_MEDIA)
                        .setContentType(AudioAttributes.CONTENT_TYPE_MOVIE)
                        .build(),
                )
                .setOnAudioFocusChangeListener(focusChange)
                .build()
            focus = request
            audio.requestAudioFocus(request)
        } else {
            @Suppress("DEPRECATION")
            audio.requestAudioFocus(focusChange, AudioManager.STREAM_MUSIC, AudioManager.AUDIOFOCUS_GAIN)
        }
    }

    private fun abandonFocus() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            focus?.let { audio.abandonAudioFocusRequest(it) }
            focus = null
        } else {
            @Suppress("DEPRECATION")
            audio.abandonAudioFocus(focusChange)
        }
    }
}
