@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import android.app.Activity
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.getValue
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.AspectRatioFrameLayout
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.player.DolbyVisionFallbackMode
import com.fluxa.app.player.ExternalSubtitleTrack
import com.fluxa.app.player.ExoPlayerEngine
import com.fluxa.app.player.LibassDebugLog
import com.fluxa.app.player.MpvPlaybackState
import com.fluxa.app.player.MkvNativeAssExtractor
import com.fluxa.app.player.NativeAssTrack
import com.fluxa.app.player.PlayerEngine
import com.fluxa.app.player.PlayerEngineRequest
import com.fluxa.app.player.TorrentStreamManager
import com.fluxa.app.player.TorrentStreamStatus
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

@Composable
internal fun ExoPlayerListenerEffect(
    exoPlayer: ExoPlayer,
    exoEngine: ExoPlayerEngine,
    useMpvBackend: Boolean,
    currentUrl: String?,
    currentStreamIndex: Int,
    currentStreamsSize: Int,
    autoFallbackOnStreamError: Boolean,
    returnToSourcesOnError: Boolean,
    lang: String,
    mergeSkipSegments: (List<IntroTimestamps>) -> Unit,
    updateEngine: (PlayerEngineSnapshot.() -> PlayerEngineSnapshot) -> Unit,
    openSourceSelectionScreen: () -> Unit,
    fallbackToNextStream: () -> Boolean,
    torrentManager: TorrentStreamManager,
    retryPlayback: () -> Unit = {}
) {
    // Capture latest callback so listener always sees current version without being recreated.
    val latestUpdateEngine = rememberUpdatedState(updateEngine)
    val latestRetryPlayback = rememberUpdatedState(retryPlayback)
    val latestMergeSkipSegments = rememberUpdatedState(mergeSkipSegments)
    val latestOpenSourceSelection = rememberUpdatedState(openSourceSelectionScreen)
    val latestFallbackToNextStream = rememberUpdatedState(fallbackToNextStream)
    val latestAutoFallbackOnStreamError by rememberUpdatedState(autoFallbackOnStreamError)
    val latestCurrentStreamIndex by rememberUpdatedState(currentStreamIndex)
    val latestCurrentStreamsSize by rememberUpdatedState(currentStreamsSize)

    DisposableEffect(exoPlayer, useMpvBackend) {
        val listener = object : Player.Listener {
            override fun onMetadata(metadata: androidx.media3.common.Metadata) {
                if (useMpvBackend) return
                val newSegments = mutableListOf<IntroTimestamps>()
                for (i in 0 until metadata.length()) {
                    val entry = metadata.get(i)
                    if (entry is androidx.media3.extractor.metadata.id3.ChapterFrame) {
                        val title = entry.chapterId.lowercase()
                        val type = when {
                            title.contains("intro") || title.contains("opening") || title.contains("op") || title.contains("balangç") || title.contains("giri") -> "intro"
                            title.contains("outro") || title.contains("ending") || title.contains("credits") || title.contains("ed") || title.contains("biti") || title.contains("jenerik") || title.contains("son") -> "outro"
                            title.contains("recap") || title.contains("previously") || title.contains("özet") -> "recap"
                            else -> null
                        }
                        if (type != null) {
                            newSegments.add(IntroTimestamps(entry.startTimeMs.toLong(), entry.endTimeMs.toLong(), type))
                        }
                    }
                }
                if (newSegments.isNotEmpty()) latestMergeSkipSegments.value(newSegments)
            }

            override fun onPlaybackStateChanged(playbackState: Int) {
                if (useMpvBackend) return
                latestUpdateEngine.value(when (playbackState) {
                    Player.STATE_READY -> { ->
                        copy(
                            playback = playback.copy(
                                isBuffering = false,
                                isPlaying = exoPlayer.playWhenReady,
                                playWhenReadyForScrobble = exoPlayer.playWhenReady,
                                hasStartedPlaying = true,
                                playbackEnded = false,
                            ),
                            timeline = timeline.copy(duration = if (exoPlayer.duration > 0) exoPlayer.duration else 0L),
                            playerError = null,
                        )
                    }
                    Player.STATE_ENDED -> { ->
                        copy(playback = playback.copy(isBuffering = false, playbackEnded = true))
                    }
                    else -> { ->
                        copy(playback = playback.copy(isBuffering = playbackState == Player.STATE_BUFFERING || playbackState == Player.STATE_IDLE))
                    }
                })
            }

            override fun onIsPlayingChanged(isPlaying: Boolean) {
                if (!useMpvBackend) latestUpdateEngine.value { copy(playback = playback.copy(isPlaying = isPlaying)) }
            }

            override fun onPlayWhenReadyChanged(playWhenReady: Boolean, reason: Int) {
                if (!useMpvBackend) latestUpdateEngine.value { copy(playback = playback.copy(playWhenReadyForScrobble = playWhenReady)) }
            }

            override fun onRenderedFirstFrame() {
                if (useMpvBackend) return
                latestUpdateEngine.value { copy(render = render.copy(isVideoRendered = true), playerError = null) }
            }

            override fun onPlayerError(error: androidx.media3.common.PlaybackException) {
                if (useMpvBackend) return
                val httpStatus = (error.cause as? androidx.media3.datasource.HttpDataSource.InvalidResponseCodeException)?.responseCode
                android.util.Log.e("PlayerScreen", " Player Error: ${error.errorCodeName} (${error.errorCode}) httpStatus=$httpStatus url=$currentUrl", error)
                if (
                    currentUrl.isTorrentPlaybackUrl() &&
                    error.errorCode == androidx.media3.common.PlaybackException.ERROR_CODE_PARSING_CONTAINER_UNSUPPORTED
                ) {
                    android.util.Log.w("PlayerScreen", "Torrent container parse error — retrying ExoPlayer")
                    latestUpdateEngine.value { copy(playerError = null, playback = playback.copy(isBuffering = true)) }
                    latestRetryPlayback.value()
                    return
                }
                if (
                    latestAutoFallbackOnStreamError &&
                    latestCurrentStreamIndex + 1 < latestCurrentStreamsSize &&
                    latestFallbackToNextStream.value()
                ) {
                    android.util.Log.w("PlayerScreen", "Playback failed; trying next Cloudstream link")
                    return
                }
                val detailedMsg = when (error.errorCode) {
                    androidx.media3.common.PlaybackException.ERROR_CODE_IO_BAD_HTTP_STATUS -> AppStrings.t(lang, "player.error_server")
                    androidx.media3.common.PlaybackException.ERROR_CODE_IO_NETWORK_CONNECTION_FAILED -> AppStrings.t(lang, "player.error_network")
                    else -> error.message
                }
                latestUpdateEngine.value {
                    copy(
                        playerError = AppStrings.format(lang, "player.error_load", detailedMsg.orEmpty()),
                        playback = playback.copy(isBuffering = false),
                    )
                }
                if (returnToSourcesOnError && currentStreamsSize > 1) latestOpenSourceSelection.value()
            }
        }
        exoPlayer.addListener(listener)
        onDispose {
            exoPlayer.removeListener(listener)
            exoPlayer.stop()
            exoPlayer.setVideoSurface(null)
            exoPlayer.clearMediaItems()
            torrentManager.stop()
            exoEngine.clear()
        }
    }
}

