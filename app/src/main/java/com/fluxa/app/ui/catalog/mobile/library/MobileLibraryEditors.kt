@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import com.fluxa.app.data.local.*
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import coil3.compose.AsyncImage

@Composable
internal fun LibraryUtilityButton(
    text: String,
    accent: Color,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    onClick: () -> Unit
) {
    Box(
        modifier = modifier
            .height(38.dp)
            .clip(RoundedCornerShape(8.dp))
            .border(1.dp, accent.copy(alpha = if (enabled) 0.68f else 0.24f), RoundedCornerShape(8.dp))
            .background(accent.copy(alpha = if (enabled) 0.08f else 0.03f))
            .clickable(enabled = enabled) { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = text,
            color = if (enabled) accent else Color.White.copy(alpha = 0.32f),
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
    }
}

internal sealed class LibraryCollectionDialogState {
    data object Create : LibraryCollectionDialogState()
    data class Edit(val collection: LibraryCollectionUi) : LibraryCollectionDialogState()
}

@Composable
internal fun LibraryImportCollectionsDialog(
    lang: String,
    accent: Color,
    amoledMode: Boolean,
    onBrowse: () -> Unit,
    onDismiss: () -> Unit,
    onImportJson: (String) -> Unit
) {
    var jsonText by remember { mutableStateOf("") }
    val trimmedJson = jsonText.trim()
    val context = LocalContext.current
    val clipboardManager = remember(context) {
        context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    }
    androidx.compose.material3.AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = if (amoledMode) Color(0xFF050505) else Color(0xFF111620),
        title = {
            Text(
                text = AppStrings.t(lang, "library.import_collections"),
                color = Color.White,
                fontWeight = FontWeight.Bold
            )
        },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(
                    value = jsonText,
                    onValueChange = { jsonText = it },
                    minLines = 6,
                    maxLines = 18,
                    placeholder = { Text(AppStrings.t(lang, "library.paste_json")) },
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = Color.White,
                        unfocusedTextColor = Color.White,
                        focusedBorderColor = accent,
                        unfocusedBorderColor = Color.White.copy(alpha = 0.18f),
                        cursorColor = accent
                    ),
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(min = 180.dp, max = 420.dp)
                )
                LibraryUtilityButton(
                    text = AppStrings.t(lang, "library.paste_from_clipboard"),
                    accent = accent,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    clipboardManager.primaryClip
                        ?.takeIf { it.itemCount > 0 }
                        ?.getItemAt(0)
                        ?.coerceToText(context)
                        ?.toString()
                        ?.let { jsonText = it }
                }
                LibraryUtilityButton(
                    text = AppStrings.t(lang, "library.browse_json_file"),
                    accent = accent,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    onBrowse()
                }
            }
        },
        confirmButton = {
            TextButton(
                enabled = trimmedJson.isNotBlank(),
                onClick = { onImportJson(trimmedJson) }
            ) {
                Text(AppStrings.t(lang, "library.import_collections"), color = if (trimmedJson.isNotBlank()) accent else Color.White.copy(alpha = 0.28f))
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(AppStrings.t(lang, "auto.cancel"), color = Color.White.copy(alpha = 0.72f))
            }
        }
    )
}

