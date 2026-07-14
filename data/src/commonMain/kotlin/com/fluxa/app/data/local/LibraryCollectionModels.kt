package com.fluxa.app.data.local

data class LibraryUserCollection(
    val id: String,
    val title: String,
    val itemIds: List<String>? = emptyList(),
    val imageUrl: String? = null,
    val showOnHome: Boolean? = false,
    val folders: List<LibraryUserCollectionFolder>? = emptyList(),
    val showAllTab: Boolean? = true,
    val viewMode: String? = "FOLLOW_LAYOUT",
    val pinToTop: Boolean? = false,
    val focusGlowEnabled: Boolean? = true
)

data class LibraryCatalogSource(
    val addonId: String? = null,
    val catalogId: String,
    val type: String
)

data class LibraryRemoteSource(
    val provider: String,
    val title: String? = null,
    val mediaType: String? = null,
    val traktListId: Long? = null,
    val tmdbSourceType: String? = null,
    val tmdbId: Long? = null,
    val sortBy: String? = null,
    val sortHow: String? = null,
    val filters: Map<String, String>? = null,
    val addonId: String? = null,
    val catalogId: String? = null,
    val type: String? = null,
    val genre: String? = null
)

data class LibraryUserCollectionFolder(
    val id: String,
    val title: String,
    val imageUrl: String? = null,
    val shape: String? = "poster",
    val catalogId: String? = null,
    val catalogTitle: String? = null,
    val genre: String? = null,
    val hideTitle: Boolean? = false,
    val focusGifEnabled: Boolean? = true,
    val catalogSources: List<LibraryCatalogSource>? = null,
    val sources: List<LibraryRemoteSource>? = null,
    val coverEmoji: String? = null,
    val coverImageUrl: String? = null,
    val focusGifUrl: String? = null,
    val titleLogoUrl: String? = null,
    val heroBackdropUrl: String? = null
)