@Composable
internal fun PlayerStallWatchdogEffect(
    currentUrl: String?,
    currentStreamIndex: Int,
    currentStreamsSize: Int,
    autoFallbackOnStreamError: Boolean,
    useMpvBackend: Boolean,
    isBuffering: Boolean,
    hasStartedPlaying: Boolean,
    isVideoRendered: Boolean,
    torrentStatus: TorrentStreamStatus,
    returnToSourcesOnError: Boolean,
    lang: String,
    updateEngine: (PlayerEngineSnapshot.() -> PlayerEngineSnapshot) -> Unit,
    openSourceSelectionScreen: () -> Unit,
    fallbackToNextStream: () -> Boolean
) {
    val latestIsBuffering by rememberUpdatedState(isBuffering)
    val latestHasStartedPlaying by rememberUpdatedState(hasStartedPlaying)
    val latestIsVideoRendered by rememberUpdatedState(isVideoRendered)
    val latestTorrentStatus by rememberUpdatedState(torrentStatus)
    val latestUpdateEngine = rememberUpdatedState(updateEngine)
    val latestOpenSourceSelection = rememberUpdatedState(openSourceSelectionScreen)
    val latestFallbackToNextStream = rememberUpdatedState(fallbackToNextStream)
    val latestAutoFallbackOnStreamError by rememberUpdatedState(autoFallbackOnStreamError)
    LaunchedEffect(currentUrl, currentStreamIndex, currentStreamsSize) {
        if (currentUrl.isNullOrBlank()) return@LaunchedEffect
        val watchedUrl = currentUrl
        val isTorrentSource = watchedUrl.isTorrentPlaybackUrl()
        delay(
            when {
                isTorrentSource -> 60_000
                useMpvBackend -> 30_000
                else -> 12_000
            }
        )
        if (watchedUrl == currentUrl && latestIsBuffering && !latestHasStartedPlaying && !latestIsVideoRendered) {
            if (
                isTorrentSource &&
                (latestTorrentStatus.bufferProgress > 0 ||
                    latestTorrentStatus.activePeers > 0 ||
                    latestTorrentStatus.totalPeers > 0 ||
                    latestTorrentStatus.detailedStatus.isNotBlank())
            ) {
                latestUpdateEngine.value { copy(playerError = null, playback = playback.copy(isBuffering = true)) }
                return@LaunchedEffect
            }
            if (
                latestAutoFallbackOnStreamError &&
                currentStreamIndex + 1 < currentStreamsSize &&
                latestFallbackToNextStream.value()
            ) {
                android.util.Log.w("PlayerScreen", "Playback stalled; trying next Cloudstream link")
                return@LaunchedEffect
            }
            latestUpdateEngine.value { copy(playerError = AppStrings.format(lang, "player.error_load", AppStrings.t(lang, "player.error_source")), playback = playback.copy(isBuffering = false)) }
            if (returnToSourcesOnError && currentStreamsSize > 1) latestOpenSourceSelection.value()
        }
    }
}