@Composable
internal fun LibraryCollectionEditorPage(
    lang: String,
    accent: Color,
    amoledMode: Boolean,
    initial: LibraryUserCollection,
    catalogOptions: List<HomeCategory>,
    userCollections: List<LibraryUserCollection>,
    onImportCollectionsClick: () -> Unit,
    onExportCollectionsClick: () -> Unit,
    onDismiss: () -> Unit,
    onSave: (LibraryUserCollection) -> Unit
) {
    var title by remember(initial.id) { mutableStateOf(initial.title) }
    var imageUrl by remember(initial.id) { mutableStateOf(initial.imageUrl.orEmpty()) }
    var showOnHome by remember(initial.id) { mutableStateOf(initial.showOnHome == true) }
    var folders by remember(initial.id) { mutableStateOf(initial.folders.orEmpty()) }
    var editingFolder by remember { mutableStateOf<LibraryUserCollectionFolder?>(null) }
    fun collectionDraft(
        nextFolders: List<LibraryUserCollectionFolder> = folders,
        nextShowOnHome: Boolean = showOnHome,
        nextImageUrl: String = imageUrl
    ): LibraryUserCollection =
        initial.copy(
            title = title.trim(),
            imageUrl = nextImageUrl.trim().takeIf { it.isNotBlank() },
            showOnHome = nextShowOnHome,
            folders = nextFolders
        )
    val context = LocalContext.current
    val clipboardManager = remember(context) {
        context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    }
    val scope = rememberCoroutineScope()

    Box(
        modifier = Modifier
            .fillMaxSize()
            .zIndex(20f)
            .background(if (amoledMode) Color.Black else Color(0xFF06080D))
    ) {
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Bottom)),
            contentPadding = PaddingValues(16.dp, 18.dp, 16.dp, 180.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            item {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    IconButton(onClick = onDismiss) { Icon(FluxaIcons.ArrowBack, null, tint = Color.White) }
                    Text(
                        text = if (initial.title.isBlank()) AppStrings.t(lang, "auto.new_collection") else AppStrings.t(lang, "auto.edit_collection"),
                        color = Color.White,
                        fontSize = 22.sp,
                        fontWeight = FontWeight.Black,
                        modifier = Modifier.weight(1f)
                    )
                }
            }
            item {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                        LibraryUtilityButton(
                            text = AppStrings.t(lang, "library.import_collections"),
                            accent = accent,
                            modifier = Modifier.weight(1f)
                        ) {
                            onImportCollectionsClick()
                        }
                        LibraryUtilityButton(
                            text = AppStrings.t(lang, "library.export_collections"),
                            accent = accent,
                            modifier = Modifier.weight(1f),
                            enabled = userCollections.isNotEmpty()
                        ) {
                            onExportCollectionsClick()
                        }
                    }
                    LibraryUtilityButton(
                        text = AppStrings.t(lang, "library.copy_collection_json"),
                        accent = accent,
                        modifier = Modifier.fillMaxWidth(),
                        enabled = title.isNotBlank()
                    ) {
                        val label = AppStrings.t(lang, "library.copy_collection_json")
                        val toastMsg = AppStrings.t(lang, "library.copied_collection_json")
                        val draft = collectionDraft()
                        scope.launch {
                            val json = withContext(Dispatchers.Default) {
                                exportLibraryCollectionsJson(listOf(draft))
                            }
                            clipboardManager.setPrimaryClip(ClipData.newPlainText(label, json))
                            android.widget.Toast.makeText(context, toastMsg, android.widget.Toast.LENGTH_SHORT).show()
                        }
                    }
                }
            }
            item {
                OutlinedTextField(
                    value = title,
                    onValueChange = { title = it },
                    placeholder = { Text(AppStrings.t(lang, "auto.collection_name")) },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedBorderColor = accent,
                        unfocusedBorderColor = Color.White.copy(alpha = 0.16f),
                        focusedTextColor = Color.White,
                        unfocusedTextColor = Color.White,
                        focusedLabelColor = accent,
                        unfocusedLabelColor = Color.White.copy(alpha = 0.58f)
                    )
                )
            }
            item {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Icon(if (showOnHome) FluxaIcons.Visibility else FluxaIcons.VisibilityOff, null, tint = Color.White.copy(alpha = 0.82f))
                    Text(
                        text = AppStrings.t(lang, "library.show_above_continue_watching"),
                        color = Color.White,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.weight(1f)
                    )
                    Switch(checked = showOnHome, onCheckedChange = {
                        showOnHome = it
                    })
                }
            }
            item {
                ImageUrlEditor(
                    label = AppStrings.t(lang, "library.collection_image"),
                    imageUrl = imageUrl,
                    accent = accent,
                    onImageUrl = { imageUrl = it },
                    lang = lang
                )
            }
            item {
                Text(
                    text = AppStrings.format(lang, "format.folders_limit", folders.size, 10),
                    color = Color.White.copy(alpha = 0.86f),
                    fontWeight = FontWeight.Black,
                    fontSize = 15.sp
                )
            }
            items(folders, key = { it.id }) { folder ->
                LibraryFolderRow(folder, accent, lang) {
                    editingFolder = folder
                }
            }
            item {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(44.dp)
                        .clip(RoundedCornerShape(8.dp))
                        .border(1.dp, accent.copy(alpha = 0.75f), RoundedCornerShape(8.dp))
                        .clickable(enabled = folders.size < 10) {
                            editingFolder = LibraryUserCollectionFolder(
                                id = "folder_${System.currentTimeMillis()}",
                                title = AppStrings.t(lang, "library.untitled"),
                                shape = "poster"
                            )
                        },
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = AppStrings.t(lang, "library.add_folder"),
                        color = accent,
                        fontWeight = FontWeight.Bold
                    )
                }
            }
            item {
                SaveEditorButton(
                    text = AppStrings.t(lang, "library.save_collection"),
                    enabled = title.isNotBlank(),
                    accent = accent
                ) {
                    if (title.isNotBlank()) {
                        onSave(collectionDraft())
                    }
                }
            }
        }
        editingFolder?.let { folder ->
            LibraryFolderEditorPage(
                lang = lang,
                accent = accent,
                amoledMode = amoledMode,
                initial = folder,
                catalogOptions = catalogOptions,
                onDismiss = { editingFolder = null },
                onSave = { saved ->
                    val nextFolders = (folders.filterNot { it.id == saved.id } + saved).take(10)
                    folders = nextFolders
                    editingFolder = null
                }
            )
        }
    }
}

