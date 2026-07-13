package com.fluxa.app.shared.feature.search

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.OutlinedTextField
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
fun SearchScreen(
    state: SearchUiState,
    language: String?,
    onQueryChanged: (String) -> Unit,
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
        OutlinedTextField(
            value = state.query,
            onValueChange = onQueryChanged,
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            label = { Text(AppStrings.t(language, "auto.search")) }
        )
        when {
            state.isLoading -> SearchLoading(modifier = Modifier.weight(1f))
            state.results.isNotEmpty() -> SearchResults(
                items = state.results,
                onItemSelected = onItemSelected,
                modifier = Modifier.weight(1f)
            )
            state.query.isNotBlank() -> SearchEmpty(
                text = AppStrings.t(language, "auto.no_results_found"),
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
        items(items, key = { it.id }) { item ->
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
