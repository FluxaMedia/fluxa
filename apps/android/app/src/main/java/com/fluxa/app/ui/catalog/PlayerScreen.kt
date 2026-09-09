@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import android.content.Context
import android.util.Log
import androidx.activity.compose.BackHandler
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.platform.LocalContext
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.stream.playableUrl
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.player.*
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.ui.playback.FluxaPlaybackSessionController
import com.fluxa.app.shared.feature.player.MediaTrack
import com.fluxa.app.shared.feature.watchtogether.WatchTogetherContent
import com.fluxa.app.shared.feature.watchtogether.LambdaWatchTogetherPlaybackEndpoint
import com.fluxa.app.shared.feature.watchtogether.WatchTogetherDialog
import com.fluxa.app.shared.feature.watchtogether.WatchTogetherManager
import com.fluxa.app.shared.feature.watchtogether.WatchTogetherPlaybackSnapshot
import com.fluxa.app.shared.feature.watchtogether.loadIntoManager
import com.fluxa.app.shared.feature.watchtogether.saveAndConfigure
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import com.google.gson.JsonObject

@Composable
fun PlayerScreen(
    meta: Meta,
    initialProgress: Long = 0L,
    initialProgressPercent: Float? = null,
    videoId: String? = null,
    onBack: () -> Unit,
    viewModel: HomeViewModel,
    exoPlayer: ExoPlayer,
    activeProfile: UserProfile?,
    onUpdateProfile: (UserProfile) -> Unit,
    initialStreamIndex: Int = 0,
    initialStreams: List<Stream> = emptyList(),
    lastStreamUrl: String? = null,
    lastStreamTitle: String? = null,
    initialBingeGroup: String? = null,
    returnToSourcesOnError: Boolean = false,
    onSelectSource: (PlayerSourceSelectionRequest) -> Unit,
    onRecommendationSelected: (Meta) -> Unit = {}
) {
    val context = LocalContext.current
    val activity = context as? android.app.Activity
    val scope = rememberCoroutineScope()
    val deviceType = LocalDeviceType.current
    val lang = activeProfile?.safeLanguage ?: "en"
    val isAnimeMeta = meta.genres?.any { g -> g.lowercase().contains("anime") } == true
    val configuredPlayer = activeProfile?.safePreferredPlayer.orEmpty()
    val backendPlan = remember(configuredPlayer, activeProfile?.safeAnimeUseMpv, isAnimeMeta) {
        val preferredPlayer = when {
            configuredPlayer == "mpv" -> "mpv"
            configuredPlayer == "exoplayer" -> "exoplayer"
            activeProfile?.safeAnimeUseMpv == true && isAnimeMeta -> "mpv"
            else -> "internal"
        }
        FluxaCoreNative.playerBackendSelection(
            JsonObject().apply { addProperty("preferredPlayer", preferredPlayer) }.toString()
        )
    }
    val useMpvBackend = backendPlan.backend == "mpv"
    val mpvCustomOptions = activeProfile?.safeMpvCustomOptions.orEmpty()
    val audioProcessingMode = activeProfile?.safeAudioProcessingMode ?: "reference"
    // ExoPlayer and MPV each own their route callback and reconfiguration.
    // Recreating the MPV composable here would race that lifecycle and reset
    // playback twice on one HDMI/Bluetooth change.
    val mpvPlayer = remember(context, useMpvBackend, mpvCustomOptions, audioProcessingMode) {
        if (useMpvBackend) runCatching { MpvEmbeddedPlayer(context, mpvCustomOptions, audioProcessingMode) }.getOrNull() else null
    }

    PlayerOrientationLock(activity, deviceType)

    val torrentManager = TorrentStreamManager.getInstance(context)
    val torrentStatus by torrentManager.status.collectAsStateWithLifecycle()

    val initialAudioManager = remember(context) {
        context.getSystemService(Context.AUDIO_SERVICE) as android.media.AudioManager
    }
    val state = rememberPlayerScreenState(
        initialVideoId = videoId,
        initialStreamIndex = initialStreamIndex,
        initialVolume = initialAudioManager.getStreamVolume(android.media.AudioManager.STREAM_MUSIC),
    )
    DisposableEffect(exoPlayer, mpvPlayer, useMpvBackend) {
        onDispose {
            // Route changes replace the ExoPlayer instance. Preserve the exact
            // playback point so the new capability-aware sink resumes there.
            state.lastSavedPosition = if (useMpvBackend) {
                state.engine.timeline.position.coerceAtLeast(0L)
            } else {
                exoPlayer.currentPosition.coerceAtLeast(0L)
            }
            state.lastSavedPlayWhenReady = if (useMpvBackend) {
                state.engine.playback.isPlaying
            } else {
                exoPlayer.playWhenReady
            }
        }
    }
    val systemControls = rememberPlayerSystemControls(context, activity, state, initialAudioManager)
    val audioManager = systemControls.audioManager
    val maxVolume = systemControls.maxVolume
    val seekBackwardMs = (activeProfile?.safeSeekBackwardSeconds ?: 10) * 1000L
    val seekForwardMs = (activeProfile?.safeSeekForwardSeconds ?: 10) * 1000L

    LaunchedEffect(activeProfile?.safeTorrentSpeedPreset, activeProfile?.safeTorrentCachePreset) {
        torrentManager.configurePreferences(
            speedPreset = activeProfile?.safeTorrentSpeedPreset,
            cacheLimitMb = FluxaCoreNative.torrentCacheLimitMb(
                preset = activeProfile?.safeTorrentCachePreset,
                platform = "android"
            )
        )
    }

    PlayerTransientFeedbackEffects(
        showVolumeBar = state.showVolumeBar,
        volumeBarVersion = state.volumeBarVersion,
        brightnessBarVersion = state.brightnessBarVersion,
        seekFeedbackVersion = state.seekFeedbackVersion,
        segmentSkipFeedbackVersion = state.segmentSkipFeedbackVersion,
        zoomOverlayVersion = state.zoomOverlayVersion,
        setShowVolumeBar = { state.showVolumeBar = it },
        setShowBrightnessBar = { state.showBrightnessBar = it },
        setShowSeekFeedback = { state.showSeekFeedback = it },
        resetPendingSeek = {
            state.pendingSeekTarget = null
            state.seekFeedbackMs = 0L
        },
        setShowSegmentSkipFeedback = { state.showSegmentSkipFeedback = it },
        setShowZoomOverlay = { state.showZoomOverlay = it }
    )

    val fluxaPlayer = remember(exoPlayer) { MediaPlayerController(context, exoPlayer) }
    val exoEngine = remember(fluxaPlayer, exoPlayer) { ExoPlayerEngine(fluxaPlayer, exoPlayer) }
    val activeEngine: PlayerEngine? = remember(useMpvBackend, mpvPlayer, exoEngine) {
        if (useMpvBackend) mpvPlayer?.let(::MpvPlayerEngine) else exoEngine
    }
    val watchNextPublisher = remember(context) { AndroidTvWatchNextPublisher(context) }
    DisposableEffect(exoPlayer, mpvPlayer, useMpvBackend) {
        FluxaPlaybackSessionController.attach(
            context = context,
            exoPlayer = exoPlayer.takeUnless { useMpvBackend },
            mpvPlayer = mpvPlayer.takeIf { useMpvBackend },
            title = meta.name,
            subtitle = state.currentEpisodeMetaLine,
            artworkUri = state.currentEpisodeArtwork ?: meta.poster,
            language = lang,
        )
        onDispose {
            FluxaPlaybackSessionController.detach(
                context = context,
                exoPlayer = exoPlayer.takeUnless { useMpvBackend },
                mpvPlayer = mpvPlayer.takeIf { useMpvBackend },
            )
        }
    }
    LaunchedEffect(meta.id, state.currentVideoId, state.currentEpisodeMetaLine, state.currentEpisodeArtwork) {
        FluxaPlaybackSessionController.updateMetadata(
            title = meta.name,
            subtitle = state.currentEpisodeMetaLine,
            artworkUri = state.currentEpisodeArtwork ?: meta.poster,
            language = lang,
        )
    }

    LaunchedEffect(activeEngine, meta.id, state.currentVideoId) {
        try {
            while (isActive) {
                val positionMs = if (useMpvBackend) state.engine.timeline.position else exoPlayer.currentPosition
                val durationMs = if (useMpvBackend) state.engine.timeline.duration else exoPlayer.duration
                val episode = state.currentVideoId
                    ?.let(::extractSeasonEpisode)
                    ?: videoId?.let(::extractSeasonEpisode)
                val episodeTitle = episode?.let { (season, number) ->
                    meta.videos.orEmpty().firstOrNull { it.season == season && it.number == number }?.name
                }
                val artworkUrl = episode?.let { (season, number) ->
                    meta.videos.orEmpty().firstOrNull { it.season == season && it.number == number }?.thumbnail
                } ?: meta.poster
                val playing = if (useMpvBackend) state.engine.playback.isPlaying else exoPlayer.isPlaying
                val buffering = if (useMpvBackend) state.engine.playback.isBuffering else exoPlayer.isLoading
                val request = JsonObject().apply {
                    addProperty("title", meta.name)
                    addProperty("episodeTitle", episodeTitle)
                    addProperty("season", episode?.first)
                    addProperty("episode", episode?.second)
                    addProperty("positionMs", positionMs.coerceAtLeast(0L))
                    addProperty("durationMs", durationMs.takeIf { it > 0L } ?: 0L)
                    addProperty("isPlaying", playing)
                    addProperty("isBuffering", buffering)
                    addProperty("artworkUrl", artworkUrl)
                }
                runCatching {
                    FluxaCoreNative.discordPresenceSnapshot(request.toString())?.let { snapshot ->
                        DiscordPresenceNative.update(
                            title = snapshot.title,
                            episodeLine = snapshot.episodeLine,
                            status = snapshot.status,
                            positionMs = snapshot.positionMs,
                            durationMs = snapshot.durationMs,
                            artworkUrl = snapshot.artworkUrl
                        )
                    }
                }
                delay(1000)
            }
        } finally {
            runCatching { DiscordPresenceNative.clear() }
        }
    }

    var showWatchParty by remember { mutableStateOf(false) }
    val watchTogetherState by WatchTogetherManager.state.collectAsStateWithLifecycle()
    val watchTogetherConfig by WatchTogetherManager.config.collectAsStateWithLifecycle()
    val watchConfigStore = remember(context) { AndroidWatchTogetherConfigStore(context) }
    LaunchedEffect(activeProfile?.id) {
        if (WatchTogetherManager.config.value.serverUrl.isBlank()) {
            watchConfigStore.loadIntoManager(activeProfile?.displayName ?: "Guest")
        }
    }
    val watchEndpoint = remember(activeEngine, state, useMpvBackend) {
        LambdaWatchTogetherPlaybackEndpoint(
            snapshotProvider = {
                WatchTogetherPlaybackSnapshot(
                    positionMs = (if (useMpvBackend) state.engine.timeline.position else state.timelinePosition).coerceAtLeast(0L),
                    durationMs = state.engine.timeline.duration.coerceAtLeast(0L),
                    isPlaying = state.engine.playback.isPlaying,
                    isBuffering = state.engine.playback.isBuffering,
                )
            },
            playingSetter = { playing -> activeEngine?.setPaused(!playing) },
            seekHandler = { positionMs ->
                val safePosition = positionMs.coerceAtLeast(0L)
                activeEngine?.seekTo(safePosition, exact = true)
                state.updateEngineSnapshot(state.engine.copy(
                    timeline = state.engine.timeline.copy(position = safePosition)
                ))
                state.timelinePosition = safePosition
            },
            speedSetter = { speed -> activeEngine?.setSpeed(speed) },
        )
    }
    val watchContent = remember(meta.id, meta.type, state.currentVideoId, meta.name) {
        WatchTogetherContent(
            id = meta.id,
            type = meta.type.ifBlank { "movie" },
            videoId = state.currentVideoId,
            title = meta.name.orEmpty(),
        )
    }
    DisposableEffect(watchEndpoint, watchContent) {
        WatchTogetherManager.attachPlayback(watchEndpoint, watchContent)
        onDispose { WatchTogetherManager.detachPlayback(watchEndpoint) }
    }
    val emptyAudioTracks = remember { MutableStateFlow(emptyList<MediaTrack>()) }
    val emptySubtitleTracks = remember { MutableStateFlow(emptyList<MediaTrack>()) }
    val emptyTrack = remember { MutableStateFlow<MediaTrack?>(null) }
    val emptyTechnicalInfo = remember { MutableStateFlow<String?>(null) }
    val availableAudios by (activeEngine?.availableAudios ?: emptyAudioTracks).collectAsStateWithLifecycle()
    val availableSubtitles by (activeEngine?.availableSubtitles ?: emptySubtitleTracks).collectAsStateWithLifecycle()
    val currentAudio by (activeEngine?.currentAudio ?: emptyTrack).collectAsStateWithLifecycle()
    val currentSubtitle by (activeEngine?.currentSubtitle ?: emptyTrack).collectAsStateWithLifecycle()
    val effectiveTechnicalInfo by (activeEngine?.technicalInfo ?: emptyTechnicalInfo).collectAsStateWithLifecycle()

    LaunchedEffect(state.currentVideoId, state.currentStreams) {
        state.failedAutoFallbackUrls = emptySet()
    }

    PlayerEngineSettingsEffects(
        activeEngine = activeEngine,
        audioDelayMs = state.audioDelayMs,
        subtitleDelayMs = state.subtitleDelayMs,
        resizeMode = state.resizeMode
    )

    PlayerExternalAudioEffects(
        activeEngine = activeEngine,
        currentStreams = state.currentStreams,
        currentUrl = state.resolvedUrl ?: state.currentUrl,
        preferredAudioLanguage = TrackSelectionState.resolvePreferredAudioLanguage(activeProfile, meta)
    )

    val updateEngine: (PlayerEngineSnapshot.() -> PlayerEngineSnapshot) -> Unit = { f ->
        state.updateEngineSnapshot(f(state.engine))
    }


    PlayerEpisodeMetadataEffect(
        meta = meta,
        currentVideoId = state.currentVideoId,
        viewModel = viewModel,
        language = activeProfile?.language ?: "en",
        setEpisodeLine = { state.currentEpisodeMetaLine = it },
        setEpisodeArtwork = { state.currentEpisodeArtwork = it }
    )

    val contentWarningsEnabled = activeProfile?.safeContentWarningsEnabled != false
    LaunchedEffect(meta.id, contentWarningsEnabled) {
        if (contentWarningsEnabled) viewModel.loadParentsGuide(meta.id)
    }
    val parentsGuide by viewModel.parentsGuide.collectAsStateWithLifecycle()

    LaunchedEffect(state.currentVideoId) {
        state.showParentsGuide = false
        state.parentsGuideShown = false
    }
    LaunchedEffect(state.engine.playback.hasStartedPlaying, parentsGuide, contentWarningsEnabled) {
        if (contentWarningsEnabled && state.engine.playback.hasStartedPlaying && parentsGuide.isNotEmpty() && !state.parentsGuideShown) {
            state.parentsGuideShown = true
            state.showParentsGuide = true
        }
    }

    val mainFocusRequester = remember { FocusRequester() }
    val playPauseFocusRequester = remember { FocusRequester() }
    val seekbarFocusRequester = remember { FocusRequester() }
    val playbackActions = PlayerPlaybackActions(
        state = state,
        scope = scope,
        useMpvBackend = useMpvBackend,
        exoPlayer = exoPlayer,
        activeEngine = activeEngine,
        meta = meta,
        videoId = videoId,
        viewModel = viewModel,
        activeProfile = activeProfile,
        availableSubtitles = availableSubtitles,
        currentSubtitle = currentSubtitle,
        seekForwardMs = seekForwardMs,
        seekBackwardMs = seekBackwardMs,
        requestPlayPauseFocus = { playPauseFocusRequester.requestFocus() },
        onSelectSource = onSelectSource,
        onBack = onBack,
    )

    val externalActions = rememberPlayerExternalPlaybackActions(
        context = context,
        state = state,
        useMpvBackend = useMpvBackend,
        exoPlayer = exoPlayer,
        meta = meta,
        videoId = videoId,
        activeProfile = activeProfile,
        viewModel = viewModel,
    )

    LaunchedEffect(Unit) { delay(1000); try { mainFocusRequester.requestFocus() } catch(e: Exception) {} }

    LaunchedEffect(Unit) {
        if (!useMpvBackend) exoPlayer.volume = 1.0f
    }

    LaunchedEffect(initialBingeGroup) {
        if (!initialBingeGroup.isNullOrBlank() && activeProfile?.safeTryBingeGroup == true) {
            state.preferredBingeGroupForNextEpisode = initialBingeGroup
        }
    }

    DisposableEffect(mpvPlayer) {
        onDispose { mpvPlayer?.release() }
    }

    PlayerMpvStateSyncEffect(
        useMpvBackend = useMpvBackend,
        mpvPlayer = mpvPlayer,
        updateEngine = updateEngine,
    )

    val playerUserAddons by viewModel.userAddons.collectAsStateWithLifecycle()
    val latestTorrentStatus by rememberUpdatedState(torrentStatus)
    val torrentParseRetryCount = remember(state.currentUrl) { androidx.compose.runtime.mutableIntStateOf(0) }
    PlayerPlaybackSideEffects(
        viewModel = viewModel,
        activeProfile = activeProfile,
        meta = meta,
        currentVideoId = state.currentVideoId,
        currentStreamIndex = state.currentStreamIndex,
        currentEpisodeMetaLine = state.currentEpisodeMetaLine,
        currentEpisodeArtwork = state.currentEpisodeArtwork,
        nextEpisode = state.nextEpisodePending,
        currentStreams = state.currentStreams,
        isPlaying = state.engine.playback.isPlaying,
        playWhenReadyForScrobble = state.engine.playback.playWhenReadyForScrobble,
        currentPositionMs = { if (useMpvBackend) state.engine.timeline.position else exoPlayer.currentPosition },
        duration = state.engine.timeline.duration,
        lastSavedTimestamp = state.lastSavedTimestamp,
        onLastSavedTimestampChanged = { state.lastSavedTimestamp = it },
        isVideoRendered = state.engine.render.isVideoRendered
    )

    PlayerStreamLoadingEffect(
        meta = meta,
        videoId = videoId,
        currentVideoId = state.currentVideoId,
        initialStreams = initialStreams,
        initialStreamIndex = initialStreamIndex,
        lastStreamUrl = lastStreamUrl,
        lastStreamTitle = lastStreamTitle,
        activeProfile = activeProfile,
        preferredBingeGroup = state.preferredBingeGroupForNextEpisode,
        viewModel = viewModel,
        lang = lang,
        setCurrentUrl = {
            if (state.currentUrl != it) state.telemetryAttemptGeneration++
            state.currentUrl = it
        },
        setCurrentStreams = { state.currentStreams = it },
        setCurrentStreamIndex = { state.currentStreamIndex = it },
        setZeroSpeedTicks = { state.zeroSpeedTicks = it },
        updateEngine = updateEngine,
        clearPreferredBingeGroup = { state.preferredBingeGroupForNextEpisode = null }
    )

    PlayerSkipSegmentsEffect(
        meta = meta,
        currentVideoId = state.currentVideoId,
        activeProfile = activeProfile,
        viewModel = viewModel,
        setSkipSegments = { state.skipSegments = it },
        setDismissedSkipSegments = { state.dismissedSkipSegments = it },
        setIntroAutoSkipped = { state.introAutoSkipped = it }
    )

    LaunchedEffect(state.currentVideoId) {
        state.markSegmentType = null
        state.markSegmentStartMs = null
        state.markSegmentEndMs = null
        state.markSegmentFeedback = null
    }

    LaunchedEffect(activeEngine, state.currentVideoId) {
        activeEngine?.chapters?.collect { chapters ->
            state.chapters = chapters
            if (chapters.isNotEmpty() && state.skipSegments.isEmpty() && activeProfile?.safeUseChapterSkip != false) {
                val derived = deriveSkipSegmentsFromChapters(chapters)
                if (derived.isNotEmpty()) {
                    state.skipSegments = derived
                }
            }
        }
    }

    ExoPlayerListenerEffect(
        exoPlayer = exoPlayer,
        exoEngine = exoEngine,
        useMpvBackend = useMpvBackend,
        currentUrl = state.currentUrl,
        telemetryAttemptGeneration = state.telemetryAttemptGeneration,
        currentStreamIndex = state.currentStreamIndex,
        currentStreamsSize = state.currentStreams.size,
        autoFallbackOnStreamError = playbackActions.isCloudstreamPlayback() || activeProfile?.safeAutoRetryNextSource == true,
        returnToSourcesOnError = returnToSourcesOnError,
        lang = lang,
        mergeSkipSegments = { newSegments ->
            state.skipSegments = (state.skipSegments + newSegments).distinctBy { "${it.startTime}-${it.type}" }
        },
        updateEngine = updateEngine,
        openSourceSelectionScreen = playbackActions::openSourceSelection,
        fallbackToNextStream = playbackActions::fallbackToNextCloudstreamStream,
        torrentManager = torrentManager,
        retryPlayback = {
            val savedUrl = state.resolvedUrl
            if (savedUrl != null && torrentParseRetryCount.intValue < 2) {
                torrentParseRetryCount.intValue++
                state.telemetryAttemptGeneration++
                scope.launch {
                    exoEngine.clear()
                    state.resolvedUrl = null
                    delay(1500)
                    state.resolvedUrl = savedUrl
                }
            } else {
                torrentParseRetryCount.intValue = 0
                state.updateEngineSnapshot(state.engine.copy(
                    playerError = AppStrings.format(lang, "player.error_load", AppStrings.t(lang, "player.error_server")),
                    playback = state.engine.playback.copy(isBuffering = false),
                ))
                if (returnToSourcesOnError && state.currentStreams.size > 1) playbackActions.openSourceSelection()
            }
        }
    )

    PlayerStallWatchdogEffect(
        currentUrl = state.currentUrl,
        currentStreamIndex = state.currentStreamIndex,
        currentStreamsSize = state.currentStreams.size,
        autoFallbackOnStreamError = playbackActions.isCloudstreamPlayback() || activeProfile?.safeAutoRetryNextSource == true,
        useMpvBackend = useMpvBackend,
        isBuffering = state.engine.playback.isBuffering,
        hasStartedPlaying = state.engine.playback.hasStartedPlaying,
        isVideoRendered = state.engine.render.isVideoRendered,
        torrentStatus = latestTorrentStatus,
        returnToSourcesOnError = returnToSourcesOnError,
        lang = lang,
        updateEngine = updateEngine,
        openSourceSelectionScreen = playbackActions::openSourceSelection,
        fallbackToNextStream = playbackActions::fallbackToNextCloudstreamStream
    )

    PlayerResolveUrlEffect(
        currentUrl = state.currentUrl,
        currentStreamIndex = state.currentStreamIndex,
        currentStreams = state.currentStreams,
        activeEngine = activeEngine,
        torrentManager = torrentManager,
        currentVideoId = state.currentVideoId,
        meta = meta,
        viewModel = viewModel,
        returnToSourcesOnError = returnToSourcesOnError,
        lang = lang,
        setSkipSegments = { state.skipSegments = it },
        skipSegments = state.skipSegments,
        updateEngine = updateEngine,
        setResolvedUrl = { state.resolvedUrl = it },
        openSourceSelectionScreen = playbackActions::openSourceSelection,
        fallbackToNextStream = playbackActions::fallbackToNextCloudstreamStream
    )

    PlayerPreparePlaybackEffect(
        resolvedUrl = state.resolvedUrl,
        activeProfile = activeProfile,
        activeEngine = activeEngine,
        exoEngine = exoEngine,
        useMpvBackend = useMpvBackend,
        currentStreams = state.currentStreams,
        currentStreamIndex = state.currentStreamIndex,
        viewModel = viewModel,
        playerUserAddons = playerUserAddons,
        meta = meta,
        currentVideoId = state.currentVideoId,
        lastSavedPosition = state.lastSavedPosition,
        lastSavedPlayWhenReady = state.lastSavedPlayWhenReady,
        shouldApplyInitialProgress = state.shouldApplyInitialProgress,
        initialProgress = initialProgress,
        resizeMode = state.resizeMode,
        audioDelayMs = state.audioDelayMs,
        subtitleDelayMs = state.subtitleDelayMs,
        updateEngine = updateEngine,
        clearLastSavedPosition = {
            state.lastSavedPosition = 0L
            state.lastSavedPlayWhenReady = null
        },
        clearInitialProgress = { state.shouldApplyInitialProgress = false },
        onExternalSubtitlesFetched = { state.currentExternalSubtitleTracks = it },
        onNativeAssTracksExtracted = { state.embeddedNativeAssTracks = it }
    )

    var pendingResumePercent by remember { mutableStateOf(initialProgressPercent) }
    LaunchedEffect(state.currentVideoId) {
        if (state.currentVideoId != videoId) pendingResumePercent = null
    }
    LaunchedEffect(state.engine.timeline.duration, state.engine.playback.hasStartedPlaying) {
        val percent = pendingResumePercent ?: return@LaunchedEffect
        val duration = state.engine.timeline.duration
        if (state.engine.playback.hasStartedPlaying && duration > 0L) {
            playbackActions.seekSafely((duration * (percent / 100f)).toLong())
            pendingResumePercent = null
        }
    }



    LaunchedEffect(state.engine.playback.isPlaying, state.skipSegments, activeProfile?.safeAutoSkipIntro) {
        while (state.engine.playback.isPlaying) {
            if (!useMpvBackend) state.timelinePosition = exoPlayer.currentPosition
            if (state.engine.playback.hasStartedPlaying && activeProfile?.safeAutoSkipIntro == true) {
                val segment = state.skipSegments.firstOrNull {
                    state.timelinePosition in it.startTime until it.endTime &&
                        it.dismissKey() !in state.dismissedSkipSegments
                }
                if (segment != null) {
                    state.dismissedSkipSegments = state.dismissedSkipSegments + segment.dismissKey()
                    if (segment.type == "outro" && state.nextEpisodePending != null) {
                        playbackActions.playNext()
                    } else {
                        playbackActions.seekSafely(segment.endTime)
                    }
                    state.introAutoSkipped = true
                    state.segmentSkipFeedbackVersion += 1
                }
            }
            // Keep controls/scrubbing responsive without waking the whole player UI twice a
            // second during ordinary hands-off playback. Auto-skip still has 1s precision
            // with controls hidden, which is well inside typical intro/outro segment margins.
            val timelinePollMs = if (state.showControls || state.showSettings || state.isScrubbing) 500L else 1_000L
            delay(timelinePollMs)
        }
    }

    PlayerTrackMemoryEffects(
        meta = meta,
        currentVideoId = state.currentVideoId,
        activeProfile = activeProfile,
        activeEngine = activeEngine,
        availableSubtitles = availableSubtitles,
        currentAudio = currentAudio,
        currentSubtitle = currentSubtitle,
        hasStartedPlaying = state.engine.playback.hasStartedPlaying,
        useMpvBackend = useMpvBackend,
        viewModel = viewModel,
        currentEpisodeArtwork = state.currentEpisodeArtwork,
        currentPositionMs = { if (useMpvBackend) state.engine.timeline.position else exoPlayer.currentPosition },
        duration = state.engine.timeline.duration,
        currentStreamIndex = state.currentStreamIndex,
        currentEpisodeMetaLine = state.currentEpisodeMetaLine,
        currentStreams = state.currentStreams
    )


    BackHandler {
        if (state.showSettings) state.showSettings = false
        else if (showWatchParty) showWatchParty = false
        else {
            WatchTogetherManager.leaveRoom()
            onBack()
        }
    }

    PlayerBufferProgressEffect(
        exoPlayer = exoPlayer,
        useMpvBackend = useMpvBackend,
        isVideoRendered = state.engine.render.isVideoRendered,
        currentUrl = state.currentUrl,
        torrentStatus = torrentStatus,
        updateEngine = updateEngine,
    )

    PlayerEpisodeNavigationEffect(
        meta = meta,
        currentVideoId = state.currentVideoId,
        viewModel = viewModel,
        language = activeProfile?.language ?: "en",
        setPreviousEpisode = { state.previousEpisodePending = it },
        setNextEpisode = { state.nextEpisodePending = it }
    )

    LaunchedEffect(meta.id, state.currentVideoId, useMpvBackend) {
        if (!com.fluxa.app.BuildConfig.IS_TV) return@LaunchedEffect
        while (isActive) {
            if (state.engine.playback.hasStartedPlaying) {
                watchNextPublisher.publish(
                    meta = meta,
                    videoId = state.currentVideoId ?: videoId,
                    positionMs = state.engine.timeline.position,
                    durationMs = state.engine.timeline.duration,
                    streamUrl = state.currentStreams.getOrNull(state.currentStreamIndex)?.playableUrl,
                    streamTitle = state.currentStreams.getOrNull(state.currentStreamIndex)?.title,
                )
            }
            delay(15_000)
        }
    }

    val latestIsScrubbing by rememberUpdatedState(state.isScrubbing)
    val latestActiveEngine by rememberUpdatedState(activeEngine)
    LaunchedEffect(state.scrubPosition) {
        if (!latestIsScrubbing) return@LaunchedEffect
        delay(80)
        if (latestIsScrubbing) latestActiveEngine?.seekTo(state.scrubPosition, exact = false)
    }

    PlayerPipEffect(
        context = context,
        lang = lang,
        isPlaying = state.engine.playback.isPlaying,
        activeEngine = activeEngine,
        seekBackward = { playbackActions.performRelativeSeek(-1) },
        seekForward = { playbackActions.performRelativeSeek(1) }
    )

    suspend fun requestTerminalRecommendations() {
        if (state.terminalRecommendationsRequested || state.terminalRecommendationsLoading) return
        state.terminalRecommendationsRequested = true
        state.terminalRecommendationsLoading = true
        Log.d("TerminalRec", "request start id=${meta.id} type=${meta.type}")
        try {
            state.terminalRecommendations = viewModel.loadTerminalRecommendations(
                meta = meta,
                hasNextEpisode = false,
                language = lang,
                profile = activeProfile,
            )
            Log.d("TerminalRec", "request result count=${state.terminalRecommendations.size}")
        } finally {
            state.terminalRecommendationsLoading = false
        }
    }

    LaunchedEffect(
        state.currentVideoId,
        useMpvBackend,
        activeProfile?.safeMovieRecommendationOutroPercent,
        activeProfile?.safeSeriesRecommendationOutroPercent,
        state.skipSegments,
    ) {
        while (isActive) {
            val isPlaying = if (useMpvBackend) state.engine.playback.isPlaying else exoPlayer.isPlaying
            val hasStartedPlaying = state.engine.playback.hasStartedPlaying || isPlaying
            val positionMs = if (useMpvBackend) state.engine.timeline.position else exoPlayer.currentPosition
            val durationMs = if (useMpvBackend) {
                state.engine.timeline.duration
            } else {
                exoPlayer.duration.takeIf { it > 0L } ?: state.engine.timeline.duration
            }
            val thresholdPercent = if (meta.type == "series") {
                activeProfile?.safeSeriesRecommendationOutroPercent ?: 85f
            } else {
                activeProfile?.safeMovieRecommendationOutroPercent ?: 85f
            }
            val outroStartSeconds = state.skipSegments
                .firstOrNull { it.type == "outro" && it.endTime >= durationMs - 1_000L }
                ?.startTime
                ?.div(1_000.0)
            val shouldShowRecommendations = if (isPlaying && durationMs > 0L && hasStartedPlaying) {
                runCatching {
                    FluxaCoreUniFfi.coreInvokeValue(
                        "recommendationOutroPlan",
                        JsonObject().apply {
                            addProperty("positionSeconds", positionMs / 1_000.0)
                            addProperty("durationSeconds", durationMs / 1_000.0)
                            addProperty("thresholdPercent", thresholdPercent)
                            addProperty("alreadyShown", state.terminalRecommendationsRequested)
                            outroStartSeconds?.let { addProperty("outroStartSeconds", it) }
                        }.toString(),
                    ).asJsonObject.get("shouldShow")?.asBoolean == true
                }.getOrDefault(false)
            } else {
                false
            }
            if (
                state.nextEpisodePending == null &&
                    shouldShowRecommendations
            ) {
                Log.d("TerminalRec", "threshold reached id=${meta.id} positionMs=$positionMs durationMs=$durationMs threshold=$thresholdPercent")
                requestTerminalRecommendations()
            }
            delay(1_000)
        }
    }


    LaunchedEffect(state.engine.playback.playbackEnded, state.nextEpisodePending) {
        if (!state.engine.playback.playbackEnded || !state.engine.playback.hasStartedPlaying) return@LaunchedEffect
        if (com.fluxa.app.BuildConfig.IS_TV) {
            watchNextPublisher.remove(meta, state.currentVideoId ?: videoId)
        }
        delay(150)
        val hasNextEpisode = state.nextEpisodePending != null
        if (hasNextEpisode) {
            playbackActions.autoPlayNextWhenEnded()
        } else {
            requestTerminalRecommendations()
            while (state.terminalRecommendationsLoading) delay(50)
            if (state.terminalRecommendations.isEmpty()) playbackActions.closePlayer()
        }
    }

    PlayerScreenContent(
        meta = meta,
        state = state,
        context = context,
        lang = lang,
        deviceType = deviceType,
        exoPlayer = exoPlayer,
        mpvPlayer = mpvPlayer,
        useMpvBackend = useMpvBackend,
        activeProfile = activeProfile,
        onUpdateProfile = onUpdateProfile,
        activeEngine = activeEngine,
        torrentStatus = torrentStatus,
        mainFocusRequester = mainFocusRequester,
        playPauseFocusRequester = playPauseFocusRequester,
        seekbarFocusRequester = seekbarFocusRequester,
        audioManager = audioManager,
        maxVolume = maxVolume,
        seekForwardMs = seekForwardMs,
        seekBackwardMs = seekBackwardMs,
        viewModel = viewModel,
        availableAudios = availableAudios,
        currentAudio = currentAudio,
        availableSubtitles = availableSubtitles,
        currentSubtitle = currentSubtitle,
        effectiveTechnicalInfo = effectiveTechnicalInfo,
        parentsGuide = parentsGuide,
        showControlsTemp = playbackActions::showControlsTemporarily,
        seekSafely = playbackActions::seekSafely,
        toggleSubtitleSelection = playbackActions::toggleSubtitleSelection,
        performRelativeSeek = playbackActions::performRelativeSeek,
        onVolumeSwipe = systemControls::adjustVolume,
        onBrightnessSwipe = systemControls::adjustBrightness,
        playPrevious = playbackActions::playPrevious,
        playNext = playbackActions::playNext,
        smartCast = externalActions::cast,
        openInExternalPlayer = externalActions::openExternalPlayer,
        openWatchParty = { showWatchParty = true },
        openSourceSelectionScreen = playbackActions::openSourceSelection,
        closePlayer = {
            WatchTogetherManager.leaveRoom()
            playbackActions.closePlayer()
        },
        switchToStream = playbackActions::switchToStream
    )

    if (state.terminalRecommendations.isNotEmpty()) {
        PlayerTerminalRecommendations(
            items = state.terminalRecommendations,
            language = lang,
            onSelected = { recommendation ->
                onRecommendationSelected(recommendation)
            },
            onDismiss = { state.terminalRecommendations = emptyList() },
        )
    }

    if (showWatchParty) {
        WatchTogetherDialog(
            state = watchTogetherState,
            config = watchTogetherConfig,
            localContent = watchContent,
            defaultDisplayName = activeProfile?.displayName.orEmpty(),
            onConfigure = { serverUrl, secret, displayName ->
                watchConfigStore.saveAndConfigure(serverUrl, secret, displayName)
            },
            onCreateRoom = WatchTogetherManager::createRoom,
            onJoinRoom = WatchTogetherManager::joinRoom,
            onLeaveRoom = { WatchTogetherManager.leaveRoom() },
            onDismiss = { showWatchParty = false },
        )
    }
}
