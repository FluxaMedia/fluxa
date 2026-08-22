package com.fluxa.app.shared.feature.detail

import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import kotlinx.coroutines.flow.Flow

data class DetailRequestUiModel(
    val id: String,
    val type: String,
    val source: CatalogSourceUiModel = CatalogSourceUiModel(),
    val initialProgress: Long? = null,
    val initialProgressPercent: Float? = null,
    val lastVideoId: String? = null,
    val lastStreamIndex: Int? = null,
    val autoPlay: Boolean = false,
    val targetSeason: Int? = null,
    val targetEpisode: Int? = null,
    val lastStreamUrl: String? = null,
    val lastStreamTitle: String? = null,
    val initialContent: DetailUiModel? = null
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
    val playableUrl: String,
    val requestHeadersJson: String = "{}",
    val name: String = "",
    val sourceKind: String = "addon"
)

data class DetailRatingUiModel(
    val source: String,
    val value: String
)

data class DetailTrailerUiModel(
    val id: String,
    val title: String,
    val thumbnailUrl: String?,
    val sourceLabel: String
)

data class DetailDiscussionCommentUiModel(
    val author: String,
    val body: String,
    val likes: Int = 0,
    val spoiler: Boolean = false
)

data class DetailCastMemberUiModel(
    val name: String,
    val character: String?,
    val profileUrl: String?
)

data class DetailUiModel(
    val id: String,
    val type: String,
    val title: String,
    val description: String,
    val posterUrl: String?,
    val backgroundUrl: String?,
    val logoUrl: String?,
    val trailerUrl: String? = null,
    val trailers: List<DetailTrailerUiModel> = emptyList(),
    val releaseLabel: String,
    val ratingLabel: String,
    val ratings: List<DetailRatingUiModel> = emptyList(),
    val traktComments: List<DetailDiscussionCommentUiModel> = emptyList(),
    val mdblistDiscussion: List<DetailDiscussionCommentUiModel> = emptyList(),
    val runtimeLabel: String?,
    val ageRating: String? = null,
    val genres: List<String> = emptyList(),
    val cast: List<DetailCastMemberUiModel> = emptyList(),
    val isInWatchlist: Boolean,
    val isLiked: Boolean = false,
    val supportsWatchlist: Boolean = true,
    val supportsLike: Boolean = true,
    val relatedItems: List<CatalogItemUiModel>,
    val availableSeasons: List<Int> = emptyList(),
    val selectedSeason: Int = 1,
    val seasonEpisodes: List<DetailEpisodeUiModel> = emptyList(),
    val selectedEpisodeId: String? = null,
    val resumeVideoId: String? = null,
    val resumeProgress: Long = 0L,
    val resumeProgressPercent: Float? = null,
    val streams: List<DetailStreamUiModel> = emptyList(),
    val isLoadingStreams: Boolean = false,
    val availableAddons: List<String> = emptyList(),
    val loadingAddonNames: List<String> = emptyList(),
    val selectedAddon: String? = null,
    val hasStreamProviders: Boolean = true,
    val addonPriorityOrder: List<String> = emptyList(),
    val tmdbId: String? = null
)


enum class DetailScreenStyle {
    Cinematic,
    Classic,
    Compact;

    companion object {
        fun from(value: String?): DetailScreenStyle = when (value?.trim()?.lowercase()) {
            "classic" -> Classic
            "compact" -> Compact
            else -> Cinematic
        }
    }
}

data class DetailPresentationOptions(
    val screenStyle: DetailScreenStyle = DetailScreenStyle.Cinematic,
    val preferClearlogo: Boolean = true,
    val showEpisodeDescriptions: Boolean = true,
    val showCast: Boolean = true,
    val showRecommendations: Boolean = true,
    val collapsingHero: Boolean = true,
    val blurUnwatchedEpisodes: Boolean = false,
    val seasonSelectorMode: String = "dropdown",
    val episodeCardsLayout: String = "carousel"
)

data class DetailUiState(
    val content: DetailUiModel? = null,
    val isLoading: Boolean = false,
    val errorKey: String? = null
)