@Composable
internal fun PlayerResolveUrlEffect(
    currentUrl: String?,
    currentStreamIndex: Int,
    currentStreams: List<Stream>,
    activeEngine: PlayerEngine?,
    torrentManager: TorrentStreamManager,
    currentVideoId: String?,
    meta: Meta,
    viewModel: HomeViewModel,
    returnToSourcesOnError: Boolean,
    lang: String,
    setSkipSegments: (List<IntroTimestamps>) -> Unit,
    skipSegments: List<IntroTimestamps>,
    updateEngine: (PlayerEngineSnapshot.() -> PlayerEngineSnapshot) -> Unit,
    setResolvedUrl: (String?) -> Unit,
    openSourceSelectionScreen: () -> Unit,
    fallbackToNextStream: () -> Boolean
) {
    LaunchedEffect(currentUrl) {
        val playbackUrl = currentUrl ?: return@LaunchedEffect
        val chosenStream = currentStreams.getOrNull(currentStreamIndex)
        chosenStream?.skipOffsets?.let { streamOffsets ->
            setSkipSegments((skipSegments + streamOffsets).distinctBy { "${it.startTime}-${it.type}" })
        }
        activeEngine?.pause()
        updateEngine { copy(playback = playback.copy(isBuffering = true, hasStartedPlaying = false), render = render.copy(isVideoRendered = false)) }
        setResolvedUrl(null)
        val resolved = viewModel.resolvePlayerPlayback(
            url = playbackUrl,
            stream = chosenStream,
            currentVideoId = currentVideoId,
            title = playbackNotificationTitle(meta, currentVideoId)
        )
        if (resolved.playerError == null) {
            setResolvedUrl(resolved.resolvedUrl)
            updateEngine { copy(playback = playback.copy(isBuffering = resolved.isBuffering)) }
        } else {
            android.util.Log.e("PlayerScreen", "Playback resolve error: ${resolved.playerError}")
            if (fallbackToNextStream()) return@LaunchedEffect
            updateEngine { copy(playerError = localizedPlayerError(lang, resolved.playerError), playback = playback.copy(isBuffering = false)) }
            if (returnToSourcesOnError && currentStreams.size > 1) {
                openSourceSelectionScreen()
            }
        }
    }
}

