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
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.zIndex
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import androidx.compose.runtime.collectAsState
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
internal fun MobileLibraryDashboard(
    lang: String,
    accent: Color,
    amoledMode: Boolean,
    selectedType: String,
    onSelectType: (String) -> Unit,
    selectedSection: String,
    onSelectSection: (String) -> Unit,
    plannedItems: List<Meta>,
    completedItems: List<Meta>,
    favoriteItems: List<Meta>,
    plannedCount: Int,
    completedCount: Int,
    downloadedCount: Int,
    favoriteCount: Int,
    collections: List<LibraryCollectionUi>,
    userCollections: List<LibraryUserCollection> = emptyList(),
    categoriesFlow: StateFlow<List<HomeCategory>>? = null,
    onMovieClick: (Meta) -> Unit,
    onCollectionClick: (LibraryCollectionUi) -> Unit,
    onCreateCollection: (String) -> Unit,
    onSaveUserCollection: (LibraryUserCollection) -> Unit = {},
    onImportCollections: (List<LibraryUserCollection>) -> Unit = {},
    onRenameCollection: (LibraryCollectionUi, String) -> Unit,
    onDeleteCollection: (LibraryCollectionUi) -> Unit
) {
    var editMode by remember { mutableStateOf(false) }
    var collectionDialog by remember { mutableStateOf<LibraryCollectionDialogState?>(null) }
    var editingCollection by remember { mutableStateOf<LibraryUserCollection?>(null) }
    var pendingExportJson by remember { mutableStateOf<String?>(null) }
    var showImportDialog by remember { mutableStateOf(false) }
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    fun importCollectionsFromJson(rawJson: String) {
        runCatching { importLibraryCollectionsJson(rawJson) }
            .onSuccess { imported ->
                if (imported.isNotEmpty()) {
                    onImportCollections(imported)
                    showImportDialog = false
                    android.widget.Toast.makeText(context, AppStrings.format(lang, "library.imported_collections", imported.size), android.widget.Toast.LENGTH_SHORT).show()
                } else {
                    android.widget.Toast.makeText(context, AppStrings.t(lang, "library.import_failed"), android.widget.Toast.LENGTH_SHORT).show()
                }
            }
            .onFailure {
                android.widget.Toast.makeText(context, AppStrings.t(lang, "library.import_failed"), android.widget.Toast.LENGTH_SHORT).show()
            }
    }
    val importLauncher = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) {
            scope.launch {
                runCatching {
                    withContext(Dispatchers.IO) {
                        context.contentResolver.openInputStream(uri)?.bufferedReader()?.use { it.readText() }.orEmpty()
                    }
                }
                    .onSuccess(::importCollectionsFromJson)
                    .onFailure {
                        android.widget.Toast.makeText(context, AppStrings.t(lang, "library.import_failed"), android.widget.Toast.LENGTH_SHORT).show()
                    }
            }
        }
    }
    val exportLauncher = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("application/json")) { uri ->
        val json = pendingExportJson
        pendingExportJson = null
        if (uri != null && json != null) {
            scope.launch {
                runCatching {
                    withContext(Dispatchers.IO) {
                        context.contentResolver.openOutputStream(uri)?.bufferedWriter()?.use { it.write(json) }
                    }
                }.onSuccess {
                    android.widget.Toast.makeText(context, AppStrings.t(lang, "library.exported_collections"), android.widget.Toast.LENGTH_SHORT).show()
                }.onFailure {
                    android.widget.Toast.makeText(context, AppStrings.t(lang, "library.export_failed"), android.widget.Toast.LENGTH_SHORT).show()
                }
            }
        }
    }
    val backgroundColor = if (amoledMode) Color.Black else Color(0xFF05070B)
    val cardColor = if (amoledMode) FluxaColors.backgroundAmoled else FluxaColors.backgroundNavy.copy(alpha = 0.82f)
    val plannedChunked = remember(plannedItems) { plannedItems.chunked(3) }
    val completedChunked = remember(completedItems) { completedItems.chunked(3) }
    val favoritesChunked = remember(favoriteItems) { favoriteItems.chunked(3) }

    LazyColumn(
        modifier = Modifier
            .fillMaxSize()
            .background(backgroundColor),
        contentPadding = PaddingValues(start = 10.dp, end = 10.dp, top = 22.dp, bottom = 132.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp)
    ) {
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = AppStrings.t(lang, "nav.library"),
                    color = Color.White,
                    fontSize = 22.sp,
                    fontWeight = FontWeight.Black,
                    modifier = Modifier.weight(1f)
                )
            }
        }

        item {
            val libChips = listOf(
                Triple("planned", AppStrings.t(lang, "auto.planned"), plannedCount),
                Triple("completed", AppStrings.t(lang, "auto.completed"), completedCount),
                Triple("favorites", AppStrings.t(lang, "auto.favorites"), favoriteCount),
                Triple("downloads", AppStrings.t(lang, "auto.downloads"), downloadedCount),
                Triple("collections", AppStrings.t(lang, "auto.my_collections"), collections.size)
            )
            LazyRow(
                contentPadding = PaddingValues(horizontal = 2.dp),
                horizontalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                items(libChips, key = { (k, _, _) -> k }) { (key, label, count) ->
                    val selected = selectedSection == key
                    val onAccentColor = if (accent.luminance() > 0.5f) Color.Black else Color.White
                    val chipBackground by animateColorAsState(
                        targetValue = if (selected) accent else cardColor,
                        animationSpec = tween(FluxaDimensions.AnimDuration.scaleAlpha, easing = FastOutSlowInEasing),
                        label = "libraryChipBackground"
                    )
                    val chipBorder by animateColorAsState(
                        targetValue = if (selected) accent else Color.White.copy(0.1f),
                        animationSpec = tween(FluxaDimensions.AnimDuration.scaleAlpha, easing = FastOutSlowInEasing),
                        label = "libraryChipBorder"
                    )
                    val chipScale by animateFloatAsState(
                        targetValue = if (selected) 1.04f else 1f,
                        animationSpec = tween(FluxaDimensions.AnimDuration.scaleAlpha, easing = FastOutSlowInEasing),
                        label = "libraryChipScale"
                    )
                    Box(
                        modifier = Modifier
                            .animateItem()
                            .graphicsLayer {
                                scaleX = chipScale
                                scaleY = chipScale
                            }
                            .clip(RoundedCornerShape(999.dp))
                            .background(chipBackground)
                            .border(1.dp, chipBorder, RoundedCornerShape(999.dp))
                            .clickable { onSelectSection(key) }
                            .padding(horizontal = 18.dp, vertical = 11.dp)
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(6.dp)
                        ) {
                            Text(
                                text = label,
                                color = if (selected) onAccentColor else Color.White.copy(0.85f),
                                fontSize = 14.sp,
                                fontWeight = FontWeight.Bold
                            )
                            if (count > 0) {
                                Text(
                                    text = count.toString(),
                                    color = (if (selected) onAccentColor else Color.White).copy(alpha = 0.55f),
                                    fontSize = 12.sp,
                                    fontWeight = FontWeight.SemiBold
                                )
                            }
                        }
                    }
                }
            }
        }

        if (selectedSection == "collections") {
            item {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = AppStrings.t(lang, "auto.my_collections"),
                        color = Color.White.copy(alpha = 0.86f),
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.weight(1f)
                    )
                    Text(
                        text = if (editMode) AppStrings.t(lang, "auto.done") else AppStrings.t(lang, "auto.edit"),
                        color = accent,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier
                            .clip(RoundedCornerShape(8.dp))
                            .clickable(enabled = collections.any { it.userCollectionId != null }) { editMode = !editMode }
                            .padding(horizontal = 10.dp, vertical = 7.dp)
                    )
                }
            }
            if (collections.isEmpty()) {
                item {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 24.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = AppStrings.t(lang, "auto.no_collections_yet"),
                            color = Color.White.copy(alpha = 0.48f),
                            fontSize = 14.sp
                        )
                    }
                }
            } else {
                items(collections, key = { it.userCollectionId ?: it.title }) { collection ->
                    LibraryCollectionRow(
                        collection = collection,
                        amoledMode = amoledMode,
                        editMode = editMode,
                        onEdit = { col ->
                            editingCollection = userCollections.firstOrNull { it.id == col.userCollectionId }
                        },
                        onDelete = { onDeleteCollection(it) },
                        onClick = { onCollectionClick(collection) }
                    )
                }
            }
            item {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(42.dp)
                        .clip(RoundedCornerShape(8.dp))
                        .border(1.dp, accent.copy(alpha = 0.78f), RoundedCornerShape(8.dp))
                        .background(accent.copy(alpha = if (amoledMode) 0.08f else 0.05f))
                        .clickable {
                            editingCollection = LibraryUserCollection(
                                id = "local_${System.currentTimeMillis()}",
                                title = ""
                            )
                        },
                    contentAlignment = Alignment.Center
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Icon(FluxaIcons.Add, null, tint = accent, modifier = Modifier.size(16.dp))
                        Text(
                            text = AppStrings.t(lang, "auto.new_collection"),
                            color = accent,
                            fontSize = 13.sp,
                            fontWeight = FontWeight.Bold
                        )
                    }
                }
            }
        } else {
            val sectionItems = when (selectedSection) {
                "planned" -> plannedItems
                "completed" -> completedItems
                "favorites" -> favoriteItems
                else -> emptyList()
            }
            val chunkedItems = when (selectedSection) {
                "planned" -> plannedChunked
                "completed" -> completedChunked
                "favorites" -> favoritesChunked
                else -> emptyList()
            }
            if (sectionItems.isEmpty() && selectedSection != "downloads") {
                item {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 48.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = AppStrings.t(lang, "library.empty"),
                            color = Color.White.copy(alpha = 0.48f),
                            fontSize = 14.sp
                        )
                    }
                }
            } else {
                itemsIndexed(chunkedItems, key = { _, rowItems -> rowItems.joinToString(",") { "${it.type}:${it.id}" } }) { _, rowItems ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = 4.dp),
                        horizontalArrangement = Arrangement.spacedBy(10.dp)
                    ) {
                        rowItems.forEach { item ->
                            LibraryPosterGridCard(
                                item = item,
                                modifier = Modifier.weight(1f),
                                onClick = { onMovieClick(item) }
                            )
                        }
                        repeat(3 - rowItems.size) {
                            Spacer(modifier = Modifier.weight(1f))
                        }
                    }
                }
            }
        }
    }

    collectionDialog?.let { state ->
        LibraryCollectionNameDialog(
            lang = lang,
            accent = accent,
            amoledMode = amoledMode,
            initialTitle = (state as? LibraryCollectionDialogState.Edit)?.collection?.title.orEmpty(),
            onDismiss = { collectionDialog = null },
            onConfirm = { title ->
                when (state) {
                    LibraryCollectionDialogState.Create -> onCreateCollection(title)
                    is LibraryCollectionDialogState.Edit -> onRenameCollection(state.collection, title)
                }
                collectionDialog = null
            }
        )
    }
    if (showImportDialog) {
        LibraryImportCollectionsDialog(
            lang = lang,
            accent = accent,
            amoledMode = amoledMode,
            onBrowse = { importLauncher.launch(arrayOf("application/json", "text/*", "*/*")) },
            onDismiss = { showImportDialog = false },
            onImportJson = ::importCollectionsFromJson
        )
    }
    editingCollection?.let { draft ->
        val catalogOptions by (categoriesFlow ?: kotlinx.coroutines.flow.MutableStateFlow(emptyList())).collectAsStateWithLifecycle()
        LibraryCollectionEditorPage(
            lang = lang,
            accent = accent,
            amoledMode = amoledMode,
            initial = draft,
            catalogOptions = catalogOptions,
            userCollections = userCollections,
            onImportCollectionsClick = { showImportDialog = true },
            onExportCollectionsClick = {
                scope.launch {
                    pendingExportJson = withContext(Dispatchers.Default) {
                        exportLibraryCollectionsJson(userCollections)
                    }
                    exportLauncher.launch("fluxa-collections.json")
                }
            },
            onDismiss = { editingCollection = null },
            onSave = {
                onSaveUserCollection(it)
                editingCollection = null
            }
        )
    }
}