sealed interface DetailAction {
    data class Play(val fromStart: Boolean = false) : DetailAction
    data object ToggleWatchlist : DetailAction
    data object ToggleLike : DetailAction
    data object ShufflePlay : DetailAction
    data class RelatedItemSelected(val item: CatalogItemUiModel) : DetailAction
    data class SeasonSelected(val season: Int) : DetailAction
    data class EpisodeSelected(val episodeId: String) : DetailAction
    data class StreamSelected(val stream: DetailStreamUiModel, val episodeId: String?) : DetailAction
    data class AddonFilterSelected(val addonName: String?) : DetailAction
    data class DownloadEpisode(val episodeId: String) : DetailAction
    data class DownloadSeason(val season: Int) : DetailAction
    data object RetrySourcesRequested : DetailAction
}

sealed interface DetailNavigationEvent {
    data class PlayStream(val stream: DetailStreamUiModel, val episodeId: String?, val resumeProgress: Long = 0L, val resumeProgressPercent: Float? = null) : DetailNavigationEvent
    data class SelectSources(val episodeId: String?, val resumeProgress: Long = 0L, val resumeProgressPercent: Float? = null) : DetailNavigationEvent
}

object DetailNavigationLogic {
    fun resumeProgressFor(resumeVideoId: String?, resumeProgress: Long, targetVideoId: String?): Long =
        if (targetVideoId != null && targetVideoId == resumeVideoId) resumeProgress else 0L

    fun resumeProgressPercentFor(resumeVideoId: String?, resumeProgressPercent: Float?, targetVideoId: String?): Float? =
        if (targetVideoId != null && targetVideoId == resumeVideoId) resumeProgressPercent else null

    fun forStream(
        contentResumeVideoId: String?,
        contentResumeProgress: Long,
        contentResumeProgressPercent: Float? = null,
        stream: DetailStreamUiModel,
        episodeId: String?
    ): DetailNavigationEvent.PlayStream {
        val targetVideoId = episodeId ?: contentResumeVideoId
        return DetailNavigationEvent.PlayStream(
            stream = stream,
            episodeId = episodeId,
            resumeProgress = resumeProgressFor(contentResumeVideoId, contentResumeProgress, targetVideoId),
            resumeProgressPercent = resumeProgressPercentFor(contentResumeVideoId, contentResumeProgressPercent, targetVideoId)
        )
    }

    fun forPlay(
        contentId: String?,
        contentResumeVideoId: String?,
        contentResumeProgress: Long,
        contentResumeProgressPercent: Float? = null,
        episodeId: String?,
        firstStreamIfCs3: DetailStreamUiModel?,
        fromStart: Boolean = false
    ): DetailNavigationEvent {
        val targetVideoId = episodeId ?: contentResumeVideoId
        val progress = if (fromStart) 0L else resumeProgressFor(contentResumeVideoId, contentResumeProgress, targetVideoId)
        val progressPercent = if (fromStart) null else resumeProgressPercentFor(contentResumeVideoId, contentResumeProgressPercent, targetVideoId)
        val isCs3 = contentId?.startsWith("cs3:") == true || targetVideoId?.startsWith("cs3:") == true
        return if (isCs3 && firstStreamIfCs3 != null) {
            DetailNavigationEvent.PlayStream(firstStreamIfCs3, episodeId, progress, progressPercent)
        } else {
            DetailNavigationEvent.SelectSources(episodeId, progress, progressPercent)
        }
    }
}

interface DetailDataSource {
    fun observeDetail(id: String, type: String): Flow<DetailUiState>
    suspend fun loadDetail(request: DetailRequestUiModel)
    suspend fun toggleWatchlist(id: String, type: String)
    suspend fun toggleLike(id: String, type: String)
    suspend fun shuffleEpisode(): String?
    suspend fun selectSeason(season: Int)
    suspend fun selectEpisode(episodeId: String)
    suspend fun loadSources(contentId: String, contentType: String, episodeId: String?)
    suspend fun selectAddonFilter(addonName: String?)
    suspend fun downloadEpisode(episodeId: String)
    suspend fun downloadSeason(season: Int)
}
