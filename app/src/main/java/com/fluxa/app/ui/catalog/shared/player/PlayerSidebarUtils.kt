@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.Crossfade
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.*
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import coil3.compose.AsyncImage
import com.fluxa.app.player.MediaTrack
import java.util.Locale

internal fun nativeLanguageName(code: String): String {
    val normalized = code.lowercase(Locale.ROOT)
    val locale = Locale.forLanguageTag(normalized)
    val native = locale.getDisplayLanguage(locale).trim()
    return native.takeIf { it.isNotBlank() }?.replaceFirstChar { ch -> ch.titlecase(locale) } ?: code
}

internal fun Meta.withCurrentEpisodeArtwork(artwork: String?): Meta {
    val episodeArtwork = artwork?.takeIf { it.isNotBlank() } ?: return this
    if (type != "series") return this
    return copy(
        continueWatchingPoster = episodeArtwork,
        continueWatchingBackground = episodeArtwork
    )
}

@Composable
fun SeekFeedback(direction: Int, seekMs: Long) {
    val seconds = (seekMs / 1000L).coerceAtLeast(1L)
    val transition = rememberInfiniteTransition(label = "seek")
    val chevronShift by transition.animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(620, easing = FastOutSlowInEasing),
            repeatMode = RepeatMode.Restart
        ), label = "chevronShift"
    )

    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Canvas(modifier = Modifier.size(120.dp, 82.dp)) {
            val sign = if (direction > 0) 1f else -1f
            val stroke = size.minDimension * 0.075f
            val centerY = size.height / 2f
            val baseX = size.width / 2f - sign * size.width * 0.14f
            repeat(3) { index ->
                val phase = ((chevronShift + index * 0.28f) % 1f)
                val alpha = 0.22f + phase * 0.58f
                val x = baseX + sign * (index * size.width * 0.12f + phase * size.width * 0.05f)
                drawLine(
                    color = Color.White.copy(alpha = alpha),
                    start = Offset(x - sign * size.width * 0.06f, centerY - size.height * 0.12f),
                    end = Offset(x + sign * size.width * 0.06f, centerY),
                    strokeWidth = stroke,
                    cap = StrokeCap.Round
                )
                drawLine(
                    color = Color.White.copy(alpha = alpha),
                    start = Offset(x + sign * size.width * 0.06f, centerY),
                    end = Offset(x - sign * size.width * 0.06f, centerY + size.height * 0.12f),
                    strokeWidth = stroke,
                    cap = StrokeCap.Round
                )
            }
        }
        Text(
            text = if (direction > 0) "+$seconds" else "-$seconds",
            color = Color.White,
            fontWeight = FontWeight.Black,
            fontSize = 22.sp,
            style = androidx.compose.ui.text.TextStyle(
                shadow = Shadow(color = Color.Black.copy(alpha = 0.55f), offset = Offset(2f, 4f), blurRadius = 8f)
            )
        )
    }
}

internal fun formatTime(ms: Long): String {
    val totalSec = ms / 1000
    val hr = totalSec / 3600
    val min = (totalSec % 3600) / 60
    val sec = totalSec % 60
    return if (hr > 0) {
        String.format(java.util.Locale.US, "%02d:%02d:%02d", hr, min, sec)
    } else {
        String.format(java.util.Locale.US, "%02d:%02d", min, sec)
    }
}