@Composable
private fun LibraryContinueInlineCard(item: Meta, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(Color.White.copy(alpha = 0.07f), RoundedCornerShape(8.dp))
            .clickable { onClick() }
            .padding(10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        AsyncImage(
            model = item.continueWatchingPoster,
            contentDescription = item.name,
            modifier = Modifier
                .width(112.dp)
                .height(64.dp)
                .clip(RoundedCornerShape(7.dp)),
            contentScale = ContentScale.Crop
        )
        Column(modifier = Modifier.weight(1f)) {
            Text(item.name, color = Color.White, fontWeight = FontWeight.Black, fontSize = 14.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
            val subtitle = item.lastEpisodeName ?: item.reason ?: item.releaseInfo
            if (!subtitle.isNullOrBlank()) {
                Text(subtitle, color = Color.White.copy(alpha = 0.58f), fontSize = 12.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            val duration = item.duration ?: 0L
            val progress = item.timeOffset ?: 0L
            if (duration > 0L && progress > 0L) {
                Box(
                    modifier = Modifier
                        .padding(top = 8.dp)
                        .fillMaxWidth()
                        .height(3.dp)
                        .clip(RoundedCornerShape(99.dp))
                        .background(Color.White.copy(alpha = 0.14f))
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth((progress.toFloat() / duration.toFloat()).coerceIn(0f, 1f))
                            .height(3.dp)
                            .background(Color.White)
                    )
                }
            }
        }
        Icon(FluxaIcons.KeyboardArrowRight, null, tint = Color.White.copy(alpha = 0.42f), modifier = Modifier.size(20.dp))
    }
}
