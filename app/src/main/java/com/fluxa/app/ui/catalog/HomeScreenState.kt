package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta

@Immutable
data class HomeBillboardState(
    val billboardIndex: Int,
    val billboardMovie: Meta?,
    val billboardPool: List<Meta>,
    val billboardLogo: String?,
    val billboardWatchlist: Boolean,
    val filteredBillboardPool: List<Meta>,
    val displayBillboardMovie: Meta?,
    val billboardSeasonPosterUrl: String?,
    val billboardTrailerUrl: String?
)

@Immutable
data class HomeCatalogState(
    val categories: List<HomeCategory>,
    val isLoading: Boolean,
    val hasLoadedHome: Boolean,
    val filter: String,
    val showHeroSection: Boolean,
    val orderedCategories: List<HomeCategory>,
    val topTenFeedKeys: Set<String>
)

@Composable
fun rememberHomeBillboardState(
    activeProfile: UserProfile?,
    viewModel: HomeViewModel
): HomeBillboardState {
    val billboardIndex by viewModel.billboardIndex.collectAsStateWithLifecycle()
    val billboardMovie by viewModel.billboardMovie.collectAsStateWithLifecycle()
    val billboardPool by viewModel.billboardPool.collectAsStateWithLifecycle()
    val billboardLogo by viewModel.billboardLogo.collectAsStateWithLifecycle()
    val billboardWatchlist by viewModel.billboardWatchlist.collectAsStateWithLifecycle(initialValue = false)
    val billboardSeasonPosterUrl by viewModel.billboardSeasonPosterUrl.collectAsStateWithLifecycle()
    val billboardTrailerUrl by viewModel.billboardTrailerUrl.collectAsStateWithLifecycle()
    val filter by viewModel.currentFilter.collectAsStateWithLifecycle()
    val filteredBillboardPool = remember(billboardPool, filter) {
        billboardPool.filter { it.matchesFilter(filter) }
    }
    val displayBillboardMovie = remember(billboardMovie, filteredBillboardPool, filter) {
        billboardMovie?.takeIf { it.matchesFilter(filter) }
            ?: filteredBillboardPool.firstOrNull()
            ?: billboardMovie
    }
    return HomeBillboardState(
        billboardIndex = billboardIndex,
        billboardMovie = billboardMovie,
        billboardPool = billboardPool,
        billboardLogo = billboardLogo,
        billboardWatchlist = billboardWatchlist,
        filteredBillboardPool = filteredBillboardPool,
        displayBillboardMovie = displayBillboardMovie,
        billboardSeasonPosterUrl = billboardSeasonPosterUrl,
        billboardTrailerUrl = billboardTrailerUrl
    )
}

@Composable
fun rememberHomeCatalogState(
    activeProfile: UserProfile?,
    viewModel: HomeViewModel
): HomeCatalogState {
    val categories by viewModel.categories.collectAsStateWithLifecycle()
    val isLoading by viewModel.isLoading.collectAsStateWithLifecycle()
    val hasLoadedHome by viewModel.hasLoadedHome.collectAsStateWithLifecycle()
    val filter by viewModel.currentFilter.collectAsStateWithLifecycle()
    val showHeroSection = activeProfile?.safeShowHeroSection != false
    val orderedCategories = remember(categories, filter) {
        orderHomeCategories(categories, filter)
    }
    val topTenFeedKeys = remember(activeProfile?.safeTopTenFeedToggles) {
        activeProfile?.safeTopTenFeedToggles.orEmpty().toSet()
    }
    return HomeCatalogState(
        categories = categories,
        isLoading = isLoading,
        hasLoadedHome = hasLoadedHome,
        filter = filter,
        showHeroSection = showHeroSection,
        orderedCategories = orderedCategories,
        topTenFeedKeys = topTenFeedKeys
    )
}

fun orderHomeCategories(
    categories: List<HomeCategory>,
    filter: String = "all"
): List<HomeCategory> {
    return categories
        .filterNot { category -> category.type == "collection_folder" }
        .mapNotNull { category ->
            val filteredItems = when {
                category.isContinueWatchingCategory() || category.id == "library" -> {
                    if (filter == "all") category.items else {
                        val byFilter = category.items.filter { item -> item.matchesFilter(filter) }
                        if (byFilter.isNotEmpty()) byFilter else category.items
                    }
                }
                filter == "all" -> category.items
                else -> category.items.filter { item -> item.matchesFilter(filter) }
            }
            if (filteredItems.isEmpty()) null
            else if (filteredItems === category.items) category
            else category.copy(items = filteredItems)
        }
}

private val YOUTUBE_ID_REGEX = Regex("""^[A-Za-z0-9_-]{11}$""")
private val YOUTUBE_URL_REGEX = Regex("""(?:youtube\.com/watch\?[^#]*v=|youtube\.com/embed/|youtube\.com/shorts/)([A-Za-z0-9_-]{6,})""", RegexOption.IGNORE_CASE)
private val YOUTUBE_SHORT_REGEX = Regex("""youtu\.be/([A-Za-z0-9_-]{6,})""", RegexOption.IGNORE_CASE)
private val VIDEO_EXTENSIONS = setOf(".mp4", ".m3u8", ".mpd", ".webm", ".mov")

internal fun String.extractYoutubeVideoId(): String? {
    val value = trim()
    if (YOUTUBE_ID_REGEX.matches(value)) return value
    return YOUTUBE_URL_REGEX.find(value)?.groupValues?.getOrNull(1)
        ?: YOUTUBE_SHORT_REGEX.find(value)?.groupValues?.getOrNull(1)
}

internal fun String.isDirectVideoPreviewUrl(): Boolean {
    val value = trim().lowercase()
    return VIDEO_EXTENSIONS.any { value.contains(it) } ||
        value.contains("googlevideo.com") ||
        value.contains("video.twimg.com")
}
