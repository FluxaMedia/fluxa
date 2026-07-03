package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video

import androidx.compose.runtime.Composable

@Composable
fun SourceSelectionScreen(
    meta: Meta,
    video: Video?,
    videoId: String?,
    initialProgress: Long,
    lastStreamIndex: Int? = null,
    lastStreamUrl: String? = null,
    lastStreamTitle: String? = null,
    autoSelectSavedSource: Boolean = true,
    downloadMode: Boolean = false,
    activeProfile: UserProfile?,
    viewModel: DetailViewModel,
    onBack: () -> Unit,
    onStreamSelected: (Stream, List<Stream>, Int, Boolean) -> Unit
) {
    if (LocalDeviceType.current == DeviceType.Mobile) {
        MobileSourceSelectionScreen(
            meta, video, videoId, initialProgress, lastStreamIndex, lastStreamUrl, lastStreamTitle,
            autoSelectSavedSource, downloadMode, activeProfile, viewModel, onBack, onStreamSelected
        )
    } else {
        TvSourceSelectionScreen(
            meta, video, videoId, initialProgress, lastStreamIndex, lastStreamUrl, lastStreamTitle,
            autoSelectSavedSource, downloadMode, activeProfile, viewModel, onBack, onStreamSelected
        )
    }
}
