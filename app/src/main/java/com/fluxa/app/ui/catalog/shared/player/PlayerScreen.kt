@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import android.content.Context
import android.net.Uri
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.platform.LocalContext
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.player.*
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.player.MediaTrack
import com.fluxa.app.player.VideoFormatBadge
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow

@Composable
fun PlayerScreen(
    meta: Meta,
    initialProgress: Long = 0L,
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
    onSelectSource: (PlayerSourceSelectionRequest) -> Unit
) {
    val context = LocalContext.current
    val activity = context as? android.app.Activity
    val scope = rememberCoroutineScope()
    val deviceType = LocalDeviceType.current
    val lang = activeProfile?.safeLanguage ?: "en"
    val isAnimeMeta = meta.genres?.any { g -> g.lowercase().contains("anime") } == true
    val useMpvBackend = activeProfile?.safePreferredPlayer == "mpv" ||
        (activeProfile?.safeAnimeUseMpv == true && isAnimeMeta)
    val mpvCustomOptions = activeProfile?.safeMpvCustomOptions.orEmpty()
    val mpvPlayer = remember(context, useMpvBackend, mpvCustomOptions) {
        if (useMpvBackend) runCatching { MpvEmbeddedPlayer(context, mpvCustomOptions) }.getOrNull() else null
    }

    PlayerOrientationLock(activity, deviceType)

    val torrentManager = TorrentStreamManager.getInstance(context)
    val torrentStatus by torrentManager.status.collectAsStateWithLifecycle()
    
    val audioManager = remember { context.getSystemService(Context.AUDIO_SERVICE) as android.media.AudioManager }
    val state = rememberPlayerScreenState(
        initialVideoId = videoId,
        initialStreamIndex = initialStreamIndex,
        initialVolume = audioManager.getStreamVolume(android.media.AudioManager.STREAM_MUSIC)
    )
    val maxVolume = remember { audioManager.getStreamMaxVolume(android.media.AudioManager.STREAM_MUSIC) }
    val seekBackwardMs = (activeProfile?.safeSeekBackwardSeconds ?: 10) * 1000L
    val seekForwardMs = (activeProfile?.safeSeekForwardSeconds ?: 10) * 1000L

    LaunchedEffect(activeProfile?.safeTorrentSpeedPreset) {
        torrentManager.configurePreferences(speedPreset = activeProfile?.safeTorrentSpeedPreset)
    }
    
    PlayerTransientFeedbackEffects(
        showVolumeBar = state.showVolumeBar,
        seekFeedbackVersion = state.seekFeedbackVersion,
        segmentSkipFeedbackVersion = state.segmentSkipFeedbackVersion,
        zoomOverlayVersion = state.zoomOverlayVersion,
        setShowVolumeBar = { state.showVolumeBar = it },
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
    val emptyAudioTracks = remember { MutableStateFlow(emptyList<MediaTrack>()) }
    val emptyVideoFormatBadge = remember { MutableStateFlow<VideoFormatBadge?>(null) }
    val emptySubtitleTracks = remember { MutableStateFlow(emptyList<MediaTrack>()) }
    val emptyTrack = remember { MutableStateFlow<MediaTrack?>(null) }
    val emptyTechnicalInfo = remember { MutableStateFlow<String?>(null) }
    val availableAudios by (activeEngine?.availableAudios ?: emptyAudioTracks).collectAsStateWithLifecycle()
    val availableSubtitles by (activeEngine?.availableSubtitles ?: emptySubtitleTracks).collectAsStateWithLifecycle()
    val currentAudio by (activeEngine?.currentAudio ?: emptyTrack).collectAsStateWithLifecycle()
    val currentSubtitle by (activeEngine?.currentSubtitle ?: emptyTrack).collectAsStateWithLifecycle()
    val effectiveTechnicalInfo by (activeEngine?.technicalInfo ?: emptyTechnicalInfo).collectAsStateWithLifecycle()
    val activeVideoFormatBadge by (activeEngine?.activeVideoFormatBadge ?: emptyVideoFormatBadge).collectAsStateWithLifecycle()

    PlayerEngineSettingsEffects(
        activeEngine = activeEngine,
        audioDelayMs = state.audioDelayMs,
        subtitleDelayMs = state.subtitleDelayMs,
        resizeMode = state.resizeMode
    )

    val updateEngine: (PlayerEngineSnapshot.() -> PlayerEngineSnapshot) -> Unit = { f ->
        state.engine = state.engine.f()
    }

    fun seekSafely(targetPosition: Long) {
        if (!state.engine.playback.hasStartedPlaying) return
        val maxDuration = when {
            useMpvBackend && state.engine.timeline.duration > 0 -> state.engine.timeline.duration
            exoPlayer.duration > 0 -> exoPlayer.duration
            state.engine.timeline.duration > 0 -> state.engine.timeline.duration
            else -> Long.MAX_VALUE
        }
        val target = targetPosition.coerceIn(0L, maxDuration)
        if (useMpvBackend) {
            state.pendingSeekTarget = target
            state.engine = state.engine.copy(timeline = state.engine.timeline.copy(position = target))
            activeEngine?.seekTo(target)
            return
        }
        val before = exoPlayer.currentPosition
        state.pendingSeekTarget = target
        state.timelinePosition = target
        state.engine = state.engine.copy(timeline = state.engine.timeline.copy(position = target))
        exoPlayer.seekTo(target)
        // Skip retry for torrent streams: a second seekTo on a torrent proxy that hasn't
        // buffered the target position yet causes the stream to restart from position 0.
        if (!state.resolvedUrl.isTorrentPlaybackUrl()) {
            scope.launch {
                delay(900)
                val after = exoPlayer.currentPosition
                if (target > before + 2_000L && after < before + 1_000L && exoPlayer.isCurrentMediaItemSeekable) {
                    exoPlayer.seekTo(target)
                    state.timelinePosition = target
                    state.engine = state.engine.copy(timeline = state.engine.timeline.copy(position = target))
                }
            }
        }
    }

    PlayerEpisodeMetadataEffect(
        meta = meta,
        currentVideoId = state.currentVideoId,
        viewModel = viewModel,
        language = activeProfile?.language ?: "en",
        setEpisodeLine = { state.currentEpisodeMetaLine = it },
        setEpisodeArtwork = { state.currentEpisodeArtwork = it }
    )

    val mainFocusRequester = remember { FocusRequester() }
    val playPauseFocusRequester = remember { FocusRequester() }
    val seekbarFocusRequester = remember { FocusRequester() }

    val externalPlayerLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        val returnedPosition = result.data?.getLongExtra("position", -1L) ?: -1L
        val positionToSave = if (returnedPosition >= 0L) returnedPosition else state.externalPlayerStartedPosition
        if (positionToSave > 0L) {
            val chosenStream = state.currentStreams.getOrNull(state.currentStreamIndex)
            viewModel.savePlaybackProgress(
                meta = meta,
                timeOffset = positionToSave,
                duration = state.engine.timeline.duration,
                videoId = state.currentVideoId ?: videoId,
                streamIndex = state.currentStreamIndex,
                episodeName = state.currentEpisodeMetaLine,
                lastStreamUrl = chosenStream?.playableUrl ?: state.currentUrl,
                lastStreamTitle = chosenStream?.title,
                lastBingeGroup = null,
                lastAudioLanguage = null,
                lastSubtitleLanguage = null,
                scrobbleTraktPause = false
            )
        }
        state.externalPlayerStartedPosition = -1L
    }

    fun openInExternalPlayer() {
        // Use the raw stream URL, not the proxy/DV-rewrite URL — external players
        // can't use Fluxa's internal local HTTP proxy.
        val rawUrl = state.currentStreams.getOrNull(state.currentStreamIndex)?.playableUrl
            ?: state.currentUrl
        if (rawUrl.isNullOrEmpty()) return
        val currentPos = if (useMpvBackend) state.engine.timeline.position else exoPlayer.currentPosition
        state.externalPlayerStartedPosition = currentPos
        val subs = state.currentExternalSubtitleTracks
        val intent = android.content.Intent(android.content.Intent.ACTION_VIEW).apply {
            setDataAndType(Uri.parse(rawUrl), "video/*")
            putExtra("title", meta.name)
            putExtra("position", currentPos)
            if (subs.isNotEmpty()) {
                // VLC
                putExtra("subtitles_location", subs.first().url)
                // MX Player — expects Uri[] not ArrayList
                putExtra("subs", subs.map { Uri.parse(it.url) }.toTypedArray())
                putExtra("subs.name", subs.map { it.label ?: it.language ?: "" }.toTypedArray())
            }
        }
        try {
            externalPlayerLauncher.launch(intent)
        } catch (e: Exception) {
            android.widget.Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, "toast.external_player_not_found"), android.widget.Toast.LENGTH_SHORT).show()
        }
    }

    fun smartCast(url: String? = state.resolvedUrl) {
        if (url.isNullOrEmpty()) return
        val videoUrl = url
        
        //  PRO-CASTING INTENT: Target specialized casting apps first
        val intent = android.content.Intent(android.content.Intent.ACTION_VIEW).apply {
            setDataAndType(android.net.Uri.parse(videoUrl), "video/*")
            addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)
            
            // Add metadata for smarter casting apps
            putExtra("title", meta.name)
            putExtra("poster", meta.poster)
        }

        try {
            // Try to find Web Video Caster or BubbleUPnP directly if possible
            val chooser = android.content.Intent.createChooser(intent, AppStrings.t(activeProfile?.safeLanguage, "auto.choose_player"))
            context.startActivity(chooser)
        } catch (e: Exception) {
            android.widget.Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, "toast.cast_app_not_found"), android.widget.Toast.LENGTH_SHORT).show()
        }
    }

    fun showControlsTemp() {
        state.showControls = true
        state.controlsTimerJob?.cancel()
        state.controlsTimerJob = scope.launch {
            delay(5000) //  5s AUTO-HIDE
            if (!state.isScrubbing) state.showControls = false 
        }
        scope.launch { delay(100); try { playPauseFocusRequester.requestFocus() } catch(e: Exception) {} }
    }

    fun switchToStream(streamIndex: Int) {
        val nextStream = state.currentStreams.getOrNull(streamIndex) ?: return
        if (streamIndex == state.currentStreamIndex || nextStream.playableUrl.isNullOrEmpty()) return
        android.util.Log.w(
            "PlayerScreen",
            "Switching stream ${state.currentStreamIndex} -> $streamIndex: fileIdx=${nextStream.fileIdx} " +
                "url=${nextStream.playableUrl} name=${nextStream.name} title=${nextStream.title}"
        )
        state.lastSavedPosition = if (useMpvBackend) state.engine.timeline.position else exoPlayer.currentPosition
        state.shouldApplyInitialProgress = false
        state.isSwitchingAudioSource = true
        state.engine = state.engine.copy(playerError = null, playback = state.engine.playback.copy(isBuffering = true, hasStartedPlaying = false), render = RenderSnapshot())
        state.currentStreamIndex = streamIndex
        state.currentUrl = nextStream.playableUrl
        scope.launch {
            delay(1400)
            state.isSwitchingAudioSource = false
        }
    }

    fun openSourceSelectionScreen() {
        val selectedStream = state.currentStreams.getOrNull(state.currentStreamIndex)
        val progress = if (useMpvBackend) state.engine.timeline.position else exoPlayer.currentPosition.takeIf { it > 0 } ?: state.engine.timeline.position
        onSelectSource(
            PlayerSourceSelectionRequest(
                meta = meta,
                videoId = state.currentVideoId ?: videoId,
                progress = progress,
                streamIndex = state.currentStreamIndex.takeIf { it >= 0 },
                streamUrl = selectedStream?.playableUrl ?: state.currentUrl,
                streamTitle = selectedStream?.title
            )
        )
    }

    LaunchedEffect(Unit) { delay(1000); try { mainFocusRequester.requestFocus() } catch(e: Exception) {} }

    LaunchedEffect(Unit) {
        if (!useMpvBackend) exoPlayer.volume = 1.0f // Ensure main player is not muted
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
        context = context,
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
        onLastSavedTimestampChanged = { state.lastSavedTimestamp = it }
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
        setCurrentUrl = { state.currentUrl = it },
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

    ExoPlayerListenerEffect(
        exoPlayer = exoPlayer,
        exoEngine = exoEngine,
        useMpvBackend = useMpvBackend,
        currentUrl = state.currentUrl,
        currentStreamsSize = state.currentStreams.size,
        returnToSourcesOnError = returnToSourcesOnError,
        lang = lang,
        mergeSkipSegments = { newSegments ->
            state.skipSegments = (state.skipSegments + newSegments).distinctBy { "${it.startTime}-${it.type}" }
        },
        updateEngine = updateEngine,
        openSourceSelectionScreen = ::openSourceSelectionScreen,
        torrentManager = torrentManager,
        retryPlayback = {
            val savedUrl = state.resolvedUrl
            if (savedUrl != null && torrentParseRetryCount.intValue < 2) {
                torrentParseRetryCount.intValue++
                scope.launch {
                    exoEngine.clear()
                    state.resolvedUrl = null
                    delay(1500)
                    state.resolvedUrl = savedUrl
                }
            } else {
                torrentParseRetryCount.intValue = 0
                state.engine = state.engine.copy(
                    playerError = AppStrings.format(lang, "player.error_load", AppStrings.t(lang, "player.error_server")),
                    playback = state.engine.playback.copy(isBuffering = false),
                )
                if (returnToSourcesOnError && state.currentStreams.size > 1) openSourceSelectionScreen()
            }
        }
    )

    PlayerStallWatchdogEffect(
        currentUrl = state.currentUrl,
        currentStreamIndex = state.currentStreamIndex,
        currentStreamsSize = state.currentStreams.size,
        useMpvBackend = useMpvBackend,
        isBuffering = state.engine.playback.isBuffering,
        hasStartedPlaying = state.engine.playback.hasStartedPlaying,
        isVideoRendered = state.engine.render.isVideoRendered,
        torrentStatus = latestTorrentStatus,
        returnToSourcesOnError = returnToSourcesOnError,
        lang = lang,
        updateEngine = updateEngine,
        openSourceSelectionScreen = ::openSourceSelectionScreen
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
        setSkipSegments = { state.skipSegments = it },
        skipSegments = state.skipSegments,
        updateEngine = updateEngine,
        setResolvedUrl = { state.resolvedUrl = it },
        openSourceSelectionScreen = ::openSourceSelectionScreen
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
        shouldApplyInitialProgress = state.shouldApplyInitialProgress,
        initialProgress = initialProgress,
        resizeMode = state.resizeMode,
        audioDelayMs = state.audioDelayMs,
        subtitleDelayMs = state.subtitleDelayMs,
        updateEngine = updateEngine,
        clearLastSavedPosition = { state.lastSavedPosition = 0L },
        clearInitialProgress = { state.shouldApplyInitialProgress = false },
        onExternalSubtitlesFetched = { state.currentExternalSubtitleTracks = it }
    )
    fun playNext() {
        state.nextEpisodePending?.let { next ->
            if (activeProfile?.safeTryBingeGroup == true) {
                state.preferredBingeGroupForNextEpisode = state.currentStreams.getOrNull(state.currentStreamIndex)?.bingeGroup
            }
            state.resetForEpisode(next.id)
        } ?: onBack()
    }

    fun playPrevious() {
        state.previousEpisodePending?.let { previous ->
            state.resetForEpisode(previous.id)
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
                        playNext()
                    } else {
                        seekSafely(segment.endTime)
                    }
                    state.introAutoSkipped = true
                    state.segmentSkipFeedbackVersion += 1
                }
            }
            delay(500)
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

    fun toggleSubtitleSelection() {
        if (currentSubtitle != null) {
            activeEngine?.disableSubtitles()
        } else {
            TrackSelectionState.findPreferredSubtitle(availableSubtitles, activeProfile, meta)?.let { track ->
                activeEngine?.enableSubtitle(track)
            }
        }
        showControlsTemp()
    }

    BackHandler {
        if (state.showMobileTrackPicker) state.showMobileTrackPicker = false
        else if (state.showSettings) state.showSettings = false
        else onBack()
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

    // Scrub live preview: seek to keyframe while user drags the seekbar.
    // Debounced 80ms so rapid slider changes coalesce into one seek call.
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
        hasNextEpisode = state.nextEpisodePending != null,
        activeEngine = activeEngine,
        playNext = ::playNext
    )

    fun closePlayer() {
        state.showSettings = false
        state.showControls = false
        onBack()
    }

    fun performRelativeSeek(direction: Int) {
        if (!state.engine.playback.hasStartedPlaying) return
        val step = if (direction > 0) seekForwardMs else seekBackwardMs
        val base = state.pendingSeekTarget ?: if (useMpvBackend) state.engine.timeline.position else exoPlayer.currentPosition
        val rawTarget = if (direction > 0) base + step else base - step
        val maxDuration = when {
            useMpvBackend && state.engine.timeline.duration > 0 -> state.engine.timeline.duration
            exoPlayer.duration > 0 -> exoPlayer.duration
            state.engine.timeline.duration > 0 -> state.engine.timeline.duration
            else -> Long.MAX_VALUE
        }
        val target = rawTarget.coerceIn(0L, maxDuration)
        seekSafely(target)
        if (state.seekDirection == direction) {
            state.seekFeedbackMs += step
        } else {
            state.seekDirection = direction
            state.seekFeedbackMs = step
        }
        state.seekFeedbackVersion += 1
        state.showSeekFeedback = true
    }

    // Auto-play when finished
    LaunchedEffect(state.engine.playback.playbackEnded) {
        if (state.engine.playback.playbackEnded && state.engine.playback.hasStartedPlaying) {
            if (activeProfile?.safeAutoPlayNextEpisode == true) {
                playNext()
            }
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
        activeVideoFormatBadge = activeVideoFormatBadge,
        showControlsTemp = ::showControlsTemp,
        seekSafely = ::seekSafely,
        toggleSubtitleSelection = ::toggleSubtitleSelection,
        performRelativeSeek = ::performRelativeSeek,
        playPrevious = ::playPrevious,
        playNext = ::playNext,
        smartCast = ::smartCast,
        openInExternalPlayer = ::openInExternalPlayer,
        openSourceSelectionScreen = ::openSourceSelectionScreen,
        closePlayer = ::closePlayer,
        switchToStream = ::switchToStream
    )
}
