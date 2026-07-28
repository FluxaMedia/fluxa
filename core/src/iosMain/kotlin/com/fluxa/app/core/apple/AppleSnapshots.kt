package com.fluxa.app.core.apple

data class AppleCatalogHomeSnapshot(
    val rows: List<AppleCatalogRowSnapshot> = emptyList(),
    val isLoading: Boolean = false
)

data class AppleCatalogRowSnapshot(
    val id: String,
    val title: String,
    val items: List<AppleCatalogItemSnapshot>,
    val canLoadMore: Boolean = false
)

data class AppleCatalogItemSnapshot(
    val id: String,
    val type: String,
    val title: String,
    val subtitle: String = "",
    val artworkUrl: String? = null,
    val logoUrl: String? = null,
    val addonTransportUrl: String? = null,
    val catalogType: String? = null,
    val progress: Float? = null,
    val topTenRank: Int? = null
)

data class AppleDetailRequestSnapshot(
    val id: String,
    val type: String,
    val addonTransportUrl: String? = null,
    val catalogType: String? = null,
    val title: String? = null
)

data class AppleDetailStreamSnapshot(
    val addonName: String,
    val title: String,
    val playableUrl: String,
    val requestHeadersJson: String = "{}"
)

data class AppleDetailSnapshot(
    val id: String,
    val type: String,
    val title: String,
    val description: String = "",
    val posterUrl: String? = null,
    val backgroundUrl: String? = null,
    val logoUrl: String? = null,
    val releaseLabel: String = "",
    val ratingLabel: String = "",
    val isInWatchlist: Boolean = false,
    val isLoading: Boolean = false,
    val errorKey: String? = null,
    val streams: List<AppleDetailStreamSnapshot> = emptyList(),
    val hasStreamProviders: Boolean = true
)

data class ApplePlaybackRequestSnapshot(
    val playableUrl: String,
    val title: String,
    val resumePositionMs: Long = 0L,
    val requestHeadersJson: String = "{}"
)

data class AppleDiscoverRequestSnapshot(
    val contentType: String,
    val catalogKey: String? = null,
    val genre: String? = null
)

data class AppleDiscoverFilterOptionSnapshot(
    val id: String? = null,
    val label: String
)

data class AppleDiscoverSnapshot(
    val request: AppleDiscoverRequestSnapshot,
    val catalogOptions: List<AppleDiscoverFilterOptionSnapshot> = emptyList(),
    val genreOptions: List<AppleDiscoverFilterOptionSnapshot> = emptyList(),
    val results: List<AppleCatalogItemSnapshot> = emptyList(),
    val isLoading: Boolean = false
)

data class AppleSearchSnapshot(
    val query: String,
    val results: List<AppleCatalogItemSnapshot> = emptyList(),
    val isLoading: Boolean = false
)
