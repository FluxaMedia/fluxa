package com.fluxa.app.shared.feature.library

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun LibraryScreen(
    state: LibraryUiState,
    language: String?,
    onAction: (LibraryAction) -> Unit,
    onItemSelected: (com.fluxa.app.shared.feature.catalog.CatalogItemUiModel) -> Unit,
    modifier: Modifier = Modifier
) {
    var section by remember { mutableStateOf(LibrarySection.Planned) }
    var typeFilter by remember { mutableStateOf(LibraryTypeFilter.All) }
    var viewingCollectionId by remember { mutableStateOf<String?>(null) }
    var viewingDownloadGroupKey by remember { mutableStateOf<String?>(null) }
    var showCreateDialog by remember { mutableStateOf(false) }
    var editingCollectionId by remember { mutableStateOf<String?>(null) }
    var isManagingCollections by remember { mutableStateOf(false) }

    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        when {
            viewingDownloadGroupKey != null -> {
                val group = state.downloadGroups.firstOrNull { it.key == viewingDownloadGroupKey }
                if (group != null) {
                    LibraryDownloadGroupPage(
                        group = group,
                        language = language,
                        onBack = { viewingDownloadGroupKey = null },
                        onOpen = { onAction(LibraryAction.DownloadOpened(it)) },
                        onCancel = { onAction(LibraryAction.DownloadCancelled(it)) }
                    )
                }
            }
            else -> {
                val collection = viewingCollectionId?.let { id -> state.collections.firstOrNull { (it.id ?: it.title) == id } }
                if (collection != null) {
                    LibraryCollectionDetailPage(
                        collection = collection,
                        language = language,
                        onBack = { viewingCollectionId = null },
                        onItemSelected = onItemSelected
                    )
                } else {
                    Column(modifier = Modifier.fillMaxSize()) {
                        Text(
                            text = AppStrings.t(language, "nav.library"),
                            color = Color.White,
                            fontWeight = FontWeight.Black,
                            fontSize = 26.sp,
                            modifier = Modifier.padding(horizontal = 16.dp, vertical = 16.dp)
                        )
                        LibrarySectionChips(
                            section = section,
                            counts = mapOf(
                                LibrarySection.Planned to state.planned.size,
                                LibrarySection.Completed to state.completed.size,
                                LibrarySection.Favorites to state.favorites.size,
                                LibrarySection.Downloads to state.downloadGroups.size,
                                LibrarySection.Collections to state.collections.size
                            ),
                            onSectionSelected = { section = it },
                            language = language
                        )
                        when {
                            state.isLoading -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                CircularProgressIndicator(color = Color.White)
                            }
                            section == LibrarySection.Collections -> LibraryCollectionsSection(
                                collections = state.collections,
                                isManaging = isManagingCollections,
                                language = language,
                                onToggleManage = { isManagingCollections = !isManagingCollections },
                                onCollectionClick = { viewingCollectionId = it.id ?: it.title },
                                onCreateClick = { showCreateDialog = true },
                                onRenameClick = { editingCollectionId = it },
                                onDeleteClick = { onAction(LibraryAction.CollectionDeleted(it)) }
                            )
                            section == LibrarySection.Downloads -> LibraryDownloadFoldersSection(
                                groups = state.downloadGroups,
                                language = language,
                                onGroupClick = { viewingDownloadGroupKey = it }
                            )
                            else -> {
                                val items = when (section) {
                                    LibrarySection.Planned -> state.planned
                                    LibrarySection.Completed -> state.completed
                                    LibrarySection.Favorites -> state.favorites
                                    else -> emptyList()
                                }
                                LibraryTypeFilterRow(typeFilter, language) { typeFilter = it }
                                LibraryItemGrid(filterItems(items, typeFilter), language, onItemSelected)
                            }
                        }
                    }
                }
            }
        }

        if (showCreateDialog) {
            LibraryCollectionNameDialog(
                initialTitle = "",
                language = language,
                onConfirm = { title ->
                    onAction(LibraryAction.CollectionCreated(title))
                    showCreateDialog = false
                },
                onDismiss = { showCreateDialog = false }
            )
        }
        editingCollectionId?.let { id ->
            val current = state.collections.firstOrNull { it.id == id }
            LibraryCollectionNameDialog(
                initialTitle = current?.title.orEmpty(),
                language = language,
                onConfirm = { title ->
                    onAction(LibraryAction.CollectionRenamed(id, title))
                    editingCollectionId = null
                },
                onDismiss = { editingCollectionId = null }
            )
        }
    }
}

