@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)

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
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRestorer
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.stableLazyKey
import com.fluxa.app.shared.feature.localmedia.LocalMediaKind
import com.fluxa.app.shared.feature.localmedia.LocalMediaSourceInput
import com.fluxa.app.shared.feature.localmedia.LocalMediaSourceType
import com.fluxa.app.shared.feature.discover.DiscoverDropdownFilter
import com.fluxa.app.shared.feature.discover.DiscoverFilterOptionUiModel
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun LibraryScreen(
    state: LibraryUiState,
    language: String?,
    onAction: (LibraryAction) -> Unit,
    onItemSelected: (com.fluxa.app.shared.feature.catalog.CatalogItemUiModel) -> Unit,
    initialSection: LibrarySection = LibrarySection.Planned,
    modifier: Modifier = Modifier
) {
    var section by remember(initialSection) { mutableStateOf(initialSection) }
    LaunchedEffect(state.completedSectionEnabled) {
        if (!state.completedSectionEnabled && section == LibrarySection.Completed) {
            section = LibrarySection.Planned
        }
    }
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
                        onItemSelected = onItemSelected,
                        onFolderClick = { folder -> onAction(LibraryAction.FolderSelected(folder)) },
                        onFolderEditClick = { folder ->
                            onAction(LibraryAction.FolderEditorOpened(collection.id ?: collection.title, folder.id))
                        },
                        onAddFolderClick = {
                            onAction(LibraryAction.FolderEditorOpened(collection.id ?: collection.title))
                        }
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
                                LibrarySection.LocalMedia to state.localMediaRows.sumOf { it.items.size },
                                LibrarySection.Downloads to state.downloadGroups.size,
                                LibrarySection.Collections to state.collections.size
                            ),
                            plannedLabelKey = state.plannedLabelKey,
                            completedLabelKey = state.completedLabelKey,
                            completedSectionEnabled = state.completedSectionEnabled,
                            localMediaSupported = state.localMediaSupported,
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
                            section == LibrarySection.LocalMedia -> LocalMediaLibrarySection(
                                state = state,
                                language = language,
                                onAction = onAction,
                                onItemSelected = onItemSelected,
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
                                    LibrarySection.LocalMedia, LibrarySection.Downloads, LibrarySection.Collections -> emptyList()
                                }
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    LibraryTypeDropdown(typeFilter, language) { typeFilter = it }
                                    if (state.availableLibrarySources.size > 1) {
                                        LibrarySourceDropdown(state.librarySource, state.availableLibrarySources, language) {
                                            onAction(LibraryAction.SourceChanged(it))
                                        }
                                    }
                                }
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
    plannedLabelKey: String,
    completedLabelKey: String,
    completedSectionEnabled: Boolean,
    localMediaSupported: Boolean,
    onSectionSelected: (LibrarySection) -> Unit,
    language: String?
) {
    val visibleSections = LibrarySection.entries.filter { entry ->
        (entry != LibrarySection.Completed || completedSectionEnabled) &&
            (entry != LibrarySection.LocalMedia || localMediaSupported)
    }
    LazyRow(
        contentPadding = PaddingValues(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        modifier = Modifier.padding(bottom = 12.dp)
    ) {
        items(visibleSections) { entry ->
            val selected = entry == section
            val label = when (entry) {
                LibrarySection.Planned -> AppStrings.t(language, plannedLabelKey)
                LibrarySection.Completed -> AppStrings.t(language, completedLabelKey)
                LibrarySection.Favorites -> AppStrings.t(language, "auto.favorites")
                LibrarySection.LocalMedia -> AppStrings.t(language, "library.local_media")
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
private fun LibraryTypeDropdown(current: LibraryTypeFilter, language: String?, onSelected: (LibraryTypeFilter) -> Unit) {
    val options = LibraryTypeFilter.entries.map { entry ->
        DiscoverFilterOptionUiModel(
            id = entry.name,
            label = when (entry) {
                LibraryTypeFilter.All -> AppStrings.t(language, "auto.all")
                LibraryTypeFilter.Movie -> AppStrings.t(language, "auto.movie")
                LibraryTypeFilter.Series -> AppStrings.t(language, "auto.series")
                LibraryTypeFilter.Anime -> AppStrings.t(language, "auto.anime")
            }
        )
    }
    Box(modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp)) {
        DiscoverDropdownFilter(
            label = AppStrings.t(language, "auto.type"),
            options = options,
            selectedId = current.name,
            onSelected = { selected ->
                LibraryTypeFilter.entries.firstOrNull { it.name == selected }?.let(onSelected)
            }
        )
    }
}

@Composable
private fun LibrarySourceDropdown(current: String, available: List<String>, language: String?, onSelected: (String) -> Unit) {
    val options = available.map { source ->
        DiscoverFilterOptionUiModel(
            id = source,
            label = AppStrings.t(language, "settings.cw_source_of_truth_$source")
        )
    }
    Box(modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp)) {
        DiscoverDropdownFilter(
            label = AppStrings.t(language, "settings.library_source_of_truth"),
            options = options,
            selectedId = current,
            onSelected = { selected -> selected?.let(onSelected) }
        )
    }
}

@Composable
private fun LibraryItemGrid(
    items: List<CatalogItemUiModel>,
    language: String?,
    onItemSelected: (CatalogItemUiModel) -> Unit
) {
    if (items.isEmpty()) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(AppStrings.t(language, "library.empty"), color = Color.White.copy(alpha = 0.5f))
        }
        return
    }
    LazyVerticalGrid(
        columns = com.fluxa.app.ui.catalog.rememberCatalogGridCells(),
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 12.dp, bottom = 120.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
        modifier = Modifier.fillMaxSize().focusRestorer()
    ) {
        items(items, key = { it.stableLazyKey() }, contentType = { "catalog-card" }) { item ->
            val card = remember(item.card) { item.card.copy(showTitleBar = true) }
            CatalogCard(model = card, onClick = { onItemSelected(item) })
        }
    }
}

@Composable
private fun LocalMediaLibrarySection(
    state: LibraryUiState,
    language: String?,
    onAction: (LibraryAction) -> Unit,
    onItemSelected: (CatalogItemUiModel) -> Unit,
) {
    var showNasDialog by remember { mutableStateOf(false) }
    Column(modifier = Modifier.fillMaxSize()) {
        LazyRow(
            modifier = Modifier.fillMaxWidth(),
            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            item {
                TextButton(onClick = { onAction(LibraryAction.LocalMediaFolderPickerRequested(LocalMediaKind.Movies)) }) {
                    Text(AppStrings.t(language, "library.local_media_add_movies_folder"), color = Color.White)
                }
            }
            item {
                TextButton(onClick = { onAction(LibraryAction.LocalMediaFolderPickerRequested(LocalMediaKind.TvShows)) }) {
                    Text(AppStrings.t(language, "library.local_media_add_tv_folder"), color = Color.White)
                }
            }
            item {
                TextButton(onClick = { onAction(LibraryAction.LocalMediaFolderPickerRequested(LocalMediaKind.Anime)) }) {
                    Text(AppStrings.t(language, "library.local_media_add_anime_folder"), color = Color.White)
                }
            }
            item {
                TextButton(onClick = { showNasDialog = true }) {
                    Text(AppStrings.t(language, "library.local_media_add_nas"), color = Color.White)
                }
            }
            item {
                TextButton(
                    onClick = { onAction(LibraryAction.LocalMediaScanRequested()) },
                    enabled = !state.localMediaIsScanning,
                ) {
                    Text(
                        AppStrings.t(language, if (state.localMediaIsScanning) "library.local_media_scanning" else "library.local_media_scan"),
                        color = Color.White,
                    )
                }
            }
        }
        if (state.localMediaSources.isNotEmpty()) {
            LazyRow(
                contentPadding = PaddingValues(horizontal = 16.dp, vertical = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(state.localMediaSources, key = { it.id }) { source ->
                    Row(
                        modifier = Modifier
                            .clip(RoundedCornerShape(10.dp))
                            .background(Color.White.copy(alpha = 0.05f))
                            .padding(horizontal = 10.dp, vertical = 7.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column {
                            Text(source.displayName, color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.Medium)
                            Text(
                                "${source.kind.label(language)} · ${source.sourceType.label(language)}",
                                color = Color.White.copy(alpha = 0.45f),
                                fontSize = 10.sp,
                            )
                        }
                        TextButton(onClick = { onAction(LibraryAction.LocalMediaSourceRemoved(source.id)) }) {
                            Text(AppStrings.t(language, "library.local_media_remove"), color = FluxaColors.errorRed, fontSize = 10.sp)
                        }
                    }
                }
            }
        }
        if (state.localMediaError != null) {
            Text(
                state.localMediaError,
                color = FluxaColors.errorRed,
                fontSize = 12.sp,
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp),
            )
        }
        Text(
            "${state.localMediaIndexedFileCount} ${AppStrings.t(language, "library.local_media_files_indexed")}" +
                if (state.localMediaUnmatchedFileCount > 0) " · ${state.localMediaUnmatchedFileCount} ${AppStrings.t(language, "library.local_media_unmatched")}" else "",
            color = Color.White.copy(alpha = 0.45f),
            fontSize = 11.sp,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp),
        )
        if (state.localMediaRows.isEmpty()) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text(AppStrings.t(language, "library.local_media_empty"), color = Color.White.copy(alpha = 0.5f))
            }
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(bottom = 120.dp),
                verticalArrangement = Arrangement.spacedBy(18.dp),
            ) {
                items(state.localMediaRows, key = { it.id }) { row ->
                    Column {
                        Text(
                            row.title,
                            color = Color.White,
                            fontWeight = FontWeight.Bold,
                            fontSize = 17.sp,
                            modifier = Modifier.padding(horizontal = 16.dp, vertical = 6.dp),
                        )
                        LazyRow(
                            contentPadding = PaddingValues(horizontal = 16.dp),
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            items(row.items, key = { it.stableLazyKey() }) { item ->
                                CatalogCard(model = item.card, onClick = { onItemSelected(item) })
                            }
                        }
                    }
                }
            }
        }
    }
    if (showNasDialog) {
        LocalMediaNasDialog(
            language = language,
            onDismiss = { showNasDialog = false },
            onConfirm = {
                onAction(LibraryAction.LocalMediaSourceAdded(it))
                onAction(LibraryAction.LocalMediaScanRequested())
                showNasDialog = false
            },
        )
    }
}

