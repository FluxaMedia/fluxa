package com.fluxa.app.shared.feature.catalog

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import com.fluxa.app.shared.image.FluxaRemoteImage

@Composable
actual fun TvHeroRow(
    items: List<CatalogItemUiModel>,
    language: String?,
    onItemClick: (CatalogItemUiModel) -> Unit,
    modifier: Modifier
) {
    val hero = items.firstOrNull() ?: return
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(360.dp)
            .clip(RoundedCornerShape(16.dp))
            .clickable { onItemClick(hero) }
    ) {
        FluxaRemoteImage(
            imageUrl = hero.backdropUrl ?: hero.card.artworkUrl,
            cacheKey = "desktop-hero:${hero.id}",
            contentDescription = hero.card.title,
            modifier = Modifier.fillMaxWidth().height(360.dp),
            contentScale = ContentScale.Crop
        )
    }
}
