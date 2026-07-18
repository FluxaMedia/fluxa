package com.fluxa.app.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.image.FluxaRemoteImage

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
fun PosterActionSheet(
    item: CatalogItemUiModel,
    language: String?,
    onDismiss: () -> Unit,
    onAddToLibrary: () -> Unit
) {
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = FluxaColors.surfaceRaised
    ) {
        Column(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp).padding(bottom = 28.dp),
            verticalArrangement = Arrangement.spacedBy(18.dp)
        ) {
            Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                FluxaRemoteImage(
                    imageUrl = item.card.artworkUrl,
                    cacheKey = "poster-sheet:${item.card.artworkUrl}",
                    contentDescription = null,
                    modifier = Modifier.width(92.dp).height(138.dp).clip(RoundedCornerShape(10.dp)),
                    contentScale = ContentScale.Crop
                )
                Column(verticalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.weight(1f)) {
                    Text(item.card.title, color = Color.White, fontSize = 20.sp, fontWeight = FontWeight.Bold, maxLines = 2, overflow = TextOverflow.Ellipsis)
                    item.card.subtitle.takeIf { it.isNotBlank() }?.let {
                        Text(it, color = Color.White.copy(alpha = 0.65f), fontSize = 13.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    }
                    val metadata = listOfNotNull(
                        item.ageRating?.takeIf { it.isNotBlank() },
                        item.runtimeLabel?.takeIf { it.isNotBlank() },
                        item.seasonsCount?.takeIf { it > 0 }?.let { "$it ${AppStrings.t(language, "auto.seasons")}" }
                    ).joinToString(" • ")
                    if (metadata.isNotBlank()) {
                        Text(metadata, color = Color.White.copy(alpha = 0.72f), fontSize = 13.sp)
                    }
                    item.description?.takeIf { it.isNotBlank() }?.let {
                        Text(it, color = Color.White.copy(alpha = 0.78f), fontSize = 13.sp, lineHeight = 17.sp, maxLines = 4, overflow = TextOverflow.Ellipsis)
                    }
                }
            }
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(50.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(Color.White)
                    .clickable(onClick = onAddToLibrary),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(Icons.Filled.Add, contentDescription = null, tint = Color.Black)
                Text(
                    AppStrings.t(language, "common.add_to_library"),
                    color = Color.Black,
                    fontWeight = FontWeight.Bold,
                    modifier = Modifier.padding(start = 8.dp)
                )
            }
        }
    }
}