@Composable
private fun LocalMediaNasDialog(
    language: String?,
    onDismiss: () -> Unit,
    onConfirm: (LocalMediaSourceInput) -> Unit,
) {
    var kind by remember { mutableStateOf(LocalMediaKind.Movies) }
    var sourceType by remember { mutableStateOf(LocalMediaSourceType.Smb) }
    var location by remember { mutableStateOf("") }
    var username by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(AppStrings.t(language, "library.local_media_add_nas_library"), color = Color.White) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    LocalMediaKind.entries.forEach { candidate ->
                        TextButton(onClick = { kind = candidate }) {
                            Text(candidate.label(language), color = if (kind == candidate) Color.White else Color.White.copy(alpha = 0.45f))
                        }
                    }
                }
                Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    listOf(LocalMediaSourceType.Smb, LocalMediaSourceType.WebDav).forEach { candidate ->
                        TextButton(onClick = { sourceType = candidate }) {
                            Text(candidate.label(language), color = if (sourceType == candidate) Color.White else Color.White.copy(alpha = 0.45f))
                        }
                    }
                }
                OutlinedTextField(
                    value = location,
                    onValueChange = { location = it },
                    label = { Text(if (sourceType == LocalMediaSourceType.Smb) "smb://server/share/Movies" else "https://server/dav/Movies/") },
                    singleLine = true,
                )
                OutlinedTextField(value = username, onValueChange = { username = it }, label = { Text(AppStrings.t(language, "library.local_media_username_optional")) }, singleLine = true)
                OutlinedTextField(
                    value = password,
                    onValueChange = { password = it },
                    label = { Text(AppStrings.t(language, "library.local_media_password_optional")) },
                    singleLine = true,
                    visualTransformation = PasswordVisualTransformation(),
                )
            }
        },
        confirmButton = {
            TextButton(
                onClick = {
                    onConfirm(LocalMediaSourceInput(kind, sourceType, location.trim(), username = username, password = password))
                },
                enabled = location.isNotBlank(),
            ) { Text(AppStrings.t(language, "library.local_media_add"), color = Color.White) }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text(AppStrings.t(language, "auto.cancel"), color = Color.White.copy(alpha = 0.6f)) } },
        containerColor = Color(0xFF1A1D26),
    )
}

