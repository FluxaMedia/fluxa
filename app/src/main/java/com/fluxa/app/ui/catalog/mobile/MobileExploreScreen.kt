@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.itemsIndexed as mobileGridItemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage

@Composable
fun MobileExploreScreen(
    activeProfile: UserProfile?,
    lang: String,
    results: List<Meta>,
    resultSources: Map<String, HomeCatalogSource>,
    searchResults: List<Meta>,
    searchRows: List<SearchResultRow>,
    isDiscoverLoading: Boolean,
    catalogOptions: List<Pair<String?, String>>,
    genreOptions: List<Pair<String?, String>>,
    selectedType: String,
    selectedCatalog: String?,
    selectedGenre: String?,
    onSelectType: (String) -> Unit,
    onSelectCatalog: (String?) -> Unit,
    onSelectGenre: (String?) -> Unit,
    onMovieClick: (Meta, String?, String?) -> Unit,
    onBack: () -> Unit,
    viewModel: HomeViewModel
) {
    var searchQuery by remember { mutableStateOf("") }
    val focusManager = LocalFocusManager.current
    val trimmedSearchQuery = searchQuery.trim()
    val isSearchMode = trimmedSearchQuery.length >= 2
    val displayedResults = remember(results, searchResults, trimmedSearchQuery) {
        val rawResults = if (isSearchMode) {
            searchResults
                .ifEmpty {
                    results.filter { meta ->
                        meta.name.contains(trimmedSearchQuery, ignoreCase = true) ||
                            meta.originalName?.contains(trimmedSearchQuery, ignoreCase = true) == true
                    }
                }
                .distinctBy { it.id }
        } else {
            results
        }
        rawResults
    }
    val displayedSearchRows = remember(searchRows, trimmedSearchQuery) {
        if (!isSearchMode) {
            emptyList()
        } else {
            searchRows.filter { it.items.isNotEmpty() }
        }
    }

    LazyVerticalGrid(
        columns = GridCells.Fixed(3),
        modifier = Modifier.fillMaxSize().background(Color(0xFF040508)),
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 18.dp, bottom = 120.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp)
    ) {
        item(span = { GridItemSpan(maxLineSpan) }) {
            Column(modifier = Modifier.fillMaxWidth()) {
                Text(
                    text = AppStrings.t(lang, "nav.discover"),
                    color = Color.White,
                    fontSize = 22.sp,
                    fontWeight = FontWeight.Black,
                    modifier = Modifier.padding(start = 16.dp, end = 16.dp, bottom = 12.dp)
                )
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(48.dp)
                            .clip(RoundedCornerShape(24.dp))
                            .background(Color.White.copy(alpha = 0.08f))
                            .border(1.dp, Color.White.copy(alpha = 0.05f), RoundedCornerShape(24.dp))
                            .padding(horizontal = 16.dp),
                        contentAlignment = Alignment.CenterStart
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(FluxaIcons.Search, null, tint = Color.White.copy(alpha = 0.4f), modifier = Modifier.size(20.dp))
                            Spacer(Modifier.width(10.dp))
                            androidx.compose.foundation.text.BasicTextField(
                                value = searchQuery,
                                onValueChange = {
                                    searchQuery = it
                                    viewModel.search(it)
                                },
                                singleLine = true,
                                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                                keyboardActions = KeyboardActions(onSearch = { focusManager.clearFocus() }),
                                textStyle = androidx.compose.ui.text.TextStyle(color = Color.White, fontSize = 15.sp),
                                cursorBrush = androidx.compose.ui.graphics.SolidColor(Color(activeProfile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt())),
                                modifier = Modifier.fillMaxWidth(),
                                decorationBox = { innerTextField ->
                                    if (searchQuery.isEmpty()) {
                                        Text(
                                            text = AppStrings.t(lang, "auto.explore_68dd9e4b"),
                                            color = Color.White.copy(alpha = 0.3f),
                                            fontSize = 15.sp
                                        )
                                    }
                                    innerTextField()
                                }
                            )
                        }
                    }
                }

                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp),
                    horizontalArrangement = Arrangement.spacedBy(10.dp)
                ) {
                    val exploreAccent = Color(activeProfile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt())
                    Box(Modifier.weight(1f)) {
                        ExploreDropdownFilter(
                            title = AppStrings.t(lang, "auto.content_type"),
                            options = exploreTypeOptions(lang),
                            selected = selectedType,
                            onSelect = { it?.let(onSelectType) },
                            showTitle = false,
                            accentColor = exploreAccent
                        )
                    }
                    Box(Modifier.weight(1f)) {
                        ExploreDropdownFilter(
                            title = AppStrings.t(lang, "explore.catalog"),
                            options = catalogOptions,
                            selected = selectedCatalog,
                            onSelect = onSelectCatalog,
                            showTitle = false,
                            accentColor = exploreAccent
                        )
                    }
                    Box(Modifier.weight(1f)) {
                        ExploreDropdownFilter(
                            title = AppStrings.t(lang, "auto.genre_78cde1de"),
                            options = genreOptions,
                            selected = selectedGenre,
                            onSelect = onSelectGenre,
                            showTitle = false,
                            accentColor = exploreAccent
                        )
                    }
                }

                if (!isSearchMode) {
                    Text(
                        text = AppStrings.t(lang, "auto.results"),
                        color = Color.White,
                        fontSize = 20.sp,
                        fontWeight = FontWeight.Black,
                        modifier = Modifier.padding(start = 16.dp, top = 24.dp, bottom = 12.dp)
                    )
                } else {
                    Spacer(Modifier.height(12.dp))
                }
            }
        }

        if (!isSearchMode && displayedResults.isEmpty() && isDiscoverLoading) {
            item(span = { GridItemSpan(maxLineSpan) }) {
                Box(
                    modifier = Modifier.fillMaxWidth().padding(top = 48.dp),
                    contentAlignment = Alignment.Center
                ) {
                    androidx.compose.material3.CircularProgressIndicator(color = Color.White)
                }
            }
        } else if (isSearchMode && displayedSearchRows.isNotEmpty()) {
            displayedSearchRows.forEachIndexed { index, row ->
                item(
                    key = "search-row:${row.title}:$index",
                    span = { GridItemSpan(maxLineSpan) }
                ) {
                    MobileBrowseCategoryRow(
                        title = row.title,
                        items = row.items,
                        onMovieClick = { meta -> onMovieClick(meta, row.sourceAddonTransportUrl, row.sourceAddonCatalogType) },
                        onSearchHistory = viewModel::addToSearchHistory,
                        cardLayout = if (activeProfile?.safePosterLandscapeMode == true) "horizontal" else activeProfile?.safeCardLayout ?: "vertical",
                        activeProfile = activeProfile
                    )
                }
            }
        } else if (displayedResults.isEmpty()) {
            item(span = { GridItemSpan(maxLineSpan) }) {
                SearchEmptyState(
                    title = if (isSearchMode) {
                        AppStrings.t(lang, "auto.no_results_found")
                    } else {
                        AppStrings.t(lang, "auto.no_results_yet")
                    },
                    subtitle = if (isSearchMode) {
                        AppStrings.t(lang, "auto.try_a_shorter_or_slightly_different_title")
                    } else {
                        AppStrings.t(lang, "auto.try_different_filters_to_discover_something__1ba561eb")
                    }
                )
            }
        } else {
            mobileGridItemsIndexed(
                items = displayedResults,
                key = { _, movie -> "${movie.type}:${movie.id}" }
            ) { _, movie ->
                val source = discoverSourceFor(movie, resultSources)
                MobileExplorePosterCard(
                    movie = movie,
                    onClick = { onMovieClick(movie, source?.transportUrl, source?.type) }
                )
            }
        }
    }
}

@Composable
private fun MobileExplorePosterCard(
    movie: Meta,
    onClick: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
    ) {
        AsyncImage(
            model = movie.poster,
            contentDescription = movie.name,
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(2f / 3f)
                .clip(RoundedCornerShape(10.dp))
                .background(Color.White.copy(alpha = 0.06f)),
            contentScale = ContentScale.Crop
        )
        Spacer(Modifier.height(6.dp))
        Text(
            text = movie.name,
            color = Color.White,
            fontSize = 14.sp,
            fontWeight = FontWeight.Black,
            lineHeight = 16.sp,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis
        )
    }
}
