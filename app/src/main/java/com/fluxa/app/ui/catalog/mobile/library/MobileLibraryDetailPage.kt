@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.remote.Meta
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import coil3.compose.AsyncImage

@Composable
internal fun MobileLibraryDetailPage(
    lang: String,
    title: String,
    items: List<Meta>,
    amoledMode: Boolean,
    onBack: () -> Unit,
    onMovieClick: (Meta) -> Unit
) {
    var selectedType by remember(items) { mutableStateOf("all") }
    var filterExpanded by remember { mutableStateOf(false) }
    val typeOptions = remember(items, lang) { libraryTypeFilterOptions(items, lang) }
    val filteredItems = remember(items, selectedType) { filterLibraryItems(items, selectedType) }
    val showAsContinueList = title == AppStrings.t(lang, "auto.continue_watching")

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(if (amoledMode) Color.Black else Color(0xFF05070B))
            .zIndex(40f)
    ) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(start = 10.dp, end = 10.dp, top = 18.dp, bottom = 132.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            item {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    IconButton(onClick = onBack) {
                        Icon(FluxaIcons.ArrowBack, AppStrings.t(lang, "common.back"), tint = Color.White)
                    }
                    Column(modifier = Modifier.weight(1f)) {
                        Text(title, color = Color.White, fontSize = 20.sp, fontWeight = FontWeight.Black, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text(
                            text = AppStrings.format(lang, "format.items_count", filteredItems.size),
                            color = Color.White.copy(alpha = 0.48f),
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold
                        )
                    }
                    if (typeOptions.size > 1) {
                        Box {
                            Row(
                                modifier = Modifier
                                    .clip(RoundedCornerShape(8.dp))
                                    .background(Color.White.copy(alpha = 0.08f))
                                    .clickable { filterExpanded = true }
                                    .padding(horizontal = 12.dp, vertical = 8.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(6.dp)
                            ) {
                                Text(
                                    text = typeOptions.firstOrNull { it.first == selectedType }?.second ?: typeOptions.first().second,
                                    color = Color.White.copy(alpha = 0.86f),
                                    fontSize = 12.sp,
                                    fontWeight = FontWeight.Bold
                                )
                                Icon(FluxaIcons.KeyboardArrowRight, null, tint = Color.White.copy(alpha = 0.48f), modifier = Modifier.size(16.dp))
                            }
                            DropdownMenu(expanded = filterExpanded, onDismissRequest = { filterExpanded = false }) {
                                typeOptions.forEach { (key, label) ->
                                    DropdownMenuItem(
                                        text = { Text(label) },
                                        onClick = {
                                            selectedType = key
                                            filterExpanded = false
                                        }
                                    )
                                }
                            }
                        }
                    }
                }
            }
            if (filteredItems.isEmpty()) {
                item {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(220.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(AppStrings.t(lang, "library.empty"), color = Color.White.copy(alpha = 0.52f), fontSize = 14.sp)
                    }
                }
            } else {
                if (showAsContinueList) {
                    items(filteredItems, key = { "${it.type}:${it.id}" }) { item ->
                        LibraryContinueListCard(item = item, onClick = { onMovieClick(item) })
                    }
                } else {
                    itemsIndexed(filteredItems.chunked(3), key = { i, _ -> i }) { _, rowItems ->
                        Row(
                            modifier = Modifier.fillMaxWidth(),
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
    }
}

@Composable
private fun LibraryContinueListCard(item: Meta, onClick: () -> Unit) {
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

internal fun libraryTypeFilterOptions(items: List<Meta>, lang: String): List<Pair<String, String>> {
    val options = mutableListOf("all" to AppStrings.t(lang, "auto.all_24e2815b"))
    if (items.any { it.type == "series" }) {
        options += "series" to AppStrings.t(lang, "auto.series_02b3bdba")
    }
    if (items.any { it.type == "movie" }) {
        options += "movie" to AppStrings.t(lang, "auto.movies")
    }
    if (items.any(::isLibraryAnime)) {
        options += "anime" to "Anime"
    }
    return options.distinctBy { it.first }
}

private fun filterLibraryItems(items: List<Meta>, selectedType: String): List<Meta> {
    return when (selectedType) {
        "movie" -> items.filter { it.type == "movie" }
        "series" -> items.filter { it.type == "series" }
        "anime" -> items.filter(::isLibraryAnime)
        else -> items
    }
}

private fun isLibraryAnime(meta: Meta): Boolean {
    return meta.type == "anime" ||
        meta.genres.orEmpty().any { it.contains("anime", ignoreCase = true) } ||
        meta.originalLanguage == "ja"
}
