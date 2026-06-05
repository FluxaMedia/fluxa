package com.fluxa.app.ui.routes

import androidx.compose.runtime.Composable
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.ui.AppNavigator
import com.fluxa.app.ui.Screen
import com.fluxa.app.ui.catalog.DetailViewModel
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.PlayerScreen
import com.fluxa.app.ui.catalog.SourceSelectionScreen

@Composable
internal fun SourcesRoute(
    screen: Screen.Sources,
    activeProfile: UserProfile?,
    navigator: AppNavigator,
    onBack: () -> Unit
) {
    val sourceViewModel: DetailViewModel = androidx.hilt.navigation.compose.hiltViewModel()
    SourceSelectionScreen(
        meta = screen.meta,
        video = screen.video,
        videoId = screen.videoId,
        initialProgress = screen.initialProgress,
        lastStreamIndex = screen.lastStreamIndex,
        lastStreamUrl = screen.lastStreamUrl,
        lastStreamTitle = screen.lastStreamTitle,
        autoSelectSavedSource = screen.autoSelectSavedSource,
        downloadMode = screen.downloadMode,
        activeProfile = activeProfile,
        viewModel = sourceViewModel,
        onBack = onBack,
        onStreamSelected = { stream, visibleStreams, streamIndex, autoSelectedSavedSource ->
            navigator.navigateTo(
                Screen.Player(
                    meta = screen.meta,
                    videoId = screen.video?.id ?: screen.videoId,
                    initialProgress = screen.initialProgress,
                    streamIndex = streamIndex,
                    initialStreams = visibleStreams,
                    lastStreamUrl = stream.playableUrl,
                    lastStreamTitle = stream.title,
                    returnToSourcesOnError = autoSelectedSavedSource
                )
            )
        }
    )
}

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
                Screen.Sources(
                    meta = request.meta,
                    videoId = request.videoId,
                    initialProgress = request.progress,
                    lastStreamIndex = request.streamIndex,
                    lastStreamUrl = request.streamUrl,
                    lastStreamTitle = request.streamTitle,
                    autoSelectSavedSource = false,
                    downloadMode = request.downloadMode
                )
            )
        }
    )
}
