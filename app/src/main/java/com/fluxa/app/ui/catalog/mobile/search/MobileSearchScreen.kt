@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.*
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.itemsIndexed
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.ExperimentalTvMaterial3Api
import androidx.tv.material3.Surface
import coil3.compose.AsyncImage

@Composable
fun MobileSearchScreen(
    activeProfile: UserProfile?,
    searchResults: List<Meta>,
    searchRows: List<SearchResultRow>,
    query: String,
    onSearch: (String) -> Unit,
    onBack: () -> Unit,
    onMovieClick: (Meta, String?, String?) -> Unit,
    focusRequester: FocusRequester,
    lang: String,
    viewModel: HomeViewModel
) {
    val searchHistory by viewModel.searchHistory.collectAsStateWithLifecycle()
    val discoverGenres by viewModel.discoverGenres.collectAsStateWithLifecycle()
    var isSearchFocused by remember { mutableStateOf(false) }
    var selectedDiscoverType by remember { mutableStateOf("all") }
    var selectedDiscoverGenre by remember { mutableStateOf<String?>(null) }
    var selectedDiscoverProvider by remember { mutableStateOf<String?>(null) }
    val showExplore = query.isEmpty() && !isSearchFocused
    val genreOptions = remember(discoverGenres) { discoverGenres.map { it.id to it.label } }

    LaunchedEffect(selectedDiscoverType, lang) {
        viewModel.loadDiscoverGenres(selectedDiscoverType)
    }

    LaunchedEffect(genreOptions, selectedDiscoverGenre) {
        if (selectedDiscoverGenre != null && genreOptions.none { it.first == selectedDiscoverGenre }) {
            selectedDiscoverGenre = null
        }
    }

    LaunchedEffect(query, selectedDiscoverType, selectedDiscoverGenre, selectedDiscoverProvider, lang) {
        if (query.isEmpty()) {
            viewModel.discover(
                type = selectedDiscoverType,
                catalogKey = null,
                genre = selectedDiscoverGenre,
                year = null,
                rating = null,
                provider = selectedDiscoverProvider,
                region = null
            )
        }
    }
    
    Column(modifier = Modifier.fillMaxSize().background(Color(0xFF0A0B10))) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 24.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            androidx.compose.material3.IconButton(
                onClick = onBack,
                modifier = Modifier.size(40.dp).background(Color.White.copy(alpha = 0.05f), CircleShape)
            ) {
                Icon(FluxaIcons.ArrowBack, null, tint = Color.White)
            }

            Box(
                modifier = Modifier
                    .weight(1f)
                    .height(56.dp)
                    .background(Color.White.copy(alpha = 0.06f), RoundedCornerShape(28.dp)),
                contentAlignment = Alignment.CenterStart
            ) {
                Row(modifier = Modifier.padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
                    Icon(FluxaIcons.Search, null, tint = Color.Gray, modifier = Modifier.size(20.dp))
                    Spacer(modifier = Modifier.width(12.dp))
                    BasicTextField(
                        value = query,
                        onValueChange = onSearch,
                        textStyle = TextStyle(color = Color.White, fontSize = 16.sp, fontWeight = FontWeight.Bold),
                        cursorBrush = SolidColor(Color.White),
                        modifier = Modifier
                            .weight(1f)
                            .focusRequester(focusRequester)
                            .onFocusChanged { isSearchFocused = it.isFocused },
                        decorationBox = { innerTextField ->
                            if (query.isEmpty()) {
                                Text(
                                    text = AppStrings.t(lang, "auto.search_movies_shows_cast"),
                                    color = Color.Gray,
                                    fontSize = 16.sp
                                )
                            }
                            innerTextField()
                        },
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search)
                    )
                }
            }
        }

        LazyVerticalGrid(
            columns = GridCells.Fixed(1),
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, bottom = 40.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            if (showExplore) {
                item {
                    Column {
                        SearchSectionTitle(AppStrings.t(lang, "auto.explore"))
                        val searchAccent = Color(activeProfile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt())
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(10.dp)
                        ) {
                            Box(Modifier.weight(1f)) {
                                ExploreDropdownFilter(
                                    title = AppStrings.t(lang, "auto.type"),
                                    options = listOf(
                                        "all" to (AppStrings.t(lang, "auto.all_24e2815b")),
                                        "movie" to (AppStrings.t(lang, "auto.movies")),
                                        "series" to (AppStrings.t(lang, "auto.series_02b3bdba"))
                                    ),
                                    selected = selectedDiscoverType,
                                    onSelect = { it?.let { selectedDiscoverType = it } },
                                    accentColor = searchAccent
                                )
                            }
                            Box(Modifier.weight(1f)) {
                                ExploreDropdownFilter(
                                    title = AppStrings.t(lang, "auto.genre"),
                                    options = genreOptions,
                                    selected = selectedDiscoverGenre,
                                    onSelect = { selectedDiscoverGenre = it },
                                    accentColor = searchAccent
                                )
                            }
                            Box(Modifier.weight(1f)) {
                                ExploreDropdownFilter(
                                    title = AppStrings.t(lang, "auto.platform"),
                                    options = listOf(
                                        null to (AppStrings.t(lang, "auto.all_24e2815b")),
                                        "8" to AppStrings.t(lang, "brand.netflix"),
                                        "9" to AppStrings.t(lang, "brand.prime_video"),
                                        "337" to AppStrings.t(lang, "brand.disney_plus"),
                                        "49" to AppStrings.t(lang, "brand.hbo"),
                                        "350" to AppStrings.t(lang, "brand.apple_tv_plus")
                                    ),
                                    selected = selectedDiscoverProvider,
                                    onSelect = { selectedDiscoverProvider = it },
                                    accentColor = searchAccent
                                )
                            }
                        }
                    }
                }
            }

            if (query.isEmpty()) {
                if (isSearchFocused && searchHistory.isNotEmpty()) {
                    item {
                        SearchSectionTitle(AppStrings.t(lang, "auto.recent_searches"))
                    }
                    itemsIndexed(searchHistory, key = { _, item -> "history:${item.type}:${item.id}" }) { _, recentItem ->
                        SearchMovieRow(recentItem, lang, modifier = Modifier.animateItem()) {
                            viewModel.addToSearchHistory(recentItem)
                            onMovieClick(recentItem, null, null)
                        }
                    }
                }

                if (searchResults.isEmpty() && showExplore) {
                    item {
                        SearchEmptyState(
                            title = AppStrings.t(lang, "auto.no_results_yet"),
                            subtitle = AppStrings.t(lang, "auto.try_different_filters_to_discover_something_")
                        )
                    }
                } else {
                    itemsIndexed(searchResults, key = { _, item -> "empty-query:${item.type}:${item.id}" }) { _, movie ->
                        SearchMovieRow(movie, lang, modifier = Modifier.animateItem()) {
                            viewModel.addToSearchHistory(movie)
                            onMovieClick(movie, null, null)
                        }
                    }
                }
            } else if (query.isNotBlank() && searchRows.isNotEmpty()) {
                itemsIndexed(searchRows, key = { index, row -> "row:${row.title}:$index" }) { _, row ->
                    MobileBrowseCategoryRow(
                        modifier = Modifier.animateItem(),
                        title = row.title,
                        items = row.items,
                        onMovieClick = { meta -> onMovieClick(meta, row.sourceAddonTransportUrl, row.sourceAddonCatalogType) },
                        onSearchHistory = viewModel::addToSearchHistory,
                        cardLayout = activeProfile?.safeCardLayout ?: "vertical",
                        activeProfile = activeProfile
                    )
                }
            } else if (searchResults.isEmpty() && query.isNotBlank()) {
                if (searchHistory.isNotEmpty()) {
                    item {
                        SearchSectionTitle(AppStrings.format(lang, "format.no_results_for", query))
                    }
                    itemsIndexed(searchHistory.take(5), key = { _, item -> "fallback-history:${item.type}:${item.id}" }) { _, recentItem ->
                        SearchMovieRow(recentItem, lang, modifier = Modifier.animateItem()) {
                            viewModel.addToSearchHistory(recentItem)
                            onMovieClick(recentItem, null, null)
                        }
                    }
                }
            } else {
                itemsIndexed(searchResults, key = { _, item -> "query-result:${item.type}:${item.id}" }) { _, movie ->
                    SearchMovieRow(movie, lang, modifier = Modifier.animateItem()) {
                        viewModel.addToSearchHistory(movie)
                        onMovieClick(movie, null, null)
                    }
                }
            }
        }
    }
}