@Composable
private fun LibraryFolderEditorPage(
    lang: String,
    accent: Color,
    amoledMode: Boolean,
    initial: LibraryUserCollectionFolder,
    catalogOptions: List<HomeCategory>,
    onDismiss: () -> Unit,
    onSave: (LibraryUserCollectionFolder) -> Unit
) {
    var title by remember(initial.id) { mutableStateOf(initial.title) }
    var imageUrl by remember(initial.id) { mutableStateOf(initial.effectiveImageUrl().orEmpty()) }
    var focusGifUrl by remember(initial.id) { mutableStateOf(initial.focusGifUrl.orEmpty()) }
    var titleLogoUrl by remember(initial.id) { mutableStateOf(initial.titleLogoUrl.orEmpty()) }
    var heroBackdropUrl by remember(initial.id) { mutableStateOf(initial.heroBackdropUrl.orEmpty()) }
    var shape by remember(initial.id) { mutableStateOf(initial.shape ?: "poster") }
    var catalogId by remember(initial.id) { mutableStateOf(initial.effectiveCatalogId().orEmpty()) }
    var catalogTitle by remember(initial.id) { mutableStateOf(initial.catalogTitle.orEmpty()) }
    var genre by remember(initial.id) { mutableStateOf(initial.genre.orEmpty()) }
    val selectedCatalog = remember(catalogId, catalogOptions) { catalogOptions.firstOrNull { it.id == catalogId } }
    val genreOptions = remember(selectedCatalog) {
        selectedCatalog?.items.orEmpty()
            .flatMap { it.genres.orEmpty() }
            .distinct()
            .sortedBy { it.lowercase() }
    }
    Box(
        modifier = Modifier
            .fillMaxSize()
            .zIndex(30f)
            .background(if (amoledMode) Color.Black else Color(0xFF080B12))
    ) {
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Bottom)),
            contentPadding = PaddingValues(16.dp, 18.dp, 16.dp, 180.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            item {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    IconButton(onClick = onDismiss) { Icon(FluxaIcons.ArrowBack, null, tint = Color.White) }
                    Text(text = AppStrings.t(lang, "library.folder"), color = Color.White, fontSize = 22.sp, fontWeight = FontWeight.Black)
                }
            }
            item {
                OutlinedTextField(
                    value = title,
                    onValueChange = { title = it },
                    placeholder = { Text(AppStrings.t(lang, "library.folder_name")) },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    colors = OutlinedTextFieldDefaults.colors(focusedBorderColor = accent, unfocusedBorderColor = Color.White.copy(alpha = 0.16f), focusedTextColor = Color.White, unfocusedTextColor = Color.White)
                )
            }
            item { ImageUrlEditor(AppStrings.t(lang, "library.folder_image"), imageUrl, accent, { imageUrl = it }, lang) }
            item { ImageUrlEditor(AppStrings.t(lang, "library.folder_focus_gif"), focusGifUrl, accent, { focusGifUrl = it }, lang) }
            item { ImageUrlEditor(AppStrings.t(lang, "library.folder_title_logo"), titleLogoUrl, accent, { titleLogoUrl = it }, lang) }
            item { ImageUrlEditor(AppStrings.t(lang, "library.folder_hero_backdrop"), heroBackdropUrl, accent, { heroBackdropUrl = it }, lang) }
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    listOf("poster" to "Poster", "square" to "Square", "wide" to "Wide").forEach { option ->
                        Box(
                            modifier = Modifier
                                .weight(1f)
                                .height(38.dp)
                                .clip(RoundedCornerShape(8.dp))
                                .background(if (shape == option.first) accent else Color.White.copy(alpha = 0.08f))
                                .clickable { shape = option.first },
                            contentAlignment = Alignment.Center
                        ) {
                            Text(
                                option.second,
                                color = if (shape == option.first) contrastOn(accent) else Color.White,
                                fontWeight = FontWeight.Bold,
                                fontSize = 12.sp
                            )
                        }
                    }
                }
            }
            item {
                Text(text = AppStrings.t(lang, "library.catalog"), color = Color.White.copy(alpha = 0.82f), fontWeight = FontWeight.Black)
            }
            items(catalogOptions.filterNot { it.isContinueWatchingCategory() || it.type == "collection" }, key = { it.id }) { catalog ->
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .background(if (catalogId == catalog.id) accent else Color.White.copy(alpha = 0.06f))
                        .clickable {
                            catalogId = catalog.id
                            catalogTitle = catalog.name
                            genre = ""
                        }
                        .padding(12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    val selectedColor = if (catalogId == catalog.id) contrastOn(accent) else Color.White
                    Text(catalog.name, color = selectedColor, fontWeight = FontWeight.Bold, modifier = Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis)
                    if (catalogId == catalog.id) Icon(FluxaIcons.CheckCircle, null, tint = selectedColor, modifier = Modifier.size(18.dp))
                }
            }
            if (genreOptions.isNotEmpty()) {
                item {
                    Text(text = AppStrings.t(lang, "auto.genre"), color = Color.White.copy(alpha = 0.82f), fontWeight = FontWeight.Black)
                }
                items(genreOptions, key = { it }) { option ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(8.dp))
                            .background(if (genre == option) accent else Color.White.copy(alpha = 0.06f))
                            .clickable { genre = option }
                            .padding(12.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        val selectedColor = if (genre == option) contrastOn(accent) else Color.White
                        Text(option, color = selectedColor, fontWeight = FontWeight.Bold, modifier = Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis)
                        if (genre == option) Icon(FluxaIcons.CheckCircle, null, tint = selectedColor, modifier = Modifier.size(18.dp))
                    }
                }
            }
            item {
                SaveEditorButton(
                    text = AppStrings.t(lang, "library.save_folder"),
                    enabled = title.isNotBlank() && catalogId.isNotBlank(),
                    accent = accent
                ) {
                    val genreSuffix = genre.takeIf { it.isNotBlank() }?.let { " - $it" }.orEmpty()
                    onSave(
                        initial.copy(
                            title = title.trim(),
                            imageUrl = imageUrl.trim().takeIf { it.isNotBlank() },
                            shape = shape,
                            catalogId = catalogId,
                            catalogTitle = "$catalogTitle$genreSuffix",
                            genre = genre.takeIf { it.isNotBlank() },
                            catalogSources = listOf(
                                LibraryCatalogSource(
                                    catalogId = catalogId,
                                    type = selectedCatalog?.type ?: initial.effectiveCatalogType() ?: "movie"
                                )
                            ),
                            coverImageUrl = imageUrl.trim().takeIf { it.isNotBlank() }
                                ?: initial.coverImageUrl,
                            focusGifUrl = focusGifUrl.trim().takeIf { it.isNotBlank() },
                            titleLogoUrl = titleLogoUrl.trim().takeIf { it.isNotBlank() },
                            heroBackdropUrl = heroBackdropUrl.trim().takeIf { it.isNotBlank() }
                        )
                    )
                }
            }
        }
    }
}

