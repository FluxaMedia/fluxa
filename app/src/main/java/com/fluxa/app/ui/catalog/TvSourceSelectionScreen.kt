@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.Border
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.Glow
import androidx.tv.material3.Surface
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import coil3.compose.AsyncImage
import kotlinx.coroutines.launch

@Composable
internal fun TvSourceSelectionScreen(
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
    val context = LocalContext.current
    val vmState by viewModel.uiState.collectAsStateWithLifecycle()
    val detail = vmState.detail
    val streams = vmState.filteredStreams
    val isLoadingStreams = vmState.isLoadingStreams
    val availableAddons = vmState.availableAddons
    val selectedAddon = vmState.selectedAddon
    val userAddons = vmState.userAddons
    val scope = rememberCoroutineScope()
    val downloadManager = remember(context) { OfflineDownloadManager.getInstance(context) }
    var subtitleDialog by remember { mutableStateOf<Pair<Stream, List<OfflineSubtitleOption>>?>(null) }
    var loadingDownloadStream by remember { mutableStateOf<Stream?>(null) }
    val accent = Color(activeProfile?.safeAccentColorArgb ?: FluxaColors.accentArgb)
    val targetId = video?.id ?: videoId ?: meta.id
    val title = detail?.name?.takeIf { it.isNotBlank() } ?: meta.name
    val logoUrl = (detail?.logo ?: meta.logo)?.takeIf { it.isNotBlank() }
    var logoLoadFailed by remember(logoUrl) { mutableStateOf(false) }
    var hasFetchedStreams by remember(targetId, activeProfile?.id) { mutableStateOf(false) }
    val episodeLine = remember(video, videoId) {
        video?.let { ep ->
            buildString {
                append("S")
                append(ep.season ?: 1)
                append("E")
                append(ep.number ?: 0)
                ep.name?.takeIf { it.isNotBlank() }?.let { append(" - ").append(it) }
            }
        } ?: videoId?.takeIf { meta.type == "series" }
    }
    val heroImage = video?.thumbnail ?: if (video == null) detail?.background else null
    val lang = activeProfile?.safeLanguage ?: "en"
    var resumedSavedSource by remember(targetId, lastStreamUrl, lastStreamTitle, lastStreamIndex) { mutableStateOf(false) }

    fun startDownloadFlow(stream: Stream) {
        if (!stream.isOfflineDownloadable()) {
            android.widget.Toast.makeText(context, AppStrings.t(lang, "downloads.unsupported_source"), android.widget.Toast.LENGTH_SHORT).show()
            return
        }
        loadingDownloadStream = stream
        scope.launch {
            val options = fetchDownloadSubtitleOptions(
                viewModel = viewModel,
                addons = userAddons,
                profile = activeProfile,
                type = meta.type,
                id = targetId,
                stream = stream
            )
            loadingDownloadStream = null
            subtitleDialog = stream to options
        }
    }

    LaunchedEffect(meta.id, meta.type, activeProfile?.id) {
        viewModel.loadDetail(meta.type, meta.id, activeProfile)
    }

    LaunchedEffect(targetId, detail?.id, activeProfile?.id) {
        if (meta.type == "series" && targetId == meta.id) return@LaunchedEffect
        if (hasFetchedStreams) return@LaunchedEffect
        hasFetchedStreams = true
        viewModel.fetchStreamsForSelection(meta.type, targetId, context)
    }

    LaunchedEffect(streams, isLoadingStreams, lastStreamUrl, lastStreamTitle, lastStreamIndex, activeProfile?.safeStreamSourceSelectionMode, activeProfile?.safeStreamSourceRegexPattern, downloadMode) {
        if (resumedSavedSource || isLoadingStreams || streams.isEmpty()) return@LaunchedEffect
        if (downloadMode) return@LaunchedEffect
        if (meta.id.startsWith("cs3:") || targetId.startsWith("cs3:")) {
            resumedSavedSource = true
            onStreamSelected(streams[0], streams, 0, true)
            return@LaunchedEffect
        }
        val mode = activeProfile?.safeStreamSourceSelectionMode ?: STREAM_SOURCE_MODE_MANUAL
        val matchedIndex = if (mode == STREAM_SOURCE_MODE_MANUAL) {
            if (!autoSelectSavedSource) return@LaunchedEffect
            when {
                !lastStreamUrl.isNullOrBlank() -> streams.indexOfFirst { it.playableUrl == lastStreamUrl }
                !lastStreamTitle.isNullOrBlank() -> streams.indexOfFirst { it.title == lastStreamTitle }
                lastStreamIndex != null && lastStreamIndex in streams.indices -> lastStreamIndex
                else -> -1
            }
        } else {
            selectStreamIndex(
                streams = streams,
                currentVideoId = targetId,
                initialStreamIndex = lastStreamIndex ?: 0,
                savedUrl = null,
                savedTitle = null,
                sourceSelectionMode = mode,
                regexPattern = activeProfile?.safeStreamSourceRegexPattern,
                preferredBingeGroup = null
            )
        }
        if (matchedIndex >= 0) {
            resumedSavedSource = true
            onStreamSelected(streams[matchedIndex], streams, matchedIndex, true)
        }
    }

    Box(modifier = Modifier.fillMaxSize().background(Color.Black)) {
        if (!heroImage.isNullOrBlank()) {
            AsyncImage(
                model = heroImage,
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
        }
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Brush.verticalGradient(listOf(Color.Black.copy(alpha = 0.45f), Color.Black.copy(alpha = 0.78f), Color.Black)))
        )
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Brush.horizontalGradient(listOf(Color.Black.copy(alpha = 0.55f), Color.Transparent)))
        )

        IconButton(
            onClick = onBack,
            modifier = Modifier
                .padding(start = 32.dp, top = 32.dp)
                .size(44.dp)
                .clip(CircleShape)
                .background(Color.White.copy(alpha = 0.08f))
        ) {
            Icon(FluxaIcons.ArrowBack, null, tint = Color.White)
        }

        Row(modifier = Modifier.fillMaxSize().padding(start = 56.dp, end = 56.dp, top = 96.dp, bottom = 40.dp)) {
            Column(
                modifier = Modifier
                    .width(420.dp)
                    .fillMaxHeight()
                    .padding(end = 32.dp),
                verticalArrangement = Arrangement.Center
            ) {
                if (logoUrl != null && !logoLoadFailed) {
                    AsyncImage(
                        model = logoUrl,
                        contentDescription = title,
                        modifier = Modifier.heightIn(max = 96.dp).widthIn(max = 360.dp),
                        contentScale = ContentScale.Fit,
                        alignment = Alignment.CenterStart,
                        onError = { logoLoadFailed = true }
                    )
                } else {
                    Text(text = title, color = Color.White, fontSize = 30.sp, fontWeight = FontWeight.Black, maxLines = 3, overflow = TextOverflow.Ellipsis)
                }
                episodeLine?.let {
                    Spacer(Modifier.height(10.dp))
                    Text(text = it, color = Color.White.copy(alpha = 0.75f), fontSize = 16.sp, fontWeight = FontWeight.Bold)
                }
                val synopsis = video?.overview?.takeIf { it.isNotBlank() } ?: detail?.description.orEmpty()
                if (synopsis.isNotBlank()) {
                    Spacer(Modifier.height(14.dp))
                    Text(text = synopsis, color = Color.White.copy(alpha = 0.62f), fontSize = 14.sp, lineHeight = 20.sp, maxLines = 6, overflow = TextOverflow.Ellipsis)
                }
            }

            Column(modifier = Modifier.weight(1f).fillMaxHeight()) {
                LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp), contentPadding = PaddingValues(vertical = 8.dp)) {
                    item {
                        TvSourceChip(AppStrings.t(lang, "auto.all_24e2815b"), selectedAddon == null) { viewModel.setSelectedAddon(null) }
                    }
                    items(availableAddons, key = { it }) { addon ->
                        TvSourceChip(addon, selectedAddon == addon) { viewModel.setSelectedAddon(addon) }
                    }
                }

                Spacer(Modifier.height(8.dp))

                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .weight(1f)
                        .clip(RoundedCornerShape(20.dp))
                        .background(Color.Black.copy(alpha = 0.45f))
                        .padding(20.dp)
                ) {
                    when {
                        isLoadingStreams && streams.isEmpty() -> {
                            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                CircularProgressIndicator(color = accent)
                            }
                        }
                        !isLoadingStreams && streams.isEmpty() -> {
                            Text(
                                text = AppStrings.t(lang, "auto.no_sources_found_3019f12c"),
                                color = Color.White.copy(alpha = 0.58f),
                                fontSize = 14.sp,
                                modifier = Modifier.padding(vertical = 24.dp)
                            )
                        }
                        else -> {
                            LazyColumn(verticalArrangement = Arrangement.spacedBy(10.dp), contentPadding = PaddingValues(bottom = 8.dp)) {
                                itemsIndexed(streams, key = { i, s -> s.playableUrl ?: "${s.title.orEmpty()}$i" }) { index, stream ->
                                    TvSourceStreamRow(
                                        stream = stream,
                                        isLoading = loadingDownloadStream == stream,
                                        onClick = {
                                            if (!downloadMode) onStreamSelected(stream, streams, index, false) else startDownloadFlow(stream)
                                        }
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    subtitleDialog?.let { (stream, options) ->
        AlertDialog(
            onDismissRequest = { subtitleDialog = null },
            title = { Text(AppStrings.t(lang, "downloads.subtitles_title")) },
            text = {
                LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.heightIn(max = 320.dp)) {
                    item {
                        SourceSubtitleOptionRow(
                            label = AppStrings.t(lang, "downloads.no_subtitle"),
                            onClick = {
                                subtitleDialog = null
                                scope.launch {
                                    enqueueOfflineDownload(downloadManager, activeProfile, meta, video, videoId, stream, null, context)
                                    onBack()
                                }
                            }
                        )
                    }
                    items(options, key = { it.url }) { option ->
                        SourceSubtitleOptionRow(
                            label = option.label,
                            onClick = {
                                subtitleDialog = null
                                scope.launch {
                                    enqueueOfflineDownload(downloadManager, activeProfile, meta, video, videoId, stream, option, context)
                                    onBack()
                                }
                            }
                        )
                    }
                }
            },
            confirmButton = {},
            dismissButton = {
                TextButton(onClick = { subtitleDialog = null }) {
                    Text(AppStrings.t(lang, "common.cancel"))
                }
            }
        )
    }
}

@Composable
private fun TvSourceChip(label: String, selected: Boolean, onClick: () -> Unit) {
    Surface(
        onClick = onClick,
        modifier = Modifier.height(36.dp),
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(18.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = if (selected) Color.White else Color.White.copy(alpha = 0.08f),
            contentColor = if (selected) Color.Black else Color.White,
            focusedContainerColor = if (selected) Color.White else Color.White.copy(alpha = 0.08f),
            focusedContentColor = if (selected) Color.Black else Color.White
        ),
        border = ClickableSurfaceDefaults.border(
            border = Border(androidx.compose.foundation.BorderStroke(1.dp, Color.White.copy(alpha = 0.12f))),
            focusedBorder = Border(androidx.compose.foundation.BorderStroke(2.dp, Color.White))
        ),
        glow = ClickableSurfaceDefaults.glow(glow = Glow.None, focusedGlow = Glow.None, pressedGlow = Glow.None),
        scale = ClickableSurfaceDefaults.scale(focusedScale = 1f)
    ) {
        Box(modifier = Modifier.fillMaxHeight().padding(horizontal = 18.dp), contentAlignment = Alignment.Center) {
            Text(label, color = if (selected) Color.Black else Color.White, fontSize = 12.sp, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

@Composable
private fun TvSourceStreamRow(stream: Stream, isLoading: Boolean, onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    val sourceName = stream.streamSourceHeader()
    val rawTitle = stream.streamRawBody()
    Surface(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth().onFocusChanged { focused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(12.dp)),
        colors = ClickableSurfaceDefaults.colors(containerColor = if (focused) Color.White.copy(alpha = 0.14f) else Color.White.copy(alpha = 0.04f)),
        border = ClickableSurfaceDefaults.border(
            focusedBorder = Border(androidx.compose.foundation.BorderStroke(2.dp, Color.White)),
            border = Border(androidx.compose.foundation.BorderStroke(1.dp, Color.White.copy(alpha = 0.1f)))
        ),
        glow = ClickableSurfaceDefaults.glow(glow = Glow.None, focusedGlow = Glow.None, pressedGlow = Glow.None),
        scale = ClickableSurfaceDefaults.scale(focusedScale = 1f)
    ) {
        Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 18.dp, vertical = 16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text(sourceName, color = Color.White, fontSize = 14.sp, lineHeight = 17.sp, fontWeight = FontWeight.Black, maxLines = 2, overflow = TextOverflow.Ellipsis)
            rawTitle?.let {
                AddonStreamBodyText(text = it, bodyMaxLines = 4)
            }
            if (isLoading) {
                CircularProgressIndicator(modifier = Modifier.size(18.dp), color = Color.White, strokeWidth = 2.dp)
            }
        }
    }
}
