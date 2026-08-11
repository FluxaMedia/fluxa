@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.shared.feature.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.tv.material3.Carousel
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.shared.shortenHeroSynopsis

@Composable
actual fun TvHeroRow(
    items: List<CatalogItemUiModel>,
    language: String?,
    onItemClick: (CatalogItemUiModel) -> Unit,
    modifier: Modifier
) {
    if (items.isEmpty()) return
    val hero = items.take(10)
    Carousel(
        itemCount = hero.size,
        modifier = modifier
            .fillMaxWidth()
            .height(360.dp)
            .clip(RoundedCornerShape(16.dp))
    ) { index ->
        val item = hero[index]
        var focused by remember(item.id) { mutableStateOf(false) }
        Box(
            modifier = Modifier
                .fillMaxSize()
                .onFocusChanged { focused = it.isFocused }
                .then(if (focused) Modifier.border(4.dp, Color.White, RoundedCornerShape(16.dp)) else Modifier)
                .clickable { onItemClick(item) }
        ) {
            FluxaRemoteImage(
                imageUrl = item.backdropUrl ?: item.card.artworkUrl,
                cacheKey = "tv-hero:${item.id}",
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(
                        Brush.horizontalGradient(
                            0f to Color.Black.copy(alpha = 0.75f),
                            0.6f to Color.Transparent
                        )
                    )
            )
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(start = 58.dp, bottom = 40.dp, end = 40.dp)
            ) {
                Text(
                    text = item.card.title,
                    color = Color.White,
                    fontSize = 36.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
                val metadataParts = remember(item.genres, item.releaseLabel, item.type, item.seasonsCount, item.runtimeLabel, item.ageRating, language) {
                    buildList {
                        item.genres.firstOrNull { it.isNotBlank() }?.let { add(it) }
                        item.releaseLabel?.takeIf { it.isNotBlank() }?.let { add(it) }
                        if (item.type == "series" && (item.seasonsCount ?: 0) > 0) {
                            add("${item.seasonsCount} ${AppStrings.t(language, "auto.seasons")}")
                        } else {
                            item.runtimeLabel?.takeIf { it.isNotBlank() }?.let { add(it) }
                        }
                        item.ageRating?.takeIf { it.isNotBlank() }?.let { add(it) }
                    }
                }
                if (metadataParts.isNotEmpty()) {
                    Row(
                        modifier = Modifier.padding(top = 8.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        metadataParts.forEachIndexed { index, part ->
                            if (index > 0) {
                                Text(text = "•", color = Color.White.copy(alpha = 0.5f), fontSize = 13.sp, fontWeight = FontWeight.Bold)
                            }
                            Text(text = part, color = Color.White.copy(alpha = 0.85f), fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                        }
                    }
                }
                val description = item.description
                if (!description.isNullOrBlank()) {
                    val shortenedDescription = remember(description) { shortenHeroSynopsis(description) }
                    Text(
                        text = shortenedDescription,
                        color = Color.White.copy(alpha = 0.8f),
                        fontSize = 15.sp,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(top = 10.dp).widthIn(max = 620.dp)
                    )
                }
            }
        }
    }
}
