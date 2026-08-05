package com.fluxa.app.desktop.detail

import com.fluxa.app.data.local.WatchlistStore
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.desktop.addonstore.DesktopAddonRegistry
import com.fluxa.app.desktop.home.CINEMETA_TRANSPORT_URL
import com.fluxa.app.domain.discovery.supportsStremioResource
import com.fluxa.app.shared.feature.detail.DetailCastMemberUiModel
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.detail.DetailEpisodeUiModel
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.detail.DetailStreamUiModel
import com.fluxa.app.shared.feature.detail.DetailUiModel
import com.fluxa.app.shared.feature.detail.DetailUiState
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

class DesktopDetailDataSource(
    private val addonRepository: AddonRepository,
    private val watchlistStore: WatchlistStore,
    private val addonRegistry: DesktopAddonRegistry
) : DetailDataSource {
    private val state = MutableStateFlow(DetailUiState())
    private var metaDetail: MetaDetail? = null
    private var addonTransportUrl: String = CINEMETA_TRANSPORT_URL
    private var selectedSeason: Int = 1
    private var selectedEpisodeId: String? = null

    override fun observeDetail(id: String, type: String): Flow<DetailUiState> = state.asStateFlow()

    override suspend fun loadDetail(request: DetailRequestUiModel) {
        addonTransportUrl = request.source.addonTransportUrl ?: CINEMETA_TRANSPORT_URL
        selectedSeason = request.targetSeason ?: 1
        selectedEpisodeId = request.lastVideoId
        state.value = DetailUiState(content = null, isLoading = true)
        val detail = addonRepository.getMetaDetailFromSpecificAddon(addonTransportUrl, request.type, request.id)
        metaDetail = detail
        if (detail == null) {
            state.value = DetailUiState(content = null, isLoading = false)
            return
        }
        publish(isInWatchlist = watchlistStore.isInWatchlist(request.id))
    }

    override suspend fun toggleWatchlist(id: String, type: String) {
        val detail = metaDetail ?: return
        watchlistStore.toggleWatchlist(
            Meta(
                id = detail.id,
                name = detail.name,
                type = detail.type,
                poster = detail.poster,
                background = detail.background,
                logo = detail.logo,
                description = detail.description
            )
        )
        publish(isInWatchlist = watchlistStore.isInWatchlist(id))
    }

    override suspend fun selectSeason(season: Int) {
        selectedSeason = season
        selectedEpisodeId = null
        publish(isInWatchlist = state.value.content?.isInWatchlist ?: false)
    }

    override suspend fun selectEpisode(episodeId: String) {
        selectedEpisodeId = episodeId
        publish(isInWatchlist = state.value.content?.isInWatchlist ?: false)
    }

    override suspend fun loadSources(contentId: String, contentType: String, episodeId: String?) {
        state.value = state.value.copy(content = state.value.content?.copy(isLoadingStreams = true))
        val targetId = episodeId ?: contentId
        val streamAddons = addonRegistry.enabledUrls().mapNotNull { url ->
            val descriptor = runCatching { addonRepository.getAddonManifest(url) }.getOrNull()
            descriptor?.takeIf { it.supportsStremioResource("stream", contentType, targetId) }
        }
        val uiStreams = coroutineScope {
            streamAddons.map { addon -> async { fetchStreams(addon, contentType, targetId) } }.awaitAll().flatten()
        }
        state.value = state.value.copy(
            content = state.value.content?.copy(
                streams = uiStreams,
                isLoadingStreams = false,
                availableAddons = uiStreams.map { it.addonName }.distinct(),
                hasStreamProviders = streamAddons.isNotEmpty()
            )
        )
    }

    private suspend fun fetchStreams(addon: AddonDescriptor, contentType: String, targetId: String): List<DetailStreamUiModel> {
        val streams = runCatching {
            addonRepository.getStreamsFromAddon(addon.transportUrl, addon.manifest.name, contentType, targetId)
        }.getOrDefault(emptyList())
        return streams.mapNotNull { stream ->
            val url = stream.url ?: return@mapNotNull null
            DetailStreamUiModel(
                addonName = addon.manifest.name,
                title = stream.title ?: stream.name ?: url,
                playableUrl = url
            )
        }
    }

    override suspend fun selectAddonFilter(addonName: String?) = Unit
    override suspend fun downloadEpisode(episodeId: String) = Unit
    override suspend fun downloadSeason(season: Int) = Unit

    private fun publish(isInWatchlist: Boolean) {
        val detail = metaDetail ?: return
        val videos = detail.videos.orEmpty()
        val availableSeasons = videos.mapNotNull { it.season }
            .filter { it >= 0 }
            .distinct()
            .sorted()
            .ifEmpty { listOf(1) }
        val seasonEpisodes = videos.filter { it.season == selectedSeason }
            .sortedBy { it.number ?: 0 }
            .map { it.toDetailEpisodeUiModel() }
        val resolvedEpisodeId = selectedEpisodeId ?: seasonEpisodes.firstOrNull()?.id
        state.value = DetailUiState(
            content = DetailUiModel(
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
                cast = detail.cast.orEmpty().mapNotNull { member ->
                    member.name.takeIf(String::isNotBlank)?.let { name ->
                        DetailCastMemberUiModel(
                            name = name,
                            character = member.character?.takeIf(String::isNotBlank),
                            profileUrl = member.profilePath?.takeIf(String::isNotBlank)
                        )
                    }
                },
                isInWatchlist = isInWatchlist,
                relatedItems = emptyList(),
                availableSeasons = availableSeasons,
                selectedSeason = selectedSeason,
                seasonEpisodes = seasonEpisodes,
                selectedEpisodeId = resolvedEpisodeId,
                hasStreamProviders = false
            ),
            isLoading = false
        )
    }
}

private fun Video.toDetailEpisodeUiModel(): DetailEpisodeUiModel = DetailEpisodeUiModel(
    id = id,
    season = season,
    number = number,
    title = name.orEmpty(),
    description = overview,
    thumbnailUrl = thumbnail,
    releaseLabel = released,
    runtimeLabel = null,
    isUpcoming = false,
    isWatched = false
)
