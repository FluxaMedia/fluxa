@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.Surface

@Composable
fun ExploreScreen(
    activeProfile: UserProfile?,
    onMovieClick: (Meta, String?, String?) -> Unit,
    onBack: () -> Unit,
    viewModel: HomeViewModel,
    initialType: String = "all",
    initialGenre: String? = null
) {
    val deviceType = LocalDeviceType.current
    val lang = activeProfile?.language ?: "en"

    var selectedType by remember(initialType) { mutableStateOf(initialType) }
    var selectedCatalog by remember { mutableStateOf<String?>(null) }
    var selectedGenre by remember(initialGenre) { mutableStateOf(initialGenre) }
    var selectedYear by remember { mutableStateOf<String?>(null) }
    var selectedRating by remember { mutableStateOf<Float?>(null) }
    var selectedProvider by remember { mutableStateOf<String?>(null) }
    var selectedRegion by remember { mutableStateOf<String?>(null) }

    val discoverUiState by viewModel.discoverUiState.collectAsStateWithLifecycle()
    val searchResults by viewModel.searchResults.collectAsStateWithLifecycle()
    val searchRows by viewModel.searchRows.collectAsStateWithLifecycle()
    val userAddons by viewModel.userAddons.collectAsStateWithLifecycle()
    val allLabel = AppStrings.t(lang, "auto.all")
    val results = discoverUiState.results
    val resultSources = discoverUiState.resultSources
    val isDiscoverLoading = discoverUiState.isLoading
    val discoverCatalogs = discoverUiState.catalogs
    val discoverGenres = discoverUiState.genres
    val catalogOptions = discoverCatalogs.map { it.key to it.label }
    val genreOptions = discoverGenres.map { it.id to it.label }.ifEmpty { listOf(null to allLabel) }

    val resetCatalogAndGenre: (String) -> Unit = { nextType ->
        selectedType = nextType
        selectedCatalog = null
        selectedGenre = null
    }
    val resetGenreForCatalog: (String?) -> Unit = { nextCatalog ->
        selectedCatalog = nextCatalog
        selectedGenre = null
    }

    LaunchedEffect(activeProfile?.id, activeProfile?.safeLanguage, activeProfile?.safeLocalAddons) {
        activeProfile?.let { viewModel.applyUpdatedProfile(it) }
    }

    LaunchedEffect(selectedType, selectedCatalog, lang, userAddons) {
        viewModel.loadDiscoverCatalogFilters(selectedType, selectedCatalog)
    }

    LaunchedEffect(discoverCatalogs, selectedCatalog) {
        if (discoverCatalogs.isNotEmpty() && (selectedCatalog == null || discoverCatalogs.none { it.key == selectedCatalog })) {
            selectedCatalog = discoverCatalogs.first().key
        }
    }

    LaunchedEffect(genreOptions, selectedGenre) {
        val supportsOptionalEmptySelection = genreOptions.any { it.first == null }
        if (genreOptions.isEmpty()) {
            selectedGenre = null
        } else if (selectedGenre != null && !selectedGenre.orEmpty().contains("|") && genreOptions.none { it.first == selectedGenre }) {
            selectedGenre = if (supportsOptionalEmptySelection) null else genreOptions.firstOrNull()?.first
        } else if (selectedGenre == null && !supportsOptionalEmptySelection) {
            selectedGenre = genreOptions.firstOrNull()?.first
        }
    }

    LaunchedEffect(selectedType, selectedCatalog, selectedGenre, selectedYear, selectedRating, selectedProvider, selectedRegion, discoverCatalogs, discoverGenres) {
        viewModel.discover(selectedType, selectedCatalog, selectedGenre, selectedYear, selectedRating, selectedProvider, selectedRegion)
    }

    if (deviceType == DeviceType.Mobile) {
        MobileExploreScreen(
            activeProfile = activeProfile,
            lang = lang,
            results = results,
            resultSources = resultSources,
            searchResults = searchResults,
            searchRows = searchRows,
            isDiscoverLoading = isDiscoverLoading,
            catalogOptions = catalogOptions,
            genreOptions = genreOptions,
            selectedType = selectedType,
            selectedCatalog = selectedCatalog,
            selectedGenre = selectedGenre,
            onSelectType = resetCatalogAndGenre,
            onSelectCatalog = resetGenreForCatalog,
            onSelectGenre = { selectedGenre = it },
            onMovieClick = onMovieClick,
            onBack = onBack,
            viewModel = viewModel
        )
    } else {
        TvExploreScreen(
            activeProfile = activeProfile,
            lang = lang,
            results = results,
            resultSources = resultSources,
            isDiscoverLoading = isDiscoverLoading,
            catalogOptions = catalogOptions,
            genreOptions = genreOptions,
            selectedType = selectedType,
            selectedCatalog = selectedCatalog,
            selectedGenre = selectedGenre,
            onSelectType = resetCatalogAndGenre,
            onSelectCatalog = resetGenreForCatalog,
            onSelectGenre = { selectedGenre = it },
            onMovieClick = { meta, source ->
                onMovieClick(meta, source?.transportUrl, source?.type)
            },
            onBack = onBack,
            viewModel = viewModel
        )
    }
}

