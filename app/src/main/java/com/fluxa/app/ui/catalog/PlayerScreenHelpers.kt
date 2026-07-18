@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.player.LibassDebugLog

import android.content.Context
import android.content.ContextWrapper
import android.app.Activity
import android.app.PendingIntent
import android.app.PictureInPictureParams
import android.app.RemoteAction
import android.content.BroadcastReceiver
import android.content.Intent
import android.content.IntentFilter
import android.graphics.drawable.Icon
import android.os.Build
import android.util.Rational
import android.view.LayoutInflater
import androidx.activity.compose.BackHandler
import androidx.core.content.ContextCompat
import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.graphics.*
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.key.*
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.ui.zIndex
import androidx.media3.common.Player
import androidx.media3.common.C
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.AspectRatioFrameLayout
import androidx.media3.ui.PlayerView
import androidx.media3.common.Metadata
import androidx.media3.extractor.metadata.id3.ChapterFrame
import coil3.compose.AsyncImage
import com.fluxa.app.R
import com.fluxa.app.player.*
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.player.MediaTrack
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.coroutines.isActive
import java.util.Locale

data class PlayerSourceSelectionRequest(
    val meta: Meta,
    val videoId: String?,
    val progress: Long,
    val streams: List<Stream>,
    val streamIndex: Int?,
    val streamUrl: String?,
    val streamTitle: String?,
    val downloadMode: Boolean = false
)

internal fun subtitleColorWithOpacity(argb: Int, opacity: Float): Int {
    return Color(argb).copy(alpha = opacity.coerceIn(0f, 1f)).toArgb()
}

internal tailrec fun Context.findActivity(): Activity? = when (this) {
    is Activity -> this
    is ContextWrapper -> baseContext.findActivity()
    else -> null
}

internal fun subtitleCaptionStyle(profile: UserProfile?): androidx.media3.ui.CaptionStyleCompat {
    val outlineOpacity = profile?.safeSubtitleOutlineOpacity ?: 0f
    val edgeType = if (outlineOpacity > 0f) {
        androidx.media3.ui.CaptionStyleCompat.EDGE_TYPE_OUTLINE
    } else {
        androidx.media3.ui.CaptionStyleCompat.EDGE_TYPE_NONE
    }
    return androidx.media3.ui.CaptionStyleCompat(
        subtitleColorWithOpacity(profile?.safeSubtitleColor ?: Color.White.toArgb(), profile?.safeSubtitleTextOpacity ?: 1f),
        subtitleColorWithOpacity(profile?.safeSubtitleBackgroundColor ?: Color.Black.toArgb(), profile?.safeSubtitleBackgroundOpacity ?: 0.75f),
        android.graphics.Color.TRANSPARENT,
        edgeType,
        subtitleColorWithOpacity(profile?.safeSubtitleOutlineColor ?: Color.Black.toArgb(), outlineOpacity),
        androidx.media3.ui.CaptionStyleCompat.DEFAULT.typeface
    )
}

internal fun AddonDescriptor.hasSubtitleResource(): Boolean {
    return manifest.hasStremioResource("subtitles")
}

internal fun SubtitleData.subtitleUrl(): String? {
    return url?.takeIf { it.isNotBlank() } ?: attributes.url.takeIf { it.isNotBlank() }
}

internal fun SubtitleData.subtitleLanguages(): List<String> {
    return attributes.languages.ifEmpty { listOfNotNull(lang) }
}

internal suspend fun fetchExternalSubtitleTracks(
    viewModel: HomeViewModel,
    addons: List<AddonDescriptor>,
    profile: UserProfile?,
    type: String,
    id: String,
    stream: Stream?
): List<ExternalSubtitleTrack> = withContext(Dispatchers.IO) {
    val subAddons = addons.filter { it.supportsStremioResource("subtitles", type, id) }
    val preferred = profile?.safePreferredSubtitleLanguage?.substringBefore('-')?.substringBefore('_')?.lowercase(Locale.ROOT)
    val secondary = profile?.safeSecondarySubtitleLanguage?.substringBefore('-')?.substringBefore('_')?.lowercase(Locale.ROOT)
    val inlineSubtitleSource = stream?.addonName.orEmpty()
    val inlineSubtitles = stream?.subtitles.orEmpty().map { inlineSubtitleSource to it }
    val subtitleExtra = stream?.subtitleExtraArgs().orEmpty()
    LibassDebugLog.d(
        "fetch external subtitles type=$type id=$id subtitleAddons=${subAddons.size} inline=${inlineSubtitles.size} subtitleExtra=${subtitleExtra.isNotBlank()}"
    )
    val subtitles = kotlinx.coroutines.supervisorScope {
        subAddons.map { addon ->
            async {
                withTimeoutOrNull(2500L) {
                    viewModel.getSubtitlesFromAddon(addon.transportUrl, type, id, subtitleExtra).map { addon.manifest.name to it }
                }.orEmpty()
            }
        }.awaitAll().flatten()
    }
    LibassDebugLog.d("external subtitle fetch completed addonSubtitles=${subtitles.size}")

    val result = (inlineSubtitles + subtitles)
        .mapNotNull { (addonName, subtitle) ->
            val url = subtitle.subtitleUrl() ?: return@mapNotNull null
            val language = subtitle.subtitleLanguages().firstOrNull()?.lowercase(Locale.ROOT)
            ExternalSubtitleTrack(
                url = url,
                language = language,
                label = listOfNotNull(language?.let { nativeLanguageName(it) }, addonName.takeIf { it.isNotBlank() })
                    .joinToString(" - ")
                    .ifBlank { null }
            )
        }
        .distinctBy { "${it.language}:${it.url}" }
        .sortedBy {
            val language = it.language?.substringBefore('-')?.substringBefore('_')?.lowercase(Locale.ROOT)
            when (language) {
                preferred -> 0
                secondary -> 1
                else -> 2
            }
        }
    LibassDebugLog.d(
        "external subtitle tracks ready count=${result.size} assLike=${result.count { track ->
            track.url.substringBefore('?').substringBefore('#').lowercase(Locale.ROOT).let { path ->
                path.endsWith(".ass") || path.endsWith(".ssa")
            }
        }} items=${result.take(6).joinToString { "${LibassDebugLog.urlSummary(it.url)} lang=${it.language} label=${it.label}" }}"
    )
    result
}

internal fun Stream.subtitleExtraArgs(): String {
    return FluxaCoreNative.streamPlaybackInfo(this).subtitleExtraArgs
}

internal fun playbackNotificationTitle(meta: Meta, videoId: String?): String {
    if (meta.type != "series" || videoId.isNullOrBlank()) return meta.name
    val parts = videoId.split(":")
    if (parts.size < 3) return meta.name
    val season = parts[parts.size - 2].toIntOrNull() ?: return meta.name
    val episode = parts[parts.size - 1].toIntOrNull() ?: return meta.name
    return "${meta.name} - S$season E$episode"
}

internal fun String?.isTorrentPlaybackUrl(): Boolean {
    return FluxaCoreNative.isTorrentPlaybackUrl(this)
}

internal fun Stream.matchesEpisode(videoId: String?): Boolean {
    return FluxaCoreNative.streamMatchesEpisode(
        videoId = videoId,
        title = title,
        name = name,
        description = description,
        filename = filename,
        effectiveFilename = effectiveFilename
    )
}