@Composable
private fun SaveEditorButton(text: String, enabled: Boolean, accent: Color, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(48.dp)
            .clip(RoundedCornerShape(8.dp))
            .background(if (enabled) accent else Color.White.copy(alpha = 0.12f))
            .clickable(enabled = enabled) { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = text,
            color = if (enabled) contrastOn(accent) else Color.White.copy(alpha = 0.62f),
            fontWeight = FontWeight.Black
        )
    }
}

private fun contrastOn(color: Color): Color {
    return if (color.luminance() > 0.62f) Color.Black else Color.White
}

@Composable
private fun ImageUrlEditor(label: String, imageUrl: String, accent: Color, onImageUrl: (String) -> Unit, lang: String? = null) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (imageUrl.isNotBlank()) {
            AsyncImage(
                model = imageUrl,
                contentDescription = null,
                modifier = Modifier.fillMaxWidth().height(132.dp).clip(RoundedCornerShape(8.dp)),
                contentScale = ContentScale.Crop
            )
        }
        OutlinedTextField(
            value = imageUrl,
            onValueChange = onImageUrl,
            placeholder = { Text(label) },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            colors = OutlinedTextFieldDefaults.colors(focusedBorderColor = accent, unfocusedBorderColor = Color.White.copy(alpha = 0.16f), focusedTextColor = Color.White, unfocusedTextColor = Color.White)
        )
    }
}

