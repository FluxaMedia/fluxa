package com.fluxa.app.shared.feature.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.LibraryBooks
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.filled.Explore
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.IconButton
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.common.AppStrings

@Composable
fun TvCatalogHomeScreen(
    state: CatalogHomeUiState,
    onAction: (CatalogAction) -> Unit,
    language: String? = null,
    onHomeRequested: () -> Unit = {},
    onSearchRequested: () -> Unit = {},
    onLibraryRequested: () -> Unit = {},
    onDiscoverRequested: () -> Unit = {},
    onSettingsRequested: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    Box(modifier = modifier.fillMaxSize()) {
        if (state.rows.isEmpty()) {
            TvCatalogHomeLoading(Modifier.fillMaxSize())
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(top = 96.dp, bottom = 64.dp),
                verticalArrangement = Arrangement.spacedBy(30.dp)
            ) {
                items(state.rows, key = { it.id }) { row ->
                    Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
                        Text(
                            text = row.title,
                            color = Color.White,
                            fontSize = 24.sp,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.padding(horizontal = 58.dp)
                        )
                        LazyRow(
                            contentPadding = PaddingValues(horizontal = 58.dp),
                            horizontalArrangement = Arrangement.spacedBy(20.dp)
                        ) {
                            items(row.items, key = { it.id }) { item ->
                                var focused by remember(item.id) { mutableStateOf(false) }
                                CatalogCard(
                                    model = item.card,
                                    onClick = { onAction(CatalogAction.ItemSelected(item)) },
                                    modifier = Modifier
                                        .clip(androidx.compose.foundation.shape.RoundedCornerShape(14.dp))
                                        .background(if (focused) Color.White.copy(alpha = 0.18f) else Color.Transparent)
                                        .onFocusChanged { focused = it.isFocused }
                                        .padding(4.dp)
                                )
                            }
                            if (row.canLoadMore) {
                                item(key = "${row.id}:load-more") {
                                    Box(
                                        modifier = Modifier
                                            .width(124.dp)
                                            .height(186.dp)
                                            .clip(androidx.compose.foundation.shape.RoundedCornerShape(14.dp))
                                            .background(FluxaColors.surfaceRaised)
                                            .clickable { onAction(CatalogAction.LoadMore(row.id)) }
                                            .padding(16.dp),
                                        contentAlignment = Alignment.Center
                                    ) {
                                        androidx.compose.material3.Icon(
                                            imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                                            contentDescription = AppStrings.t(null, "common.view_all"),
                                            tint = Color.White
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Row(
            modifier = Modifier
                .align(Alignment.TopStart)
                .fillMaxWidth()
                .padding(horizontal = 46.dp, vertical = 20.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            TvHomeAction(Icons.Filled.Home, AppStrings.t(language, "nav.home"), onHomeRequested)
            TvHomeAction(Icons.Filled.Search, AppStrings.t(language, "auto.search"), onSearchRequested)
            TvHomeAction(Icons.AutoMirrored.Filled.LibraryBooks, AppStrings.t(language, "nav.library"), onLibraryRequested)
            TvHomeAction(Icons.Filled.Explore, AppStrings.t(language, "nav.discover"), onDiscoverRequested)
            TvHomeAction(Icons.Filled.Settings, AppStrings.t(language, "nav.settings"), onSettingsRequested)
        }
    }
}

@Composable
private fun TvHomeAction(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    contentDescription: String,
    onClick: () -> Unit
) {
    IconButton(onClick = onClick) {
        androidx.compose.material3.Icon(
            imageVector = icon,
            contentDescription = contentDescription,
            tint = Color.White
        )
    }
}

@Composable
private fun TvCatalogHomeLoading(modifier: Modifier) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 58.dp, vertical = 44.dp),
        verticalArrangement = Arrangement.spacedBy(28.dp)
    ) {
        repeat(3) {
            Box(
                modifier = Modifier
                    .fillMaxWidth(0.28f)
                    .height(28.dp)
                    .background(FluxaColors.surfaceRaised)
            )
            Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                repeat(6) {
                    Box(
                        modifier = Modifier
                            .width(132.dp)
                            .height(198.dp)
                            .background(FluxaColors.surfaceRaised)
                    )
                }
            }
        }
    }
}
