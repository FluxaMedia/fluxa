@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.foundation.ExperimentalFoundationApi::class, androidx.compose.ui.ExperimentalComposeUiApi::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

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

@Composable
internal fun MobileLibraryListItem(
    movie: Meta,
    lang: String,
    onClick: () -> Unit,
    onForgetProgress: () -> Unit
) {
    val context = LocalContext.current
    val timeOffset = movie.timeOffset ?: 0L
    val duration = movie.duration ?: 0L
    val isUpNext = movie.isUpNextContinueItem()
    val posterRequest = remember(movie.poster) {
        ImageRequest.Builder(context)
            .data(movie.poster)
            .memoryCacheKey("lib-list:${movie.poster}")
            .diskCacheKey(movie.poster)
            .size(224, 396)
            .build()
    }
    val progress = if (duration > 0L) (timeOffset.toFloat() / duration.toFloat()).coerceIn(0f, 1f) else 0f
    val remainingMs = (duration - timeOffset).coerceAtLeast(0L)
    val watchedMs = timeOffset.coerceAtLeast(0L)
    val metaLine = remember(movie.id, movie.type, movie.releaseInfo, movie.lastVideoId, movie.lastEpisodeName, lang) {
        val episodeInfo = continueWatchingEpisodeLabel(movie)
        buildList {
            movie.releaseInfo?.takeIf { it.isNotBlank() }?.let { add(it) }
            add(if (movie.type == "movie") AppStrings.t(lang, "auto.movie") else AppStrings.t(lang, "auto.series"))
            episodeInfo?.takeIf { it.isNotBlank() }?.let { add(it) }
        }.joinToString("  ")
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(FluxaDimensions.LibraryListItem.height)
            .clip(RoundedCornerShape(FluxaDimensions.LibraryListItem.rowCornerRadius))
            .background(Color.White.copy(alpha = FluxaDimensions.Alpha.emptyCardBackground))
            .border(1.dp, Color.White.copy(alpha = FluxaDimensions.Alpha.subtleBorder), RoundedCornerShape(FluxaDimensions.LibraryListItem.rowCornerRadius))
            .clickable { onClick() }
            .padding(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        AsyncImage(
            model = posterRequest,
            contentDescription = null,
            modifier = Modifier
                .width(FluxaDimensions.LibraryListItem.thumbnailWidth)
                .fillMaxHeight()
                .clip(RoundedCornerShape(FluxaDimensions.LibraryListItem.thumbnailCornerRadius)),
            contentScale = ContentScale.Crop
        )

        Spacer(Modifier.width(14.dp))

        Column(
            modifier = Modifier
                .weight(1f)
                .fillMaxHeight()
        ) {
            Text(
                text = movie.name,
                color = Color.White,
                fontSize = 16.sp,
                fontWeight = FontWeight.Black,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )

            if (metaLine.isNotBlank()) {
                Spacer(Modifier.height(4.dp))
                Text(
                    text = metaLine,
                    color = Color.White.copy(alpha = 0.56f),
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
            }

            Spacer(Modifier.height(10.dp))

            if (!isUpNext) {
                LinearProgressIndicator(
                    progress = { progress },
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(5.dp)
                        .clip(RoundedCornerShape(999.dp)),
                    color = Color.White,
                    trackColor = Color.White.copy(alpha = FluxaDimensions.Alpha.progressBarTrack)
                )
            }

            Spacer(Modifier.height(8.dp))

            Text(
                text = if (isUpNext) {
                    AppStrings.t(lang, "auto.up_next")
                } else {
                    AppStrings.format(
                        lang,
                        "format.watch_progress",
                        formatElapsedTime(watchedMs, lang),
                        formatRemainingTime(remainingMs, lang)
                    )
                },
                color = Color.White.copy(alpha = 0.76f),
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )

            movie.description?.takeIf { it.isNotBlank() }?.let { summary ->
                Spacer(Modifier.height(6.dp))
                Text(
                    text = summary,
                    color = Color.White.copy(alpha = 0.48f),
                    fontSize = 12.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }

        Column(
            modifier = Modifier.fillMaxHeight(),
            verticalArrangement = Arrangement.SpaceBetween,
            horizontalAlignment = Alignment.End
        ) {
            IconButton(onClick = onForgetProgress) {
                Icon(
                    imageVector = FluxaIcons.DeleteOutline,
                    contentDescription = null,
                    tint = Color.White.copy(alpha = 0.75f)
                )
            }
            Icon(
                FluxaIcons.KeyboardArrowRight,
                null,
                tint = Color.White.copy(alpha = 0.22f)
            )
        }
    }
}
