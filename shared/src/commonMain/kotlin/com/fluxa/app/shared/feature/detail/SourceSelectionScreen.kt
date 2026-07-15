package com.fluxa.app.shared.feature.detail

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun SourceSelectionScreen(
    content: DetailUiModel,
    language: String?,
    onBack: () -> Unit,
    onStreamSelected: (DetailStreamUiModel) -> Unit,
    modifier: Modifier = Modifier
) {
    Column(modifier = modifier.fillMaxSize().padding(16.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Button(onClick = onBack) {
                Text(AppStrings.t(language, "common.back"))
            }
            Column {
                Text(
                    text = AppStrings.t(language, "auto.sources"),
                    color = Color.White,
                    fontSize = 24.sp,
                    fontWeight = FontWeight.Bold
                )
                Text(text = content.title, color = Color.White.copy(alpha = 0.7f))
            }
        }
        when {
            content.isLoadingStreams -> CircularProgressIndicator(
                color = Color.White,
                modifier = Modifier.align(Alignment.CenterHorizontally).padding(32.dp)
            )
            content.streams.isEmpty() -> Text(
                text = AppStrings.t(language, "auto.no_sources_found_3019f12c"),
                color = Color.White.copy(alpha = 0.7f),
                modifier = Modifier.padding(top = 32.dp)
            )
            else -> LazyColumn(
                modifier = Modifier.fillMaxSize().padding(top = 16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                items(content.streams, key = { it.playableUrl }) { stream ->
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(8.dp))
                            .clickable { onStreamSelected(stream) }
                            .padding(horizontal = 16.dp, vertical = 14.dp)
                    ) {
                        Text(text = stream.title, color = Color.White, fontWeight = FontWeight.SemiBold)
                        Text(text = stream.addonName, color = FluxaColors.accent, fontSize = 12.sp)
                    }
                }
            }
        }
    }
}
