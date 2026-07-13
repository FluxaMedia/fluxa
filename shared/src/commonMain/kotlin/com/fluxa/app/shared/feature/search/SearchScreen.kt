package com.fluxa.app.shared.feature.search

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.grid.items as gridItems
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.text.KeyboardOptions
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun SearchScreen(
    state: SearchUiState,
    language: String?,
    onQueryChanged: (String) -> Unit,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    onClearHistory: () -> Unit = {},
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
            value = state.query,
            onValueChange = onQueryChanged,
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
            label = { Text(AppStrings.t(language, "auto.search")) }
        )
        when {
            state.isLoading -> SearchLoading(modifier = Modifier.weight(1f))
            state.resultRows.isNotEmpty() -> SearchResultRows(
                rows = state.resultRows,
                onItemSelected = onItemSelected,
                modifier = Modifier.weight(1f)
            )
            state.results.isNotEmpty() -> SearchResults(
                items = state.results,
                onItemSelected = onItemSelected,
                modifier = Modifier.weight(1f)
            )
            state.query.isNotBlank() -> SearchNoResultsForQuery(
                query = state.query,
                language = language,
                recentItems = state.recentItems,
                onItemSelected = onItemSelected,
                modifier = Modifier.weight(1f)
            )
            state.recentItems.isNotEmpty() -> SearchHistory(
                title = AppStrings.t(language, "auto.recent_searches"),
                clearLabel = AppStrings.t(language, "auto.clear_search"),
                items = state.recentItems,
                onItemSelected = onItemSelected,
                onClearHistory = onClearHistory,
                modifier = Modifier.weight(1f)
            )
            else -> SearchEmpty(
                text = AppStrings.t(language, "auto.start_searching"),
                modifier = Modifier.weight(1f)
            )
        }
    }
}

@Composable
private fun SearchNoResultsForQuery(
    query: String,
    language: String?,
    recentItems: List<CatalogItemUiModel>,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    modifier: Modifier
) {
    Column(modifier = modifier.fillMaxSize()) {
        Text(
            text = AppStrings.format(language, "auto.no_results_found_for", query),
            color = Color.White,
            fontWeight = FontWeight.Medium,
            modifier = Modifier.padding(vertical = 12.dp)
        )
        if (recentItems.isNotEmpty()) {
            SearchResults(
                items = recentItems.take(5),
                onItemSelected = onItemSelected,
                modifier = Modifier.fillMaxSize()
            )
        }
    }
}

@Composable
private fun SearchHistory(
    title: String,
    clearLabel: String,
    items: List<CatalogItemUiModel>,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    onClearHistory: () -> Unit,
    modifier: Modifier
) {
    Column(modifier = modifier) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(bottom = 12.dp),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(text = title, color = Color.White, fontWeight = FontWeight.Bold)
            Text(
                text = clearLabel,
                color = Color.White.copy(alpha = 0.7f),
                modifier = Modifier.clickable { onClearHistory() }
            )
        }
        SearchResults(
            items = items,
            onItemSelected = onItemSelected,
            modifier = Modifier.fillMaxSize()
        )
    }
}

@Composable
private fun SearchResultRows(
    rows: List<CatalogRowUiModel>,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    modifier: Modifier
) {
    LazyColumn(
        modifier = modifier.fillMaxWidth(),
        contentPadding = PaddingValues(bottom = 20.dp),
        verticalArrangement = Arrangement.spacedBy(20.dp)
    ) {
        items(rows, key = { it.id }) { row ->
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    text = row.title,
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis
                )
                LazyRow(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    items(row.items, key = { it.id }) { item ->
                        CatalogCard(model = item.card, onClick = { onItemSelected(item) })
                    }
                }
            }
        }
    }
}

@Composable
private fun SearchResults(
    items: List<CatalogItemUiModel>,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    modifier: Modifier
) {
    LazyVerticalGrid(
        columns = GridCells.Adaptive(minSize = 128.dp),
        modifier = modifier.fillMaxWidth(),
        contentPadding = PaddingValues(bottom = 20.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        gridItems(items, key = { it.id }) { item ->
            CatalogCard(model = item.card, onClick = { onItemSelected(item) })
        }
    }
}

@Composable
private fun SearchLoading(modifier: Modifier) {
    androidx.compose.foundation.layout.Box(
        modifier = modifier.fillMaxWidth(),
        contentAlignment = Alignment.Center
    ) {
        CircularProgressIndicator(color = Color.White)
    }
}

@Composable
private fun SearchEmpty(text: String, modifier: Modifier) {
    androidx.compose.foundation.layout.Box(
        modifier = modifier.fillMaxWidth(),
        contentAlignment = Alignment.Center
    ) {
        Text(text = text, color = Color.White, fontWeight = FontWeight.Medium)
    }
}
