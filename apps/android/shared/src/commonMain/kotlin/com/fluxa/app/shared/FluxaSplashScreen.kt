package com.fluxa.app.shared

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import fluxa.shared.generated.resources.Res
import fluxa.shared.generated.resources.fluxa_mark
import org.jetbrains.compose.resources.painterResource
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaUiTokens

@Composable
fun FluxaSplashScreen(
    backgroundUrl: String? = null,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaUiTokens.colorBackground),
        contentAlignment = Alignment.Center,
    ) {
        if (!backgroundUrl.isNullOrBlank()) {
            FluxaRemoteImage(
                imageUrl = backgroundUrl,
                cacheKey = "profile-picker-background:$backgroundUrl",
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
            )
            Box(Modifier.fillMaxSize().background(androidx.compose.ui.graphics.Color.Black.copy(alpha = 0.78f)))
        }

        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(28.dp),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                Image(
                    painter = painterResource(Res.drawable.fluxa_mark),
                    contentDescription = "Fluxa",
                    contentScale = ContentScale.Fit,
                    modifier = Modifier.size(56.dp),
                )
                Text(
                    text = "fluxa",
                    color = MaterialTheme.colorScheme.onBackground,
                    style = MaterialTheme.typography.displaySmall,
                    fontWeight = FontWeight.Bold,
                )
            }
            CircularProgressIndicator(
                modifier = Modifier.size(28.dp),
                color = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.86f),
                strokeWidth = 3.dp,
            )
        }
    }
}
