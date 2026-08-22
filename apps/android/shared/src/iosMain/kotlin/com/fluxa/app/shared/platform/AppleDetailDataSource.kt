package com.fluxa.app.shared.platform

import com.fluxa.app.core.apple.AppleDetailEpisodeSnapshot
import com.fluxa.app.core.apple.AppleDetailRequestSnapshot
import com.fluxa.app.core.apple.AppleDetailSeasonRequestSnapshot
import com.fluxa.app.core.apple.AppleDetailSnapshot
import com.fluxa.app.core.apple.AppleDetailStreamSnapshot
import com.fluxa.app.core.apple.AppleDetailStreamsRequestSnapshot
import com.fluxa.app.core.apple.ApplePlaybackRequestSnapshot
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.detail.DetailEpisodeUiModel
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

class AppleDetailDataSource(
    private val watchlistStore: WatchlistStore
) : DetailDataSource {
    private val state = MutableStateFlow(DetailUiState())
    private var onLoadRequested: (AppleDetailRequestSnapshot) -> Unit = {}
    private var onWatchlistRequested: (AppleDetailRequestSnapshot) -> Unit = {}
    private var onSeasonRequested: (AppleDetailSeasonRequestSnapshot) -> Unit = {}
    private var onStreamsRequested: (AppleDetailStreamsRequestSnapshot) -> Unit = {}
    private var onAddonFilterRequested: (String?) -> Unit = {}
    private var onDownloadEpisodeRequested: (AppleDetailStreamsRequestSnapshot) -> Unit = {}
    private var onDownloadSeasonRequested: (AppleDetailSeasonRequestSnapshot) -> Unit = {}
    private var subtitleUrlsByStreamUrl: Map<String, List<String>> = emptyMap()

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

    override suspend fun toggleLike(id: String, type: String) {
        val content = state.value.content ?: return
        val wasLiked = watchlistStore.feedback(id) == true
        watchlistStore.setFeedback(
            id,
            if (wasLiked) null else true,
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
        state.value = state.value.copy(content = content.copy(isLiked = watchlistStore.feedback(id) == true))
    }

    override suspend fun shuffleEpisode(): String? {
        val content = state.value.content ?: return null
        if (content.type != "series") return null
        return content.seasonEpisodes.filterNot { it.isUpcoming }.randomOrNull()?.id
    }

    override suspend fun selectSeason(season: Int) {
        val content = state.value.content ?: return
        state.value = state.value.copy(content = content.copy(selectedSeason = season, seasonEpisodes = emptyList()))
        onSeasonRequested(AppleDetailSeasonRequestSnapshot(content.id, content.type, season))
    }

    override suspend fun selectEpisode(episodeId: String) {
        val content = state.value.content ?: return
        state.value = state.value.copy(content = content.copy(selectedEpisodeId = episodeId))
    }

    override suspend fun loadSources(contentId: String, contentType: String, episodeId: String?) {
        val content = state.value.content
        if (content != null) {
            state.value = state.value.copy(content = content.copy(selectedEpisodeId = episodeId, isLoadingStreams = true))
        }
        onStreamsRequested(AppleDetailStreamsRequestSnapshot(contentId, contentType, episodeId))
    }

    override suspend fun selectAddonFilter(addonName: String?) {
        val content = state.value.content ?: return
        state.value = state.value.copy(content = content.copy(selectedAddon = addonName))
        onAddonFilterRequested(addonName)
    }

    override suspend fun downloadEpisode(episodeId: String) {
        val content = state.value.content ?: return
        onDownloadEpisodeRequested(AppleDetailStreamsRequestSnapshot(content.id, content.type, episodeId))
    }

    override suspend fun downloadSeason(season: Int) {
        val content = state.value.content ?: return
        onDownloadSeasonRequested(AppleDetailSeasonRequestSnapshot(content.id, content.type, season))
    }

    fun setHandlers(
        load: (AppleDetailRequestSnapshot) -> Unit,
        watchlist: (AppleDetailRequestSnapshot) -> Unit,
        season: (AppleDetailSeasonRequestSnapshot) -> Unit = {},
        streams: (AppleDetailStreamsRequestSnapshot) -> Unit = {},
        addonFilter: (String?) -> Unit = {},
        downloadEpisode: (AppleDetailStreamsRequestSnapshot) -> Unit = {},
        downloadSeason: (AppleDetailSeasonRequestSnapshot) -> Unit = {}
    ) {
        onLoadRequested = load
        onWatchlistRequested = watchlist
        onSeasonRequested = season
        onStreamsRequested = streams
        onAddonFilterRequested = addonFilter
        onDownloadEpisodeRequested = downloadEpisode
        onDownloadSeasonRequested = downloadSeason
    }

    fun update(snapshot: AppleDetailSnapshot) {
        subtitleUrlsByStreamUrl = snapshot.streams.associate { it.playableUrl to it.subtitleUrls }
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
                availableSeasons = snapshot.availableSeasons.mapNotNull { it.toIntOrNull() },
                selectedSeason = snapshot.selectedSeason,
                seasonEpisodes = snapshot.seasonEpisodes.map { it.toUiModel() },
                selectedEpisodeId = snapshot.selectedEpisodeId,
                streams = snapshot.streams.map { stream ->
                    DetailStreamUiModel(stream.addonName, stream.title, stream.playableUrl, stream.requestHeadersJson)
                },
                isLoadingStreams = snapshot.isLoadingStreams,
                availableAddons = snapshot.availableAddons.ifEmpty { snapshot.streams.map { it.addonName }.distinct() },
                loadingAddonNames = snapshot.loadingAddonNames,
                selectedAddon = snapshot.selectedAddon,
                hasStreamProviders = snapshot.hasStreamProviders
            ),
            isLoading = snapshot.isLoading,
            errorKey = snapshot.errorKey
        )
    }

    fun playbackRequest(
        stream: DetailStreamUiModel,
        resumePositionMs: Long,
        videoId: String? = null,
    ): ApplePlaybackRequestSnapshot {
        val content = state.value.content
        return ApplePlaybackRequestSnapshot(
            playableUrl = stream.playableUrl,
            title = stream.title,
            resumePositionMs = resumePositionMs,
            requestHeadersJson = stream.requestHeadersJson,
            subtitleUrls = subtitleUrlsByStreamUrl[stream.playableUrl].orEmpty(),
            contentId = content?.id.orEmpty(),
            contentType = content?.type ?: "movie",
            videoId = videoId ?: content?.selectedEpisodeId,
        )
    }

    fun firstPlaybackRequest(resumePositionMs: Long): ApplePlaybackRequestSnapshot? {
        val content = state.value.content ?: return null
        val stream = content.streams.firstOrNull() ?: return null
        return playbackRequest(stream, resumePositionMs, content.selectedEpisodeId)
    }
}

private fun DetailRequestUiModel.toAppleSnapshot() = AppleDetailRequestSnapshot(id, type, source.addonTransportUrl, source.catalogType)

private fun AppleDetailEpisodeSnapshot.toUiModel() = DetailEpisodeUiModel(
    id = id,
    season = season.takeIf { it > 0 },
    number = number.takeIf { it > 0 },
    title = title,
    description = description,
    thumbnailUrl = thumbnailUrl,
    releaseLabel = releaseLabel,
    runtimeLabel = runtimeLabel,
    isUpcoming = isUpcoming,
    isWatched = isWatched
)
