package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import com.fluxa.app.shared.feature.detail.DetailStreamUiModel
import com.fluxa.app.shared.feature.detail.DetailUiModel
import com.fluxa.app.shared.feature.detail.SourceSelectionScreen
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.PlayerScreen

@Composable
internal fun PlayerRoute(
    screen: Screen.Player,
    activeProfile: UserProfile?,
    profileManager: ProfileManager,
    homeViewModel: HomeViewModel,
    mainPlayer: ExoPlayer,
    navigator: AppNavigator,
    onBack: () -> Unit,
    onProfileChanged: (UserProfile) -> Unit
) {
    if (screen.showSourceSelection) {
        val streams = screen.initialStreams.map { stream ->
            DetailStreamUiModel(
                addonName = stream.addonName.orEmpty(),
                title = stream.title.orEmpty(),
                playableUrl = stream.playableUrl.orEmpty()
            )
        }.filter { it.playableUrl.isNotBlank() }
        SourceSelectionScreen(
            content = DetailUiModel(
                id = screen.meta.id,
                type = screen.meta.type,
                title = screen.meta.name,
                description = screen.meta.description.orEmpty(),
                posterUrl = screen.meta.poster,
                backgroundUrl = screen.meta.background,
                logoUrl = screen.meta.logo,
                releaseLabel = screen.meta.releaseInfo.orEmpty(),
                ratingLabel = screen.meta.imdbRating.orEmpty(),
                runtimeLabel = screen.meta.runtime,
                isInWatchlist = false,
                relatedItems = emptyList(),
                streams = streams
            ),
            language = activeProfile?.safeLanguage,
            onBack = onBack,
            onStreamSelected = { selected ->
                val index = screen.initialStreams.indexOfFirst {
                    it.playableUrl == selected.playableUrl
                }.coerceAtLeast(0)
                navigator.navigateTo(
                    screen.copy(
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
        screen.meta,
        screen.initialProgress,
        screen.videoId,
        onBack,
        homeViewModel,
        mainPlayer,
        activeProfile,
        {
            onProfileChanged(it)
            profileManager.saveProfile(it)
        },
        initialStreamIndex = screen.streamIndex,
        initialStreams = screen.initialStreams,
        lastStreamUrl = screen.lastStreamUrl,
        lastStreamTitle = screen.lastStreamTitle,
        initialBingeGroup = screen.preferredBingeGroup,
        returnToSourcesOnError = screen.returnToSourcesOnError,
        onSelectSource = { request ->
            navigator.navigateTo(
                Screen.Player(
                    meta = request.meta,
                    videoId = request.videoId,
                    initialProgress = request.progress,
                    streamIndex = request.streamIndex ?: 0,
                    initialStreams = request.streams,
                    lastStreamUrl = request.streamUrl,
                    lastStreamTitle = request.streamTitle,
                    showSourceSelection = true
                )
            )
        }
    )
}