@Composable
private fun MobileBrowseFilterChip(text: String, selected: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(999.dp))
            .background(if (selected) Color.White else Color.White.copy(alpha = 0.06f))
            .clickable { onClick() }
            .padding(horizontal = 14.dp, vertical = 8.dp)
    ) {
        Text(
            text = text,
            color = if (selected) Color.Black else Color.White,
            fontSize = 13.sp,
            fontWeight = FontWeight.Bold
        )
    }
}

@Composable
fun MobileBrowseCategoryRow(
    modifier: Modifier = Modifier,
    title: String,
    items: List<Meta>,
    onMovieClick: (Meta) -> Unit,
    onSearchHistory: (Meta) -> Unit,
    cardLayout: String,
    activeProfile: UserProfile? = null
) {
    Column(modifier = modifier.animateContentSize(animationSpec = tween(220))) {
        SearchSectionTitle(title)
        androidx.compose.foundation.lazy.LazyRow(
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            contentPadding = PaddingValues(end = 4.dp)
        ) {
            items(
                count = items.size,
                key = { index -> "${items[index].type}:${items[index].id}" }
            ) { index ->
                val item = items[index]
                MovieCard(
                    meta = item,
                    onFocus = {},
                    onClick = {
                        onSearchHistory(item)
                        onMovieClick(item)
                    },
                    cardLayout = if (activeProfile?.safePosterLandscapeMode == true) "horizontal" else cardLayout,
                    profile = activeProfile
                )
            }
        }
    }
}

