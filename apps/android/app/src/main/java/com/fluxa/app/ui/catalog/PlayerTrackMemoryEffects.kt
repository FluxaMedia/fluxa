package com.fluxa.app.ui.catalog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.shared.feature.player.MediaTrack
import com.fluxa.app.player.PlayerEngine
import com.fluxa.app.data.remote.withCurrentEpisodeArtwork
import java.util.Locale

@Composable
internal fun PlayerTrackMemoryEffects(
    meta: Meta,
    currentVideoId: String?,
    activeProfile: UserProfile?,
    activeEngine: PlayerEngine?,
    availableSubtitles: List<MediaTrack>,
    currentAudio: MediaTrack?,
    currentSubtitle: MediaTrack?,
    hasStartedPlaying: Boolean,
    useMpvBackend: Boolean,
    viewModel: HomeViewModel,
    currentEpisodeArtwork: String?,
    currentPositionMs: () -> Long,
    duration: Long,
    currentStreamIndex: Int,
    currentEpisodeMetaLine: String?,
    currentStreams: List<Stream>
) {
    LaunchedEffect(activeProfile?.preferredAudioLanguage, activeProfile?.secondaryAudioLanguage, meta.originalLanguage, meta.genres) {
        val preferred = TrackSelectionState.resolvePreferredAudioLanguage(activeProfile, meta)
        activeEngine?.applyPreferredAudioLanguage(preferred)
    }

    LaunchedEffect(
        activeProfile?.safeSubtitleSize,
        activeProfile?.safeSubtitleColor,
        activeProfile?.safeSubtitleBackgroundColor,
        activeProfile?.safeSubtitleOutlineColor,
        activeProfile?.safeSubtitleTextOpacity,
        activeProfile?.safeSubtitleBackgroundOpacity,
        activeProfile?.safeSubtitleOutlineOpacity,
        activeProfile?.safeSubtitleShadow,
        activeEngine
    ) {
        activeEngine?.applySubtitleStyle(activeProfile)
    }

    var subtitleMemoryApplied by remember(meta.id, currentVideoId) { mutableStateOf(false) }
    LaunchedEffect(availableSubtitles, hasStartedPlaying) {
        if (subtitleMemoryApplied || !hasStartedPlaying) return@LaunchedEffect
        val memory = meta.lastSubtitleLanguage
        if (memory == "__off__") {
            activeEngine?.disableSubtitles()
            subtitleMemoryApplied = true
        } else if (!memory.isNullOrBlank() && availableSubtitles.isNotEmpty()) {
            val match = availableSubtitles.firstOrNull {
                TrackSelectionState.matchesSubtitleLanguage(it, memory)
            }
            if (match != null) {
                activeEngine?.enableSubtitle(match)
                subtitleMemoryApplied = true
            }
        }
    }

    LaunchedEffect(currentAudio?.language, hasStartedPlaying, useMpvBackend) {
        if (!hasStartedPlaying) return@LaunchedEffect
        val lang = currentAudio?.language?.lowercase(Locale.ROOT)?.takeIf { it.isNotBlank() }
            ?: return@LaunchedEffect
        viewModel.savePlaybackProgress(
            meta.withCurrentEpisodeArtwork(currentEpisodeArtwork),
            currentPositionMs(),
            duration,
            currentVideoId,
            currentStreamIndex,
            currentEpisodeMetaLine,
            currentStreams.getOrNull(currentStreamIndex)?.playableUrl,
            currentStreams.getOrNull(currentStreamIndex)?.title,
            lastAudioLanguage = lang,
            scrobbleTraktPause = false
        )
    }

    LaunchedEffect(currentSubtitle?.language, currentSubtitle == null, hasStartedPlaying, subtitleMemoryApplied) {
        if (!hasStartedPlaying || !subtitleMemoryApplied) return@LaunchedEffect
        val subtitleLang = currentSubtitle?.language?.lowercase(Locale.ROOT)?.takeIf { it.isNotBlank() } ?: "__off__"
        viewModel.savePlaybackProgress(
            meta.withCurrentEpisodeArtwork(currentEpisodeArtwork),
            currentPositionMs(),
            duration,
            currentVideoId,
            currentStreamIndex,
            currentEpisodeMetaLine,
            currentStreams.getOrNull(currentStreamIndex)?.playableUrl,
            currentStreams.getOrNull(currentStreamIndex)?.title,
            lastSubtitleLanguage = subtitleLang,
            scrobbleTraktPause = false
        )
    }
}
