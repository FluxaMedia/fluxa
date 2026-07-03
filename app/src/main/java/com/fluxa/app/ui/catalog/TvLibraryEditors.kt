@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.LibraryCatalogSource
import com.fluxa.app.data.local.LibraryUserCollection
import com.fluxa.app.data.local.LibraryUserCollectionFolder

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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.Surface
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import androidx.compose.ui.platform.LocalContext

@Composable
internal fun TvCollectionEditorPage(
    lang: String,
    initial: LibraryUserCollection,
    catalogOptions: List<HomeCategory>,
    onDismiss: () -> Unit,
    onSave: (LibraryUserCollection) -> Unit
) {
    var title by remember(initial.id) { mutableStateOf(initial.title) }
    var imageUrl by remember(initial.id) { mutableStateOf(initial.imageUrl.orEmpty()) }
    var showOnHome by remember(initial.id) { mutableStateOf(initial.showOnHome == true) }
    var folders by remember(initial.id) { mutableStateOf(initial.folders.orEmpty()) }
    var editingFolder by remember { mutableStateOf<LibraryUserCollectionFolder?>(null) }

    Box(modifier = Modifier.fillMaxSize().background(Color.Black)) {
        LazyColumn(
            modifier = Modifier.fillMaxSize().padding(horizontal = 58.dp, vertical = 40.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            item {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    IconButton(onClick = onDismiss) { Icon(FluxaIcons.ArrowBack, null, tint = Color.White) }
                    Text(
                        text = if (initial.title.isBlank()) AppStrings.t(lang, "auto.new_collection") else AppStrings.t(lang, "auto.edit_collection"),
                        color = Color.White,
                        fontSize = 26.sp,
                        fontWeight = FontWeight.Black
                    )
                }
            }
            item {
                OutlinedTextField(
                    value = title,
                    onValueChange = { title = it },
                    label = { Text(AppStrings.t(lang, "auto.collection_name")) },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    colors = tvEditorFieldColors()
                )
            }
            item {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Text(
                        text = AppStrings.t(lang, "library.show_above_continue_watching"),
                        color = Color.White,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.weight(1f)
                    )
                    Switch(checked = showOnHome, onCheckedChange = { showOnHome = it })
                }
            }
            item {
                TvImageUrlField(AppStrings.t(lang, "library.collection_image"), imageUrl) { imageUrl = it }
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
                TvFolderRow(folder) { editingFolder = folder }
            }
            item {
                Surface(
                    onClick = {
                        if (folders.size < 10) {
                            editingFolder = LibraryUserCollectionFolder(
                                id = "folder_${System.currentTimeMillis()}",
                                title = AppStrings.t(lang, "library.untitled"),
                                shape = "poster"
                            )
                        }
                    },
                    modifier = Modifier.fillMaxWidth().height(48.dp),
                    shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(10.dp)),
                    colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(alpha = 0.08f))
                ) {
                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Text(AppStrings.t(lang, "library.add_folder"), color = Color.White, fontWeight = FontWeight.Bold)
                    }
                }
            }
            item {
                TvSaveButton(
                    text = AppStrings.t(lang, "library.save_collection"),
                    enabled = title.isNotBlank()
                ) {
                    onSave(
                        initial.copy(
                            title = title.trim(),
                            imageUrl = imageUrl.trim().takeIf { it.isNotBlank() },
                            showOnHome = showOnHome,
                            folders = folders
                        )
                    )
                }
            }
        }

        editingFolder?.let { folder ->
            TvFolderEditorPage(
                lang = lang,
                initial = folder,
                catalogOptions = catalogOptions,
                onDismiss = { editingFolder = null },
                onSave = { saved ->
                    folders = (folders.filterNot { it.id == saved.id } + saved).take(10)
                    editingFolder = null
                }
            )
        }
    }
}

