package com.fluxa.app.ui.catalog

import android.app.Activity
import android.content.Context
import android.media.AudioManager
import android.provider.Settings
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember

internal class PlayerSystemControls(
    private val activity: Activity?,
    val audioManager: AudioManager,
    val maxVolume: Int,
    private val state: PlayerScreenState,
) {
    fun adjustVolume(delta: Float) {
        val next = (state.currentVolumeExact + delta * maxVolume)
            .coerceIn(0f, maxVolume.toFloat())
        state.currentVolumeExact = next
        val rounded = next.toInt()
        if (rounded != state.currentVolume) {
            audioManager.setStreamVolume(AudioManager.STREAM_MUSIC, rounded, 0)
            state.currentVolume = rounded
        }
        state.volumeBarVersion += 1
    }

    fun adjustBrightness(delta: Float) {
        val next = (state.currentBrightness + delta).coerceIn(MIN_BRIGHTNESS, 1f)
        state.currentBrightness = next
        activity?.window?.let { window ->
            window.attributes = window.attributes.apply { screenBrightness = next }
        }
        state.brightnessBarVersion += 1
    }

    companion object {
        private const val MIN_BRIGHTNESS = 0.02f
    }
}

@Composable
internal fun rememberPlayerSystemControls(
    context: Context,
    activity: Activity?,
    state: PlayerScreenState,
    audioManager: AudioManager,
): PlayerSystemControls {
    val maxVolume = remember(audioManager) {
        audioManager.getStreamMaxVolume(AudioManager.STREAM_MUSIC)
    }

    LaunchedEffect(context, activity) {
        val windowBrightness = activity?.window?.attributes?.screenBrightness ?: -1f
        state.currentBrightness = if (windowBrightness in 0f..1f) {
            windowBrightness
        } else {
            runCatching {
                Settings.System.getInt(
                    context.contentResolver,
                    Settings.System.SCREEN_BRIGHTNESS,
                ) / 255f
            }.getOrDefault(0.5f)
        }
    }

    return remember(activity, audioManager, maxVolume, state) {
        PlayerSystemControls(
            activity = activity,
            audioManager = audioManager,
            maxVolume = maxVolume,
            state = state,
        )
    }
}
