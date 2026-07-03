@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.foundation.ExperimentalFoundationApi::class, androidx.compose.ui.ExperimentalComposeUiApi::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Intent
import android.graphics.drawable.BitmapDrawable
import android.net.Uri
import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerDefaults
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.runtime.*
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.*
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.key.*
import androidx.compose.ui.layout.*
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.PlatformTextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import androidx.compose.ui.viewinterop.AndroidView
import androidx.palette.graphics.Palette
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items as mobileGridItems
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import coil3.imageLoader
import coil3.request.ImageRequest
import coil3.size.Size
import androidx.compose.ui.res.painterResource
import com.fluxa.app.R
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.distinctUntilChanged
import androidx.compose.runtime.snapshotFlow
import java.text.SimpleDateFormat
import java.util.*
import kotlin.math.abs

internal fun formatRemainingTime(ms: Long, lang: String? = "en"): String {
    val totalSeconds = ms / 1000
    val totalMinutes = totalSeconds / 60
    val hours = totalMinutes / 60
    val minutes = totalMinutes % 60
    return when {
        hours > 0 -> AppStrings.format(lang, "format.remaining_hours", hours, minutes)
        minutes > 0 -> AppStrings.format(lang, "format.remaining_minutes", minutes)
        else -> AppStrings.t(lang, "format.remaining_almost_done")
    }
}

internal fun formatElapsedTime(ms: Long, lang: String? = "en"): String {
    val totalSeconds = ms / 1000
    val totalMinutes = totalSeconds / 60
    val hours = totalMinutes / 60
    val minutes = totalMinutes % 60
    return when {
        hours > 0 -> AppStrings.format(lang, "format.elapsed_hours_minutes", hours, minutes)
        minutes > 0 -> AppStrings.format(lang, "format.elapsed_minutes", minutes)
        else -> AppStrings.t(lang, "format.elapsed_just_started")
    }
}

internal fun parseShortVideoId(id: String?): String? {
    if (id.isNullOrBlank() || id.startsWith("cs3:")) return null
    val parts = id.split(":")
    if (parts.size >= 3) {
        val season = parts[parts.size - 2].toIntOrNull()
        val episode = parts[parts.size - 1].toIntOrNull()
        if (season != null && episode != null) {
            return "S$season, E$episode"
        }
    }
    return null
}

fun continueWatchingEpisodeLabel(meta: Meta): String? {
    val code = parseShortVideoId(meta.lastVideoId)
    val title = continueWatchingEpisodeTitle(meta)
    return when {
        code != null && title != null && !title.startsWith(code, ignoreCase = true) -> "$code: $title"
        code != null && title != null -> title
        code != null -> code
        else -> title
    }
}

private val EPISODE_TITLE_PREFIX_REGEX = Regex("""(?i)^S\s*\d+\s*[,: \-]*E\s*\d+\s*[:\-]?\s*""")

fun continueWatchingEpisodeTitle(meta: Meta): String? {
    return meta.lastEpisodeName
        ?.trim()
        ?.replace(EPISODE_TITLE_PREFIX_REGEX, "")
        ?.trim()
        ?.takeIf { it.isNotBlank() }
}

fun continueWatchingEpisodeProgressLabel(meta: Meta, lang: String? = "en"): String? {
    val code = parseShortVideoId(meta.lastVideoId) ?: return null
    val duration = meta.duration ?: return code
    val remainingMs = (duration - (meta.timeOffset ?: 0L)).coerceAtLeast(0L)
    return AppStrings.format(lang, "format.continue_watching_episode_progress", code, formatRemainingTime(remainingMs, lang))
}

fun Meta.isUpNextContinueItem(): Boolean {
    val isSeries = type == "series" || type == "tv" || type == "anime"
    return isSeries &&
        !lastVideoId.isNullOrBlank() &&
        (timeOffset ?: 0L) <= 0L &&
        (duration ?: 0L) <= 0L
}
