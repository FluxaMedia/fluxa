package com.fluxa.app.shared.feature.discover

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.shared.feature.search.SearchResultRows
import com.fluxa.app.shared.feature.search.SearchResults
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun DiscoverScreen(
    state: DiscoverUiState,
    language: String?,
    onFiltersChanged: (DiscoverFiltersUiModel) -> Unit,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    searchQuery: String = "",
    onSearchQueryChanged: (String) -> Unit = {},
    searchResultRows: List<CatalogRowUiModel> = emptyList(),
    searchResults: List<CatalogItemUiModel> = emptyList(),
    isSearching: Boolean = false,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaColors.background)
            .padding(horizontal = 20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        OutlinedTextField(
            value = searchQuery,
            onValueChange = onSearchQueryChanged,
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            label = { Text(AppStrings.t(language, "auto.search")) }
        )
        if (searchQuery.isNotBlank()) {
            when {
                isSearching -> Box(
                    modifier = Modifier.weight(1f).fillMaxWidth(),
                    contentAlignment = Alignment.Center
                ) { CircularProgressIndicator(color = Color.White) }
                searchResultRows.isNotEmpty() -> SearchResultRows(
                    rows = searchResultRows,
                    onItemSelected = onItemSelected,
                    modifier = Modifier.weight(1f)
                )
                searchResults.isNotEmpty() -> SearchResults(
                    items = searchResults,
                    onItemSelected = onItemSelected,
                    modifier = Modifier.weight(1f)
                )
                else -> Box(
                    modifier = Modifier.weight(1f).fillMaxWidth(),
                    contentAlignment = Alignment.Center
                ) { Text(AppStrings.t(language, "auto.no_results_found"), color = Color.White) }
            }
        } else {
            DiscoverFilters(
                filters = state.filters,
                catalogOptions = state.catalogOptions,
                genreOptions = state.genreOptions,
                language = language,
                onFiltersChanged = onFiltersChanged
            )
            when {
                state.isLoading && state.results.isEmpty() -> DiscoverSkeletonGrid(modifier = Modifier.weight(1f))
                state.results.isEmpty() -> Box(
                    modifier = Modifier.weight(1f).fillMaxWidth(),
                    contentAlignment = Alignment.Center
                ) { Text(AppStrings.t(language, "auto.no_results_yet"), color = Color.White) }
                else -> LazyVerticalGrid(
                    columns = GridCells.Fixed(3),
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
}

@Composable
private fun DiscoverSkeletonGrid(modifier: Modifier = Modifier) {
    LazyVerticalGrid(
        columns = GridCells.Fixed(3),
        modifier = modifier.fillMaxWidth(),
        contentPadding = PaddingValues(bottom = 20.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        items(18, key = { it }) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(2f / 3f)
                    .background(FluxaColors.surfaceCard, RoundedCornerShape(10.dp))
            )
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
    Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        DiscoverDropdownFilter(
            label = AppStrings.t(language, "auto.type"),
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
            DiscoverDropdownFilter(
                label = AppStrings.t(language, "auto.catalog"),
                options = catalogOptions,
                selectedId = filters.catalogKey,
                onSelected = { value ->
                    onFiltersChanged(filters.copy(catalogKey = value, genre = null))
                }
            )
        }
        if (genreOptions.isNotEmpty()) {
            DiscoverDropdownFilter(
                label = AppStrings.t(language, "auto.genre"),
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
private fun DiscoverDropdownFilter(
    label: String,
    options: List<DiscoverFilterOptionUiModel>,
    selectedId: String?,
    onSelected: (String?) -> Unit
) {
    var expanded by remember { mutableStateOf(false) }
    val selectedLabel = options.firstOrNull { it.id == selectedId }?.label ?: label
    Box {
        Box(
            modifier = Modifier
                .background(Color.White.copy(alpha = 0.12f), RoundedCornerShape(999.dp))
                .clickable { expanded = true }
                .padding(horizontal = 14.dp, vertical = 8.dp)
        ) {
            Text(
                text = selectedLabel,
                color = Color.White,
                fontWeight = FontWeight.Medium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            options.forEach { option ->
                DropdownMenuItem(
                    text = { Text(option.label) },
                    onClick = {
                        onSelected(option.id)
                        expanded = false
                    }
                )
            }
        }
    }
}
