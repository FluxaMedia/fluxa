@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)

package com.fluxa.app.shared.feature.discover

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.ui.focus.focusRestorer
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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items as columnItems
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.focusGroup
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.AlertDialog
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.foundation.border
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogRowUiModel
import com.fluxa.app.shared.feature.catalog.stableLazyKey
import com.fluxa.app.shared.skeletonShimmer
import com.fluxa.app.shared.feature.search.SearchResultRows
import com.fluxa.app.shared.feature.search.SearchResults
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.LocalDeviceType

@Composable
fun DiscoverScreen(
    state: DiscoverUiState,
    language: String?,
    onFiltersChanged: (DiscoverFiltersUiModel) -> Unit,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    onLoadMore: () -> Unit = {},
    searchQuery: String = "",
    onSearchQueryChanged: (String) -> Unit = {},
    searchResultRows: List<CatalogRowUiModel> = emptyList(),
    searchResults: List<CatalogItemUiModel> = emptyList(),
    isSearching: Boolean = false,
    modifier: Modifier = Modifier
) {
    val isTv = LocalDeviceType.current == DeviceType.TV
    LaunchedEffect(state.catalogOptions, state.filters.contentType) {
        if (state.filters.catalogKey == null && state.catalogOptions.isNotEmpty()) {
            onFiltersChanged(state.filters.copy(catalogKey = state.catalogOptions.first().id))
        }
    }

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
                typeOptions = state.typeOptions,
                catalogOptions = state.catalogOptions,
                genreOptions = state.genreOptions,
                language = language,
                onFiltersChanged = onFiltersChanged,
                isTv = isTv
            )
            when {
                state.isLoading && state.results.isEmpty() -> DiscoverSkeletonGrid(modifier = Modifier.weight(1f))
                state.results.isEmpty() -> Box(
                    modifier = Modifier.weight(1f).fillMaxWidth(),
                    contentAlignment = Alignment.Center
                ) { Text(AppStrings.t(language, "auto.no_results_yet"), color = Color.White) }
                else -> {
                    val gridState = rememberLazyGridState()
                    val shouldLoadMore by remember {
                        derivedStateOf {
                            val lastVisible = gridState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0
                            val totalItems = gridState.layoutInfo.totalItemsCount
                            totalItems > 0 && lastVisible >= totalItems - 6
                        }
                    }
                    LaunchedEffect(shouldLoadMore, state.isLoading) {
                        if (shouldLoadMore && !state.isLoading) {
                            onLoadMore()
                        }
                    }
                    LazyVerticalGrid(
                        state = gridState,
                        columns = com.fluxa.app.ui.catalog.rememberCatalogGridCells(),
                        modifier = Modifier.weight(1f).fillMaxWidth().focusRestorer(),
                        contentPadding = PaddingValues(top = 12.dp, bottom = 120.dp),
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        items(state.results, key = { it.stableLazyKey() }, contentType = { "catalog-card" }) { item ->
                            CatalogCard(model = item.card, onClick = { onItemSelected(item) })
                        }
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
            Icon(
                imageVector = Icons.Filled.Search,
                contentDescription = null,
                tint = Color.White.copy(alpha = 0.5f),
                modifier = Modifier.width(20.dp)
            )
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
        columns = com.fluxa.app.ui.catalog.rememberCatalogGridCells(),
        modifier = modifier.fillMaxWidth(),
        contentPadding = PaddingValues(bottom = 120.dp),
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
    typeOptions: List<DiscoverFilterOptionUiModel>,
    catalogOptions: List<DiscoverFilterOptionUiModel>,
    genreOptions: List<DiscoverFilterOptionUiModel>,
    language: String?,
    onFiltersChanged: (DiscoverFiltersUiModel) -> Unit,
    isTv: Boolean
) {
    val effectiveTypeOptions = typeOptions.ifEmpty {
        listOf(
            DiscoverFilterOptionUiModel("movie", AppStrings.t(language, "auto.movie")),
            DiscoverFilterOptionUiModel("series", AppStrings.t(language, "auto.series"))
        )
    }
    Row(
        modifier = Modifier.horizontalScroll(rememberScrollState()),
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        DiscoverDropdownFilter(
            label = AppStrings.t(language, "auto.type"),
            options = effectiveTypeOptions,
            selectedId = filters.contentType,
            isTv = isTv,
            onSelected = { value ->
                onFiltersChanged(DiscoverFiltersUiModel(contentType = value.orEmpty()))
            }
        )
        if (catalogOptions.isNotEmpty()) {
            DiscoverDropdownFilter(
                label = AppStrings.t(language, "auto.catalog"),
                options = catalogOptions,
                selectedId = filters.catalogKey,
                isTv = isTv,
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
                isTv = isTv,
                onSelected = { value ->
                    onFiltersChanged(filters.copy(genre = value))
                }
            )
        }
    }
}

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
internal fun DiscoverDropdownFilter(
    label: String,
    options: List<DiscoverFilterOptionUiModel>,
    selectedId: String?,
    onSelected: (String?) -> Unit,
    isTv: Boolean = false
) {
    var showSheet by remember { mutableStateOf(false) }
    var triggerFocused by remember { mutableStateOf(false) }
    val normalizedSelectedId = selectedId?.trim()?.lowercase()
    val selectedLabel = options.firstOrNull { it.id?.trim()?.lowercase() == normalizedSelectedId }?.label ?: label
    Row(
        modifier = Modifier
            .clip(RoundedCornerShape(999.dp))
            .onFocusChanged { triggerFocused = it.isFocused }
            .background(if (triggerFocused) Color.White else Color.White.copy(alpha = 0.08f), RoundedCornerShape(999.dp))
            .clickable { showSheet = true }
            .padding(horizontal = 14.dp, vertical = 9.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = selectedLabel,
            color = if (triggerFocused) Color.Black else Color.White,
            fontWeight = FontWeight.Medium,
            fontSize = 14.sp,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.widthIn(max = 140.dp)
        )
        Spacer(modifier = Modifier.width(2.dp))
        Icon(
            imageVector = Icons.Filled.KeyboardArrowDown,
            contentDescription = null,
            tint = if (triggerFocused) Color.Black else Color.White.copy(alpha = 0.6f),
            modifier = Modifier.width(18.dp)
        )
    }

    if (showSheet && isTv) {
        val selectedRowFocusRequester = remember { FocusRequester() }
        Dialog(
            onDismissRequest = { showSheet = false },
        ) {
            Column(
                modifier = Modifier
                    .widthIn(min = 360.dp, max = 560.dp)
                    .focusGroup()
                    .clip(RoundedCornerShape(22.dp))
                    .background(FluxaColors.surfaceRaised)
                    .padding(horizontal = 24.dp, vertical = 20.dp)
            ) {
                LaunchedEffect(Unit) {
                    runCatching { selectedRowFocusRequester.requestFocus() }
                }
                Text(
                    text = label,
                    color = Color.White,
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 24.sp,
                    modifier = Modifier.padding(bottom = 12.dp)
                )
                options.forEach { option ->
                    val selected = option.id?.trim()?.lowercase() == normalizedSelectedId
                    val optionId = option.id?.trim()?.lowercase()
                    var rowFocused by remember { mutableStateOf(false) }
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .then(if (selected) Modifier.focusRequester(selectedRowFocusRequester) else Modifier)
                            .onFocusChanged { rowFocused = it.isFocused }
                            .clip(RoundedCornerShape(10.dp))
                            .background(
                                when {
                                    rowFocused -> Color.White.copy(alpha = 0.16f)
                                    else -> Color.Transparent
                                }
                            )
                            .then(
                                if (rowFocused) Modifier.border(2.dp, Color.White, RoundedCornerShape(10.dp))
                                else Modifier
                            )
                            .clickable {
                                optionId?.let(onSelected)
                                showSheet = false
                            }
                            .padding(horizontal = 16.dp, vertical = 14.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = option.label,
                            color = Color.White,
                            fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal
                        )
                        if (selected) Icon(Icons.Filled.Check, contentDescription = null, tint = Color.White)
                    }
                }
            }
        }
    }

    if (showSheet && !isTv) {
        val sheetState = androidx.compose.material3.rememberModalBottomSheetState(skipPartiallyExpanded = true)
        val selectedRowFocusRequester = remember { FocusRequester() }
        LaunchedEffect(showSheet) {
            if (showSheet) runCatching { selectedRowFocusRequester.requestFocus() }
        }
        androidx.compose.material3.ModalBottomSheet(
            onDismissRequest = { showSheet = false },
            sheetState = sheetState,
            containerColor = FluxaColors.surfaceRaised
        ) {
            Text(
                text = label,
                color = Color.White.copy(alpha = 0.6f),
                fontWeight = FontWeight.SemiBold,
                modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp)
            )
            LazyColumn(
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(max = 420.dp),
                contentPadding = PaddingValues(bottom = 24.dp)
            ) {
                columnItems(options, key = { it.id ?: it.label }) { option ->
                    val selected = option.id?.trim()?.lowercase() == normalizedSelectedId
                    var rowFocused by remember { mutableStateOf(false) }
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .let { if (selected) it.focusRequester(selectedRowFocusRequester) else it }
                            .onFocusChanged { rowFocused = it.isFocused }
                            .background(if (rowFocused) Color.White.copy(alpha = 0.12f) else Color.Transparent)
                            .clickable {
                                onSelected(option.id)
                                showSheet = false
                            }
                            .padding(horizontal = 20.dp, vertical = 14.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = option.label,
                            color = if (selected) FluxaColors.accent else Color.White,
                            fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal
                        )
                        if (selected) {
                            Icon(
                                imageVector = Icons.Filled.Check,
                                contentDescription = null,
                                tint = FluxaColors.accent
                            )
                        }
                    }
                }
            }
        }
    }
}
