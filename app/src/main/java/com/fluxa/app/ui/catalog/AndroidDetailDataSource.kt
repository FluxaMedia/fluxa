package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.detail.DetailCastMemberUiModel
import com.fluxa.app.shared.feature.detail.DetailEpisodeUiModel
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.detail.DetailRatingUiModel
import com.fluxa.app.shared.feature.detail.DetailDiscussionCommentUiModel
import com.fluxa.app.shared.feature.detail.DetailStreamUiModel
import com.fluxa.app.shared.feature.detail.DetailTrailerUiModel
import com.fluxa.app.shared.feature.detail.DetailUiModel
import com.fluxa.app.shared.feature.detail.DetailUiState as SharedDetailUiState
import com.fluxa.app.shared.feature.localmedia.LocalMediaLibraryService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.withContext
import com.fluxa.app.ui.catalog.formatRuntimeLabel

class AndroidDetailDataSource(
    val detailViewModel: DetailViewModel,
    private val activeProfile: () -> UserProfile?,
    private val localMediaLibrary: LocalMediaLibraryService,
    private val deviceType: DeviceType = DeviceType.Mobile,
) : DetailDataSource {

    private var selectedSeason: Int = 1
    private val selectedEpisodeIdFlow = MutableStateFlow<String?>(null)
    private var requestedVideoId: String? = null
    private var requestedProgress: Long? = null
    private var requestedProgressPercent: Float? = null
    private var initialMeta: Meta? = null
    private var strictProviderData: Boolean = false

    fun setInitialMeta(value: Meta?) {
        initialMeta = value
    }

    override fun observeDetail(id: String, type: String): Flow<SharedDetailUiState> {
        return combine(detailViewModel.uiState, selectedEpisodeIdFlow, localMediaLibrary.state) { state, selectedEpisodeId, _ ->
            val profile = activeProfile()
            val currentSelectedSeason = selectedSeason
            val currentRequestedVideoId = requestedVideoId
            val currentRequestedProgress = requestedProgress
            val currentRequestedProgressPercent = requestedProgressPercent
            withContext(Dispatchers.Default) {
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
                        val effectiveResumeVideoId = currentRequestedVideoId ?: state.savedPlayback?.lastVideoId
                        val savedPlaybackProgressPercent = state.savedPlayback?.resumeProgressPercent
                            ?: state.savedPlayback?.let { saved ->
                                val offset = saved.timeOffset
                                val total = saved.duration
                                if (offset != null && total != null && total > 0L) {
                                    (offset.toFloat() / total.toFloat()) * 100f
                                } else {
                                    null
                                }
                            }
                        val effectiveResumeProgressPercent = currentRequestedProgressPercent ?: savedPlaybackProgressPercent
                        val effectiveResumeProgress = if (effectiveResumeProgressPercent == null) {
                            currentRequestedProgress ?: state.savedPlayback?.timeOffset ?: 0L
                        } else {
                            0L
                        }
                        val localStreams = localMediaLibrary.playbackStreams(detail.id, detail.type, currentEpisodeId).map { stream ->
                            DetailStreamUiModel(
                                addonName = "Local · ${stream.sourceLabel}",
                                title = stream.title,
                                playableUrl = stream.playableUrl,
                                sourceKind = "local",
                            )
                        }
                        val remoteStreams = state.filteredStreams.toUiModels()
                        DetailUiModel(
                            id = detail.id,
                            type = detail.type,
                            title = detail.name,
                            description = detail.description.orEmpty(),
                            tmdbId = detail.tmdbId,
                            posterUrl = detail.poster,
                            backgroundUrl = detail.background,
                            logoUrl = detail.logo,
                            trailerUrl = state.trailerUrl,
                            trailers = state.trailers.map { trailer ->
                                DetailTrailerUiModel(
                                    id = trailer.id,
                                    title = trailer.title,
                                    thumbnailUrl = trailer.thumbnail,
                                    sourceLabel = trailer.source.takeUnless { it.equals("addon", ignoreCase = true) }
                                        .orEmpty()
                                )
                            },
                            releaseLabel = detail.releaseInfo.orEmpty(),
                            ratingLabel = detail.imdbRating.orEmpty(),
                            ratings = detail.ratings.orEmpty().mapNotNull { rating ->
                                rating.value?.takeIf(String::isNotBlank)?.let { DetailRatingUiModel(rating.source, it) }
                            },
                            traktComments = state.traktComments.map { DetailDiscussionCommentUiModel(it.author, it.body, it.likes, it.spoiler) },
                            mdblistDiscussion = state.mdblistDiscussion.map { DetailDiscussionCommentUiModel(it.author, it.body, it.likes, it.spoiler) },
                            runtimeLabel = formatRuntimeLabel(detail.runtime),
                            ageRating = detail.ageRating,
                            genres = detail.genres.orEmpty(),
                            cast = detail.cast.orEmpty().mapNotNull { member ->
                                member.name.takeIf(String::isNotBlank)?.let { name ->
                                    DetailCastMemberUiModel(
                                        name = name,
                                        character = member.character?.takeIf(String::isNotBlank),
                                        profileUrl = member.profilePath?.takeIf(String::isNotBlank)
                                    )
                                }
                            },
                            isInWatchlist = state.isInWatchlist,
                            isLiked = state.feedback == true,
                            supportsWatchlist = state.supportsWatchlist,
                            supportsLike = state.supportsLike,
                            relatedItems = state.similarItems.toCatalogItems(profile, deviceType = deviceType),
                            availableSeasons = availableSeasons,
                            selectedSeason = currentSelectedSeason,
                            seasonEpisodes = state.seasonEpisodes.map { it.toUiModel(effectiveWatched) },
                            selectedEpisodeId = currentEpisodeId,
                            resumeVideoId = effectiveResumeVideoId,
                            resumeProgress = effectiveResumeProgress,
                            resumeProgressPercent = effectiveResumeProgressPercent,
                            streams = (localStreams + remoteStreams).distinctBy { it.playableUrl },
                            isLoadingStreams = state.isLoadingStreams,
                            availableAddons = (localStreams.map { it.addonName } + state.availableAddons).distinct(),
                            loadingAddonNames = state.loadingAddonNames,
                            selectedAddon = state.selectedAddon,
                            hasStreamProviders = state.hasStreamProviders || localStreams.isNotEmpty(),
                            addonPriorityOrder = (listOf("Local") + state.userAddons.map { it.manifest.name }).distinct()
                        )
                    },
                    isLoading = state.isLoading
                )
            }
        }
    }

    override suspend fun loadDetail(request: DetailRequestUiModel) {
        selectedSeason = request.targetSeason ?: 1
        selectedEpisodeIdFlow.value = request.lastVideoId
        requestedVideoId = request.lastVideoId
        requestedProgress = request.initialProgress
        requestedProgressPercent = request.initialProgressPercent
        strictProviderData = request.source.strictProviderData
        val providerMeta = initialMeta?.takeIf { it.id == request.id && it.type == request.type }
            ?: request.initialContent?.toProviderMeta()
        detailViewModel.loadDetail(
            type = request.type,
            id = request.id,
            profile = activeProfile(),
            sourceAddonTransportUrl = request.source.addonTransportUrl,
            sourceAddonCatalogType = request.source.catalogType,
            initialMeta = providerMeta,
            strictProviderData = strictProviderData,
            providerId = request.source.providerId,
            providerAccountId = request.source.providerAccountId
        )
    }

    override suspend fun toggleWatchlist(id: String, type: String) {
        detailViewModel.toggleWatchlist()
    }

    override suspend fun toggleLike(id: String, type: String) {
        val wasLiked = detailViewModel.uiState.value.feedback == true
        detailViewModel.setFeedback(!wasLiked)
    }

    override suspend fun selectSeason(season: Int) {
        selectedSeason = season
        selectedEpisodeIdFlow.value = null
        val id = detailViewModel.uiState.value.detail?.id ?: return
        detailViewModel.loadSeason(id, season)
    }

    override suspend fun selectEpisode(episodeId: String) {
        selectedEpisodeIdFlow.value = episodeId
        val type = detailViewModel.uiState.value.detail?.type ?: return
        detailViewModel.fetchStreamsForSelection(type, episodeId)
    }

    override suspend fun loadSources(contentId: String, contentType: String, episodeId: String?) {
        selectedEpisodeIdFlow.value = episodeId
        detailViewModel.fetchStreamsForSelection(contentType, episodeId ?: contentId)
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

    fun resolveStream(playableUrl: String): com.fluxa.app.data.remote.Stream? {
        detailViewModel.uiState.value.streams.firstOrNull { it.playableUrl == playableUrl }?.let { return it }
        val detail = detailViewModel.uiState.value.detail ?: return null
        val videoId = selectedEpisodeIdFlow.value
        val local = localMediaLibrary.playbackStreams(detail.id, detail.type, videoId)
            .firstOrNull { it.playableUrl == playableUrl }
            ?: return null
        return com.fluxa.app.data.remote.Stream(
            name = local.sourceLabel,
            title = local.title,
            url = local.playableUrl,
            addonName = "Local",
        )
    }
}