private fun filterItems(
    items: List<com.fluxa.app.shared.feature.catalog.CatalogItemUiModel>,
    filter: LibraryTypeFilter
): List<com.fluxa.app.shared.feature.catalog.CatalogItemUiModel> = when (filter) {
    LibraryTypeFilter.All -> items
    LibraryTypeFilter.Movie -> items.filter { it.type == "movie" }
    LibraryTypeFilter.Series -> items.filter { it.type == "series" }
    LibraryTypeFilter.Anime -> items.filter { it.type == "anime" }
}

@Composable
private fun LibrarySectionChips(
    section: LibrarySection,
    counts: Map<LibrarySection, Int>,
    onSectionSelected: (LibrarySection) -> Unit,
    language: String?
) {
    LazyRow(
        contentPadding = PaddingValues(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        modifier = Modifier.padding(bottom = 12.dp)
    ) {
        items(LibrarySection.entries.toList()) { entry ->
            val selected = entry == section
            val label = when (entry) {
                LibrarySection.Planned -> AppStrings.t(language, "auto.planned")
                LibrarySection.Completed -> AppStrings.t(language, "auto.completed")
                LibrarySection.Favorites -> AppStrings.t(language, "auto.favorites")
                LibrarySection.Downloads -> AppStrings.t(language, "auto.downloads")
                LibrarySection.Collections -> AppStrings.t(language, "auto.my_collections")
            }
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(if (selected) Color.White.copy(alpha = 0.16f) else Color.White.copy(alpha = 0.06f))
                    .clickable { onSectionSelected(entry) }
                    .padding(horizontal = 16.dp, vertical = 10.dp)
            ) {
                Text(
                    text = "$label (${counts[entry] ?: 0})",
                    color = if (selected) Color.White else Color.White.copy(alpha = 0.65f),
                    fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                    fontSize = 13.sp
                )
            }
        }
    }
}

@Composable
private fun LibraryTypeFilterRow(current: LibraryTypeFilter, language: String?, onSelected: (LibraryTypeFilter) -> Unit) {
    LazyRow(
        contentPadding = PaddingValues(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        modifier = Modifier.padding(bottom = 8.dp)
    ) {
        items(LibraryTypeFilter.entries.toList()) { entry ->
            val selected = entry == current
            val label = when (entry) {
                LibraryTypeFilter.All -> AppStrings.t(language, "auto.all")
                LibraryTypeFilter.Movie -> AppStrings.t(language, "auto.movie")
                LibraryTypeFilter.Series -> AppStrings.t(language, "auto.series")
                LibraryTypeFilter.Anime -> AppStrings.t(language, "auto.anime")
            }
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(if (selected) Color.White.copy(alpha = 0.14f) else Color.Transparent)
                    .clickable { onSelected(entry) }
                    .padding(horizontal = 12.dp, vertical = 6.dp)
            ) {
                Text(label, color = if (selected) Color.White else Color.White.copy(alpha = 0.5f), fontSize = 12.sp)
            }
        }
    }
}

