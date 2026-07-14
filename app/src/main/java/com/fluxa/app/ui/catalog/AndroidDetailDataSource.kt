package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Video
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.detail.DetailEpisodeUiModel
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.detail.DetailStreamUiModel
import com.fluxa.app.shared.feature.detail.DetailUiModel
import com.fluxa.app.shared.feature.detail.DetailUiState as SharedDetailUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class AndroidDetailDataSource(
    val detailViewModel: DetailViewModel,
    private val activeProfile: () -> UserProfile?
) : DetailDataSource {

    private var selectedSeason: Int = 1
    private var selectedEpisodeId: String? = null
    private var requestedVideoId: String? = null
    private var requestedProgress: Long? = null

    override fun observeDetail(id: String, type: String): Flow<SharedDetailUiState> {
        return detailViewModel.uiState.map { state ->
            SharedDetailUiState(
                content = state.detail?.let { detail ->
                    val effectiveWatched = state.watchedVideoIds.toSet() + state.localWatchedVideoIds
                    val availableSeasons = buildList {
                        val seasonsCount = detail.seasonsCount ?: 0
                        if (seasonsCount > 0) addAll(1..seasonsCount)
                        addAll(detail.videos?.mapNotNull { it.season }.orEmpty().filter { it > 0 })
                        if (detail.videos?.any { it.season == 0 } == true) add(0)
                    }.distinct().sortedWith(compareBy<Int> { if (it == 0) 1 else 0 }.thenBy { it }).ifEmpty { listOf(1) }
                    val currentEpisodeId = selectedEpisodeId
                        ?: state.seasonEpisodes.firstOrNull { !detailIsUpcoming(it.released) }?.id
                        ?: state.seasonEpisodes.firstOrNull()?.id
                    val effectiveResumeVideoId = requestedVideoId ?: state.savedPlayback?.lastVideoId
                    val effectiveResumeProgress = requestedProgress ?: state.savedPlayback?.timeOffset ?: 0L
                    DetailUiModel(
                        id = detail.id,
                        type = detail.type,
                        title = detail.name,
                        description = detail.description.orEmpty(),
                        posterUrl = detail.poster,
                        backgroundUrl = detail.background,
                        logoUrl = detail.logo,
                        releaseLabel = detail.releaseInfo.orEmpty(),
                        ratingLabel = detail.imdbRating.orEmpty(),
                        runtimeLabel = detail.runtime,
                        ageRating = detail.ageRating,
                        castNames = detail.cast.orEmpty().mapNotNull { it.name.takeIf { name -> name.isNotBlank() } },
                        isInWatchlist = state.isInWatchlist,
                        relatedItems = state.similarItems.toCatalogItems(activeProfile()),
                        availableSeasons = availableSeasons,
                        selectedSeason = selectedSeason,
                        seasonEpisodes = state.seasonEpisodes.map { it.toUiModel(effectiveWatched) },
                        selectedEpisodeId = currentEpisodeId,
                        resumeVideoId = effectiveResumeVideoId,
                        resumeProgress = effectiveResumeProgress,
                        streams = state.filteredStreams.toUiModels(),
                        isLoadingStreams = state.isLoadingStreams,
                        availableAddons = state.availableAddons,
                        selectedAddon = state.selectedAddon,
                        hasStreamProviders = state.hasStreamProviders
                    )
                },
                isLoading = state.isLoading
            )
        }
    }

    override suspend fun loadDetail(request: DetailRequestUiModel) {
        selectedSeason = request.targetSeason ?: 1
        selectedEpisodeId = request.lastVideoId
        requestedVideoId = request.lastVideoId
        requestedProgress = request.initialProgress
        detailViewModel.loadDetail(
            type = request.type,
            id = request.id,
            profile = activeProfile(),
            sourceAddonTransportUrl = request.source.addonTransportUrl,
            sourceAddonCatalogType = request.source.catalogType
        )
    }

    override suspend fun toggleWatchlist(id: String, type: String) {
        detailViewModel.toggleWatchlist()
    }

    override suspend fun selectSeason(season: Int) {
        selectedSeason = season
        selectedEpisodeId = null
        val id = detailViewModel.uiState.value.detail?.id ?: return
        detailViewModel.loadSeason(id, season)
    }

    override suspend fun selectEpisode(episodeId: String) {
        selectedEpisodeId = episodeId
        val type = detailViewModel.uiState.value.detail?.type ?: return
        detailViewModel.fetchStreamsForSelection(type, episodeId)
    }

    override suspend fun selectAddonFilter(addonName: String?) {
        detailViewModel.setSelectedAddon(addonName)
    }

    override suspend fun downloadEpisode(episodeId: String) {
        val episode = detailViewModel.uiState.value.seasonEpisodes.firstOrNull { it.id == episodeId } ?: return
        detailViewModel.downloadEpisodes(listOf(episode))
    }

    override suspend fun downloadSeason(season: Int) {
        val episodes = detailViewModel.uiState.value.seasonEpisodes.filter { (it.season ?: season) == season }
        detailViewModel.downloadEpisodes(episodes)
    }

    fun resolveStream(playableUrl: String): com.fluxa.app.data.remote.Stream? =
        detailViewModel.uiState.value.streams.firstOrNull { it.playableUrl == playableUrl }
}

private fun Video.toUiModel(watchedIds: Set<String>): DetailEpisodeUiModel = DetailEpisodeUiModel(
    id = id,
    season = season,
    number = number,
    title = name.orEmpty(),
    description = overview,
    thumbnailUrl = thumbnail,
    releaseLabel = released,
    runtimeLabel = episodeRuntime?.takeIf { it > 0 }?.let { "${it}m" },
    isUpcoming = detailIsUpcoming(released),
    isWatched = id in watchedIds
)

internal fun com.fluxa.app.data.remote.Stream.toDetailStreamUiModel(): DetailStreamUiModel? {
    val url = playableUrl ?: return null
    return DetailStreamUiModel(
        addonName = addonName.orEmpty(),
        title = title ?: name.orEmpty(),
        playableUrl = url
    )
}

private fun List<com.fluxa.app.data.remote.Stream>.toUiModels(): List<DetailStreamUiModel> =
    mapNotNull { stream ->
        stream.toDetailStreamUiModel()
    }
