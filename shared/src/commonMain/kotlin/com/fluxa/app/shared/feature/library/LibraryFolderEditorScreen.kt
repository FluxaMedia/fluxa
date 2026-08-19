package com.fluxa.app.shared.feature.library

import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun LibraryFolderEditorScreen(
    state: LibraryFolderEditorUiState,
    language: String?,
    catalogOptions: List<LibraryCatalogOptionUiModel>,
    onDraftChanged: (LibraryFolderEditorUiModel) -> Unit,
    onSaveRequested: () -> Unit,
    onDeleteRequested: () -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier
) {
    val draft = state.draft

    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
            LazyColumn(
                modifier = Modifier.fillMaxWidth(0.96f).widthIn(max = 700.dp),
                contentPadding = PaddingValues(horizontal = 16.dp, vertical = 20.dp),
                verticalArrangement = Arrangement.spacedBy(18.dp)
            ) {
                item {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(18.dp)) {
                        Box(
                            modifier = Modifier
                                .size(40.dp)
                                .clip(CircleShape)
                                .background(Color.White.copy(alpha = 0.05f))
                                .clickable(onClick = onBack),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(Icons.AutoMirrored.Filled.ArrowBack, null, tint = Color.White)
                        }
                        Text(
                            text = AppStrings.t(
                                language,
                                if (draft.id != null) "library.folder_editor.edit_title" else "library.folder_editor.new_title"
                            ),
                            color = Color.White,
                            fontSize = 20.sp
                        )
                    }
                }

                item {
                    LabeledField(AppStrings.t(language, "library.folder_name")) {
                        OutlinedTextField(
                            value = draft.title,
                            onValueChange = { onDraftChanged(draft.copy(title = it)) },
                            singleLine = true,
                            colors = fieldColors(),
                            modifier = Modifier.fillMaxWidth()
                        )
                    }
                }

                item {
                    LabeledField(AppStrings.t(language, "library.folder_editor.cover_emoji")) {
                        OutlinedTextField(
                            value = draft.coverEmoji.orEmpty(),
                            onValueChange = { onDraftChanged(draft.copy(coverEmoji = it.take(4))) },
                            singleLine = true,
                            colors = fieldColors(),
                            modifier = Modifier.fillMaxWidth()
                        )
                    }
                }

                item {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(
                            text = AppStrings.t(language, "library.folder_editor.source"),
                            color = Color.White.copy(alpha = 0.6f),
                            fontSize = 12.sp
                        )
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            SourceTypeChip(
                                label = AppStrings.t(language, "library.folder_editor.source_tmdb"),
                                selected = draft.sourceKind == LibraryFolderSourceKind.Tmdb
                            ) { onDraftChanged(draft.copy(sourceKind = LibraryFolderSourceKind.Tmdb)) }
                            SourceTypeChip(
                                label = AppStrings.t(language, "library.folder_editor.source_trakt"),
                                selected = draft.sourceKind == LibraryFolderSourceKind.Trakt
                            ) { onDraftChanged(draft.copy(sourceKind = LibraryFolderSourceKind.Trakt)) }
                            SourceTypeChip(
                                label = AppStrings.t(language, "library.folder_editor.source_catalog"),
                                selected = draft.sourceKind == LibraryFolderSourceKind.AddonCatalog
                            ) { onDraftChanged(draft.copy(sourceKind = LibraryFolderSourceKind.AddonCatalog)) }
                        }
                    }
                }

                when (draft.sourceKind) {
                    LibraryFolderSourceKind.Tmdb -> {
                        item {
                            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                SourceTypeChip(
                                    label = AppStrings.t(language, "library.folder_editor.tmdb_list"),
                                    selected = draft.tmdbSourceType == "LIST"
                                ) { onDraftChanged(draft.copy(tmdbSourceType = "LIST")) }
                                SourceTypeChip(
                                    label = AppStrings.t(language, "library.folder_editor.tmdb_collection"),
                                    selected = draft.tmdbSourceType == "COLLECTION"
                                ) { onDraftChanged(draft.copy(tmdbSourceType = "COLLECTION")) }
                            }
                        }
                        item {
                            LabeledField(AppStrings.t(language, "library.folder_editor.tmdb_id")) {
                                OutlinedTextField(
                                    value = draft.tmdbId,
                                    onValueChange = { value -> onDraftChanged(draft.copy(tmdbId = value.filter { it.isDigit() })) },
                                    placeholder = { Text(AppStrings.t(language, "library.folder_editor.tmdb_id_placeholder"), color = Color.White.copy(alpha = 0.35f)) },
                                    singleLine = true,
                                    colors = fieldColors(),
                                    modifier = Modifier.fillMaxWidth()
                                )
                            }
                        }
                    }
                    LibraryFolderSourceKind.Trakt -> {
                        item {
                            LabeledField(AppStrings.t(language, "library.folder_editor.trakt_input")) {
                                OutlinedTextField(
                                    value = draft.traktInput,
                                    onValueChange = { value -> onDraftChanged(draft.copy(traktInput = value)) },
                                    placeholder = { Text(AppStrings.t(language, "library.folder_editor.trakt_input_placeholder"), color = Color.White.copy(alpha = 0.35f)) },
                                    singleLine = true,
                                    colors = fieldColors(),
                                    modifier = Modifier.fillMaxWidth()
                                )
                            }
                        }
                    }
                    LibraryFolderSourceKind.AddonCatalog -> {
                        item {
                            LabeledField(AppStrings.t(language, "library.catalog")) {
                                if (catalogOptions.isEmpty()) {
                                    Text(
                                        AppStrings.t(language, "library.folder_editor.no_catalogs"),
                                        color = Color.White.copy(alpha = 0.4f),
                                        fontSize = 12.sp
                                    )
                                } else {
                                    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                                        catalogOptions.forEach { option ->
                                            val selected = draft.catalogAddonId == option.addonId && draft.catalogId == option.catalogId
                                            SourceTypeChip(
                                                label = "${option.addonName} · ${option.catalogName}",
                                                selected = selected
                                            ) {
                                                onDraftChanged(
                                                    draft.copy(
                                                        catalogAddonId = option.addonId,
                                                        catalogId = option.catalogId,
                                                        catalogGenre = null
                                                    )
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        val selectedOption = catalogOptions.firstOrNull {
                            it.addonId == draft.catalogAddonId && it.catalogId == draft.catalogId
                        }
                        if (selectedOption != null && selectedOption.genreOptions.isNotEmpty()) {
                            item {
                                LabeledField(AppStrings.t(language, "auto.genre")) {
                                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                        selectedOption.genreOptions.forEach { genre ->
                                            SourceTypeChip(
                                                label = genre,
                                                selected = draft.catalogGenre == genre
                                            ) { onDraftChanged(draft.copy(catalogGenre = genre)) }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if (state.error != null) {
                    item {
                        Text(
                            text = AppStrings.t(language, "library.folder_editor.save_failed"),
                            color = Color(0xFFFF7171),
                            fontSize = 12.sp
                        )
                    }
                }

                item {
                    Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        TextButton(
                            onClick = onSaveRequested,
                            enabled = !state.isSaving && draft.title.isNotBlank()
                        ) {
                            Text(
                                AppStrings.t(language, if (state.isSaving) "library.folder_editor.saving" else "library.save_folder"),
                                color = Color.White
                            )
                        }
                        if (draft.id != null) {
                            TextButton(onClick = onDeleteRequested) {
                                Text(AppStrings.t(language, "auto.remove"), color = Color(0xFFFF7171))
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun LabeledField(label: String, content: @Composable () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(text = label, color = Color.White.copy(alpha = 0.6f), fontSize = 12.sp)
        content()
    }
}

@Composable
private fun SourceTypeChip(label: String, selected: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(999.dp))
            .background(if (selected) Color.White else Color.White.copy(alpha = 0.08f))
            .clickable(onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 8.dp)
    ) {
        Text(label, color = if (selected) Color.Black else Color.White, fontSize = 12.sp)
    }
}

@Composable
private fun fieldColors() = OutlinedTextFieldDefaults.colors(
    unfocusedBorderColor = Color.White.copy(alpha = 0.1f),
    focusedTextColor = Color.White,
    unfocusedTextColor = Color.White,
    cursorColor = Color.White,
    focusedContainerColor = Color.Black.copy(alpha = 0.2f),
    unfocusedContainerColor = Color.Black.copy(alpha = 0.2f)
)