@Composable
private fun TvFolderEditorPage(
    lang: String,
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

    Box(modifier = Modifier.fillMaxSize().background(FluxaColors.backgroundAmoled)) {
        LazyColumn(
            modifier = Modifier.fillMaxSize().padding(horizontal = 58.dp, vertical = 40.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            item {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    IconButton(onClick = onDismiss) { Icon(FluxaIcons.ArrowBack, null, tint = Color.White) }
                    Text(AppStrings.t(lang, "library.folder"), color = Color.White, fontSize = 26.sp, fontWeight = FontWeight.Black)
                }
            }
            item {
                OutlinedTextField(
                    value = title,
                    onValueChange = { title = it },
                    label = { Text(AppStrings.t(lang, "library.folder_name")) },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    colors = tvEditorFieldColors()
                )
            }
            item { TvImageUrlField(AppStrings.t(lang, "library.folder_image"), imageUrl) { imageUrl = it } }
            item { TvImageUrlField(AppStrings.t(lang, "library.folder_focus_gif"), focusGifUrl) { focusGifUrl = it } }
            item { TvImageUrlField(AppStrings.t(lang, "library.folder_title_logo"), titleLogoUrl) { titleLogoUrl = it } }
            item { TvImageUrlField(AppStrings.t(lang, "library.folder_hero_backdrop"), heroBackdropUrl) { heroBackdropUrl = it } }
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    listOf("poster" to "Poster", "square" to "Square", "wide" to "Wide").forEach { option ->
                        Surface(
                            onClick = { shape = option.first },
                            modifier = Modifier.weight(1f).height(40.dp),
                            shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(8.dp)),
                            colors = ClickableSurfaceDefaults.colors(
                                containerColor = if (shape == option.first) Color.White else Color.White.copy(alpha = 0.08f)
                            )
                        ) {
                            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                Text(
                                    option.second,
                                    color = if (shape == option.first) Color.Black else Color.White,
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 13.sp
                                )
                            }
                        }
                    }
                }
            }
            item {
                Text(AppStrings.t(lang, "library.catalog"), color = Color.White.copy(alpha = 0.82f), fontWeight = FontWeight.Black)
            }
            items(catalogOptions.filterNot { it.isContinueWatchingCategory() || it.type == "collection" }, key = { it.id }) { catalog ->
                TvPickerRow(catalog.name, catalogId == catalog.id) {
                    catalogId = catalog.id
                    catalogTitle = catalog.name
                    genre = ""
                }
            }
            if (genreOptions.isNotEmpty()) {
                item {
                    Text(AppStrings.t(lang, "auto.genre"), color = Color.White.copy(alpha = 0.82f), fontWeight = FontWeight.Black)
                }
                items(genreOptions, key = { it }) { option ->
                    TvPickerRow(option, genre == option) { genre = option }
                }
            }
            item {
                TvSaveButton(
                    text = AppStrings.t(lang, "library.save_folder"),
                    enabled = title.isNotBlank() && catalogId.isNotBlank()
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
                            coverImageUrl = imageUrl.trim().takeIf { it.isNotBlank() } ?: initial.coverImageUrl,
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
private fun TvImageUrlField(label: String, value: String, onValueChange: (String) -> Unit) {
    val context = LocalContext.current
    val previewRequest = remember(value) {
        ImageRequest.Builder(context).data(value).memoryCacheKey("tv-library-url:$value").diskCacheKey(value).build()
    }
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (value.isNotBlank()) {
            AsyncImage(
                model = previewRequest,
                contentDescription = null,
                modifier = Modifier.fillMaxWidth().height(140.dp).clip(RoundedCornerShape(10.dp)),
                contentScale = ContentScale.Crop
            )
        }
        OutlinedTextField(
            value = value,
            onValueChange = onValueChange,
            label = { Text(label) },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            colors = tvEditorFieldColors()
        )
    }
}

@Composable
private fun TvFolderRow(folder: LibraryUserCollectionFolder, onClick: () -> Unit) {
    val context = LocalContext.current
    val folderImageUrl = folder.effectiveImageUrl()
    val folderImageRequest = remember(folder.id, folderImageUrl) {
        ImageRequest.Builder(context).data(folderImageUrl).memoryCacheKey("tv-library-folder:${folder.id}").diskCacheKey(folderImageUrl).build()
    }
    var focused by remember { mutableStateOf(false) }
    Surface(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth().height(64.dp).onFocusChanged { focused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(10.dp)),
        colors = ClickableSurfaceDefaults.colors(containerColor = if (focused) Color.White.copy(alpha = 0.16f) else Color.White.copy(alpha = 0.06f))
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            AsyncImage(
                model = folderImageRequest,
                contentDescription = null,
                modifier = Modifier.height(44.dp).clip(RoundedCornerShape(8.dp)),
                contentScale = ContentScale.Crop
            )
            Column(modifier = Modifier.weight(1f)) {
                Text(folder.title, color = Color.White, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(folder.catalogTitle.orEmpty(), color = Color.White.copy(alpha = 0.52f), fontSize = 12.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}

@Composable
private fun TvPickerRow(label: String, selected: Boolean, onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    Surface(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth().height(48.dp).onFocusChanged { focused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(8.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = when {
                selected -> Color.White
                focused -> Color.White.copy(alpha = 0.16f)
                else -> Color.White.copy(alpha = 0.06f)
            }
        )
    ) {
        Row(modifier = Modifier.fillMaxSize().padding(horizontal = 14.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(
                label,
                color = if (selected) Color.Black else Color.White,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.weight(1f),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            if (selected) Icon(FluxaIcons.CheckCircle, null, tint = Color.Black, modifier = Modifier.height(18.dp))
        }
    }
}

@Composable
private fun TvSaveButton(text: String, enabled: Boolean, onClick: () -> Unit) {
    Surface(
        onClick = onClick,
        enabled = enabled,
        modifier = Modifier.fillMaxWidth().height(52.dp),
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(10.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = if (enabled) Color.White else Color.White.copy(alpha = 0.12f)
        )
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(text, color = if (enabled) Color.Black else Color.White.copy(alpha = 0.62f), fontWeight = FontWeight.Black)
        }
    }
}

@Composable
private fun tvEditorFieldColors() = OutlinedTextFieldDefaults.colors(
    focusedBorderColor = Color.White,
    unfocusedBorderColor = Color.White.copy(alpha = 0.18f),
    focusedTextColor = Color.White,
    unfocusedTextColor = Color.White,
    cursorColor = Color.White,
    focusedContainerColor = Color.Transparent,
    unfocusedContainerColor = Color.Transparent
)
