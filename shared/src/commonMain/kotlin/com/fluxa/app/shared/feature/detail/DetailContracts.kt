package com.fluxa.app.shared.feature.detail

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import kotlinx.coroutines.flow.Flow

data class DetailRequestUiModel(
    val id: String,
    val type: String,
    val source: CatalogSourceUiModel = CatalogSourceUiModel(),
    val initialProgress: Long? = null,
    val lastVideoId: String? = null,
    val lastStreamIndex: Int? = null,
    val autoPlay: Boolean = false,
    val targetSeason: Int? = null,
    val targetEpisode: Int? = null,
    val lastStreamUrl: String? = null,
    val lastStreamTitle: String? = null
)

data class DetailEpisodeUiModel(
    val id: String,
    val season: Int?,
    val number: Int?,
    val title: String,
    val description: String?,
    val thumbnailUrl: String?,
    val releaseLabel: String?,
    val runtimeLabel: String?,
    val isUpcoming: Boolean,
    val isWatched: Boolean
)

data class DetailStreamUiModel(
    val addonName: String,
    val title: String,
    val playableUrl: String
)

data class DetailUiModel(
    val id: String,
    val type: String,
    val title: String,
    val description: String,
    val posterUrl: String?,
    val backgroundUrl: String?,
    val logoUrl: String?,
    val releaseLabel: String,
    val ratingLabel: String,
    val runtimeLabel: String?,
    val ageRating: String? = null,
    val castNames: List<String> = emptyList(),
    val isInWatchlist: Boolean,
    val relatedItems: List<CatalogItemUiModel>,
    val availableSeasons: List<Int> = emptyList(),
    val selectedSeason: Int = 1,
    val seasonEpisodes: List<DetailEpisodeUiModel> = emptyList(),
    val selectedEpisodeId: String? = null,
    val resumeVideoId: String? = null,
    val resumeProgress: Long = 0L,
    val streams: List<DetailStreamUiModel> = emptyList(),
    val isLoadingStreams: Boolean = false,
    val availableAddons: List<String> = emptyList(),
    val selectedAddon: String? = null,
    val hasStreamProviders: Boolean = true
)

data class DetailUiState(
    val content: DetailUiModel? = null,
    val isLoading: Boolean = false,
    val errorKey: String? = null
)

sealed interface DetailAction {
    data class Play(val fromStart: Boolean = false) : DetailAction
    data object ToggleWatchlist : DetailAction
    data class RelatedItemSelected(val item: CatalogItemUiModel) : DetailAction
    data class SeasonSelected(val season: Int) : DetailAction
    data class EpisodeSelected(val episodeId: String) : DetailAction
    data class StreamSelected(val stream: DetailStreamUiModel, val episodeId: String?) : DetailAction
    data class AddonFilterSelected(val addonName: String?) : DetailAction
    data class DownloadEpisode(val episodeId: String) : DetailAction
    data class DownloadSeason(val season: Int) : DetailAction
}

sealed interface DetailNavigationEvent {
    data class PlayStream(val stream: DetailStreamUiModel, val episodeId: String?, val resumeProgress: Long = 0L) : DetailNavigationEvent
    data class SelectSources(val episodeId: String?, val resumeProgress: Long = 0L) : DetailNavigationEvent
}

object DetailNavigationLogic {
    fun resumeProgressFor(resumeVideoId: String?, resumeProgress: Long, targetVideoId: String?): Long =
        if (targetVideoId != null && targetVideoId == resumeVideoId) resumeProgress else 0L

    fun forStream(
        contentResumeVideoId: String?,
        contentResumeProgress: Long,
        stream: DetailStreamUiModel,
        episodeId: String?
    ): DetailNavigationEvent.PlayStream {
        val targetVideoId = episodeId ?: contentResumeVideoId
        return DetailNavigationEvent.PlayStream(
            stream = stream,
            episodeId = episodeId,
            resumeProgress = resumeProgressFor(contentResumeVideoId, contentResumeProgress, targetVideoId)
        )
    }

    fun forPlay(
        contentId: String?,
        contentResumeVideoId: String?,
        contentResumeProgress: Long,
        episodeId: String?,
        firstStreamIfCs3: DetailStreamUiModel?,
        fromStart: Boolean = false
    ): DetailNavigationEvent {
        val targetVideoId = episodeId ?: contentResumeVideoId
        val progress = if (fromStart) 0L else resumeProgressFor(contentResumeVideoId, contentResumeProgress, targetVideoId)
        val isCs3 = contentId?.startsWith("cs3:") == true || targetVideoId?.startsWith("cs3:") == true
        return if (isCs3 && firstStreamIfCs3 != null) {
            DetailNavigationEvent.PlayStream(firstStreamIfCs3, episodeId, progress)
        } else {
            DetailNavigationEvent.SelectSources(episodeId, progress)
        }
    }
}

interface DetailDataSource {
    fun observeDetail(id: String, type: String): Flow<DetailUiState>
    suspend fun loadDetail(request: DetailRequestUiModel)
    suspend fun toggleWatchlist(id: String, type: String)
    suspend fun selectSeason(season: Int)
    suspend fun selectEpisode(episodeId: String)
    suspend fun selectAddonFilter(addonName: String?)
    suspend fun downloadEpisode(episodeId: String)
    suspend fun downloadSeason(season: Int)
}
