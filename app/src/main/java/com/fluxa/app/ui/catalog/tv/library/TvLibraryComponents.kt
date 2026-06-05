@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

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
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
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
import coil3.compose.AsyncImage
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope

@Composable
internal fun TvLibraryScreenContent(
    lang: String,
    horizontalPadding: Dp,
    activeProfile: UserProfile?,
    filteredFavorites: List<Meta>,
    favoriteItems: List<Meta>,
    onBack: () -> Unit,
    onMovieClick: (Meta) -> Unit,
    viewModel: HomeViewModel
) {
Column(modifier = Modifier.fillMaxSize()) {
    // Header
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(24.dp),
        modifier = Modifier.fillMaxWidth().padding(horizontal = horizontalPadding, vertical = 40.dp)
    ) {
        androidx.compose.material3.IconButton(onClick = onBack, modifier = Modifier.size(48.dp).background(Color.White.copy(alpha = 0.05f), CircleShape)) {
            Icon(FluxaIcons.ArrowBack, null, tint = Color.White)
        }
        Column {
            Text(text = AppStrings.t(lang, "auto.my_library_a6c93797"), style = androidx.compose.material3.MaterialTheme.typography.displaySmall, color = Color.White, fontWeight = FontWeight.Black, letterSpacing = 2.sp, fontSize = 32.sp)
            Text(text = AppStrings.t(lang, "auto.movies_and_shows_you_saved_to_watch_later"), color = Color.White.copy(alpha = 0.5f), style = androidx.compose.material3.MaterialTheme.typography.bodyMedium)
        }
    }

    Spacer(Modifier.height(8.dp))

    if (favoriteItems.isEmpty()) {
        EmptyLibraryState(lang)
    } else {
        LazyVerticalGrid(
            columns = GridCells.Adaptive(160.dp),
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(start = horizontalPadding, end = horizontalPadding, bottom = 60.dp),
            horizontalArrangement = Arrangement.spacedBy(20.dp),
            verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            gridItemsIndexed(filteredFavorites, key = { _: Int, item: Meta -> "${item.type}:${item.id}" }) { i, item ->
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
}
