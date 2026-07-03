@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items as mobileGridItems
import androidx.compose.foundation.lazy.grid.itemsIndexed as gridItemsIndexed
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.zIndex
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.*
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import coil3.compose.AsyncImage
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope

private enum class TvLibrarySection { Planned, Completed, Favorites, Downloads, Collections }

@Composable
internal fun TvLibraryScreenContent(
    lang: String,
    horizontalPadding: Dp,
    activeProfile: UserProfile?,
    selectedType: String,
    onSelectType: (String) -> Unit,
    plannedItems: List<Meta>,
    completedItems: List<Meta>,
    favoriteItems: List<Meta>,
    filteredPlannedItems: List<Meta>,
    filteredCompletedItems: List<Meta>,
    filteredFavorites: List<Meta>,
    downloadGroups: List<OfflineDownloadGroup>,
    libraryCollections: List<LibraryCollectionUi>,
    userCollections: List<LibraryUserCollection>,
    onBack: () -> Unit,
    onMovieClick: (Meta) -> Unit,
    onOpenDownload: (OfflineDownloadItem) -> Unit,
    onDeleteCollection: (LibraryCollectionUi) -> Unit,
    onSaveUserCollection: (LibraryUserCollection) -> Unit,
    viewModel: HomeViewModel,
    tvNavActions: TvNavActions
) {
    var section by remember { mutableStateOf(TvLibrarySection.Planned) }
    var selectedDownloadGroup by remember { mutableStateOf<OfflineDownloadGroup?>(null) }
    var selectedCollection by remember { mutableStateOf<LibraryCollectionUi?>(null) }
    var editingCollection by remember { mutableStateOf<LibraryUserCollection?>(null) }
    val catalogOptions by viewModel.categories.collectAsStateWithLifecycle()
    val useTopBar = activeProfile?.safeTvNavLayout == "top"
    val railGutter = if (useTopBar) 56.dp else 126.dp
    val contentTopPadding = if (useTopBar) 108.dp else 40.dp

    LaunchedEffect(section) {
        selectedDownloadGroup = null
        selectedCollection = null
    }

    Box(modifier = Modifier.fillMaxSize()) {
    Column(modifier = Modifier.fillMaxSize().padding(start = railGutter, top = contentTopPadding)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(24.dp),
            modifier = Modifier.fillMaxWidth().padding(horizontal = horizontalPadding, vertical = 40.dp)
        ) {
            IconButton(onClick = onBack, modifier = Modifier.size(48.dp).background(Color.White.copy(alpha = 0.05f), CircleShape)) {
                Icon(FluxaIcons.ArrowBack, null, tint = Color.White)
            }
            Column {
                Text(text = AppStrings.t(lang, "auto.my_library_a6c93797"), style = MaterialTheme.typography.displaySmall, color = Color.White, fontWeight = FontWeight.Black, letterSpacing = 2.sp, fontSize = 32.sp)
                Text(text = AppStrings.t(lang, "auto.movies_and_shows_you_saved_to_watch_later"), color = Color.White.copy(alpha = 0.5f), style = MaterialTheme.typography.bodyMedium)
            }
        }

        Row(
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            modifier = Modifier.padding(horizontal = horizontalPadding, vertical = 4.dp)
        ) {
            listOf(
                TvLibrarySection.Planned to AppStrings.t(lang, "auto.planned"),
                TvLibrarySection.Completed to AppStrings.t(lang, "auto.completed"),
                TvLibrarySection.Favorites to AppStrings.t(lang, "auto.favorites"),
                TvLibrarySection.Downloads to AppStrings.t(lang, "auto.downloads"),
                TvLibrarySection.Collections to AppStrings.t(lang, "auto.my_collections")
            ).forEach { (value, label) ->
                SettingsPill(label, section == value) { section = value }
            }
        }

        val sectionItems = when (section) {
            TvLibrarySection.Planned -> plannedItems
            TvLibrarySection.Completed -> completedItems
            TvLibrarySection.Favorites -> favoriteItems
            else -> emptyList()
        }
        if (section == TvLibrarySection.Planned || section == TvLibrarySection.Completed || section == TvLibrarySection.Favorites) {
            val typeOptions = remember(sectionItems, lang) { libraryTypeFilterOptions(sectionItems, lang) }
            if (typeOptions.size > 1) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier.padding(horizontal = horizontalPadding, vertical = 4.dp)
                ) {
                    typeOptions.forEach { (value, label) ->
                        SettingsPill(label, selectedType == value) { onSelectType(value) }
                    }
                }
            }
        }

        Spacer(Modifier.height(8.dp))

        when (section) {
            TvLibrarySection.Planned -> TvLibraryMovieGrid(filteredPlannedItems, horizontalPadding, activeProfile, viewModel, lang, onMovieClick)
            TvLibrarySection.Completed -> TvLibraryMovieGrid(filteredCompletedItems, horizontalPadding, activeProfile, viewModel, lang, onMovieClick)
            TvLibrarySection.Favorites -> TvLibraryMovieGrid(filteredFavorites, horizontalPadding, activeProfile, viewModel, lang, onMovieClick)
            TvLibrarySection.Downloads -> {
                val group = selectedDownloadGroup
                if (group != null) {
                    TvDownloadEpisodeList(group, horizontalPadding, onOpenDownload, onBack = { selectedDownloadGroup = null })
                } else if (downloadGroups.isEmpty()) {
                    EmptyLibraryState(lang)
                } else {
                    LazyVerticalGrid(
                        columns = GridCells.Adaptive(160.dp),
                        modifier = Modifier.fillMaxSize(),
                        contentPadding = PaddingValues(start = horizontalPadding, end = horizontalPadding, bottom = 60.dp),
                        horizontalArrangement = Arrangement.spacedBy(20.dp),
                        verticalArrangement = Arrangement.spacedBy(24.dp)
                    ) {
                        gridItemsIndexed(downloadGroups, key = { _: Int, g: OfflineDownloadGroup -> g.key }) { _, g ->
                            TvDownloadGroupTile(g) { selectedDownloadGroup = g }
                        }
                    }
                }
            }
            TvLibrarySection.Collections -> {
                val collection = selectedCollection
                if (collection != null) {
                    Column {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.padding(horizontal = horizontalPadding, vertical = 4.dp)
                        ) {
                            IconButton(onClick = { selectedCollection = null }) {
                                Icon(FluxaIcons.ArrowBack, null, tint = Color.White)
                            }
                            Text(collection.title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 18.sp)
                            if (collection.userCollectionId != null) {
                                Spacer(Modifier.weight(1f))
                                IconButton(onClick = {
                                    editingCollection = userCollections.firstOrNull { it.id == collection.userCollectionId }
                                        ?: LibraryUserCollection(id = collection.userCollectionId!!, title = collection.title)
                                }) {
                                    Icon(FluxaIcons.Edit, null, tint = Color.White)
                                }
                                IconButton(onClick = { onDeleteCollection(collection); selectedCollection = null }) {
                                    Icon(FluxaIcons.Delete, null, tint = FluxaColors.errorRed)
                                }
                            }
                        }
                        TvLibraryMovieGrid(collection.items, horizontalPadding, activeProfile, viewModel, lang, onMovieClick)
                    }
                } else {
                    Column {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.padding(horizontal = horizontalPadding, vertical = 4.dp)
                        ) {
                            Spacer(Modifier.weight(1f))
                            SettingsActionTile(
                                title = AppStrings.t(lang, "auto.new_collection"),
                                subtitle = "",
                                icon = FluxaIcons.Add,
                                onClick = {
                                    editingCollection = LibraryUserCollection(
                                        id = "local_${System.currentTimeMillis()}",
                                        title = ""
                                    )
                                }
                            )
                        }
                        if (libraryCollections.isEmpty()) {
                            EmptyLibraryState(lang)
                        } else {
                            LazyVerticalGrid(
                                columns = GridCells.Adaptive(160.dp),
                                modifier = Modifier.fillMaxSize(),
                                contentPadding = PaddingValues(start = horizontalPadding, end = horizontalPadding, bottom = 60.dp),
                                horizontalArrangement = Arrangement.spacedBy(20.dp),
                                verticalArrangement = Arrangement.spacedBy(24.dp)
                            ) {
                                gridItemsIndexed(libraryCollections, key = { _: Int, c: LibraryCollectionUi -> c.userCollectionId ?: c.title }) { _, c ->
                                    TvCollectionTile(c) { selectedCollection = c }
                                }
                            }
                        }
                    }
                }
            }
        }

        editingCollection?.let { draft ->
            TvCollectionEditorPage(
                lang = lang,
                initial = draft,
                catalogOptions = catalogOptions,
                onDismiss = { editingCollection = null },
                onSave = { saved ->
                    onSaveUserCollection(saved)
                    editingCollection = null
                }
            )
        }
    }
        if (useTopBar) {
            TvHomeTopBar(
                lang = lang,
                selected = TvNavDestination.Library,
                onHomeClick = tvNavActions.onHome,
                onSearchClick = tvNavActions.onSearch,
                onWatchlistClick = {},
                onExploreClick = tvNavActions.onExplore,
                onProfileClick = tvNavActions.onSettings,
                contentFocusRequester = null
            )
        } else {
            TvHomeNavRail(
                lang = lang,
                selected = TvNavDestination.Library,
                onHomeClick = tvNavActions.onHome,
                onSearchClick = tvNavActions.onSearch,
                onWatchlistClick = {},
                onExploreClick = tvNavActions.onExplore,
                onProfileClick = tvNavActions.onSettings,
                contentFocusRequester = null
            )
        }
    }
}

