package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.detail.DetailStreamUiModel
import com.fluxa.app.shared.feature.detail.DetailUiModel
import com.fluxa.app.shared.feature.detail.DetailUiState
import com.fluxa.app.data.local.WatchlistStore
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

data class AppleDetailRequestSnapshot(
    val id: String,
    val type: String,
    val addonTransportUrl: String? = null,
    val catalogType: String? = null,
    val title: String? = null
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

data class AppleDetailStreamSnapshot(
    val addonName: String,
    val title: String,
    val playableUrl: String,
    val requestHeadersJson: String = "{}"
)

class AppleDetailDataSource(
    private val watchlistStore: WatchlistStore
) : DetailDataSource {
    private val state = MutableStateFlow(DetailUiState())
    private var onLoadRequested: (AppleDetailRequestSnapshot) -> Unit = {}
    private var onWatchlistRequested: (AppleDetailRequestSnapshot) -> Unit = {}

    override fun observeDetail(id: String, type: String): Flow<DetailUiState> = state.asStateFlow()

    override suspend fun loadDetail(request: DetailRequestUiModel) {
        onLoadRequested(request.toAppleSnapshot())
    }

    override suspend fun toggleWatchlist(id: String, type: String) {
        val content = state.value.content
        if (content != null) {
            watchlistStore.toggleWatchlist(
                Meta(
                    id = content.id,
                    name = content.title,
                    type = content.type,
                    poster = content.posterUrl,
                    background = content.backgroundUrl,
                    logo = content.logoUrl,
                    description = content.description
                )
            )
            state.value = state.value.copy(
                content = content.copy(isInWatchlist = watchlistStore.isInWatchlist(id))
            )
        }
        onWatchlistRequested(AppleDetailRequestSnapshot(id, type, title = state.value.content?.title))
    }

    override suspend fun selectSeason(season: Int) {
        Unit
    }

    override suspend fun selectEpisode(episodeId: String) {
        Unit
    }

    override suspend fun loadSources(contentId: String, contentType: String, episodeId: String?) {
        Unit
    }

    override suspend fun selectAddonFilter(addonName: String?) {
        Unit
    }

    override suspend fun downloadEpisode(episodeId: String) {
        Unit
    }

    override suspend fun downloadSeason(season: Int) {
        Unit
    }

    fun setHandlers(
        load: (AppleDetailRequestSnapshot) -> Unit,
        watchlist: (AppleDetailRequestSnapshot) -> Unit
    ) {
        onLoadRequested = load
        onWatchlistRequested = watchlist
    }

    fun update(snapshot: AppleDetailSnapshot) {
        state.value = DetailUiState(
            content = DetailUiModel(
                id = snapshot.id,
                type = snapshot.type,
                title = snapshot.title,
                description = snapshot.description,
                posterUrl = snapshot.posterUrl,
                backgroundUrl = snapshot.backgroundUrl,
                logoUrl = snapshot.logoUrl,
                releaseLabel = snapshot.releaseLabel,
                ratingLabel = snapshot.ratingLabel,
                runtimeLabel = null,
                isInWatchlist = snapshot.isInWatchlist,
                relatedItems = emptyList(),
                streams = snapshot.streams.map { stream ->
                    DetailStreamUiModel(stream.addonName, stream.title, stream.playableUrl, stream.requestHeadersJson)
                },
                availableAddons = snapshot.streams.map { it.addonName }.distinct(),
                hasStreamProviders = snapshot.hasStreamProviders
            ),
            isLoading = snapshot.isLoading,
            errorKey = snapshot.errorKey
        )
    }

    fun firstPlaybackRequest(resumePositionMs: Long): ApplePlaybackRequestSnapshot? {
        val content = state.value.content ?: return null
        val stream = content.streams.firstOrNull() ?: return null
        return ApplePlaybackRequestSnapshot(stream.playableUrl, content.title, resumePositionMs, stream.requestHeadersJson)
    }
}

private fun DetailRequestUiModel.toAppleSnapshot() = AppleDetailRequestSnapshot(id, type, source.addonTransportUrl, source.catalogType)
