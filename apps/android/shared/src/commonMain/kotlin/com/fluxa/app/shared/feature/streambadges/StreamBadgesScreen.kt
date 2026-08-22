package com.fluxa.app.shared.feature.streambadges

import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaIcons

import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun StreamBadgesScreen(
    state: StreamBadgesUiState,
    language: String?,
    onAction: (StreamBadgesAction) -> Unit,
    onBackRequested: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
            LazyColumn(
                modifier = Modifier.fillMaxWidth(0.96f).widthIn(max = 900.dp),
                contentPadding = PaddingValues(horizontal = 16.dp, vertical = 20.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                item {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(18.dp)) {
                        var backFocused by remember { mutableStateOf(false) }
                        Box(
                            modifier = Modifier
                                .size(40.dp)
                                .clip(CircleShape)
                                .background(if (backFocused) Color.White else Color.White.copy(alpha = 0.05f))
                                .clickable(onClick = onBackRequested),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(FluxaIcons.ArrowBack, null, tint = if (backFocused) Color.Black else Color.White)
                        }
                        Text(
                            text = AppStrings.t(language, "settings.stream_badges.title"),
                            color = Color.White,
                            fontSize = 22.sp,
                            fontWeight = FontWeight.Bold
                        )
                    }
                }

                item {
                    Text(
                        text = AppStrings.t(language, "settings.stream_badges.desc"),
                        color = Color.White.copy(alpha = 0.6f),
                        fontSize = 13.sp
                    )
                }

                item {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = AppStrings.t(language, "settings.stream_badges.position"),
                            color = Color.White.copy(alpha = 0.6f),
                            fontSize = 13.sp
                        )
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            SourceTypeChip(
                                label = AppStrings.t(language, "settings.stream_badges.position_top"),
                                selected = state.badgePlacement == "top"
                            ) { onAction(StreamBadgesAction.PlacementChanged("top")) }
                            SourceTypeChip(
                                label = AppStrings.t(language, "settings.stream_badges.position_bottom"),
                                selected = state.badgePlacement != "top"
                            ) { onAction(StreamBadgesAction.PlacementChanged("bottom")) }
                        }
                    }
                }

                item {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        OutlinedTextField(
                            value = state.importUrlDraft,
                            onValueChange = { onAction(StreamBadgesAction.ImportUrlChanged(it)) },
                            placeholder = { Text(AppStrings.t(language, "settings.stream_badges.url_placeholder"), color = Color.White.copy(alpha = 0.35f)) },
                            singleLine = true,
                            colors = OutlinedTextFieldDefaults.colors(
                                focusedTextColor = Color.White,
                                unfocusedTextColor = Color.White,
                                focusedBorderColor = Color.White.copy(alpha = 0.4f),
                                unfocusedBorderColor = Color.White.copy(alpha = 0.15f)
                            ),
                            modifier = Modifier.weight(1f)
                        )
                        TextButton(
                            onClick = { onAction(StreamBadgesAction.ImportRequested) },
                            enabled = !state.isImporting && state.importUrlDraft.isNotBlank()
                        ) {
                            Text(
                                text = AppStrings.t(
                                    language,
                                    if (state.isImporting) "settings.stream_badges.importing" else "settings.stream_badges.import"
                                ),
                                color = Color.White
                            )
                        }
                    }
                }

                if (state.error != null) {
                    item {
                        Text(
                            text = state.error,
                            color = Color(0xFFFF7171),
                            fontSize = 12.sp,
                            modifier = Modifier.clickable { onAction(StreamBadgesAction.ErrorDismissed) }
                        )
                    }
                }

                if (state.imports.isEmpty()) {
                    item {
                        Text(
                            text = AppStrings.t(language, "settings.stream_badges.empty"),
                            color = Color.White.copy(alpha = 0.4f),
                            fontSize = 13.sp
                        )
                    }
                } else {
                    items(state.imports, key = { it.sourceUrl }) { import ->
                        StreamBadgeImportRow(import, language, onAction)
                    }
                }
            }
        }
    }
}

@Composable
private fun StreamBadgeImportRow(
    import: StreamBadgeImportUiModel,
    language: String?,
    onAction: (StreamBadgesAction) -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .padding(horizontal = 14.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Box(
            modifier = Modifier
                .size(14.dp)
                .clip(CircleShape)
                .border(1.dp, Color.White.copy(alpha = 0.4f), CircleShape)
                .background(if (import.isActive) Color.White else Color.Transparent)
                .clickable(enabled = !import.isActive) { onAction(StreamBadgesAction.SourceActivated(import.sourceUrl)) }
        )
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = import.sourceUrl,
                color = Color.White,
                fontSize = 13.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            Text(
                text = AppStrings.format(language, "settings.stream_badges.filter_count", import.filterCount),
                color = Color.White.copy(alpha = 0.45f),
                fontSize = 11.sp
            )
        }
        TextButton(onClick = { onAction(StreamBadgesAction.SourceRemoved(import.sourceUrl)) }) {
            Text(AppStrings.t(language, "auto.remove"), color = Color.White.copy(alpha = 0.5f), fontSize = 12.sp)
        }
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