@Composable
private fun LibraryItemGrid(
    items: List<com.fluxa.app.shared.feature.catalog.CatalogItemUiModel>,
    language: String?,
    onItemSelected: (com.fluxa.app.shared.feature.catalog.CatalogItemUiModel) -> Unit
) {
    if (items.isEmpty()) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(AppStrings.t(language, "library.empty"), color = Color.White.copy(alpha = 0.5f))
        }
        return
    }
    LazyVerticalGrid(
        columns = GridCells.Adaptive(minSize = 128.dp),
        contentPadding = PaddingValues(horizontal = 16.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
        modifier = Modifier.fillMaxSize()
    ) {
        items(items, key = { "${it.type}:${it.id}" }) { item ->
            CatalogCard(model = item.card, onClick = { onItemSelected(item) })
        }
    }
}

@Composable
private fun LibraryCollectionsSection(
    collections: List<LibraryCollectionUiModel>,
    isManaging: Boolean,
    language: String?,
    onToggleManage: () -> Unit,
    onCollectionClick: (LibraryCollectionUiModel) -> Unit,
    onCreateClick: () -> Unit,
    onRenameClick: (String) -> Unit,
    onDeleteClick: (String) -> Unit
) {
    Column(modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp)) {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text(AppStrings.t(language, "auto.my_collections"), color = Color.White, fontWeight = FontWeight.Bold)
            if (collections.any { it.id != null }) {
                TextButton(onClick = onToggleManage) {
                    Text(
                        if (isManaging) AppStrings.t(language, "auto.done") else AppStrings.t(language, "auto.edit"),
                        color = Color.White
                    )
                }
            }
        }
        LazyColumn(modifier = Modifier.weight(1f), contentPadding = PaddingValues(vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(collections, key = { it.id ?: it.title }) { collection ->
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(14.dp))
                        .background(Color.White.copy(alpha = 0.04f))
                        .clickable(enabled = !isManaging) { onCollectionClick(collection) }
                        .padding(12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Box(modifier = Modifier.size(48.dp).clip(RoundedCornerShape(8.dp)).background(Color.White.copy(alpha = 0.08f))) {
                        val artUrl = collection.items.firstOrNull()?.card?.artworkUrl
                        if (!artUrl.isNullOrBlank()) {
                            FluxaRemoteImage(
                                imageUrl = artUrl,
                                cacheKey = "library-collection:$artUrl",
                                contentDescription = null,
                                modifier = Modifier.fillMaxSize(),
                                contentScale = androidx.compose.ui.layout.ContentScale.Crop
                            )
                        }
                    }
                    Spacer(Modifier.width(12.dp))
                    Column(modifier = Modifier.weight(1f)) {
                        Text(collection.title, color = Color.White, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text(collection.subtitle, color = Color.White.copy(alpha = 0.5f), fontSize = 12.sp)
                    }
                    if (isManaging && collection.id != null) {
                        TextButton(onClick = { onRenameClick(collection.id) }) { Text(AppStrings.t(language, "auto.edit"), color = Color.White.copy(alpha = 0.7f), fontSize = 12.sp) }
                        TextButton(onClick = { onDeleteClick(collection.id) }) { Text(AppStrings.t(language, "profiles.delete"), color = FluxaColors.errorRed, fontSize = 12.sp) }
                    }
                }
            }
            item {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(14.dp))
                        .background(Color.White.copy(alpha = 0.03f))
                        .clickable(onClick = onCreateClick)
                        .padding(16.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Text(AppStrings.t(language, "auto.new_collection"), color = Color.White.copy(alpha = 0.7f), fontWeight = FontWeight.Medium)
                }
            }
        }
    }
}

@Composable
private fun LibraryCollectionNameDialog(
    initialTitle: String,
    language: String?,
    onConfirm: (String) -> Unit,
    onDismiss: () -> Unit
) {
    var text by remember { mutableStateOf(initialTitle) }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(AppStrings.t(language, "auto.collection_name"), color = Color.White) },
        text = {
            OutlinedTextField(value = text, onValueChange = { text = it }, singleLine = true)
        },
        confirmButton = {
            TextButton(onClick = { if (text.isNotBlank()) onConfirm(text.trim()) }, enabled = text.isNotBlank()) {
                Text(AppStrings.t(language, "auto.save"), color = Color.White)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text(AppStrings.t(language, "auto.cancel"), color = Color.White.copy(alpha = 0.6f)) }
        },
        containerColor = Color(0xFF1A1D26)
    )
}