@Composable
internal fun PlayerPreparePlaybackEffect(
    resolvedUrl: String?,
    activeProfile: UserProfile?,
    activeEngine: PlayerEngine?,
    exoEngine: ExoPlayerEngine,
    useMpvBackend: Boolean,
    currentStreams: List<Stream>,
    currentStreamIndex: Int,
    viewModel: HomeViewModel,
    playerUserAddons: List<com.fluxa.app.data.remote.AddonDescriptor>,
    meta: Meta,
    currentVideoId: String?,
    lastSavedPosition: Long,
    shouldApplyInitialProgress: Boolean,
    initialProgress: Long,
    resizeMode: Int,
    audioDelayMs: Long,
    subtitleDelayMs: Long,
    updateEngine: (PlayerEngineSnapshot.() -> PlayerEngineSnapshot) -> Unit,
    clearLastSavedPosition: () -> Unit,
    clearInitialProgress: () -> Unit,
    onExternalSubtitlesFetched: (List<ExternalSubtitleTrack>) -> Unit = {},
    onNativeAssTracksExtracted: (List<NativeAssTrack>) -> Unit = {}
) {
    LaunchedEffect(resolvedUrl) {
        onExternalSubtitlesFetched(emptyList())
        onNativeAssTracksExtracted(emptyList())
        val playbackUrl = resolvedUrl
        if (!playbackUrl.isNullOrEmpty()) {
            val chosenStream = currentStreams.getOrNull(currentStreamIndex)
            LibassDebugLog.d(
                "playback effect start useMpv=$useMpvBackend url=${LibassDebugLog.urlSummary(playbackUrl)} streamIndex=$currentStreamIndex title=${chosenStream?.effectiveFilename ?: chosenStream?.rawDisplayTitle}"
            )
            val target = when {
                lastSavedPosition > 0L -> lastSavedPosition
                shouldApplyInitialProgress -> initialProgress
                else -> 0L
            }
            if (activeProfile?.safePreferredPlayer == "mpv" && activeEngine == null) {
                updateEngine { copy(playerError = AppStrings.t(activeProfile.safeLanguage, "player.error_source"), playback = playback.copy(isBuffering = false)) }
                return@LaunchedEffect
            }
            val dolbyVisionFallbackMode = when (activeProfile?.safeDolbyVisionFallbackMode) {
                "off" -> DolbyVisionFallbackMode.Off
                "force_hdr10" -> DolbyVisionFallbackMode.ForceHdr10
                else -> DolbyVisionFallbackMode.Auto
            }
            if (useMpvBackend) {
                exoEngine.pause()
                exoEngine.clear()
            }
            activeEngine?.applySubtitleStyle(activeProfile)
            activeEngine?.setZoomed(resizeMode == AspectRatioFrameLayout.RESIZE_MODE_ZOOM)
            activeEngine?.setAudioDelayMs(audioDelayMs)
            activeEngine?.setSubtitleDelayMs(subtitleDelayMs)
            val externalSubtitles = runCatching {
                fetchExternalSubtitleTracks(
                    viewModel = viewModel,
                    addons = playerUserAddons,
                    profile = activeProfile,
                    type = meta.type,
                    id = currentVideoId ?: meta.id,
                    stream = chosenStream
                )
            }.getOrElse { error ->
                LibassDebugLog.w("external subtitle fetch failed", error)
                emptyList()
            }
            LibassDebugLog.d("playback effect externalSubtitles=${externalSubtitles.size}")
            if (externalSubtitles.isNotEmpty()) onExternalSubtitlesFetched(externalSubtitles)
            activeEngine?.prepareAndPlay(
                PlayerEngineRequest(
                    url = playbackUrl,
                    stream = chosenStream,
                    subtitles = externalSubtitles,
                    startPositionMs = target,
                    preferredAudioLanguage = TrackSelectionState.resolvePreferredAudioLanguage(activeProfile, meta),
                    preferredSubtitleLanguage = activeProfile?.safePreferredSubtitleLanguage,
                    dolbyVisionFallbackMode = dolbyVisionFallbackMode,
                    dvRpuMode = activeProfile?.safeDvRpuMode ?: 2,
                    dvZeroLevel5 = activeProfile?.safeDvZeroLevel5 ?: false,
                    dvHdr10PlusMode = activeProfile?.safeDvHdr10PlusMode ?: "auto",
                    audioDecoderMode = activeProfile?.safeAudioDecoderMode ?: "hw_prefer"
                )
            )
            val isLocalUrl = playbackUrl.startsWith("file:", ignoreCase = true) ||
                playbackUrl.startsWith("content:", ignoreCase = true)
            if (isLocalUrl) {
                launch(Dispatchers.IO) {
                    LibassDebugLog.d("starting local embedded ASS extraction url=${LibassDebugLog.urlSummary(playbackUrl)}")
                    val embeddedNativeAss = MkvNativeAssExtractor.extract(
                        url = playbackUrl,
                        headers = chosenStream?.getHeaders().orEmpty()
                    )
                    LibassDebugLog.d("local embedded ASS extraction result tracks=${embeddedNativeAss.size}")
                    if (embeddedNativeAss.isNotEmpty()) {
                        launch(Dispatchers.Main) {
                            onNativeAssTracksExtracted(embeddedNativeAss)
                        }
                    }
                }
            } else {
                LibassDebugLog.d("skipping pre-extraction for non-local stream; embedded ASS depends on ExoPlayer relay")
            }
            clearLastSavedPosition()
            clearInitialProgress()
        }
    }
}

