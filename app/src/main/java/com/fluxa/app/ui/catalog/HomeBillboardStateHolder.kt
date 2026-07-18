package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.player.TrailerCue
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class HomeBillboardStateHolder {
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()
    private val _pool = MutableStateFlow<List<Meta>>(emptyList())
    val pool: StateFlow<List<Meta>> = _pool.asStateFlow()
    private val _index = MutableStateFlow(0)
    val index: StateFlow<Int> = _index.asStateFlow()
    private val _movie = MutableStateFlow<Meta?>(null)
    val movie: StateFlow<Meta?> = _movie.asStateFlow()
    private val _logo = MutableStateFlow<String?>(null)
    val logo: StateFlow<String?> = _logo.asStateFlow()
    private val _watchlist = MutableStateFlow(false)
    val watchlist: StateFlow<Boolean> = _watchlist.asStateFlow()
    private val _nextEpisode = MutableStateFlow<String?>(null)
    val nextEpisode: StateFlow<String?> = _nextEpisode.asStateFlow()
    private val _trailerUrl = MutableStateFlow<String?>(null)
    val trailerUrl: StateFlow<String?> = _trailerUrl.asStateFlow()
    private val _trailerSubtitleCues = MutableStateFlow<List<TrailerCue>>(emptyList())
    val trailerSubtitleCues: StateFlow<List<TrailerCue>> = _trailerSubtitleCues.asStateFlow()
    private val _seasonPosterUrl = MutableStateFlow<String?>(null)
    val seasonPosterUrl: StateFlow<String?> = _seasonPosterUrl.asStateFlow()

    var errorValue: String?
        get() = _error.value
        set(value) { _error.value = value }
    var poolValue: List<Meta>
        get() = _pool.value
        set(value) { _pool.value = value }
    var indexValue: Int
        get() = _index.value
        set(value) { _index.value = value }
    var movieValue: Meta?
        get() = _movie.value
        set(value) { _movie.value = value }
    var logoValue: String?
        get() = _logo.value
        set(value) { _logo.value = value }
    var watchlistValue: Boolean
        get() = _watchlist.value
        set(value) { _watchlist.value = value }
    var nextEpisodeValue: String?
        get() = _nextEpisode.value
        set(value) { _nextEpisode.value = value }
    var trailerUrlValue: String?
        get() = _trailerUrl.value
        set(value) { _trailerUrl.value = value }
    var trailerSubtitleCuesValue: List<TrailerCue>
        get() = _trailerSubtitleCues.value
        set(value) { _trailerSubtitleCues.value = value }
    var seasonPosterUrlValue: String?
        get() = _seasonPosterUrl.value
        set(value) { _seasonPosterUrl.value = value }
}
