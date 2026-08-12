package com.fluxa.app.ui.catalog

import com.fluxa.app.core.StremioId
import com.fluxa.app.core.rust.NativeHeadlessEngineResult
import com.fluxa.app.data.local.ThirdPartyProviderId
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.local.safeContinueWatchingSource
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.local.safeStreamSourceRegexPattern
import com.fluxa.app.data.local.safeStreamSourceSelectionMode
import com.fluxa.app.data.remote.DetailTrailer
import com.fluxa.app.data.repository.IntroDbSubmitResult
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.SubtitleData
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.player.STREAM_SOURCE_MODE_MANUAL
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.Dispatchers

/**
 * Owns the headless-core playback/watchlist protocol used by [HomeViewModel].
 * The ViewModel remains the public facade while wire payload construction and
 * state decoding live in one focused collaborator.
 */
internal class HomeHeadlessPlaybackCoordinator(
    private val scope: CoroutineScope,
    private val gson: Gson,
    private val dispatch: suspend (Any) -> NativeHeadlessEngineResult,
    private val repository: StremioRepository,
    private val watchlistManager: WatchlistManager,
    private val platformContentGateway: HomePlatformContentGateway,
    private val activeProfile: () -> UserProfile?,
    private val setDirectLoading: (Boolean) -> Unit,
    private val setWatchlist: (List<Meta>) -> Unit,
    private val loadLibraryData: (UserProfile?) -> Unit,
    private val refreshDynamicRows: () -> Unit,
    private val billboardMovie: () -> Meta?,
    private val setBillboardWatchlist: (Boolean) -> Unit,
) {
    private val metaListType = object : TypeToken<List<Meta>>() {}.type
    private val streamListType = object : TypeToken<List<Stream>>() {}.type
    private val trailerListType = object : TypeToken<List<DetailTrailer>>() {}.type
    private val videoListType = object : TypeToken<List<Video>>() {}.type
    private val subtitleListType = object : TypeToken<List<SubtitleData>>() {}.type
    private val introTimestampsListType = object : TypeToken<List<IntroTimestamps>>() {}.type

    fun toggleWatchlist(meta: Meta) {
        scope.launch {
            val result = dispatch(
                mapOf(
                    "type" to "toggleWatchlistRequested",
                    "item" to meta,
                    "profile" to activeProfile(),
                ),
            )
            val write = (result.state["library"] as? Map<*, *>)?.get("lastWrite") as? Map<*, *>
            setWatchlist(decodeList(write?.get("watchlist"), metaListType))
            refreshDynamicRows()
        }
    }

    fun addToWatchlist(meta: Meta) {
        scope.launch {
            if (!watchlistManager.isInWatchlist(meta.id)) toggleWatchlist(meta)
        }
    }

    fun toggleBillboardWatchlist() {
        val movie = billboardMovie() ?: return
        scope.launch {
            val result = dispatch(
                mapOf(
                    "type" to "toggleWatchlistRequested",
                    "item" to movie,
                    "profile" to activeProfile(),
                ),
            )
            val write = (result.state["library"] as? Map<*, *>)?.get("lastWrite") as? Map<*, *>
            setWatchlist(decodeList(write?.get("watchlist"), metaListType))
            (write?.get("isInWatchlist") as? Boolean)?.let(setBillboardWatchlist)
            refreshDynamicRows()
        }
    }

    fun setFeedback(movie: Meta, isLike: Boolean) {
        scope.launch {
            dispatch(
                mapOf(
                    "type" to "setFeedbackRequested",
                    "id" to movie.id,
                    "value" to isLike,
                    "meta" to movie,
                ),
            )
        }
    }

    fun savePlaybackProgress(
        meta: Meta,
        timeOffset: Long,
        duration: Long,
        videoId: String? = null,
        streamIndex: Int? = null,
        episodeName: String? = null,
        lastStreamUrl: String? = null,
        lastStreamTitle: String? = null,
        lastBingeGroup: String? = null,
        lastAudioLanguage: String? = null,
        lastSubtitleLanguage: String? = null,
        scrobbleTraktPause: Boolean = true,
    ) {
        scope.launch {
            dispatch(
                mapOf(
                    "type" to "savePlaybackProgressRequested",
                    "profile" to activeProfile(),
                    "meta" to meta,
                    "timeOffset" to timeOffset,
                    "duration" to duration,
                    "lastVideoId" to videoId,
                    "lastStreamIndex" to streamIndex,
                    "lastEpisodeName" to episodeName,
                    "lastStreamUrl" to lastStreamUrl,
                    "lastStreamTitle" to lastStreamTitle,
                    "lastBingeGroup" to lastBingeGroup,
                    "lastAudioLanguage" to lastAudioLanguage,
                    "lastSubtitleLanguage" to lastSubtitleLanguage,
                    "scrobbleTraktPause" to scrobbleTraktPause,
                ),
            )
            loadLibraryData(activeProfile())
        }
    }

    fun scrobblePlayback(
        token: String,
        metaType: String,
        itemId: String,
        progress: Float,
        action: String,
    ) {
        scope.launch {
            dispatch(
                mapOf(
                    "type" to "scrobbleRequested",
                    "token" to token,
                    "metaType" to metaType,
                    "itemId" to itemId,
                    "progress" to progress,
                    "actionName" to action,
                    "profile" to activeProfile(),
                ),
            )
        }
    }

    fun onNextEpisodeCardShown(meta: Meta, nextVideoId: String, profile: UserProfile?) {
        val effectiveProfile = profile ?: activeProfile()
        scope.launch {
            dispatch(
                mapOf(
                    "type" to "playerNextEpisodeCardShown",
                    "contentType" to meta.type,
                    "seriesId" to meta.id,
                    "nextVideoId" to nextVideoId,
                    "title" to meta.name,
                    "originalName" to meta.originalName,
                    "year" to meta.releaseInfo?.toIntOrNull(),
                    "language" to (effectiveProfile?.safeLanguage ?: "en"),
                    "profile" to effectiveProfile,
                ),
            )
        }
    }

    fun markWatchedFromPlayback(
        meta: Meta,
        videoId: String? = null,
        episodeName: String? = null,
        nextEpisode: Video? = null,
        watchedDuration: Long = 0L,
    ) {
        scope.launch {
            val profile = activeProfile()
            profile?.id?.let(watchlistManager::setActiveProfile)
            watchlistManager.recordWatchedContentDuration(meta.id, videoId, watchedDuration)
            val episodes = if (meta.type == "series" && !videoId.isNullOrBlank()) {
                val parsed = StremioId.parseEpisodeLocator(videoId)
                listOf(
                    Video(
                        id = videoId,
                        name = episodeName,
                        season = parsed?.first,
                        number = parsed?.second,
                        released = null,
                        thumbnail = meta.background,
                    ),
                )
            } else {
                emptyList()
            }
            dispatch(
                mapOf(
                    "type" to "markWatchedRequested",
                    "seriesId" to meta.id,
                    "videoIds" to listOfNotNull(videoId),
                    "watched" to true,
                    "meta" to meta,
                    "episodes" to episodes,
                    "profile" to profile,
                ),
            )
            if (meta.type == "series" && nextEpisode != null) {
                savePlaybackProgress(
                    meta = meta.copy(
                        lastVideoId = nextEpisode.id,
                        continueWatchingPoster = nextEpisode.thumbnail ?: meta.continueWatchingPoster,
                        continueWatchingBackground = nextEpisode.thumbnail ?: meta.continueWatchingBackground,
                    ),
                    timeOffset = 0L,
                    duration = 0L,
                    videoId = nextEpisode.id,
                    episodeName = nextEpisode.continueWatchingTitleForHome(),
                )
            }
            loadLibraryData(profile)
        }
    }

    fun forgetPlaybackProgress(meta: Meta) {
        scope.launch {
            dispatch(
                mapOf(
                    "type" to "clearPlaybackProgressRequested",
                    "profile" to activeProfile(),
                    "meta" to meta,
                ),
            )
            loadLibraryData(activeProfile())
            refreshDynamicRows()
        }
    }

    suspend fun getStreams(type: String, id: String): List<Stream> {
        val result = dispatch(
            mapOf(
                "type" to "playerLoadStreamsRequested",
                "contentType" to type,
                "id" to id,
                "currentVideoId" to id,
                "initialVideoId" to id,
                "initialStreams" to emptyList<Stream>(),
                "initialStreamIndex" to 0,
            ),
        )
        val player = result.state["player"] as? Map<*, *>
        return decodeList(player?.get("currentStreams"), streamListType)
    }

    suspend fun loadPlayerStreams(
        meta: Meta,
        currentVideoId: String?,
        initialVideoId: String?,
        initialStreams: List<Stream>,
        initialStreamIndex: Int,
        savedUrl: String?,
        savedTitle: String?,
        profileOverride: UserProfile?,
        preferredBingeGroup: String?,
    ): PlayerRuntimeCoreState {
        val profile = profileOverride ?: activeProfile()
        val result = dispatch(
            mapOf(
                "type" to "playerLoadStreamsRequested",
                "contentType" to meta.type,
                "id" to (currentVideoId ?: meta.id),
                "currentVideoId" to currentVideoId,
                "initialVideoId" to initialVideoId,
                "initialStreams" to initialStreams,
                "initialStreamIndex" to initialStreamIndex,
                "savedUrl" to savedUrl,
                "savedTitle" to savedTitle,
                "sourceSelectionMode" to (profile?.safeStreamSourceSelectionMode ?: STREAM_SOURCE_MODE_MANUAL),
                "regexPattern" to profile?.safeStreamSourceRegexPattern,
                "preferredBingeGroup" to preferredBingeGroup,
                "title" to meta.name,
                "originalName" to meta.originalName,
                "year" to meta.releaseInfo?.toIntOrNull(),
                "language" to (profile?.safeLanguage ?: "en"),
                "profile" to profile,
            ),
        )
        return decodeObject(result.state["player"], PlayerRuntimeCoreState::class.java)
            ?: PlayerRuntimeCoreState(playerError = "generic")
    }

    suspend fun resolvePlayerPlayback(
        url: String,
        stream: Stream?,
        currentVideoId: String?,
        title: String,
    ): PlayerRuntimeCoreState {
        val result = dispatch(
            mapOf(
                "type" to "playerResolvePlaybackRequested",
                "url" to url,
                "stream" to stream,
                "currentVideoId" to currentVideoId,
                "title" to title,
            ),
        )
        return decodeObject(result.state["player"], PlayerRuntimeCoreState::class.java)
            ?: PlayerRuntimeCoreState(playerError = "generic")
    }

    suspend fun prepareDirectPlayback(meta: Meta): DirectPlaybackTarget? {
        setDirectLoading(true)
        return try {
            val profile = activeProfile()
            val result = dispatch(
                mapOf(
                    "type" to "directPlaybackRequested",
                    "meta" to meta,
                    "profile" to profile,
                    "language" to (profile?.safeLanguage ?: "en"),
                ),
            )
            val player = result.state["player"] as? Map<*, *>
            decodeObject(player?.get("directPlaybackTarget"), DirectPlaybackTarget::class.java)
        } finally {
            setDirectLoading(false)
        }
    }

    suspend fun getSeasonEpisodes(id: String, seasonNumber: Int, language: String): List<Video> {
        val result = dispatch(
            mapOf(
                "type" to "detailSeasonRequested",
                "seriesId" to id,
                "season" to seasonNumber,
                "profile" to activeProfile(),
                "language" to language,
            ),
        )
        val detail = result.state["detail"] as? Map<*, *>
        return decodeList(detail?.get("seasonEpisodes"), videoListType)
    }

    suspend fun getSubtitlesFromAddon(
        baseUrl: String,
        type: String,
        id: String,
        extra: String = "",
    ): List<SubtitleData> {
        val result = dispatch(
            mapOf(
                "type" to "addonResourceRequested",
                "transportUrl" to baseUrl,
                "resource" to "subtitles",
                "contentType" to type,
                "id" to id,
                "extra" to mapOf("extraArgs" to extra),
            ),
        )
        val addons = result.state["addons"] as? Map<*, *>
        return decodeList(addons?.get("lastResourceResult"), subtitleListType)
    }

    suspend fun getIntroSegments(
        imdbId: String,
        season: Int,
        episode: Int,
        title: String?,
        useIntroDb: Boolean,
        useAniSkip: Boolean,
    ): List<IntroTimestamps> {
        val result = dispatch(
            mapOf(
                "type" to "introSegmentsRequested",
                "imdbId" to imdbId,
                "season" to season,
                "episode" to episode,
                "title" to title,
                "useIntroDb" to useIntroDb,
                "useAniSkip" to useAniSkip,
            ),
        )
        val player = result.state["player"] as? Map<*, *>
        return decodeList(player?.get("introSegments"), introTimestampsListType)
    }

    suspend fun submitIntroSegment(
        apiKey: String,
        segmentType: String,
        imdbId: String,
        season: Int,
        episode: Int,
        startSec: Double,
        endSec: Double,
    ): IntroDbSubmitResult = repository.submitIntroSegment(
        apiKey,
        segmentType,
        imdbId,
        season,
        episode,
        startSec,
        endSec,
    )

    suspend fun resolvePlaybackIntroImdbId(
        meta: Meta,
        videoId: String?,
        language: String,
    ): String? {
        val result = dispatch(
            mapOf(
                "type" to "introImdbIdRequested",
                "meta" to meta,
                "videoId" to videoId,
                "language" to language,
            ),
        )
        return (result.state["player"] as? Map<*, *>)?.get("introImdbId") as? String
    }

    suspend fun getConfiguredMetaDetail(type: String, id: String, language: String): MetaDetail? =
        getConfiguredMetaDetailResult(type, id, language).detail

    suspend fun getConfiguredMetaDetailResult(
        type: String,
        id: String,
        language: String,
    ): HomeMetaDetailResult {
        val result = dispatch(
            mapOf(
                "type" to "metaDetailRequested",
                "contentType" to type,
                "id" to id,
                "language" to language,
                "profile" to activeProfile(),
            ),
        )
        val lookup = result.state["lookup"] as? Map<*, *>
        val detail = decodeObject(lookup?.get("metaDetail"), MetaDetail::class.java)
        val addonTrailers = decodeList<DetailTrailer>(lookup?.get("trailers"), trailerListType)
        if (addonTrailers.isNotEmpty()) {
            return HomeMetaDetailResult(detail = detail, trailers = addonTrailers)
        }

        val profile = activeProfile()
        val tmdbTrailers = if (
            profile?.safeTmdbApiKey?.isNotBlank() == true &&
            profile.safeTmdbTrailersEnabled
        ) {
            runCatching {
                platformContentGateway.trailers(type, id, language, profile.safeTmdbApiKey)
            }.getOrElse { emptyList() }
        } else {
            emptyList()
        }
        return HomeMetaDetailResult(detail = detail, trailers = tmdbTrailers)
    }

    suspend fun resolveExpandedPosterTrailer(meta: Meta): String? {
        val language = activeProfile()?.safeLanguage ?: "en"
        val trailers = runCatching {
            getConfiguredMetaDetailResult(meta.type, meta.id, language).trailers
        }.getOrElse { emptyList() }
        return resolvePlayableTrailerUrl(trailers, dispatch)
    }

    private suspend fun <T> decodeList(value: Any?, type: java.lang.reflect.Type): List<T> = withContext(Dispatchers.Default) {
        runCatching { gson.fromJson<List<T>>(gson.toJson(value), type).orEmpty() }
            .getOrElse { emptyList() }
    }

    private suspend fun <T> decodeObject(value: Any?, clazz: Class<T>): T? = withContext(Dispatchers.Default) {
        runCatching { gson.fromJson(gson.toJson(value), clazz) }.getOrNull()
    }
}