private fun DetailUiModel.toProviderMeta(): Meta = Meta(
    id = id,
    name = title,
    type = type,
    poster = posterUrl,
    background = backgroundUrl,
    logo = logoUrl,
    description = description,
    releaseInfo = releaseLabel,
    runtime = formatRuntimeLabel(runtimeLabel),
    ageRating = ageRating,
    genres = genres,
    timeOffset = resumeProgress,
    resumeProgressPercent = resumeProgressPercent,
    lastVideoId = resumeVideoId,
    continueWatchingPoster = posterUrl,
    continueWatchingBackground = backgroundUrl
)

private fun Video.toUiModel(watchedIds: Set<String>): DetailEpisodeUiModel = DetailEpisodeUiModel(
    id = id,
    season = season,
    number = number,
    title = name.orEmpty(),
    description = overview,
    thumbnailUrl = thumbnail,
    releaseLabel = released,
    runtimeLabel = episodeRuntime?.takeIf { it > 0 }?.let { formatRuntimeLabel("${it}m") },
    isUpcoming = detailIsUpcoming(released),
    isWatched = id in watchedIds
)

internal fun com.fluxa.app.data.remote.Stream.toDetailStreamUiModel(): DetailStreamUiModel? {
    val url = playableUrl ?: return null
    return DetailStreamUiModel(
        addonName = addonName.orEmpty(),
        title = title ?: name.orEmpty(),
        playableUrl = url,
        name = name.orEmpty()
    )
}

private fun List<com.fluxa.app.data.remote.Stream>.toUiModels(): List<DetailStreamUiModel> =
    toList().mapNotNull { stream ->
        stream.toDetailStreamUiModel()
    }
