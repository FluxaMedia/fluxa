@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items

@Composable
fun TvExploreScreen(
    activeProfile: UserProfile?,
    lang: String,
    results: List<Meta>,
    resultSources: Map<String, HomeCatalogSource>,
    isDiscoverLoading: Boolean,
    catalogOptions: List<Pair<String?, String>>,
    genreOptions: List<Pair<String?, String>>,
    selectedType: String,
    selectedCatalog: String?,
    selectedGenre: String?,
    onSelectType: (String) -> Unit,
    onSelectCatalog: (String?) -> Unit,
    onSelectGenre: (String?) -> Unit,
    onMovieClick: (Meta, HomeCatalogSource?) -> Unit,
    onBack: () -> Unit,
    viewModel: HomeViewModel
) {
    Column(modifier = Modifier.fillMaxSize().background(Color(0xFF040508))) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 24.dp, vertical = 20.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            androidx.compose.material3.IconButton(
                onClick = onBack,
                modifier = Modifier.size(44.dp).background(Color.White.copy(0.05f), CircleShape)
            ) {
                Icon(FluxaIcons.ArrowBack, null, tint = Color.White)
            }
            Text(AppStrings.t(lang, "auto.explore"), fontSize = 28.sp, fontWeight = FontWeight.Black, color = Color.White)
        }

        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 24.dp),
            horizontalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Box(modifier = Modifier.weight(1f)) {
                ExploreDropdownFilter(
                    title = AppStrings.t(lang, "auto.content_type_06574d00"),
                    options = exploreTypeOptions(lang),
                    selected = selectedType,
                    onSelect = { it?.let(onSelectType) }
                )
            }
            Box(modifier = Modifier.weight(1f)) {
                ExploreDropdownFilter(
                    title = AppStrings.t(lang, "explore.catalog"),
                    options = catalogOptions,
                    selected = selectedCatalog,
                    onSelect = onSelectCatalog
                )
            }
            if (genreOptions.isNotEmpty()) {
            Box(modifier = Modifier.weight(1f)) {
                ExploreDropdownFilter(
                    title = AppStrings.t(lang, "auto.genres"),
                    options = genreOptions,
                    selected = selectedGenre,
                    onSelect = onSelectGenre
                )
            }
            }
        }

        Spacer(Modifier.height(24.dp))

        LazyVerticalGrid(
            columns = GridCells.Adaptive(140.dp),
            modifier = Modifier.fillMaxWidth().weight(1f).padding(horizontal = 24.dp),
            horizontalArrangement = Arrangement.spacedBy(16.dp),
            verticalArrangement = Arrangement.spacedBy(20.dp),
            contentPadding = PaddingValues(bottom = 120.dp)
        ) {
            if (results.isEmpty() && isDiscoverLoading) {
                item {
                    androidx.compose.material3.CircularProgressIndicator(color = Color.White)
                }
            }
            items(results, key = { "${it.type}:${it.id}" }) { movie ->
                val source = discoverSourceFor(movie, resultSources)
                MovieCard(
                    movie,
                    { viewModel.onMovieFocused(it) },
                    { onMovieClick(movie, source) },
                    if (activeProfile?.safePosterLandscapeMode == true) "horizontal" else activeProfile?.safeCardLayout ?: "vertical",
                    profile = activeProfile
                )
            }
        }
    }
}
