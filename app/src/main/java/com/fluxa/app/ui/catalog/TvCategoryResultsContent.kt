@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.itemsIndexed

@Composable
internal fun TvCategoryResultsContent(
    activeProfile: UserProfile?,
    title: String,
    category: HomeCategory?,
    items: List<Meta>,
    layout: String,
    onMovieClick: (Meta) -> Unit,
    onBack: () -> Unit,
    viewModel: HomeViewModel
) {
    Column(modifier = Modifier.fillMaxSize().background(Color(0xFF040508))) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 18.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            IconButton(
                onClick = onBack,
                modifier = Modifier.size(44.dp).background(Color.White.copy(alpha = 0.06f), CircleShape)
            ) {
                Icon(FluxaIcons.ArrowBack, null, tint = Color.White)
            }
            Text(
                text = title,
                color = Color.White,
                fontSize = 28.sp,
                fontWeight = FontWeight.Black,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )
        }

        LazyVerticalGrid(
            columns = GridCells.Adaptive(140.dp),
            modifier = Modifier.fillMaxSize().padding(horizontal = 24.dp),
            horizontalArrangement = Arrangement.spacedBy(16.dp),
            verticalArrangement = Arrangement.spacedBy(20.dp),
            contentPadding = PaddingValues(bottom = 80.dp)
        ) {
            itemsIndexed(
                items = items,
                key = { _, movie -> "${movie.type}:${movie.id}" }
            ) { index, movie ->
                if (category != null && category.canLoadMore && index == (items.size - 6).coerceAtLeast(0)) {
                    LaunchedEffect(category.id, items.size) { viewModel.loadMore(category.id) }
                }
                MovieCard(
                    movie,
                    { viewModel.onMovieFocused(it) },
                    { onMovieClick(movie) },
                    layout,
                    profile = activeProfile
                )
            }
        }
    }
}