internal fun discoverSourceFor(meta: Meta, sources: Map<String, HomeCatalogSource>): HomeCatalogSource? {
    return sources["${meta.type}:${meta.id}"] ?: sources[meta.id]
}

internal fun exploreTypeOptions(lang: String): List<Pair<String?, String>> {
    return listOf(
        "movie" to AppStrings.t(lang, "auto.movie"),
        "series" to AppStrings.t(lang, "auto.series")
    )
}

@Composable
fun ExploreDropdownFilter(
    title: String,
    options: List<Pair<String?, String>>,
    selected: String?,
    onSelect: (String?) -> Unit,
    showTitle: Boolean = true,
    accentColor: Color = Color.White
) {
    val deviceType = LocalDeviceType.current
    var showSelector by remember { mutableStateOf(false) }
    val selectedLabel = options.find { it.first == selected }?.second
        ?: options.firstOrNull { it.first == null }?.second
        ?: options.firstOrNull()?.second.orEmpty()
    val shape = RoundedCornerShape(if (deviceType == DeviceType.Mobile) 10.dp else 14.dp)

    Column {
        if (showTitle) {
            Text(text = title, color = Color.Gray, fontSize = 11.sp, fontWeight = FontWeight.Bold, modifier = Modifier.padding(bottom = 6.dp), letterSpacing = 1.sp)
        }
        val interactionModifier = if (deviceType == DeviceType.TV) {
            Modifier.fillMaxWidth()
        } else {
            Modifier.fillMaxWidth().clickable { showSelector = true }
        }
        val isActive = selected != null && options.firstOrNull()?.first != selected
        Surface(
            onClick = { if(deviceType == DeviceType.TV) showSelector = true },
            shape = ClickableSurfaceDefaults.shape(shape),
            colors = ClickableSurfaceDefaults.colors(
                containerColor = if (isActive) accentColor.copy(alpha = 0.12f) else Color(0xFF151515),
                focusedContainerColor = Color(0xFF222222)
            ),
            scale = ClickableSurfaceDefaults.scale(focusedScale = 1.03f),
            modifier = interactionModifier
                .height(if (deviceType == DeviceType.Mobile) 34.dp else 52.dp)
                .border(1.dp, if (isActive) accentColor.copy(alpha = 0.45f) else Color.White.copy(alpha = 0.08f), shape)
        ) {
            Row(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(horizontal = if (deviceType == DeviceType.Mobile) 10.dp else 14.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                Text(
                    text = selectedLabel,
                    color = Color.White,
                    fontSize = if (deviceType == DeviceType.Mobile) 11.sp else 14.sp,
                    fontWeight = FontWeight.Black,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f)
                )
                Icon(FluxaIcons.KeyboardArrowDown, null, tint = Color.White.copy(alpha = 0.72f), modifier = Modifier.size(if (deviceType == DeviceType.Mobile) 16.dp else 20.dp))
            }
        }

        if (showSelector) {
            ExploreOptionSelector(
                title = title,
                options = options,
                selected = selected,
                onSelect = {
                    onSelect(it)
                    showSelector = false
                },
                onDismiss = { showSelector = false },
                accentColor = accentColor
            )
        }
    }
}