@Composable
private fun LibraryFolderRow(folder: LibraryUserCollectionFolder, accent: Color, lang: String?, onClick: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(Color.White.copy(alpha = 0.06f)).clickable { onClick() }.padding(10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        AsyncImage(folder.effectiveImageUrl(), null, modifier = Modifier.size(50.dp).clip(RoundedCornerShape(8.dp)), contentScale = ContentScale.Crop)
        Column(modifier = Modifier.weight(1f)) {
            Text(folder.title.ifBlank { AppStrings.t(lang, "library.untitled") }, color = Color.White, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(folder.catalogTitle.orEmpty(), color = Color.White.copy(alpha = 0.52f), fontSize = 11.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        Text(folder.effectiveShape().uppercase(), color = accent, fontSize = 10.sp, fontWeight = FontWeight.Black)
    }
}

@Composable
internal fun LibraryCollectionNameDialog(
    lang: String,
    accent: Color,
    amoledMode: Boolean,
    initialTitle: String,
    onDismiss: () -> Unit,
    onConfirm: (String) -> Unit
) {
    var title by remember(initialTitle) { mutableStateOf(initialTitle) }
    val trimmedTitle = title.trim()
    androidx.compose.material3.AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = if (amoledMode) Color(0xFF050505) else Color(0xFF111620),
        title = {
            Text(
                text = if (initialTitle.isBlank()) AppStrings.t(lang, "auto.new_collection") else AppStrings.t(lang, "auto.edit_collection"),
                color = Color.White,
                fontWeight = FontWeight.Bold
            )
        },
        text = {
            OutlinedTextField(
                value = title,
                onValueChange = { title = it.take(40) },
                singleLine = true,
                placeholder = { Text(AppStrings.t(lang, "auto.collection_name")) },
                colors = OutlinedTextFieldDefaults.colors(
                    focusedTextColor = Color.White,
                    unfocusedTextColor = Color.White,
                    focusedBorderColor = accent,
                    unfocusedBorderColor = Color.White.copy(alpha = 0.18f),
                    focusedLabelColor = accent,
                    unfocusedLabelColor = Color.White.copy(alpha = 0.52f),
                    cursorColor = accent
                )
            )
        },
        confirmButton = {
            TextButton(
                enabled = trimmedTitle.isNotBlank(),
                onClick = { onConfirm(trimmedTitle) }
            ) {
                Text(AppStrings.t(lang, "auto.save"), color = if (trimmedTitle.isNotBlank()) accent else Color.White.copy(alpha = 0.28f))
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(AppStrings.t(lang, "auto.cancel"), color = Color.White.copy(alpha = 0.72f))
            }
        }
    )
}
