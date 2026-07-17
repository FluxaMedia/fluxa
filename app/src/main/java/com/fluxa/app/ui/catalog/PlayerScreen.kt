@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
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
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.player.*
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.player.MediaTrack
import com.fluxa.app.shared.feature.player.dismissKey
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import java.net.Inet4Address
import java.net.NetworkInterface

private fun castReachableUrl(rawUrl: String): String {
    val uri = runCatching { Uri.parse(rawUrl) }.getOrNull() ?: return rawUrl
    val host = uri.host ?: return rawUrl
    if (uri.scheme != "http" || host !in setOf("127.0.0.1", "localhost", "::1")) {
        return rawUrl
    }
    val lanIp = deviceLanIpv4() ?: return rawUrl
    val token = TorrentServerEngine.castAccessToken
    val builder = uri.buildUpon()
        .authority(if (uri.port > 0) "$lanIp:${uri.port}" else lanIp)
    if (token.isNotBlank() && uri.getQueryParameter("access_token").isNullOrBlank()) {
        builder.appendQueryParameter("access_token", token)
    }
    return builder.build().toString()
}

private fun deviceLanIpv4(): String? {
    return runCatching {
        NetworkInterface.getNetworkInterfaces().asSequence()
            .filter { network -> network.isUp && !network.isLoopback }
            .flatMap { network -> network.inetAddresses.asSequence() }
            .filterIsInstance<Inet4Address>()
            .map { address -> address.hostAddress }
            .filterNotNull()
            .firstOrNull { address ->
                !address.startsWith("127.") &&
                    !address.startsWith("169.254.") &&
                    address != "0.0.0.0"
            }
    }.getOrNull()
}

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

    LaunchedEffect(Unit) {
        val windowBrightness = activity?.window?.attributes?.screenBrightness ?: -1f
        state.currentBrightness = if (windowBrightness in 0f..1f) {
            windowBrightness
        } else {
            runCatching {
                android.provider.Settings.System.getInt(
                    context.contentResolver,
                    android.provider.Settings.System.SCREEN_BRIGHTNESS
                ) / 255f
            }.getOrDefault(0.5f)
        }
    }

    fun adjustVolumeByDelta(delta: Float) {
        val next = (state.currentVolumeExact + delta * maxVolume).coerceIn(0f, maxVolume.toFloat())
        state.currentVolumeExact = next
        val rounded = next.toInt()
        if (rounded != state.currentVolume) {
            audioManager.setStreamVolume(android.media.AudioManager.STREAM_MUSIC, rounded, 0)
            state.currentVolume = rounded
        }
        state.volumeBarVersion += 1
    }

    fun adjustBrightnessByDelta(delta: Float) {
        val next = (state.currentBrightness + delta).coerceIn(0.02f, 1f)
        state.currentBrightness = next
        activity?.window?.let { window ->
            val params = window.attributes
            params.screenBrightness = next
            window.attributes = params
        }
        state.brightnessBarVersion += 1
    }

    LaunchedEffect(activeProfile?.safeTorrentSpeedPreset) {
        torrentManager.configurePreferences(speedPreset = activeProfile?.safeTorrentSpeedPreset)
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

    LaunchedEffect(meta.id) { viewModel.loadParentsGuide(meta.id) }
    val parentsGuide by viewModel.parentsGuide.collectAsStateWithLifecycle()

    LaunchedEffect(state.currentVideoId) {
        state.showParentsGuide = false
        state.parentsGuideShown = false
    }
    LaunchedEffect(state.engine.playback.hasStartedPlaying, parentsGuide) {
        if (state.engine.playback.hasStartedPlaying && parentsGuide.isNotEmpty() && !state.parentsGuideShown) {
            state.parentsGuideShown = true
            state.showParentsGuide = true
        }
    }

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
        PlayerPipSuppression.suppressAutoEnter = false
    }

    fun openInExternalPlayer() {
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
                putExtra("subtitles_location", subs.first().url)
                putExtra("subs", subs.map { Uri.parse(it.url) }.toTypedArray())
                putExtra("subs.name", subs.map { it.label ?: it.language ?: "" }.toTypedArray())
            }
        }
        PlayerPipSuppression.suppressAutoEnter = true
        try {
            externalPlayerLauncher.launch(intent)
        } catch (e: Exception) {
            PlayerPipSuppression.suppressAutoEnter = false
            android.widget.Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, "toast.external_player_not_found"), android.widget.Toast.LENGTH_SHORT).show()
        }
    }

    fun smartCast(url: String? = state.resolvedUrl) {
        if (url.isNullOrEmpty()) return
        val videoUrl = castReachableUrl(url)
        
        val intent = android.content.Intent(android.content.Intent.ACTION_VIEW).apply {
            setDataAndType(android.net.Uri.parse(videoUrl), "video/*")
            addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)
            
            putExtra("title", meta.name)
            putExtra("poster", meta.poster)
        }

        try {
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
            delay(5000)
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

    fun isCloudstreamPlayback(): Boolean {
        return meta.id.startsWith("cs3:") ||
            state.currentVideoId?.startsWith("cs3:") == true ||
            state.currentStreams.getOrNull(state.currentStreamIndex)?.addonName?.trim() in viewModel.loadedCs3ApiNames.value
    }

    fun fallbackToNextCloudstreamStream(): Boolean {
        if (!isCloudstreamPlayback()) return false
        val failedUrl = state.currentStreams.getOrNull(state.currentStreamIndex)?.playableUrl
            ?: state.currentUrl
        if (!failedUrl.isNullOrBlank()) {
            state.failedAutoFallbackUrls = state.failedAutoFallbackUrls + failedUrl
        }
        val indices = state.currentStreams.indices.drop(state.currentStreamIndex + 1) +
            state.currentStreams.indices.take(state.currentStreamIndex)
        val nextIndex = indices.firstOrNull { index ->
            val candidateUrl = state.currentStreams[index].playableUrl
            !candidateUrl.isNullOrBlank() && candidateUrl !in state.failedAutoFallbackUrls
        }
            ?: return false
        switchToStream(nextIndex)
        return true
    }

    fun openSourceSelectionScreen() {
        val selectedStream = state.currentStreams.getOrNull(state.currentStreamIndex)
        val progress = if (useMpvBackend) state.engine.timeline.position else exoPlayer.currentPosition.takeIf { it > 0 } ?: state.engine.timeline.position
        onSelectSource(
            PlayerSourceSelectionRequest(
                meta = meta,
                videoId = state.currentVideoId ?: videoId,
                progress = progress,
                streams = state.currentStreams,
                streamIndex = state.currentStreamIndex.takeIf { it >= 0 },
                streamUrl = selectedStream?.playableUrl ?: state.currentUrl,
                streamTitle = selectedStream?.title
            )
        )
    }

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
        currentStreamIndex = state.currentStreamIndex,
        currentStreamsSize = state.currentStreams.size,
        autoFallbackOnStreamError = isCloudstreamPlayback() || activeProfile?.safeAutoRetryNextSource == true,
        returnToSourcesOnError = returnToSourcesOnError,
        lang = lang,
        mergeSkipSegments = { newSegments ->
            state.skipSegments = (state.skipSegments + newSegments).distinctBy { "${it.startTime}-${it.type}" }
        },
        updateEngine = updateEngine,
        openSourceSelectionScreen = ::openSourceSelectionScreen,
        fallbackToNextStream = ::fallbackToNextCloudstreamStream,
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
        autoFallbackOnStreamError = isCloudstreamPlayback() || activeProfile?.safeAutoRetryNextSource == true,
        useMpvBackend = useMpvBackend,
        isBuffering = state.engine.playback.isBuffering,
        hasStartedPlaying = state.engine.playback.hasStartedPlaying,
        isVideoRendered = state.engine.render.isVideoRendered,
        torrentStatus = latestTorrentStatus,
        returnToSourcesOnError = returnToSourcesOnError,
        lang = lang,
        updateEngine = updateEngine,
        openSourceSelectionScreen = ::openSourceSelectionScreen,
        fallbackToNextStream = ::fallbackToNextCloudstreamStream
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
        openSourceSelectionScreen = ::openSourceSelectionScreen,
        fallbackToNextStream = ::fallbackToNextCloudstreamStream
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
        onExternalSubtitlesFetched = { state.currentExternalSubtitleTracks = it },
        onNativeAssTracksExtracted = { state.embeddedNativeAssTracks = it }
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
        if (state.showSettings) state.showSettings = false
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
        parentsGuide = parentsGuide,
        showControlsTemp = ::showControlsTemp,
        seekSafely = ::seekSafely,
        toggleSubtitleSelection = ::toggleSubtitleSelection,
        performRelativeSeek = ::performRelativeSeek,
        onVolumeSwipe = ::adjustVolumeByDelta,
        onBrightnessSwipe = ::adjustBrightnessByDelta,
        playPrevious = ::playPrevious,
        playNext = ::playNext,
        smartCast = ::smartCast,
        openInExternalPlayer = ::openInExternalPlayer,
        openSourceSelectionScreen = ::openSourceSelectionScreen,
        closePlayer = ::closePlayer,
        switchToStream = ::switchToStream
    )
}
