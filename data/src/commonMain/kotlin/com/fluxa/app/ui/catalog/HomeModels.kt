package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.LibraryRemoteSource
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.domain.discovery.DiscoverCatalogOption

data class HomeCatalogSource(
    val transportUrl: String,
    val catalogId: String,
    val type: String,
    val genre: String? = null,
    val displayName: String? = null,
    val emoji: String? = null
)

data class HomeCategory(
    val name: String,
    val items: List<Meta>,
    val id: String,
    val type: String,
    val semanticName: String = name,
    val movieGenre: String? = null,
    val seriesGenre: String? = null,
    val skip: Int = 0,
    val canLoadMore: Boolean = true,
    val catalogId: String = id,
    val addonTransportUrl: String? = null,
    val addonGenre: String? = null,
    val catalogSources: List<HomeCatalogSource>? = emptyList(),
    val showAllSourcesTab: Boolean = false,
    val folderViewMode: String? = null,
    val folderHeroImageUrl: String? = null,
    val folderSourcesLoading: Boolean = false,
    val remoteSources: List<LibraryRemoteSource>? = emptyList(),
    val resultSources: Map<String, HomeCatalogSource> = emptyMap(),
    val addonIconUrl: String? = null
)

typealias SearchResultRow = com.fluxa.app.data.repository.SearchResultRow

data class DiscoverGenreOption(
    val id: String?,
    val label: String
)

data class DirectPlaybackTarget(
    val meta: Meta,
    val videoId: String?,
    val streams: List<Stream>
)

data class DiscoverUiState(
    val results: List<Meta> = emptyList(),
    val isLoading: Boolean = false,
    val genres: List<DiscoverGenreOption> = emptyList(),
    val catalogs: List<DiscoverCatalogOption> = emptyList(),
    val contentTypes: List<String> = emptyList(),
    val resultSources: Map<String, HomeCatalogSource> = emptyMap()
)

data class CalendarUiState(
    val items: List<CalendarUpcomingItem> = emptyList(),
    val isLoading: Boolean = false
)

data class CalendarUpcomingItem(
    val dateIso: String,
    val meta: Meta,
    val title: String,
    val subtitle: String?,
    val poster: String?,
    val episodePoster: String? = null,
    val seasonNumber: Int? = null,
    val episodeNumber: Int? = null,
    val episodeTitle: String? = null
)
