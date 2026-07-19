package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.DetailTrailer
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video

data class DetailUiState(
    val detail: MetaDetail? = null,
    val streams: List<Stream> = emptyList(),
    val isLoading: Boolean = false,
    val isLoadingStreams: Boolean = false,
    val autoSelectedStream: Stream? = null,
    val isInWatchlist: Boolean = false,
    val feedback: Boolean? = null,
    val availableAddons: List<String> = emptyList(),
    val loadingAddonNames: List<String> = emptyList(),
    val hasStreamProviders: Boolean = true,
    val selectedAddon: String? = null,
    val filteredStreams: List<Stream> = emptyList(),
    val seasonEpisodes: List<Video> = emptyList(),
    val watchedVideoIds: List<String> = emptyList(),
    val localWatchedVideoIds: Set<String> = emptySet(),
    val savedPlayback: Meta? = null,
    val similarItems: List<Meta> = emptyList(),
    val trailers: List<DetailTrailer> = emptyList(),
    val trailerUrl: String? = null,
    val userAddons: List<AddonDescriptor> = emptyList()
)