@Composable
internal fun PlayerBufferProgressEffect(
    exoPlayer: ExoPlayer,
    useMpvBackend: Boolean,
    isVideoRendered: Boolean,
    currentUrl: String?,
    torrentStatus: TorrentStreamStatus,
    updateEngine: (PlayerEngineSnapshot.() -> PlayerEngineSnapshot) -> Unit,
) {
    val torrentStatusState = rememberUpdatedState(torrentStatus)
    val isVideoRenderedState = rememberUpdatedState(isVideoRendered)
    val currentUrlState = rememberUpdatedState(currentUrl)
    val latestUpdateEngine = rememberUpdatedState(updateEngine)

    // MPV: binary buffer state driven by isVideoRendered
    LaunchedEffect(useMpvBackend, isVideoRendered) {
        if (useMpvBackend) {
            val ready = isVideoRendered
            latestUpdateEngine.value { copy(buffer = buffer.copy(bufferPercent = if (ready) 100 else 0, loadProgress = if (ready) 1f else 0f, seekbarBufferedProgress = if (ready) 1f else 0f)) }
        }
    }

    // ExoPlayer: event-driven — one atomic snapshot update per player event
    DisposableEffect(exoPlayer) {
        fun update() {
            if (useMpvBackend) return
            val ts = torrentStatusState.value
            val videoRendered = isVideoRenderedState.value
            val url = currentUrlState.value
            latestUpdateEngine.value {
                copy(
                    buffer = buffer.copy(
                        bufferPercent = exoPlayer.bufferedPercentage,
                        loadProgress = when {
                            videoRendered || exoPlayer.playbackState == Player.STATE_READY -> 1f
                            url.isTorrentPlaybackUrl() -> (ts.bufferProgress / 100f).coerceIn(0f, 1f)
                            exoPlayer.bufferedPercentage > 0 -> (exoPlayer.bufferedPercentage / 100f).coerceIn(0f, 1f)
                            exoPlayer.duration > 0L -> (exoPlayer.bufferedPosition.toFloat() / exoPlayer.duration.toFloat()).coerceIn(0f, 1f)
                            else -> 0f
                        },
                        seekbarBufferedProgress = when {
                            exoPlayer.duration > 0L -> maxOf(
                                (exoPlayer.bufferedPosition.toFloat() / exoPlayer.duration.toFloat()).coerceIn(0f, 1f),
                                ((exoPlayer.currentPosition + exoPlayer.totalBufferedDuration).toFloat() / exoPlayer.duration.toFloat()).coerceIn(0f, 1f),
                                (exoPlayer.bufferedPercentage / 100f).coerceIn(0f, 1f)
                            )
                            exoPlayer.bufferedPercentage > 0 -> (exoPlayer.bufferedPercentage / 100f).coerceIn(0f, 1f)
                            else -> 0f
                        }
                    )
                )
            }
        }
        val listener = object : Player.Listener {
            override fun onPlaybackStateChanged(state: Int) = update()
            override fun onIsPlayingChanged(isPlaying: Boolean) = update()
            override fun onPositionDiscontinuity(
                oldPosition: Player.PositionInfo,
                newPosition: Player.PositionInfo,
                reason: Int
            ) {
                // Update buffer bar after seek; skip auto-transitions (frequent HLS/DASH period changes)
                if (reason != Player.DISCONTINUITY_REASON_AUTO_TRANSITION) update()
            }
        }
        exoPlayer.addListener(listener)
        onDispose { exoPlayer.removeListener(listener) }
    }

    // Torrent only: slow poll for load progress (torrent progress has no ExoPlayer events)
    LaunchedEffect(currentUrl) {
        if (!currentUrl.isTorrentPlaybackUrl()) return@LaunchedEffect
        while (true) {
            delay(500)
            if (!useMpvBackend && !isVideoRenderedState.value &&
                exoPlayer.playbackState != Player.STATE_READY
            ) {
                val progress = (torrentStatusState.value.bufferProgress / 100f).coerceIn(0f, 1f)
                latestUpdateEngine.value { copy(buffer = buffer.copy(loadProgress = progress)) }
            }
        }
    }
}
