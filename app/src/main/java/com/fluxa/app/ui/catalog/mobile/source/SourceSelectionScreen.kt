package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
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
import androidx.compose.material3.TextButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.withContext
import java.text.SimpleDateFormat
import java.util.Locale

@Composable
fun MobileSourceSelectionScreen(
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
    val loadingAddonNames = vmState.loadingAddonNames
    val hasStreamProviders = vmState.hasStreamProviders
    val selectedAddon = vmState.selectedAddon
    val userAddons = vmState.userAddons
    val scope = rememberCoroutineScope()
    val downloadManager = remember(context) { OfflineDownloadManager.getInstance(context) }
    var subtitleDialog by remember { androidx.compose.runtime.mutableStateOf<Pair<Stream, List<OfflineSubtitleOption>>?>(null) }
    var downloadActionStream by remember { androidx.compose.runtime.mutableStateOf<Stream?>(null) }
    var loadingDownloadStream by remember { androidx.compose.runtime.mutableStateOf<Stream?>(null) }
    val accent = Color(activeProfile?.safeAccentColorArgb ?: FluxaColors.accentArgb)
    val targetId = video?.id ?: videoId ?: meta.id
    val title = detail?.name?.takeIf { it.isNotBlank() } ?: meta.name
    var logoLoadFailed by remember(targetId) { androidx.compose.runtime.mutableStateOf(false) }
    var hasFetchedStreams by remember(targetId, activeProfile?.id) { androidx.compose.runtime.mutableStateOf(false) }
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
    val logoUrl = (detail?.logo ?: meta.logo)?.takeIf { it.isNotBlank() }
    val lang = activeProfile?.safeLanguage ?: "en"
    val episodeInfoLine = remember(video, detail?.runtime, lang) {
        listOfNotNull(
            video?.episodeRuntime?.takeIf { it > 0 }?.let { "$it min" } ?: detail?.runtime,
            formatSourceEpisodeDate(video?.released, lang)
        ).joinToString(" - ").takeIf { it.isNotBlank() }
    }
    val episodeSynopsis = video?.overview?.takeIf { it.isNotBlank() } ?: detail?.description.orEmpty()
    var resumedSavedSource by remember(targetId, lastStreamUrl, lastStreamTitle, lastStreamIndex) { androidx.compose.runtime.mutableStateOf(false) }
    fun startDownloadFlow(stream: Stream) {
        if (!stream.isOfflineDownloadable()) {
            android.widget.Toast.makeText(context, AppStrings.t(activeProfile?.safeLanguage, "downloads.unsupported_source"), android.widget.Toast.LENGTH_SHORT).show()
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

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFF090909))
    ) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = 28.dp)
        ) {
            item {
                Column {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(300.dp)
                    ) {
                        if (!heroImage.isNullOrBlank()) {
                            AsyncImage(
                                model = heroImage,
                                contentDescription = title,
                                modifier = Modifier.fillMaxSize(),
                                contentScale = ContentScale.Crop
                            )
                        }
                        Box(
                            modifier = Modifier
                                .fillMaxSize()
                                .background(
                                    Brush.verticalGradient(
                                        listOf(
                                            Color.Black.copy(alpha = 0.10f),
                                            Color.Black.copy(alpha = 0.18f),
                                            Color(0xFF090909)
                                        )
                                    )
                                )
                        )
                        IconButton(
                            onClick = onBack,
                            modifier = Modifier
                                .padding(start = 4.dp, top = 8.dp)
                                .size(42.dp)
                                .clip(CircleShape)
                                .background(Color.Black.copy(alpha = 0.24f))
                        ) {
                            Icon(FluxaIcons.ArrowBack, AppStrings.t(lang, "common.back"), tint = Color.White)
                        }
                        if (logoUrl != null && !logoLoadFailed) {
                            AsyncImage(
                                model = logoUrl,
                                contentDescription = null,
                                modifier = Modifier
                                    .align(Alignment.BottomCenter)
                                    .padding(horizontal = 28.dp, vertical = 18.dp)
                                    .height(56.dp)
                                    .widthIn(max = 240.dp),
                                contentScale = ContentScale.Fit,
                                onError = { logoLoadFailed = true }
                            )
                        } else {
                            Text(
                                text = title,
                                color = Color.White,
                                fontSize = 24.sp,
                                lineHeight = 27.sp,
                                fontWeight = FontWeight.Black,
                                maxLines = 2,
                                overflow = TextOverflow.Ellipsis,
                                modifier = Modifier
                                    .align(Alignment.BottomCenter)
                                    .padding(horizontal = 20.dp, vertical = 20.dp)
                            )
                        }
                    }
                    Column(
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 14.dp),
                        verticalArrangement = Arrangement.spacedBy(7.dp)
                    ) {
                        episodeInfoLine?.let {
                            Text(
                                text = it,
                                color = Color.White.copy(alpha = 0.70f),
                                fontSize = 12.sp,
                                fontWeight = FontWeight.SemiBold,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                        episodeLine?.let {
                            Text(
                                text = it,
                                color = Color.White,
                                fontSize = 18.sp,
                                lineHeight = 22.sp,
                                fontWeight = FontWeight.Black,
                                maxLines = 2,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                        if (episodeSynopsis.isNotBlank()) {
                            Text(
                                text = episodeSynopsis,
                                color = Color.White.copy(alpha = 0.78f),
                                fontSize = 13.sp,
                                lineHeight = 19.sp,
                                maxLines = 5,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                    }
                }
            }

            item {
                LazyRow(
                    contentPadding = PaddingValues(horizontal = 8.dp, vertical = 10.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    item {
                        SourceChip(
                            modifier = Modifier.animateItem(),
                            label = AppStrings.t(activeProfile?.safeLanguage ?: "en", "auto.all_24e2815b"),
                            selected = selectedAddon == null,
                            accent = accent,
                            onClick = { viewModel.setSelectedAddon(null) }
                        )
                    }
                    items(availableAddons, key = { it }) { addon ->
                        SourceChip(
                            modifier = Modifier.animateItem(),
                            label = addon,
                            selected = selectedAddon == addon,
                            accent = accent,
                            onClick = { viewModel.setSelectedAddon(addon) }
                        )
                    }
                }
            }

            if (loadingAddonNames.isNotEmpty()) {
                item {
                    Column(
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 2.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        loadingAddonNames.take(4).forEach { addonName ->
                            Text(
                                text = AppStrings.format(activeProfile?.safeLanguage ?: "en", "sources.addon_still_loading", addonName),
                                color = Color.White.copy(alpha = 0.48f),
                                fontSize = 11.sp,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                    }
                }
            }

            item {
                Text(
                    text = selectedAddon ?: AppStrings.t(activeProfile?.safeLanguage ?: "en", "auto.sources"),
                    color = Color.White,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Black,
                    modifier = Modifier.padding(start = 16.dp, top = 6.dp, bottom = 6.dp)
                )
            }

            if (isLoadingStreams && streams.isEmpty()) {
                item {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 34.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        CircularProgressIndicator(color = accent)
                    }
                }
            }

            if (!isLoadingStreams && streams.isEmpty()) {
                item {
                    Text(
                        text = AppStrings.t(activeProfile?.safeLanguage ?: "en", "auto.no_sources_found_3019f12c"),
                        color = Color.White.copy(alpha = 0.58f),
                        fontSize = 14.sp,
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 24.dp)
                    )
                }
            }

            itemsIndexed(streams, key = { i, s -> "$i:${s.playableUrl ?: s.title.orEmpty()}" }) { index, stream ->
                SourcePageStreamRow(
                    modifier = Modifier.animateItem(),
                    stream = stream,
                    isLoading = loadingDownloadStream == stream,
                    onClick = {
                        if (!downloadMode) {
                            onStreamSelected(stream, streams, index, false)
                        } else {
                            startDownloadFlow(stream)
                        }
                    },
                    onLongClick = {
                        if (!downloadMode) {
                            downloadActionStream = stream
                        }
                    }
                )
            }
        }
        AnimatedVisibility(
            visible = downloadActionStream != null,
            enter = fadeIn(tween(FluxaDimensions.AnimDuration.quick)) + slideInVertically(tween(FluxaDimensions.AnimDuration.contentExpand, easing = FastOutSlowInEasing)) { it / 4 },
            exit = fadeOut(tween(FluxaDimensions.AnimDuration.heightAnim)) + slideOutVertically(tween(FluxaDimensions.AnimDuration.scaleAlpha, easing = FastOutSlowInEasing)) { it / 4 }
        ) {
            downloadActionStream?.let { stream ->
                MobileSourceDownloadActionSheet(
                    stream = stream,
                    lang = activeProfile?.safeLanguage ?: "en",
                    onDismiss = { downloadActionStream = null },
                    onDownload = {
                        downloadActionStream = null
                        startDownloadFlow(stream)
                    }
                )
            }
        }
        subtitleDialog?.let { (stream, options) ->
            AlertDialog(
                onDismissRequest = { subtitleDialog = null },
                title = { Text(AppStrings.t(activeProfile?.safeLanguage, "downloads.subtitles_title")) },
                text = {
                    LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.heightIn(max = 320.dp)) {
                        item {
                            SourceSubtitleOptionRow(
                                label = AppStrings.t(activeProfile?.safeLanguage, "downloads.no_subtitle"),
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
                        Text(AppStrings.t(activeProfile?.safeLanguage, "common.cancel"))
                    }
                }
            )
        }
    }
}

@Composable
private fun SourceChip(
    label: String,
    selected: Boolean,
    accent: Color,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    val background by animateColorAsState(
        targetValue = if (selected) accent else Color.White.copy(alpha = 0.08f),
        animationSpec = tween(FluxaDimensions.AnimDuration.scaleAlpha, easing = FastOutSlowInEasing),
        label = "sourceChipBackground"
    )
    val scale by animateFloatAsState(
        targetValue = if (selected) 1.04f else 1f,
        animationSpec = tween(FluxaDimensions.AnimDuration.scaleAlpha, easing = FastOutSlowInEasing),
        label = "sourceChipScale"
    )
    Box(
        modifier = modifier
            .height(34.dp)
            .graphicsLayer {
                scaleX = scale
                scaleY = scale
            }
            .clip(RoundedCornerShape(18.dp))
            .background(background)
            .clickable { onClick() }
            .padding(horizontal = 18.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = label,
            color = if (selected) Color.Black else Color.White,
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
    }
}

@Composable
private fun SourcePageStreamRow(
    modifier: Modifier = Modifier,
    stream: Stream,
    isLoading: Boolean = false,
    onClick: () -> Unit,
    onLongClick: () -> Unit
) {
    val sourceName = stream.streamSourceHeader()
    val rawTitle = stream.streamRawBody()
    val rowAlpha by animateFloatAsState(
        targetValue = if (isLoading) 0.72f else 1f,
        animationSpec = tween(FluxaDimensions.AnimDuration.scaleAlpha),
        label = "sourceRowAlpha"
    )
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 8.dp, vertical = 5.dp)
            .graphicsLayer { alpha = rowAlpha }
            .animateContentSize(animationSpec = tween(FluxaDimensions.AnimDuration.contentExpand, easing = FastOutSlowInEasing))
            .clip(RoundedCornerShape(8.dp))
            .background(Color(0xFF2B2A34))
            .border(1.dp, Color.White.copy(alpha = 0.05f), RoundedCornerShape(8.dp))
            .pointerInput(stream) {
                detectTapGestures(
                    onTap = { onClick() },
                    onLongPress = { onLongClick() }
                )
            }
            .padding(horizontal = 14.dp, vertical = 13.dp),
        verticalArrangement = Arrangement.spacedBy(7.dp)
    ) {
        Text(
            text = sourceName,
            color = Color.White,
            fontSize = 13.sp,
            lineHeight = 15.sp,
            fontWeight = FontWeight.Black,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis
        )
        rawTitle?.let {
            AddonStreamBodyText(
                text = it,
                bodyMaxLines = 6
            )
        }
        if (isLoading) {
            CircularProgressIndicator(modifier = Modifier.size(18.dp), color = Color.White, strokeWidth = 2.dp)
        }
    }
}

@Composable
private fun MobileSourceDownloadActionSheet(
    stream: Stream,
    lang: String,
    onDismiss: () -> Unit,
    onDownload: () -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.48f))
            .clickable { onDismiss() },
        contentAlignment = Alignment.BottomCenter
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp))
                .background(FluxaColors.surfaceDark)
                .pointerInput(stream) {
                    detectTapGestures(onTap = {})
                }
                .padding(horizontal = 18.dp, vertical = 18.dp)
                .navigationBarsPadding(),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                text = stream.streamSourceHeader(),
                color = Color.White,
                fontSize = 17.sp,
                lineHeight = 21.sp,
                fontWeight = FontWeight.Black,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )
            stream.streamRawBody()?.let { body ->
                AddonStreamBodyText(
                    text = body,
                    bodyMaxLines = 5
                )
            }
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(48.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(Color.White.copy(alpha = 0.10f))
                    .clickable { onDownload() },
                contentAlignment = Alignment.Center
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    Icon(FluxaIcons.Download, null, tint = Color.White, modifier = Modifier.size(20.dp))
                    Text(
                        text = AppStrings.t(lang, "downloads.download"),
                        color = Color.White,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Bold
                    )
                }
            }
        }
    }
}

