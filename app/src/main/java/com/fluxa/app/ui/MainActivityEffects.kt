package com.fluxa.app.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.HomeViewModel

@Composable
internal fun TraktDeviceAuthDialog(
    state: TraktDeviceAuthUiState?,
    lang: String?,
    onDismiss: () -> Unit
) {
    state ?: return
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(AppStrings.t(lang, "brand.trakt")) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text(AppStrings.t(lang, "trakt.device.open_url"))
                Text(
                    text = state.verificationUrl,
                    color = Color.White,
                    fontWeight = FontWeight.SemiBold
                )
                Text(AppStrings.t(lang, "trakt.device.enter_code"))
                Text(
                    text = state.userCode,
                    fontSize = 30.sp,
                    fontWeight = FontWeight.Black,
                    letterSpacing = 2.sp
                )
                if (state.isWaiting) {
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        CircularProgressIndicator(modifier = Modifier.size(18.dp), strokeWidth = 2.dp)
                        Text(AppStrings.t(lang, "trakt.device.waiting"))
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) {
                Text(AppStrings.t(lang, "common.close"))
            }
        }
    )
}

@Composable
internal fun PlayerLifecycleEffect(
    currentScreen: Screen,
    activeProfile: UserProfile?,
    mainPlayer: ExoPlayer,
    previewPlayer: ExoPlayer,
    homeViewModel: HomeViewModel,
    enterPictureInPicture: () -> Unit
) {
    val lifecycleOwner = LocalLifecycleOwner.current
    val latestScreen by rememberUpdatedState(currentScreen)
    val latestActiveProfile by rememberUpdatedState(activeProfile)
    val latestBackgroundPlayback by rememberUpdatedState(activeProfile?.safeBackgroundPlayback == true)
    val latestPictureInPicture by rememberUpdatedState(activeProfile?.safePictureInPicture == true)
    DisposableEffect(lifecycleOwner, mainPlayer, previewPlayer) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_RESUME) {
                if (latestActiveProfile != null) {
                    homeViewModel.refreshInstalledAddons(forceRefresh = true)
                }
                homeViewModel.refreshExternalContinueWatching()
            } else if (event == Lifecycle.Event.ON_PAUSE) {
                previewPlayer.pause()
                val isPlayerScreen = latestScreen is Screen.Player
                val shouldEnterPip = isPlayerScreen &&
                    latestPictureInPicture

                if (shouldEnterPip) {
                    runCatching { enterPictureInPicture() }
                }

                if (!isPlayerScreen || (!latestBackgroundPlayback && !shouldEnterPip)) {
                    mainPlayer.pause()
                }
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
            mainPlayer.release()
            previewPlayer.release()
        }
    }
}
