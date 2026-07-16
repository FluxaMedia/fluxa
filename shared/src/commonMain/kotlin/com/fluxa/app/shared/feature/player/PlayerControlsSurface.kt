package com.fluxa.app.shared.feature.player

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings

@Composable
fun PlayerControlsSurface(
    state: PlayerRenderState,
    language: String?,
    onAction: (PlayerRenderAction) -> Unit,
    modifier: Modifier = Modifier
) {
    Box(modifier = modifier.fillMaxSize().background(Color.Transparent)) {
        if (state.isBuffering) {
            CircularProgressIndicator(modifier = Modifier.align(Alignment.Center), color = Color.White)
        }
        if (state.controlsVisible) {
            Column(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .background(Color.Black.copy(alpha = 0.68f))
                    .padding(20.dp)
            ) {
                state.content?.let { content ->
                    Text(content.title, color = Color.White, fontWeight = FontWeight.Bold)
                    if (content.subtitle.isNotBlank()) {
                        Text(content.subtitle, color = Color.White.copy(alpha = 0.7f))
                    }
                }
                Spacer(Modifier.height(14.dp))
                LinearProgressIndicator(
                    progress = { state.progress },
                    modifier = Modifier.fillMaxWidth(),
                    color = Color.White,
                    trackColor = Color.White.copy(alpha = 0.25f)
                )
                Spacer(Modifier.height(12.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(18.dp), verticalAlignment = Alignment.CenterVertically) {
                    PlayerTextAction(
                        label = AppStrings.t(language, if (state.isPlaying) "player.pause" else "player.play"),
                        onClick = { onAction(PlayerRenderAction.PlayPause) }
                    )
                    if (state.sources.isNotEmpty()) {
                        PlayerTextAction(
                            label = AppStrings.t(language, "auto.sources"),
                            onClick = { onAction(PlayerRenderAction.SourceSelected(state.sources.first().id)) }
                        )
                    }
                    if (state.audioTracks.isNotEmpty()) {
                        PlayerTextAction(
                            label = AppStrings.t(language, "player.audio"),
                            onClick = { onAction(PlayerRenderAction.AudioTrackSelected(state.audioTracks.first().id)) }
                        )
                    }
                    if (state.subtitleTracks.isNotEmpty()) {
                        PlayerTextAction(
                            label = AppStrings.t(language, "player.subtitles"),
                            onClick = { onAction(PlayerRenderAction.SubtitleTrackSelected(state.subtitleTracks.first().id)) }
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun PlayerTextAction(label: String, onClick: () -> Unit) {
    Text(
        text = label,
        color = Color.White,
        fontWeight = FontWeight.SemiBold,
        modifier = Modifier.clickable(onClick = onClick).padding(vertical = 8.dp)
    )
}
