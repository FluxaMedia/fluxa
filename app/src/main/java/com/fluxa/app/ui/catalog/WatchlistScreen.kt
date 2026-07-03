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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items as mobileGridItems
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
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
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import java.io.File
import java.util.Locale

@Composable
fun WatchlistScreen(
    activeProfile: UserProfile?,
    onMovieClick: (Meta) -> Unit,
    onPlayDirect: (Meta) -> Unit,
    onBack: () -> Unit,
    viewModel: HomeViewModel,
    offlineDownloadManager: OfflineDownloadManager? = null,
    onOpenDownload: (OfflineDownloadItem) -> Unit = {},
    onUpdateProfile: (UserProfile) -> Unit = {}
) {
    val watchlist by viewModel.watchlist.collectAsState()
    val likedItems by viewModel.likedItems.collectAsState()
    val libraryUiState by viewModel.libraryUiState.collectAsState()
    LaunchedEffect(activeProfile?.id, activeProfile?.traktAccessToken, activeProfile?.malAccessToken, activeProfile?.simklAccessToken) {
        viewModel.loadLibraryItems(activeProfile)
    }
    val deviceType = LocalDeviceType.current
    val lang = activeProfile?.safeLanguage ?: "en"
    val horizontalPadding = if (deviceType == DeviceType.TV) 58.dp else 16.dp
    var selectedType by remember { mutableStateOf("all") }
    var selectedLibrarySection by remember { mutableStateOf("planned") }
    var selectedMobileDetail by remember { mutableStateOf<LibraryDetailUi?>(null) }
    var selectedDownloadGroup by remember { mutableStateOf<OfflineDownloadGroup?>(null) }
    val amoledMode = activeProfile?.safeAmoledMode == true
    val offlineDownloads by (offlineDownloadManager?.items ?: remember { kotlinx.coroutines.flow.MutableStateFlow(emptyList()) }).collectAsState()
    val profileDownloads = remember(offlineDownloads, activeProfile?.id) {
        offlineDownloads.filter { it.profileId == null || it.profileId == activeProfile?.id }
    }
    val downloadGroups = remember(profileDownloads) { profileDownloads.toOfflineDownloadGroups() }

    LaunchedEffect(
        activeProfile?.authKey,
        activeProfile?.traktAccessToken,
        activeProfile?.malAccessToken,
        activeProfile?.simklAccessToken
    ) {
        viewModel.loadLibraryData(activeProfile)
    }

    val plannedItems = remember(watchlist, libraryUiState) {
        (watchlist + libraryUiState.traktPlanned + libraryUiState.malPlanned + libraryUiState.simklPlanned).distinctBy { it.id }
    }
    val completedItems = remember(libraryUiState) {
        (libraryUiState.traktWatched + libraryUiState.malCompleted + libraryUiState.simklCompleted).distinctBy { it.id }
    }
    val favoriteItems = remember(likedItems) { likedItems.distinctBy { it.id } }
    val filteredFavorites = remember(favoriteItems, selectedType) { filterWatchlistLibraryItems(favoriteItems, selectedType) }
    val filteredPlannedItems = remember(plannedItems, selectedType) { filterWatchlistLibraryItems(plannedItems, selectedType) }
    val filteredCompletedItems = remember(completedItems, selectedType) { filterWatchlistLibraryItems(completedItems, selectedType) }
    val userCollections = remember(activeProfile?.safeLibraryCollections) { activeProfile?.safeLibraryCollections.orEmpty() }
    val libraryCollections = remember(
        lang, favoriteItems, libraryUiState.traktCollection,
        plannedItems, completedItems,
        libraryUiState.malWatching, libraryUiState.malPlanned, libraryUiState.malCompleted,
        libraryUiState.simklWatching, libraryUiState.simklPlanned, libraryUiState.simklCompleted,
        userCollections
    ) {
        buildLibraryCollections(
            lang = lang,
            favorites = favoriteItems,
            traktCollection = libraryUiState.traktCollection,
            planned = plannedItems,
            completed = completedItems,
            malWatching = libraryUiState.malWatching,
            malPlanned = libraryUiState.malPlanned,
            malCompleted = libraryUiState.malCompleted,
            simklWatching = libraryUiState.simklWatching,
            simklPlanned = libraryUiState.simklPlanned,
            simklCompleted = libraryUiState.simklCompleted,
            userCollections = userCollections
        )
    }
    Box(modifier = Modifier.fillMaxSize().background(if (amoledMode) Color.Black else Color(0xFF0A0A0A))) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        if (amoledMode) {
                            listOf(Color.Black, Color.Black, Color.Black)
                        } else {
                            listOf(Color.White.copy(alpha = 0.05f), Color.Transparent, Color.Black)
                        }
                    )
                )
        )

        if (deviceType == DeviceType.Mobile) {
            MobileLibraryDashboard(
                lang = lang,
                accent = Color(activeProfile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt()),
                amoledMode = amoledMode,
                selectedType = selectedType,
                onSelectType = { selectedType = it },
                selectedSection = selectedLibrarySection,
                onSelectSection = {
                    selectedLibrarySection = it
                    selectedDownloadGroup = null
                },
                plannedItems = filteredPlannedItems,
                completedItems = filteredCompletedItems,
                favoriteItems = filteredFavorites,
                plannedCount = plannedItems.size,
                completedCount = completedItems.size,
                downloadedCount = profileDownloads.size,
                favoriteCount = favoriteItems.size,
                collections = libraryCollections,
                userCollections = userCollections,
                categoriesFlow = viewModel.categories,
                onMovieClick = onMovieClick,
                onCollectionClick = { selectedMobileDetail = LibraryDetailUi(it.title, it.items) },
                onCreateCollection = { title ->
                    activeProfile?.let { profile ->
                        onUpdateProfile(
                            profile.copy(
                                libraryCollections = profile.safeLibraryCollections + LibraryUserCollection(
                                    id = "local_${System.currentTimeMillis()}",
                                    title = title.trim()
                                )
                            )
                        )
                    }
                },
                onSaveUserCollection = { collection ->
                    activeProfile?.let { profile ->
                        val current = profile.safeLibraryCollections
                        onUpdateProfile(
                            profile.copy(
                                libraryCollections = if (current.any { it.id == collection.id }) {
                                    current.map { if (it.id == collection.id) collection else it }
                                } else {
                                    current + collection
                                }
                            )
                        )
                    }
                },
                onImportCollections = { imported ->
                    activeProfile?.let { profile ->
                        val current = profile.safeLibraryCollections
                        val importedIds = imported.map { it.id }.toSet()
                        onUpdateProfile(
                            profile.copy(
                                libraryCollections = current.filterNot { it.id in importedIds } + imported
                            )
                        )
                    }
                },
                onRenameCollection = { collection, title ->
                    activeProfile?.let { profile ->
                        onUpdateProfile(
                            profile.copy(
                                libraryCollections = profile.safeLibraryCollections.map {
                                    if (it.id == collection.userCollectionId) it.copy(title = title.trim()) else it
                                }
                            )
                        )
                    }
                },
                onDeleteCollection = { collection ->
                    activeProfile?.let { profile ->
                        onUpdateProfile(
                            profile.copy(
                                libraryCollections = profile.safeLibraryCollections.filterNot { it.id == collection.userCollectionId }
                            )
                        )
                    }
                }
            )
            if (selectedLibrarySection == "downloads") {
                LaunchedEffect(offlineDownloadManager) {
                    while (offlineDownloadManager != null) {
                        offlineDownloadManager.refresh()
                        kotlinx.coroutines.delay(1000L)
                    }
                }
                selectedDownloadGroup?.let { group ->
                    MobileOfflineDownloadGroupPage(
                        lang = lang,
                        group = group,
                        amoledMode = amoledMode,
                        onBack = { selectedDownloadGroup = null },
                        onOpenDownload = {
                            offlineDownloadManager?.refresh()
                            if (it.isPlayable) onOpenDownload(it)
                        }
                    )
                } ?: MobileOfflineDownloadFoldersPage(
                    lang = lang,
                    groups = downloadGroups,
                    amoledMode = amoledMode,
                    onBack = {
                        selectedLibrarySection = "planned"
                        selectedMobileDetail = null
                    },
                    onGroupClick = { selectedDownloadGroup = it }
                )
                return@Box
            }
            selectedMobileDetail?.let { detail ->
                MobileLibraryDetailPage(
                    lang = lang,
                    title = detail.title,
                    items = detail.items,
                    amoledMode = amoledMode,
                    onBack = { selectedMobileDetail = null },
                    onMovieClick = onMovieClick
                )
            }
            return@Box
        }

        TvLibraryScreenContent(
            lang = lang,
            horizontalPadding = 58.dp,
            activeProfile = activeProfile,
            selectedType = selectedType,
            onSelectType = { selectedType = it },
            plannedItems = plannedItems,
            completedItems = completedItems,
            favoriteItems = favoriteItems,
            filteredPlannedItems = filteredPlannedItems,
            filteredCompletedItems = filteredCompletedItems,
            filteredFavorites = filteredFavorites,
            downloadGroups = downloadGroups,
            libraryCollections = libraryCollections,
            userCollections = userCollections,
            onBack = onBack,
            onMovieClick = onMovieClick,
            onOpenDownload = {
                offlineDownloadManager?.refresh()
                if (it.isPlayable) onOpenDownload(it)
            },
            onDeleteCollection = { collection ->
                activeProfile?.let { profile ->
                    onUpdateProfile(
                        profile.copy(
                            libraryCollections = profile.safeLibraryCollections.filterNot { it.id == collection.userCollectionId }
                        )
                    )
                }
            },
            onSaveUserCollection = { collection ->
                activeProfile?.let { profile ->
                    val current = profile.safeLibraryCollections
                    onUpdateProfile(
                        profile.copy(
                            libraryCollections = if (current.any { it.id == collection.id }) {
                                current.map { if (it.id == collection.id) collection else it }
                            } else {
                                current + collection
                            }
                        )
                    )
                }
            },
            viewModel = viewModel
        )
    }
}
