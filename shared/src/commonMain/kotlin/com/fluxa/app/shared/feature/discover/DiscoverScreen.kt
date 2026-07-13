package com.fluxa.app.shared.feature.discover

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun DiscoverScreen(
    state: DiscoverUiState,
    language: String?,
    onFiltersChanged: (DiscoverFiltersUiModel) -> Unit,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaColors.background)
            .padding(horizontal = 20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = AppStrings.t(language, "nav.discover"),
            color = Color.White,
            fontWeight = FontWeight.Bold
        )
        DiscoverFilters(
            filters = state.filters,
            catalogOptions = state.catalogOptions,
            genreOptions = state.genreOptions,
            language = language,
            onFiltersChanged = onFiltersChanged
        )
        when {
            state.isLoading && state.results.isEmpty() -> Box(
                modifier = Modifier.weight(1f).fillMaxWidth(),
                contentAlignment = Alignment.Center
            ) { CircularProgressIndicator(color = Color.White) }
            state.results.isEmpty() -> Box(
                modifier = Modifier.weight(1f).fillMaxWidth(),
                contentAlignment = Alignment.Center
            ) { Text(AppStrings.t(language, "auto.no_results_yet"), color = Color.White) }
            else -> LazyVerticalGrid(
                columns = GridCells.Adaptive(minSize = 128.dp),
                modifier = Modifier.weight(1f).fillMaxWidth(),
                contentPadding = PaddingValues(bottom = 20.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                items(state.results, key = { "${it.type}:${it.id}" }) { item ->
                    CatalogCard(model = item.card, onClick = { onItemSelected(item) })
                }
            }
        }
    }
}

@Composable
private fun DiscoverFilters(
    filters: DiscoverFiltersUiModel,
    catalogOptions: List<DiscoverFilterOptionUiModel>,
    genreOptions: List<DiscoverFilterOptionUiModel>,
    language: String?,
    onFiltersChanged: (DiscoverFiltersUiModel) -> Unit
) {
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        DiscoverFilterRow(
            options = listOf(
                DiscoverFilterOptionUiModel("movie", AppStrings.t(language, "auto.movie")),
                DiscoverFilterOptionUiModel("series", AppStrings.t(language, "auto.series"))
            ),
            selectedId = filters.contentType,
            onSelected = { value ->
                onFiltersChanged(DiscoverFiltersUiModel(contentType = value.orEmpty()))
            }
        )
        if (catalogOptions.isNotEmpty()) {
            DiscoverFilterRow(
                options = catalogOptions,
                selectedId = filters.catalogKey,
                onSelected = { value ->
                    onFiltersChanged(filters.copy(catalogKey = value, genre = null))
                }
            )
        }
        if (genreOptions.isNotEmpty()) {
            DiscoverFilterRow(
                options = genreOptions,
                selectedId = filters.genre,
                onSelected = { value ->
                    onFiltersChanged(filters.copy(genre = value))
                }
            )
        }
    }
}

@Composable
private fun DiscoverFilterRow(
    options: List<DiscoverFilterOptionUiModel>,
    selectedId: String?,
    onSelected: (String?) -> Unit
) {
    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        options.forEach { option ->
            val selected = option.id == selectedId
            Text(
                text = option.label,
                color = if (selected) Color.White else Color.White.copy(alpha = 0.6f),
                fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                modifier = Modifier.clickable { onSelected(option.id) }
            )
        }
    }
}
