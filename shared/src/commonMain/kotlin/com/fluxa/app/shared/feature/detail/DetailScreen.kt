package com.fluxa.app.shared.feature.detail

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun DetailScreen(
    state: DetailUiState,
    language: String?,
    onAction: (DetailAction) -> Unit,
    modifier: Modifier = Modifier
) {
    val content = state.content
    when {
        state.isLoading -> DetailLoading(modifier)
        content == null -> DetailEmpty(
            text = AppStrings.t(language, state.errorKey ?: "auto.no_results_found"),
            modifier = modifier
        )
        else -> DetailContent(
            content = content,
            language = language,
            onAction = onAction,
            modifier = modifier
        )
    }
}

@Composable
private fun DetailContent(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    modifier: Modifier
) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaColors.background)
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(280.dp)
                .background(FluxaColors.surface)
        ) {
            FluxaRemoteImage(
                imageUrl = content.backgroundUrl ?: content.posterUrl,
                cacheKey = "detail-artwork:${content.id}",
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
        }
        Column(
            modifier = Modifier.padding(horizontal = 20.dp, vertical = 16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(text = content.title, color = Color.White, fontWeight = FontWeight.Bold)
            if (content.releaseLabel.isNotBlank() || content.ratingLabel.isNotBlank()) {
                Text(
                    text = listOf(content.releaseLabel, content.ratingLabel)
                        .filter { it.isNotBlank() }
                        .joinToString(AppStrings.t(language, "auto.detail_metadata_separator")),
                    color = Color.White.copy(alpha = 0.7f)
                )
            }
            if (content.description.isNotBlank()) {
                Text(
                    text = content.description,
                    color = Color.White.copy(alpha = 0.86f),
                    maxLines = 5,
                    overflow = TextOverflow.Ellipsis
                )
            }
            Button(onClick = { onAction(DetailAction.Play) }) {
                Text(AppStrings.t(language, "common.play"))
            }
            Button(onClick = { onAction(DetailAction.ToggleWatchlist) }) {
                Text(AppStrings.t(language, if (content.isInWatchlist) "auto.in_list" else "auto.my_list"))
            }
        }
        if (content.relatedItems.isNotEmpty()) {
            Text(
                text = AppStrings.t(language, "auto.similar_titles"),
                color = Color.White,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(horizontal = 20.dp)
            )
            RelatedItems(items = content.relatedItems, onAction = onAction)
        }
    }
}

@Composable
private fun RelatedItems(items: List<CatalogItemUiModel>, onAction: (DetailAction) -> Unit) {
    LazyRow(
        contentPadding = PaddingValues(horizontal = 20.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        items(items, key = { it.id }) { item ->
            CatalogCard(model = item.card, onClick = { onAction(DetailAction.RelatedItemSelected(item)) })
        }
    }
}

@Composable
private fun DetailLoading(modifier: Modifier) {
    Box(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaColors.background),
        contentAlignment = Alignment.Center
    ) {
        CircularProgressIndicator(color = Color.White)
    }
}

@Composable
private fun DetailEmpty(text: String, modifier: Modifier) {
    Box(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaColors.background),
        contentAlignment = Alignment.Center
    ) {
        Text(text = text, color = Color.White)
    }
}