@Composable
internal fun SourceSubtitleOptionRow(label: String, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(Color.White.copy(alpha = 0.08f))
            .clickable { onClick() }
            .padding(horizontal = 12.dp, vertical = 12.dp)
    ) {
        Text(label, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.Bold)
    }
}

internal suspend fun fetchDownloadSubtitleOptions(
    viewModel: DetailViewModel,
    addons: List<AddonDescriptor>,
    profile: UserProfile?,
    type: String,
    id: String,
    stream: Stream
): List<OfflineSubtitleOption> {
    val inline = stream.subtitles.orEmpty().mapNotNull { subtitle ->
        val url = subtitle.subtitleUrl() ?: return@mapNotNull null
        val lang = subtitle.subtitleLanguages().firstOrNull()?.lowercase(Locale.ROOT)
        OfflineSubtitleOption(
            label = listOfNotNull(lang?.let { nativeLanguageName(it) }, stream.addonName).joinToString(" - ").ifBlank { url },
            language = lang,
            url = url
        )
    }
    val remote = withContext(Dispatchers.IO) {
        supervisorScope {
            addons.filter { it.supportsStremioResource("subtitles", type, id) }.map { addon ->
                async {
                    viewModel.getSubtitlesFromAddon(addon.transportUrl, type, id, stream.subtitleExtraArgs()).mapNotNull { subtitle ->
                        val url = subtitle.subtitleUrl() ?: return@mapNotNull null
                        val lang = subtitle.subtitleLanguages().firstOrNull()?.lowercase(Locale.ROOT)
                        OfflineSubtitleOption(
                            label = listOfNotNull(lang?.let { nativeLanguageName(it) }, addon.manifest.name).joinToString(" - ").ifBlank { url },
                            language = lang,
                            url = url
                        )
                    }
                }
            }.let { deferred -> deferred.map { it.await() }.flatten() }
        }
    }
    val preferred = profile?.safePreferredSubtitleLanguage?.substringBefore('-')?.substringBefore('_')?.lowercase(Locale.ROOT)
    return (inline + remote).distinctBy { "${it.language}:${it.url}" }.sortedBy {
        when (it.language?.substringBefore('-')?.substringBefore('_')?.lowercase(Locale.ROOT)) {
            preferred -> 0
            else -> 1
        }
    }
}

