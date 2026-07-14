package com.fluxa.app.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.shared.image.FluxaRemoteImage

@Composable
fun MobileExplorePosterCard(
    title: String,
    artworkUrl: String?,
    onClick: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
    ) {
        FluxaRemoteImage(
            imageUrl = artworkUrl,
            cacheKey = artworkUrl?.let { "explore-poster:$it" },
            contentDescription = title,
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(2f / 3f)
                .clip(RoundedCornerShape(10.dp))
                .background(Color.White.copy(alpha = 0.06f)),
            contentScale = ContentScale.Crop
        )
        Spacer(Modifier.height(6.dp))
        androidx.compose.material3.Text(
            text = title,
            color = Color.White,
            fontSize = 14.sp,
            fontWeight = FontWeight.Black,
            lineHeight = 16.sp,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis
        )
    }
}
