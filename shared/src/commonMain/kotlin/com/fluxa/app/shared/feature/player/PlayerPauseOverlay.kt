package com.fluxa.app.shared.feature.player

import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaDimensions

import coil3.compose.AsyncImage
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.BiasAlignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun PlayerPauseMetadataOverlay(
    content: PlayerContentUiModel,
    episodeMetaLine: String?,
    lang: String,
    modifier: Modifier = Modifier
) {
    var logoLoadError by remember(content.logoUrl) { mutableStateOf(false) }
    val logoUrl = content.logoUrl?.takeIf { it.isNotBlank() }

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(
                Brush.horizontalGradient(
                    colors = listOf(
                        Color.Black.copy(alpha = 0.85f),
                        Color.Black.copy(alpha = 0.45f),
                        Color.Transparent
                    )
                )
            )
            .padding(horizontal = FluxaDimensions.PlayerChrome.edgeMargin, vertical = 40.dp),
        verticalArrangement = Arrangement.Bottom
    ) {
        Text(
            text = AppStrings.t(lang, "player.youre_watching"),
            color = Color(0xFFB8B8B8),
            fontSize = 15.sp
        )
        Spacer(modifier = Modifier.height(10.dp))

        if (logoUrl != null && !logoLoadError) {
            AsyncImage(
                model = logoUrl,
                contentDescription = content.title,
                contentScale = ContentScale.Fit,
                alignment = BiasAlignment(-1f, 1f),
                modifier = Modifier.height(72.dp).width(240.dp),
                onError = { logoLoadError = true }
            )
        } else {
            Text(
                text = content.title,
                color = Color.White,
                fontSize = 30.sp,
                fontWeight = FontWeight.ExtraBold,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.fillMaxWidth(0.62f)
            )
        }

        if (!episodeMetaLine.isNullOrBlank()) {
            Text(
                text = episodeMetaLine,
                color = Color(0xFFCCCCCC),
                fontSize = 15.sp,
                modifier = Modifier.padding(top = 8.dp)
            )
        }

        if (content.subtitle.isNotBlank()) {
            Text(
                text = content.subtitle,
                color = Color(0xFFD6D6D6),
                fontSize = 14.sp,
                lineHeight = 20.sp,
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier
                    .padding(top = 12.dp)
                    .fillMaxWidth(0.62f)
            )
        }
    }
}