internal suspend fun enqueueOfflineDownload(
    downloadManager: OfflineDownloadManager,
    profile: UserProfile?,
    meta: Meta,
    video: Video?,
    videoId: String?,
    stream: Stream,
    subtitle: OfflineSubtitleOption?,
    context: android.content.Context
) {
    val result = downloadManager.enqueue(
        profileId = profile?.id,
        meta = meta,
        video = video,
        videoId = videoId,
        stream = stream,
        subtitle = subtitle,
        profileLanguage = profile?.safeLanguage
    )
    android.widget.Toast.makeText(
        context,
        AppStrings.t(profile?.safeLanguage, if (result.isSuccess) "downloads.queued" else "downloads.failed"),
        android.widget.Toast.LENGTH_SHORT
    ).show()
}

internal fun Stream.isOfflineDownloadable(): Boolean {
    val url = playableUrl.orEmpty()
    if (!url.startsWith("http://") && !url.startsWith("https://")) return false
    val normalized = url.lowercase(Locale.ROOT)
    val path = normalized.substringBefore('?')
    if (listOf(".srt", ".vtt", ".ass", ".ssa", ".ttml", ".sub").any { path.endsWith(it) }) return false
    if (listOf(".m3u8", ".mpd").any { path.endsWith(it) }) return false
    val text = listOfNotNull(name, title, description, addonName).joinToString(" ").lowercase(Locale.ROOT)
    if (text.contains("opensubtitles") || text.contains("subtitle") || text.contains("altyazi") || text.contains("altyazı")) return false
    return true
}

internal fun formatSourceEpisodeDate(value: String?, lang: String): String? {
    val trimmed = value?.trim().orEmpty()
    if (trimmed.length < 10) return null
    val date = trimmed.take(10)
    if (date[4] != '-' || date[7] != '-') return null
    return runCatching {
        val parsed = SimpleDateFormat("yyyy-MM-dd", Locale.US).parse(date)
        parsed?.let { SimpleDateFormat("d MMMM yyyy", AppStrings.locale(lang)).format(it) }
    }.getOrNull()
}