@Composable
private fun TvLibraryMovieGrid(
    items: List<Meta>,
    horizontalPadding: Dp,
    activeProfile: UserProfile?,
    viewModel: HomeViewModel,
    lang: String,
    onMovieClick: (Meta) -> Unit
) {
    if (items.isEmpty()) {
        EmptyLibraryState(lang)
    } else {
        LazyVerticalGrid(
            columns = GridCells.Adaptive(160.dp),
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(start = horizontalPadding, end = horizontalPadding, bottom = 60.dp),
            horizontalArrangement = Arrangement.spacedBy(20.dp),
            verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            gridItemsIndexed(items, key = { _: Int, item: Meta -> "${item.type}:${item.id}" }) { _, item ->
                MovieCard(
                    item,
                    { viewModel.onMovieFocused(it) },
                    { onMovieClick(item) },
                    if (activeProfile?.safePosterLandscapeMode == true) "horizontal" else "vertical",
                    profile = activeProfile,
                    onForgetProgress = { viewModel.toggleWatchlist(item) }
                )
            }
        }
    }
}

@Composable
private fun TvDownloadGroupTile(group: OfflineDownloadGroup, onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    Surface(
        onClick = onClick,
        modifier = Modifier.aspectRatio(0.66f).onFocusChanged { focused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(14.dp)),
        colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(alpha = if (focused) 0.12f else 0.05f))
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            AsyncImage(
                model = group.poster,
                contentDescription = group.title,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
            Box(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .fillMaxWidth()
                    .background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = 0.85f))))
                    .padding(10.dp)
            ) {
                Column {
                    Text(group.title, color = Color.White, fontWeight = FontWeight.Bold, fontSize = 13.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text("${group.episodes.size}", color = Color.White.copy(alpha = 0.6f), fontSize = 11.sp)
                }
            }
        }
    }
}

