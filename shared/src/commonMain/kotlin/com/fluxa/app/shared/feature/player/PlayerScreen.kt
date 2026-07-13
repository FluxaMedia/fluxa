package com.fluxa.app.shared.feature.player

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun PlayerScreen(
    state: PlayerUiState,
    language: String?,
    onAction: (PlayerAction) -> Unit,
    modifier: Modifier = Modifier
) {
    Box(
        modifier = modifier
            .fillMaxSize()
            .background(Color.Black)
    ) {
        if (state.isBuffering) {
            CircularProgressIndicator(
                color = Color.White,
                modifier = Modifier.align(Alignment.Center)
            )
        }
        state.content?.let { content ->
            Column(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .background(FluxaColors.background.copy(alpha = 0.86f))
                    .padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Text(text = content.title, color = Color.White, fontWeight = FontWeight.Bold)
                if (content.subtitle.isNotBlank()) {
                    Text(text = content.subtitle, color = Color.White.copy(alpha = 0.72f))
                }
                if (content.streamLabel.isNotBlank()) {
                    Text(text = content.streamLabel, color = Color.White.copy(alpha = 0.72f))
                }
                Slider(
                    value = state.positionMs.toFloat(),
                    onValueChange = { onAction(PlayerAction.SeekTo(it.toLong())) },
                    valueRange = 0f..state.durationMs.coerceAtLeast(1L).toFloat()
                )
                Button(onClick = { onAction(PlayerAction.PlayPause) }) {
                    Text(AppStrings.t(language, if (state.isPlaying) "common.pause" else "common.play"))
                }
            }
        }
    }
}
