package com.fluxa.app.ui.catalog

import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.core.StremioId
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.SubtitleData
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.domain.discovery.StreamDiscoveryRequest
import com.fluxa.app.domain.discovery.StreamDiscoveryUseCase
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull

internal class HomePlaybackStreamCoordinator(
    private val repository: StremioRepository,
    private val streamDiscovery: StreamDiscoveryUseCase,
    private val activeProfile: () -> UserProfile?,
    private val userAddons: () -> List<com.fluxa.app.data.remote.AddonDescriptor>,
    private val setDirectLoading: (Boolean) -> Unit
) {
    suspend fun getStreams(type: String, id: String): List<Stream> = withContext(Dispatchers.IO) {
        val language = activeProfile()?.safeLanguage ?: "en"
        val addons = userAddons().ifEmpty {
            repository.getUserAddons(activeProfile()?.authKey ?: "", activeProfile()?.safeLocalAddons)
        }
        val requestIds = buildPlaybackStreamRequestIds(type, id, language)
        for (requestId in requestIds) {
            val streams = streamDiscovery.discover(
                StreamDiscoveryRequest(
                    addons = addons,
                    type = type,
                    id = requestId,
                    language = language,
                    preferFastStart = true
                )
            )
            if (streams.isNotEmpty()) return@withContext streams
        }
        emptyList()
    }

    suspend fun prepareDirectPlayback(meta: Meta): DirectPlaybackTarget? = withContext(Dispatchers.IO) {
        setDirectLoading(true)
        try {
            val language = activeProfile()?.safeLanguage ?: "en"
            val detail = withTimeoutOrNull(3500) { getConfiguredMetaDetail(meta.type, meta.id, language) }
            val plan = FluxaCoreNative.directPlaybackPlan(meta, detail, ReleaseDateUtils.todayIso())
            val playbackMeta = plan.meta ?: meta
            val targetVideoId = plan.targetVideoId
            val lookupId = plan.lookupId.ifBlank { targetVideoId ?: detail?.id ?: meta.id }
            val streams = getStreams(meta.type, lookupId)
            if (streams.isEmpty()) null else DirectPlaybackTarget(
                meta = playbackMeta,
                videoId = targetVideoId,
                streams = streams
            )
        } finally {
            setDirectLoading(false)
        }
    }

    suspend fun getSeasonEpisodes(id: String, seasonNumber: Int, language: String): List<Video> {
        val profile = activeProfile()
        return repository.getTvSeason(
            id = id,
            seasonNumber = seasonNumber,
            language = language,
            authKey = profile?.authKey.orEmpty(),
            localAddons = profile?.safeLocalAddons.orEmpty(),
            useConfiguredAddons = true
        )
    }

    suspend fun getSubtitlesFromAddon(baseUrl: String, type: String, id: String, extra: String = ""): List<SubtitleData> {
        return repository.getSubtitlesFromAddon(baseUrl, type, id, extra)
    }

    suspend fun getIntroSegments(
        imdbId: String,
        season: Int,
        episode: Int,
        title: String?,
        useIntroDb: Boolean,
        useAniSkip: Boolean
    ): List<IntroTimestamps> {
        return repository.getIntro(
            imdbId = imdbId,
            season = season,
            episode = episode,
            title = title,
            useIntroDb = useIntroDb,
            useAniSkip = useAniSkip
        )
    }

    suspend fun resolvePlaybackIntroImdbId(meta: Meta, videoId: String?, language: String): String? {
        StremioId.imdbId(videoId)?.let { return it }
        StremioId.imdbId(meta.id)?.let { return it }
        StremioId.imdbId(FluxaCoreNative.playbackIntroLookupContentId(videoId ?: meta.id))?.let { return it }
        return null
    }

    suspend fun getConfiguredMetaDetail(type: String, id: String, language: String): MetaDetail? {
        val profile = activeProfile()
        return repository.getMetaDetail(
            type = type,
            id = id,
            language = language,
            authKey = profile?.authKey.orEmpty(),
            localAddons = profile?.safeLocalAddons.orEmpty(),
            useConfiguredAddons = true
        )
    }

    private suspend fun buildPlaybackStreamRequestIds(type: String, id: String, language: String): List<String> {
        val detail = if (StremioId.isTmdbLikeContentId(id) || type != "series") {
            withTimeoutOrNull(2500) { getConfiguredMetaDetail(type, StremioId.baseContentId(id), language) }
        } else {
            null
        }
        return FluxaCoreNative.playbackStreamRequestIds(type, id, detail?.id)
    }

}
