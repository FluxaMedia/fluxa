package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.shared.feature.detail.DetailStreamUiModel
import com.fluxa.app.shared.feature.detail.DetailUiModel
import com.fluxa.app.shared.feature.detail.SourceSelectionScreen
import com.fluxa.app.ui.PlayerLaunchRequest
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.PlayerScreen

@Composable
internal fun PlayerRoute(
    request: PlayerLaunchRequest,
    activeProfile: UserProfile?,
    profileManager: ProfileManager,
    homeViewModel: HomeViewModel,
    mainPlayer: ExoPlayer,
    onUpdatePlayerRequest: (PlayerLaunchRequest) -> Unit,
    onBack: () -> Unit,
    onProfileChanged: (UserProfile) -> Unit
) {
    if (request.showSourceSelection) {
        val streams = request.initialStreams.map { stream ->
            DetailStreamUiModel(
                addonName = stream.addonName.orEmpty(),
                title = stream.title.orEmpty(),
                playableUrl = stream.playableUrl.orEmpty()
            )
        }.filter { it.playableUrl.isNotBlank() }
        SourceSelectionScreen(
            content = DetailUiModel(
                id = request.meta.id,
                type = request.meta.type,
                title = request.meta.name,
                description = request.meta.description.orEmpty(),
                posterUrl = request.meta.poster,
                backgroundUrl = request.meta.background,
                logoUrl = request.meta.logo,
                releaseLabel = request.meta.releaseInfo.orEmpty(),
                ratingLabel = request.meta.imdbRating.orEmpty(),
                runtimeLabel = request.meta.runtime,
                isInWatchlist = false,
                relatedItems = emptyList(),
                streams = streams
            ),
            language = activeProfile?.safeLanguage,
            onBack = onBack,
            onStreamSelected = { selected ->
                val index = request.initialStreams.indexOfFirst {
                    it.playableUrl == selected.playableUrl
                }.coerceAtLeast(0)
                onUpdatePlayerRequest(
                    request.copy(
                        streamIndex = index,
                        lastStreamUrl = selected.playableUrl,
                        lastStreamTitle = selected.title,
                        showSourceSelection = false
                    )
                )
            }
        )
        return
    }
    PlayerScreen(
        request.meta,
        request.initialProgress,
        request.videoId,
        onBack,
        homeViewModel,
        mainPlayer,
        activeProfile,
        {
            onProfileChanged(it)
            profileManager.saveProfile(it)
        },
        initialStreamIndex = request.streamIndex,
        initialStreams = request.initialStreams,
        lastStreamUrl = request.lastStreamUrl,
        lastStreamTitle = request.lastStreamTitle,
        initialBingeGroup = request.preferredBingeGroup,
        returnToSourcesOnError = request.returnToSourcesOnError,
        onSelectSource = { sourceRequest ->
            onUpdatePlayerRequest(
                PlayerLaunchRequest(
                    meta = sourceRequest.meta,
                    videoId = sourceRequest.videoId,
                    initialProgress = sourceRequest.progress,
                    streamIndex = sourceRequest.streamIndex ?: 0,
                    initialStreams = sourceRequest.streams,
                    lastStreamUrl = sourceRequest.streamUrl,
                    lastStreamTitle = sourceRequest.streamTitle,
                    showSourceSelection = true
                )
            )
        }
    )
}