@Composable
fun SearchMovieRow(
    movie: Meta,
    lang: String,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .height(130.dp)
            .background(Color.White.copy(alpha = 0.05f), RoundedCornerShape(12.dp))
            .clickable { onClick() }
            .padding(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        AsyncImage(
            model = movie.poster,
            contentDescription = null,
            modifier = Modifier
                .width(80.dp)
                .fillMaxHeight()
                .clip(RoundedCornerShape(8.dp)),
            contentScale = ContentScale.Crop
        )
        
        Spacer(Modifier.width(16.dp))
        
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = movie.name,
                color = Color.White,
                fontSize = 16.sp,
                fontWeight = FontWeight.Black,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )
            
            Spacer(Modifier.height(6.dp))
            
            Row(verticalAlignment = Alignment.CenterVertically) {
                val metaChips = buildList {
                    movie.releaseInfo?.takeIf { it.isNotBlank() }?.let { add(it) }
                    add(AppStrings.t(lang, if (movie.type == "movie") "auto.movie" else "auto.series"))
                    addAll(movie.genres.orEmpty().take(1))
                }
                Text(
                    text = metaChips.joinToString("  "),
                    color = Color.White.copy(alpha = 0.5f),
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
            }
            
            Spacer(Modifier.weight(1f))
        }
        
        Icon(
            FluxaIcons.KeyboardArrowRight,
            null,
            tint = Color.White.copy(alpha = 0.2f),
            modifier = Modifier.padding(start = 8.dp)
        )
    }
}
