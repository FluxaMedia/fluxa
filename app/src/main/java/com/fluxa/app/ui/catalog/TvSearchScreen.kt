@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.*
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
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
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.ExperimentalTvMaterial3Api
import androidx.tv.material3.Surface
import coil3.compose.AsyncImage

@Composable
fun TvSearchScreenContent(
    activeProfile: UserProfile?,
    searchResults: List<Meta>,
    searchRows: List<SearchResultRow>,
    query: String,
    onSearch: (String) -> Unit,
    onBack: () -> Unit,
    onMovieClick: (Meta, String?, String?) -> Unit,
    focusRequester: FocusRequester,
    lang: String,
    viewModel: HomeViewModel,
    tvNavActions: TvNavActions
) {
    val searchHistory by viewModel.searchHistory.collectAsState()
    val suggestions = remember(lang) { recommendedSearches(lang) }
    val useTopBar = activeProfile?.safeTvNavLayout == "top"
    val railGutter = if (useTopBar) 56.dp else 126.dp
    val contentTopPadding = if (useTopBar) 108.dp else 40.dp

    Box(modifier = Modifier.fillMaxSize().background(Color(0xFF040508))) {
    Row(
        modifier = Modifier
            .fillMaxSize()
            .padding(start = railGutter, end = 56.dp, top = contentTopPadding, bottom = 40.dp)
    ) {
        Column(modifier = Modifier.width(340.dp)) {
                Text(
                    text = query.ifEmpty { AppStrings.t(lang, "auto.search") },
                    color = Color.White,
                    fontSize = 32.sp,
                    fontWeight = FontWeight.Black,
                    modifier = Modifier.padding(bottom = 8.dp)
                )
                Text(
                    text = if (query.isBlank()) {
                        AppStrings.t(lang, "auto.start_typing_to_find_something_fast")
                    } else {
                        AppStrings.t(lang, "auto.results_are_ranked_by_closest_match")
                    },
                    color = Color.White.copy(alpha = 0.55f),
                    fontSize = 14.sp,
                    modifier = Modifier.padding(bottom = 24.dp)
                )

                val keys = listOf(
                    "a", "b", "c", "d", "e", "f",
                    "g", "h", "i", "j", "k", "l",
                    "m", "n", "o", "p", "q", "r",
                    "s", "t", "u", "v", "w", "x",
                    "y", "z", "1", "2", "3", "4",
                    "5", "6", "7", "8", "9", "0"
                )

                LazyVerticalGrid(
                    columns = GridCells.Fixed(6),
                    modifier = Modifier.width(300.dp).height(240.dp),
                    userScrollEnabled = false,
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    items(keys, key = { it }) { key ->
                        KeyButton(key) { onSearch(query + key) }
                    }
                }

                Spacer(Modifier.height(24.dp))
                Box(modifier = Modifier.height(1.dp).width(280.dp).background(Color.White.copy(0.1f)))
                Spacer(Modifier.height(16.dp))

                Row(modifier = Modifier.width(280.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Surface(
                        onClick = { onSearch(query + " ") },
                        modifier = Modifier.height(40.dp).weight(1f),
                        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(4.dp)),
                        colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(0.05f), focusedContainerColor = Color.White)
                    ) {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                             Box(modifier = Modifier.width(40.dp).height(2.dp).background(Color.Gray))
                        }
                    }
                    Surface(
                        onClick = { if(query.isNotEmpty()) onSearch(query.dropLast(1)) },
                        modifier = Modifier.size(40.dp),
                        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(4.dp)),
                        colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(0.05f), focusedContainerColor = Color.White)
                    ) {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            androidx.tv.material3.Icon(FluxaIcons.Backspace, null, modifier = Modifier.size(20.dp))
                        }
                    }
                }

                Spacer(Modifier.height(16.dp))
                Row(modifier = Modifier.width(280.dp), horizontalArrangement = Arrangement.SpaceBetween) {
                    Text("@#&", color = Color.White.copy(0.4f), fontWeight = FontWeight.Bold)
                    Text("ËÜ", color = Color.White.copy(0.4f), fontWeight = FontWeight.Bold)
                }
                
                Spacer(Modifier.height(24.dp))
                var clearFocused by remember { mutableStateOf(false) }
                Surface(
                    onClick = { onSearch("") },
                    modifier = Modifier.height(44.dp).width(280.dp).onFocusChanged { clearFocused = it.isFocused },
                    shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(4.dp)),
                    colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(0.05f), focusedContainerColor = Color.White)
                ) {
                    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Text(AppStrings.t(lang, "auto.clear_search"), color = if (clearFocused) Color.Black else Color.White, fontWeight = FontWeight.Bold)
                    }
                }
            }

            Spacer(Modifier.width(48.dp))

            Column(modifier = Modifier.weight(1f)) {
                val topChips = if (query.isBlank()) {
                    (searchHistory.take(6).map { it.name } + suggestions).distinct().take(8)
                } else {
                    searchHistory.take(4).map { it.name }
                }

                if (topChips.isNotEmpty()) {
                    Text(
                        text = if (query.isBlank()) {
                            AppStrings.t(lang, "auto.recent_searches_and_suggestions")
                        } else {
                            AppStrings.t(lang, "auto.search_again")
                        },
                        color = Color.White.copy(alpha = 0.72f),
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(bottom = 12.dp)
                    )
                }

                Row(modifier = Modifier.padding(bottom = 32.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    topChips.forEach { tag ->
                        TagButton(tag) { onSearch(tag) }
                    }
                }

                LazyVerticalGrid(
                    columns = if (query.isNotBlank() && searchRows.isNotEmpty()) GridCells.Fixed(1) else GridCells.Adaptive(180.dp),
                    modifier = Modifier.fillMaxSize(),
                    horizontalArrangement = Arrangement.spacedBy(20.dp),
                    verticalArrangement = Arrangement.spacedBy(24.dp),
                    contentPadding = PaddingValues(bottom = 40.dp)
                ) {
                    if (query.isBlank() && searchResults.isEmpty()) {
                        item(span = { GridItemSpan(maxLineSpan) }) {
                            SearchEmptyState(
                                title = AppStrings.t(lang, "auto.start_searching"),
                                subtitle = AppStrings.t(lang, "auto.use_the_chips_above_or_type_a_few_letters_wi")
                            )
                        }
                    } else if (query.isNotBlank() && searchResults.isEmpty()) {
                        item(span = { GridItemSpan(maxLineSpan) }) {
                            SearchEmptyState(
                                title = AppStrings.t(lang, "auto.no_results_found"),
                                subtitle = AppStrings.t(lang, "auto.try_a_shorter_or_slightly_different_title")
                            )
                        }
                    } else if (query.isNotBlank() && searchRows.isNotEmpty()) {
                        items(searchRows, key = { it.id }) { row ->
                            HomeShelfRow(
                                title = row.title,
                                items = row.items,
                                cardLayout = if (activeProfile?.safePosterLandscapeMode == true) "horizontal" else activeProfile?.safeCardLayout ?: "vertical",
                                artworkPreference = null,
                                activeProfile = activeProfile,
                                topTenEnabled = false,
                                onItemClick = { meta ->
                                    viewModel.addToSearchHistory(meta)
                                    onMovieClick(meta, row.sourceAddonTransportUrl, row.sourceAddonCatalogType)
                                },
                                onItemFocus = {},
                                onNeedMore = {}
                            )
                        }
                    } else {
                        items(searchResults, key = { "${it.type}:${it.id}" }) { movie ->
                            MovieCard(
                                movie,
                                { viewModel.onMovieFocused(it) },
                                {
                                    viewModel.addToSearchHistory(movie)
                                    onMovieClick(movie, null, null)
                                },
                                if (activeProfile?.safePosterLandscapeMode == true) "horizontal" else activeProfile?.safeCardLayout ?: "vertical",
                                profile = activeProfile
                            )
                        }
                    }
                }
            }
        }
        if (useTopBar) {
            TvHomeTopBar(
                lang = lang,
                selected = TvNavDestination.Search,
                onHomeClick = tvNavActions.onHome,
                onSearchClick = {},
                onWatchlistClick = tvNavActions.onWatchlist,
                onExploreClick = tvNavActions.onExplore,
                onProfileClick = tvNavActions.onSettings,
                contentFocusRequester = null
            )
        } else {
            TvHomeNavRail(
                lang = lang,
                selected = TvNavDestination.Search,
                onHomeClick = tvNavActions.onHome,
                onSearchClick = {},
                onWatchlistClick = tvNavActions.onWatchlist,
                onExploreClick = tvNavActions.onExplore,
                onProfileClick = tvNavActions.onSettings,
                contentFocusRequester = null
            )
        }
    }
}