@Composable
fun ExploreOptionSelector(
    title: String,
    options: List<Pair<String?, String>>,
    selected: String?,
    onSelect: (String?) -> Unit,
    onDismiss: () -> Unit,
    accentColor: Color = Color.White
) {
    val deviceType = LocalDeviceType.current

    androidx.compose.ui.window.Dialog(
        onDismissRequest = onDismiss,
        properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
    ) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .clickable(onClick = onDismiss, indication = null, interactionSource = remember { androidx.compose.foundation.interaction.MutableInteractionSource() }),
            contentAlignment = if (deviceType == DeviceType.TV) Alignment.CenterEnd else Alignment.BottomCenter
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth(if(deviceType == DeviceType.TV) 0.34f else 1f)
                    .then(
                        if (deviceType == DeviceType.TV) {
                            Modifier.fillMaxHeight()
                        } else {
                            Modifier
                                .wrapContentHeight()
                                .heightIn(max = 420.dp)
                        }
                    )
                    .clip(if (deviceType == DeviceType.TV) RoundedCornerShape(topStart = 22.dp, bottomStart = 22.dp) else RoundedCornerShape(topStart = 22.dp, topEnd = 22.dp))
                    .background(Color(0xFF111111))
                    .border(
                        1.dp,
                        Color.White.copy(0.1f),
                        if (deviceType == DeviceType.TV) RoundedCornerShape(topStart = 22.dp, bottomStart = 22.dp) else RoundedCornerShape(topStart = 22.dp, topEnd = 22.dp)
                    )
                    .padding(18.dp)
            ) {
                if (deviceType == DeviceType.Mobile) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.CenterHorizontally)
                            .width(42.dp)
                            .height(4.dp)
                            .clip(RoundedCornerShape(999.dp))
                            .background(Color.White.copy(alpha = 0.22f))
                    )
                    Spacer(Modifier.height(16.dp))
                }
                if (deviceType != DeviceType.Mobile) {
                    Text(
                        text = title,
                        color = Color.White,
                        fontSize = 20.sp,
                        fontWeight = FontWeight.Black,
                        modifier = Modifier.padding(bottom = 24.dp)
                    )
                }

                androidx.compose.foundation.lazy.LazyColumn(
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = if (deviceType == DeviceType.Mobile) {
                        Modifier
                            .fillMaxWidth()
                            .heightIn(max = 320.dp)
                    } else {
                        Modifier.fillMaxSize()
                    }
                ) {
                    items(options, key = { (id, _) -> id ?: "" }) { (id, label) ->
                        val isSelected = id == selected
                        val optionModifier = if (deviceType == DeviceType.TV) {
                            Modifier.fillMaxWidth()
                        } else {
                            Modifier.fillMaxWidth().clickable { onSelect(id) }
                        }

                        val accentLuma = accentColor.red * 0.299f + accentColor.green * 0.587f + accentColor.blue * 0.114f
                        val onAccent = if (accentLuma > 0.68f) Color.Black else Color.White
                        Surface(
                            onClick = { if(deviceType == DeviceType.TV) onSelect(id) },
                            shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(12.dp)),
                            colors = ClickableSurfaceDefaults.colors(
                                containerColor = if (isSelected) accentColor else Color.White.copy(0.04f),
                                contentColor = if (isSelected) onAccent else Color.White
                            ),
                            scale = ClickableSurfaceDefaults.scale(focusedScale = 1.05f),
                            modifier = optionModifier
                        ) {
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(if (deviceType == DeviceType.Mobile) 44.dp else 56.dp)
                                    .padding(horizontal = 14.dp),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                val textColor = if (isSelected) onAccent else Color.White
                                Text(
                                    text = label,
                                    color = textColor,
                                    fontWeight = if(isSelected) FontWeight.Black else FontWeight.Medium,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                    modifier = Modifier.weight(1f)
                                )
                                if (isSelected) {
                                    Icon(FluxaIcons.Check, null, modifier = Modifier.size(20.dp), tint = onAccent)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
