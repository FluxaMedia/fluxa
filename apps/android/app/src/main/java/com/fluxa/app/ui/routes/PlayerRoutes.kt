package com.fluxa.app.ui.routes

import android.content.Intent
import android.provider.Settings
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
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
import com.fluxa.app.ui.catalog.AndroidExternalPlayerLauncher
import com.fluxa.app.ui.catalog.PlayerPipSuppression
import com.fluxa.app.ui.catalog.fetchExternalSubtitleTracks
import com.fluxa.app.ui.catalog.playbackNotificationTitle
import com.fluxa.app.ui.catalog.formatRuntimeLabel

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
    var externalLaunchFailed by remember(request) { mutableStateOf(false) }
    var externalLaunchStarted by remember(request) { mutableStateOf(false) }
    var externalTrackingPermissionPrompted by remember(request) { mutableStateOf(false) }
    val externalTrackingSettingsLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.StartActivityForResult(),
    ) {
        externalLaunchStarted = false
    }
    val persistentExternalResultLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.StartActivityForResult(),
    ) { result ->
        val returnedPosition = result.data?.getLongExtra("position", -1L)?.takeIf { it >= 0L }
            ?: result.data?.getLongExtra("position_ms", -1L)?.takeIf { it >= 0L }
        val returnedDuration = result.data?.getLongExtra("duration", -1L)?.takeIf { it > 0L }
            ?: result.data?.getLongExtra("duration_ms", -1L)?.takeIf { it > 0L }
        homeViewModel.finishExternalPlaybackTracking(returnedPosition, returnedDuration)
        PlayerPipSuppression.suppressAutoEnter = false
        onBack()
    }
    if (
        !request.showSourceSelection &&
        !externalLaunchFailed &&
        activeProfile?.preferredPlayer?.equals("external", ignoreCase = true) == true
    ) {
        val selectedStream = request.initialStreams.getOrNull(request.streamIndex)
            ?: request.initialStreams.firstOrNull { it.playableUrl == request.lastStreamUrl }
        val externalUrl = request.lastStreamUrl?.takeIf { it.isNotBlank() }
            ?: selectedStream?.playableUrl?.takeIf { it.isNotBlank() }
        if (externalUrl != null) {
            val context = LocalContext.current
            val externalTitle = request.lastStreamTitle ?: selectedStream?.title ?: request.meta.name
            LaunchedEffect(request, activeProfile.externalPlayerTarget, externalUrl, externalLaunchStarted) {
                if (externalLaunchStarted) return@LaunchedEffect
                externalLaunchStarted = true
                if (!homeViewModel.externalPlaybackMediaSessionAccessAvailable() && !externalTrackingPermissionPrompted) {
                    externalTrackingPermissionPrompted = true
                    val opened = runCatching {
                        externalTrackingSettingsLauncher.launch(Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS))
                        true
                    }.getOrDefault(false)
                    if (opened) return@LaunchedEffect
                }
                val resolved = homeViewModel.resolvePlayerPlayback(
                    url = externalUrl,
                    stream = selectedStream,
                    currentVideoId = request.videoId,
                    title = externalTitle,
                )
                val launchUrl = resolved.resolvedUrl?.takeIf { it.isNotBlank() }
                    ?: externalUrl.takeUnless { it.startsWith("stremio://torrent", ignoreCase = true) }
                if (launchUrl == null) {
                    externalLaunchStarted = false
                    externalLaunchFailed = true
                    return@LaunchedEffect
                }
                val subtitleId = request.videoId?.takeIf(String::isNotBlank) ?: request.meta.id
                val subtitles = fetchExternalSubtitleTracks(
                    viewModel = homeViewModel,
                    addons = homeViewModel.userAddons.value,
                    profile = activeProfile,
                    type = request.meta.type,
                    id = subtitleId,
                    stream = selectedStream,
                )
                val intent = AndroidExternalPlayerLauncher.createLaunchIntent(
                    context = context,
                    url = launchUrl,
                    title = playbackNotificationTitle(request.meta, request.videoId),
                    positionMs = request.initialProgress,
                    packageName = activeProfile.externalPlayerTarget,
                    headers = selectedStream?.resolveHeaders().orEmpty(),
                    subtitles = subtitles,
                )
                if (intent == null) {
                    externalLaunchStarted = false
                    externalLaunchFailed = true
                    return@LaunchedEffect
                }
                PlayerPipSuppression.suppressAutoEnter = true
                runCatching { persistentExternalResultLauncher.launch(intent) }
                    .onSuccess {
                        homeViewModel.startExternalPlaybackTracking(
                            meta = request.meta,
                            videoId = request.videoId,
                            initialPositionMs = request.initialProgress,
                            initialDurationMs = request.meta.duration ?: 0L,
                            streamIndex = request.streamIndex,
                            episodeName = playbackNotificationTitle(request.meta, request.videoId),
                            streamUrl = launchUrl,
                            streamTitle = externalTitle,
                            targetPackage = activeProfile.externalPlayerTarget,
                        )
                    }
                    .onFailure {
                        PlayerPipSuppression.suppressAutoEnter = false
                        externalLaunchStarted = false
                        externalLaunchFailed = true
                    }
            }
            return
        }
    }

    if (request.showSourceSelection) {
        val streams = request.initialStreams.map { stream ->
            DetailStreamUiModel(
                addonName = stream.addonName.orEmpty(),
                title = stream.title.orEmpty(),
                playableUrl = stream.playableUrl.orEmpty(),
                name = stream.name.orEmpty()
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
                runtimeLabel = formatRuntimeLabel(request.meta.runtime),
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
            },
            onRetry = {}
        )
        return
    }
    PlayerScreen(
        request.meta,
        request.initialProgress,
        request.initialProgressPercent,
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