private fun LocalMediaKind.label(language: String?): String = when (this) {
    LocalMediaKind.Movies -> AppStrings.t(language, "library.local_media_movies")
    LocalMediaKind.TvShows -> AppStrings.t(language, "library.local_media_tv_shows")
    LocalMediaKind.Anime -> AppStrings.t(language, "auto.anime")
}

private fun LocalMediaSourceType.label(language: String?): String = when (this) {
    LocalMediaSourceType.LocalFolder -> AppStrings.t(language, "library.local_media_folder")
    LocalMediaSourceType.Smb -> "SMB"
    LocalMediaSourceType.WebDav -> "WebDAV"
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
        LazyColumn(modifier = Modifier.weight(1f), contentPadding = PaddingValues(top = 12.dp, bottom = 120.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
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
    val fieldFocusRequester = remember { FocusRequester() }
    LaunchedEffect(Unit) { runCatching { fieldFocusRequester.requestFocus() } }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(AppStrings.t(language, "auto.collection_name"), color = Color.White) },
        text = {
            OutlinedTextField(
                value = text,
                onValueChange = { text = it },
                singleLine = true,
                modifier = Modifier.focusRequester(fieldFocusRequester)
            )
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
    onItemSelected: (com.fluxa.app.shared.feature.catalog.CatalogItemUiModel) -> Unit,
    onFolderClick: (LibraryFolderUiModel) -> Unit,
    onFolderEditClick: (LibraryFolderUiModel) -> Unit,
    onAddFolderClick: () -> Unit
) {
    Column(modifier = Modifier.fillMaxSize()) {
        LibraryDetailHeader(title = collection.title, onBack = onBack)
        LibraryCollectionFoldersRow(
            folders = collection.folders,
            language = language,
            onFolderClick = onFolderClick,
            onFolderEditClick = onFolderEditClick,
            onAddFolderClick = onAddFolderClick
        )
        LibraryItemGrid(collection.items, language, onItemSelected)
    }
}

@Composable
private fun LibraryCollectionFoldersRow(
    folders: List<LibraryFolderUiModel>,
    language: String?,
    onFolderClick: (LibraryFolderUiModel) -> Unit,
    onFolderEditClick: (LibraryFolderUiModel) -> Unit,
    onAddFolderClick: () -> Unit
) {
    LazyRow(
        contentPadding = PaddingValues(horizontal = 16.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        items(folders, key = { it.id }) { folder ->
            Column(
                modifier = Modifier
                    .clip(RoundedCornerShape(12.dp))
                    .background(Color.White.copy(alpha = 0.05f))
                    .clickable { onFolderClick(folder) }
                    .padding(12.dp)
                    .widthIn(min = 120.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                Text(
                    folder.coverEmoji?.takeIf { it.isNotBlank() } ?: "📁",
                    fontSize = 20.sp
                )
                Text(folder.title, color = Color.White, fontSize = 12.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                TextButton(onClick = { onFolderEditClick(folder) }) {
                    Text(AppStrings.t(language, "auto.edit"), color = Color.White.copy(alpha = 0.5f), fontSize = 11.sp)
                }
            }
        }
        item {
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(12.dp))
                    .background(Color.White.copy(alpha = 0.03f))
                    .clickable(onClick = onAddFolderClick)
                    .padding(16.dp)
                    .widthIn(min = 120.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(AppStrings.t(language, "library.add_folder"), color = Color.White.copy(alpha = 0.7f), fontSize = 12.sp)
            }
        }
    }
}

@Composable
private fun LibraryDetailHeader(title: String, onBack: () -> Unit) {
    Row(modifier = Modifier.fillMaxWidth().padding(16.dp), verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier.size(36.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.06f)).clickable(onClick = onBack),
            contentAlignment = Alignment.Center
        ) {
            Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = null, tint = Color.White, modifier = Modifier.size(20.dp))
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
    LazyColumn(contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 120.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
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
                Icon(Icons.AutoMirrored.Filled.KeyboardArrowRight, contentDescription = null, tint = Color.White.copy(alpha = 0.35f), modifier = Modifier.size(20.dp))
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
        LazyColumn(contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 120.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
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
                        Icon(
                            Icons.Filled.Close,
                            contentDescription = null,
                            tint = FluxaColors.errorRed,
                            modifier = Modifier.size(18.dp).clickable { onCancel(episode.id) }.padding(2.dp)
                        )
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