@Composable
private fun TvCollectionTile(collection: LibraryCollectionUi, onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    val poster = collection.items.firstOrNull()?.poster
    Surface(
        onClick = onClick,
        modifier = Modifier.aspectRatio(0.66f).onFocusChanged { focused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(14.dp)),
        colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(alpha = if (focused) 0.12f else 0.05f))
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            if (poster != null) {
                AsyncImage(model = poster, contentDescription = collection.title, modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
            }
            Box(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .fillMaxWidth()
                    .background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = 0.85f))))
                    .padding(10.dp)
            ) {
                Column {
                    Text(collection.title, color = Color.White, fontWeight = FontWeight.Bold, fontSize = 13.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text("${collection.items.size}", color = Color.White.copy(alpha = 0.6f), fontSize = 11.sp)
                }
            }
        }
    }
}

@Composable
private fun TvDownloadEpisodeList(
    group: OfflineDownloadGroup,
    horizontalPadding: Dp,
    onOpenDownload: (OfflineDownloadItem) -> Unit,
    onBack: () -> Unit
) {
    Column(modifier = Modifier.fillMaxSize()) {
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(horizontal = horizontalPadding, vertical = 4.dp)) {
            IconButton(onClick = onBack) {
                Icon(FluxaIcons.ArrowBack, null, tint = Color.White)
            }
            Text(group.title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 18.sp)
        }
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(horizontal = horizontalPadding, vertical = 8.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            items(group.episodes, key = { it.id }) { episode ->
                var focused by remember { mutableStateOf(false) }
                Surface(
                    onClick = { if (episode.status == "completed") onOpenDownload(episode) },
                    modifier = Modifier.fillMaxWidth().height(72.dp).onFocusChanged { focused = it.isFocused },
                    shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(16.dp)),
                    colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(alpha = if (focused) 0.12f else 0.05f))
                ) {
                    Row(
                        modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text(episode.episodeTitle ?: episode.title, color = Color.White, fontWeight = FontWeight.Bold, fontSize = 14.sp, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
                        Text(
                            if (episode.status == "completed") "%.1f GB".format(episode.totalBytes / 1_073_741_824.0) else episode.status,
                            color = Color.White.copy(alpha = 0.6f),
                            fontSize = 12.sp
                        )
                    }
                }
            }
        }
    }
}