@Composable
private fun LibraryCollectionDetailPage(
    collection: LibraryCollectionUiModel,
    language: String?,
    onBack: () -> Unit,
    onItemSelected: (com.fluxa.app.shared.feature.catalog.CatalogItemUiModel) -> Unit
) {
    Column(modifier = Modifier.fillMaxSize()) {
        LibraryDetailHeader(title = collection.title, onBack = onBack)
        LibraryItemGrid(collection.items, language, onItemSelected)
    }
}

@Composable
private fun LibraryDetailHeader(title: String, onBack: () -> Unit) {
    Row(modifier = Modifier.fillMaxWidth().padding(16.dp), verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier.size(36.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.06f)).clickable(onClick = onBack),
            contentAlignment = Alignment.Center
        ) {
            Text("←", color = Color.White)
        }
        Spacer(Modifier.width(12.dp))
        Text(title, color = Color.White, fontWeight = FontWeight.Bold, fontSize = 18.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

@Composable
private fun LibraryDownloadFoldersSection(
    groups: List<LibraryDownloadGroupUiModel>,
    language: String?,
    onGroupClick: (String) -> Unit
) {
    if (groups.isEmpty()) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(AppStrings.t(language, "downloads.empty"), color = Color.White.copy(alpha = 0.5f))
        }
        return
    }
    LazyColumn(contentPadding = PaddingValues(horizontal = 16.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        items(groups, key = { it.key }) { group ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(14.dp))
                    .background(Color.White.copy(alpha = 0.04f))
                    .clickable { onGroupClick(group.key) }
                    .padding(12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Box(modifier = Modifier.size(58.dp).clip(RoundedCornerShape(8.dp)).background(Color.White.copy(alpha = 0.08f))) {
                    if (!group.posterUrl.isNullOrBlank()) {
                        FluxaRemoteImage(
                            imageUrl = group.posterUrl,
                            cacheKey = "library-download:${group.posterUrl}",
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize(),
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop
                        )
                    }
                }
                Spacer(Modifier.width(12.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(group.title, color = Color.White, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text("${group.episodes.size} · ${group.totalSizeLabel}", color = Color.White.copy(alpha = 0.5f), fontSize = 12.sp)
                }
                Text("›", color = Color.White.copy(alpha = 0.4f), fontSize = 18.sp)
            }
        }
    }
}

@Composable
private fun LibraryDownloadGroupPage(
    group: LibraryDownloadGroupUiModel,
    language: String?,
    onBack: () -> Unit,
    onOpen: (String) -> Unit,
    onCancel: (String) -> Unit
) {
    Column(modifier = Modifier.fillMaxSize()) {
        LibraryDetailHeader(title = group.title, onBack = onBack)
        LazyColumn(contentPadding = PaddingValues(horizontal = 16.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(group.episodes, key = { it.id }) { episode ->
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(12.dp))
                        .background(Color.White.copy(alpha = 0.04f))
                        .clickable(enabled = episode.isPlayable) { onOpen(episode.id) }
                        .padding(12.dp)
                ) {
                    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                        Text(episode.title, color = Color.White, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
                        Text("✕", color = FluxaColors.errorRed, modifier = Modifier.clickable { onCancel(episode.id) }.padding(8.dp))
                    }
                    Text("${episode.statusLabel} · ${episode.sizeLabel}", color = Color.White.copy(alpha = 0.5f), fontSize = 12.sp)
                    if (!episode.isDownloaded) {
                        Spacer(Modifier.height(6.dp))
                        LinearProgressIndicator(
                            progress = { episode.progressPercent / 100f },
                            modifier = Modifier.fillMaxWidth(),
                            color = Color.White
                        )
                    }
                }
            }
        }
    }
}
