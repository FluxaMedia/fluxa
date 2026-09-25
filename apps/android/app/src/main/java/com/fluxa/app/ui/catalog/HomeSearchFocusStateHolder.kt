package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class HomeSearchFocusStateHolder(
    initialHistory: List<Meta>
) {

    private val _searchResults = MutableStateFlow<List<Meta>>(emptyList())
    val searchResults: StateFlow<List<Meta>> = _searchResults.asStateFlow()

    private val _searchRows = MutableStateFlow<List<SearchResultRow>>(emptyList())
    val searchRows: StateFlow<List<SearchResultRow>> = _searchRows.asStateFlow()

    private val _searchHistory = MutableStateFlow(initialHistory)
    val searchHistory: StateFlow<List<Meta>> = _searchHistory.asStateFlow()

    private val _focusedMovie = MutableStateFlow<Meta?>(null)
    val focusedMovie: StateFlow<Meta?> = _focusedMovie.asStateFlow()

    private val _focusedMovieTrailerUrl = MutableStateFlow<String?>(null)
    val focusedMovieTrailerUrl: StateFlow<String?> = _focusedMovieTrailerUrl.asStateFlow()

    private val _previewUrl = MutableStateFlow<String?>(null)
    val previewUrl: StateFlow<String?> = _previewUrl.asStateFlow()

    var searchResultsValue: List<Meta>
        get() = _searchResults.value
        set(value) { _searchResults.value = value }

    var searchRowsValue: List<SearchResultRow>
        get() = _searchRows.value
        set(value) { _searchRows.value = value }

    var searchHistoryValue: List<Meta>
        get() = _searchHistory.value
        set(value) { _searchHistory.value = value }

    var focusedMovieValue: Meta?
        get() = _focusedMovie.value
        set(value) { _focusedMovie.value = value }

    var focusedMovieTrailerUrlValue: String?
        get() = _focusedMovieTrailerUrl.value
        set(value) { _focusedMovieTrailerUrl.value = value }

    var previewUrlValue: String?
        get() = _previewUrl.value
        set(value) { _previewUrl.value = value }
}
