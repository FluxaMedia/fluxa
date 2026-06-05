@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.ui.catalog

import android.app.Activity
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.getValue
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.AspectRatioFrameLayout
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.player.DolbyVisionFallbackMode
import com.fluxa.app.player.ExoPlayerEngine
import com.fluxa.app.player.MpvPlaybackState
import com.fluxa.app.player.PlayerEngine
import com.fluxa.app.player.PlayerEngineRequest
import com.fluxa.app.player.TorrentStreamManager
import com.fluxa.app.player.TorrentStreamResult
import com.fluxa.app.player.TorrentStreamStatus
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

@Composable
internal fun PlayerEpisodeMetadataEffect(
    meta: Meta,
    currentVideoId: String?,
    viewModel: HomeViewModel,
    language: String,
    setEpisodeLine: (String?) -> Unit,
    setEpisodeArtwork: (String?) -> Unit
) {
    LaunchedEffect(currentVideoId, meta.id) {
        val fallbackEpisodeLine = meta.lastEpisodeName?.takeIf { it.isNotBlank() }
        setEpisodeLine(if (meta.type == "series") fallbackEpisodeLine else null)
        setEpisodeArtwork(null)
        if (meta.type != "series") return@LaunchedEffect
        val targetId = currentVideoId ?: return@LaunchedEffect
        val parts = targetId.split(":")
        if (parts.size < 3) return@LaunchedEffect
        val season = parts[parts.size - 2].toIntOrNull() ?: return@LaunchedEffect
        val episode = parts[parts.size - 1].toIntOrNull() ?: return@LaunchedEffect
        val shortEpisodeLine = "S$season E$episode"
        setEpisodeLine(fallbackEpisodeLine ?: shortEpisodeLine)
        runCatching {
            val episodes = viewModel.getSeasonEpisodes(meta.id, season, language)
            val matched = episodes.firstOrNull { it.id == targetId || (it.season == season && it.number == episode) }
            setEpisodeArtwork(matched?.thumbnail)
            setEpisodeLine(
                matched?.name?.takeIf { it.isNotBlank() }?.let { "$shortEpisodeLine $it" }
                    ?: fallbackEpisodeLine
                    ?: shortEpisodeLine
            )
        }
    }
}

@Composable
internal fun PlayerStreamLoadingEffect(
    meta: Meta,
    videoId: String?,
    currentVideoId: String?,
    initialStreams: List<Stream>,
    initialStreamIndex: Int,
    lastStreamUrl: String?,
    lastStreamTitle: String?,
    activeProfile: UserProfile?,
    preferredBingeGroup: String?,
    viewModel: HomeViewModel,
    lang: String,
    setCurrentUrl: (String?) -> Unit,
    setCurrentStreams: (List<Stream>) -> Unit,
    setCurrentStreamIndex: (Int) -> Unit,
    setZeroSpeedTicks: (Int) -> Unit,
    updateEngine: (PlayerEngineSnapshot.() -> PlayerEngineSnapshot) -> Unit,
    clearPreferredBingeGroup: () -> Unit
) {
    LaunchedEffect(currentVideoId, meta, preferredBingeGroup, activeProfile?.safeStreamSourceSelectionMode, activeProfile?.safeStreamSourceRegexPattern) {
        android.util.Log.d("PlayerScreen", "Loading streams for ${meta.id} / $currentVideoId")
        fun applyCoreState(state: PlayerRuntimeCoreState) {
            setCurrentUrl(state.currentUrl)
            setCurrentStreams(state.currentStreams)
            setCurrentStreamIndex(state.currentStreamIndex)
            setZeroSpeedTicks(state.zeroSpeedTicks)
            updateEngine {
                copy(
                    playback = playback.copy(isBuffering = state.isBuffering),
                    render = render.copy(isVideoRendered = state.isVideoRendered),
                    playerError = when (state.playerError) {
                        null -> null
                        "no_source" -> AppStrings.t(lang, "player.error_no_source")
                        else -> AppStrings.format(lang, "player.error_generic", state.playerError)
                    }
                )
            }
        }

        applyCoreState(
            PlayerRuntimeCoreState(
                currentVideoId = currentVideoId,
                currentStreamIndex = initialStreamIndex,
                isBuffering = true
            )
        )
        try {
            val loaded = viewModel.loadPlayerStreams(
                meta = meta,
                currentVideoId = currentVideoId,
                initialVideoId = videoId,
                initialStreams = initialStreams,
                initialStreamIndex = initialStreamIndex,
                savedUrl = lastStreamUrl ?: meta.lastStreamUrl,
                savedTitle = lastStreamTitle ?: meta.lastStreamTitle,
                activeProfile = activeProfile,
                preferredBingeGroup = preferredBingeGroup
            )
            applyCoreState(loaded)
            if (loaded.playerError == null) {
                clearPreferredBingeGroup()
                android.util.Log.i(
                    "PlayerScreen",
                    "Loaded ${loaded.currentStreams.size} streams, picked index ${loaded.currentStreamIndex}: " +
                        "url=${loaded.currentUrl}"
                )
            }
        } catch (e: Exception) {
            android.util.Log.e("PlayerScreen", "Error loading streams", e)
            applyCoreState(PlayerRuntimeCoreState(playerError = e.message.orEmpty().ifBlank { "generic" }))
        }
    }
}

@Composable
internal fun PlayerSkipSegmentsEffect(
    meta: Meta,
    currentVideoId: String?,
    activeProfile: UserProfile?,
    viewModel: HomeViewModel,
    setSkipSegments: (List<IntroTimestamps>) -> Unit,
    setDismissedSkipSegments: (Set<String>) -> Unit,
    setIntroAutoSkipped: (Boolean) -> Unit
) {
    LaunchedEffect(currentVideoId, meta.id, meta.type, activeProfile?.safeUseIntroDb, activeProfile?.safeUseAniSkip) {
        setSkipSegments(emptyList())
        setDismissedSkipSegments(emptySet())
        setIntroAutoSkipped(false)
        if (meta.type != "series") return@LaunchedEffect
        val useIntroDb = activeProfile?.safeUseIntroDb != false
        val useAniSkip = activeProfile?.safeUseAniSkip != false
        if (!useIntroDb && !useAniSkip) return@LaunchedEffect
        val imdbId = resolveIntroImdbId(
            viewModel = viewModel,
            meta = meta,
            videoId = currentVideoId,
            language = activeProfile?.safeLanguage ?: "en"
        ) ?: run {
            android.util.Log.w("PlayerScreen", "Skip intro unavailable; IMDb id could not be resolved for ${meta.id} / $currentVideoId")
            return@LaunchedEffect
        }
        val seasonEpisode = extractSeasonEpisode(currentVideoId) ?: extractSeasonEpisode(meta.id) ?: return@LaunchedEffect
        setSkipSegments(
            viewModel.getIntroSegments(
                imdbId,
                seasonEpisode.first,
                seasonEpisode.second,
                title = meta.name,
                useIntroDb = useIntroDb,
                useAniSkip = useAniSkip
            ).also {
                android.util.Log.d("PlayerScreen", "Loaded ${it.size} skip segments for $imdbId S${seasonEpisode.first}E${seasonEpisode.second}")
            }
        )
    }
}
