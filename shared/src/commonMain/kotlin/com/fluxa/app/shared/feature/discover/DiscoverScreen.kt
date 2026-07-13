package com.fluxa.app.shared.feature.discover

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.shared.skeletonShimmer
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
        DiscoverSearchField(
            query = searchQuery,
            placeholder = AppStrings.t(language, "auto.search"),
            onQueryChanged = onSearchQueryChanged
        )
        if (searchQuery.isNotBlank()) {
            when {
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
                isSearching -> DiscoverSkeletonGrid(modifier = Modifier.weight(1f))
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
private fun DiscoverSearchField(
    query: String,
    placeholder: String,
    onQueryChanged: (String) -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(52.dp)
            .background(Color.White.copy(alpha = 0.06f), RoundedCornerShape(26.dp)),
        contentAlignment = Alignment.CenterStart
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(text = "⌕", color = Color.White.copy(alpha = 0.5f), fontSize = 18.sp)
            Spacer(modifier = Modifier.width(10.dp))
            Box(modifier = Modifier.fillMaxWidth()) {
                if (query.isEmpty()) {
                    Text(text = placeholder, color = Color.White.copy(alpha = 0.4f), fontSize = 15.sp)
                }
                BasicTextField(
                    value = query,
                    onValueChange = onQueryChanged,
                    textStyle = TextStyle(color = Color.White, fontSize = 15.sp, fontWeight = FontWeight.Medium),
                    singleLine = true,
                    cursorBrush = androidx.compose.ui.graphics.SolidColor(Color.White),
                    modifier = Modifier.fillMaxWidth()
                )
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
                    .skeletonShimmer()
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
        Row(
            modifier = Modifier
                .background(Color.White.copy(alpha = 0.08f), RoundedCornerShape(999.dp))
                .clickable { expanded = true }
                .padding(horizontal = 14.dp, vertical = 9.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = selectedLabel,
                color = Color.White,
                fontWeight = FontWeight.Medium,
                fontSize = 14.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            Spacer(modifier = Modifier.width(4.dp))
            Text(text = "⌄", color = Color.White.copy(alpha = 0.6f), fontSize = 12.sp)
        }
        DropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            modifier = Modifier.background(FluxaColors.surfaceRaised, RoundedCornerShape(12.dp))
        ) {
            options.forEach { option ->
                val selected = option.id == selectedId
                DropdownMenuItem(
                    text = {
                        Text(
                            text = option.label,
                            color = if (selected) FluxaColors.accent else Color.White,
                            fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal
                        )
                    },
                    onClick = {
                        onSelected(option.id)
                        expanded = false
                    }
                )
            }
        }
    }
}