@Composable
fun KeyButton(text: String, onClick: () -> Unit) {
    var isFocused by remember { mutableStateOf(false) }
    Surface(
        onClick = onClick,
        modifier = Modifier.size(40.dp).onFocusChanged { isFocused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(4.dp)),
        colors = ClickableSurfaceDefaults.colors(containerColor = Color.Transparent, focusedContainerColor = Color.White)
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(text = text, color = if(isFocused) Color.Black else Color.White, fontSize = 18.sp, fontWeight = FontWeight.Bold)
        }
    }
}

@Composable
fun TagButton(text: String, onClick: () -> Unit) {
    var isFocused by remember { mutableStateOf(false) }
    Surface(
        onClick = onClick,
        modifier = Modifier.onFocusChanged { isFocused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(20.dp)),
        colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(0.05f), focusedContainerColor = Color.White)
    ) {
        Text(text = text, color = if(isFocused) Color.Black else Color.White, fontWeight = FontWeight.Bold, modifier = Modifier.padding(horizontal = 16.dp, vertical = 6.dp))
    }
}

@Composable
fun SuggestionCard(text: String, onClick: () -> Unit) {
    val deviceType = LocalDeviceType.current
    var isFocused by remember { mutableStateOf(false) }
    
    if (deviceType == DeviceType.Mobile) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp)
                .clip(RoundedCornerShape(8.dp))
                .background(Color.White.copy(alpha = 0.05f))
                .clickable { onClick() }
                .padding(horizontal = 16.dp),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = text,
                color = Color.White,
                fontWeight = FontWeight.Bold
            )
        }
    } else {
        Surface(
            onClick = onClick,
            shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(8.dp)),
            colors = ClickableSurfaceDefaults.colors(
                containerColor = Color.White.copy(alpha = 0.05f),
                focusedContainerColor = Color.White
            ),
            modifier = Modifier.height(48.dp).onFocusChanged { isFocused = it.isFocused }
        ) {
            Box(contentAlignment = Alignment.Center, modifier = Modifier.fillMaxSize()) {
                Text(
                    text = text,
                    color = if(isFocused) Color.Black else Color.White,
                    fontWeight = FontWeight.Bold
                )
            }
        }
    }
}
