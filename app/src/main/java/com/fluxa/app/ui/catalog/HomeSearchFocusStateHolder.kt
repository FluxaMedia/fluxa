package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.core.rust.FluxaUniFfiCoreStateHandle
import com.fluxa.app.data.remote.Meta
import com.google.gson.Gson
import java.io.Closeable
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class HomeSearchFocusStateHolder(
    initialHistory: List<Meta>,
    private val coreState: FluxaUniFfiCoreStateHandle = FluxaCoreUniFfi.createAppCoreState(
        mapOf(
            "homeSearch" to mapOf(
                "searchHistory" to initialHistory
            )
        )
    ),
    private val ownsCoreState: Boolean = true,
    private val gson: Gson = Gson()
) : Closeable {

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
        set(value) { dispatchHomeSearch("setSearchResults", value) }

    var searchRowsValue: List<SearchResultRow>
        get() = _searchRows.value
        set(value) { dispatchHomeSearch("setSearchRows", value) }

    var searchHistoryValue: List<Meta>
        get() = _searchHistory.value
        set(value) { dispatchHomeSearch("setSearchHistory", value) }

    var focusedMovieValue: Meta?
        get() = _focusedMovie.value
        set(value) { dispatchHomeSearch("setFocusedMovie", value) }

    var focusedMovieTrailerUrlValue: String?
        get() = _focusedMovieTrailerUrl.value
        set(value) { dispatchHomeSearch("setFocusedMovieTrailerUrl", value) }

    var previewUrlValue: String?
        get() = _previewUrl.value
        set(value) { dispatchHomeSearch("setPreviewUrl", value) }

    private fun dispatchHomeSearch(type: String, value: Any?) {
        applySnapshot(coreState.dispatch(CoreAction(type = type, value = value)))
    }

    private fun applySnapshot(snapshotJson: String) {
        val state = gson.fromJson(snapshotJson, CoreStateSnapshot::class.java) ?: return
        _searchResults.value = state.homeSearch.searchResults ?: emptyList()
        _searchRows.value = state.homeSearch.searchRows ?: emptyList()
        _searchHistory.value = state.homeSearch.searchHistory ?: emptyList()
        _focusedMovie.value = state.homeSearch.focusedMovie
        _focusedMovieTrailerUrl.value = state.homeSearch.focusedMovieTrailerUrl
        _previewUrl.value = state.homeSearch.previewUrl
    }

    override fun close() {
        if (ownsCoreState) {
            coreState.close()
        }
    }

    private data class CoreAction(
        val type: String,
        val value: Any?
    )

    private data class CoreStateSnapshot(
        val homeSearch: CoreHomeSearchSnapshot = CoreHomeSearchSnapshot()
    )

    private data class CoreHomeSearchSnapshot(
        val searchResults: List<Meta> = emptyList(),
        val searchRows: List<SearchResultRow> = emptyList(),
        val searchHistory: List<Meta> = emptyList(),
        val focusedMovie: Meta? = null,
        val focusedMovieTrailerUrl: String? = null,
        val previewUrl: String? = null
    )
}
