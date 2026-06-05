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
import coil3.request.crossfade
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

private val CARD_BOTTOM_GRADIENT = Brush.verticalGradient(
    colors = listOf(Color.Transparent, Color.Black.copy(alpha = 0.6f)),
    startY = 150f
)
private val EPISODE_CARD_GRADIENT = Brush.verticalGradient(0f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.76f))
private val EPISODE_CARD_GRADIENT_FOCUSED = Brush.verticalGradient(0f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.88f))

@Composable
fun MovieCardContent(
    movie: Meta,
    isUpcoming: Boolean,
    isFocused: Boolean,
    cardLayout: String,
    artworkPreference: String? = null,
    cornerRadius: Dp = 12.dp,
    hideTitles: Boolean = false,
    showHorizontalLogo: Boolean = true,
    lang: String? = "en",
    isContinueWatchingCard: Boolean = false,
    hideContinueWatchingTitles: Boolean = false,
    contentWidth: Dp = Dp.Unspecified,
    contentHeight: Dp = Dp.Unspecified,
    loadArtwork: Boolean = true
) {
    val isHorizontal = cardLayout == "horizontal"
    val isEpisodeStyle = cardLayout == "episode"
    val isCatalogFolder = movie.type == "catalog_folder"
    val context = LocalContext.current
    val deviceType = LocalDeviceType.current
    val baseCandidates = remember(
        movie.id,
        movie.type,
        movie.poster,
        movie.background,
        movie.focusGifUrl,
        movie.continueWatchingPoster,
        movie.continueWatchingBackground,
        isHorizontal,
        artworkPreference
    ) {
        if (isCatalogFolder) {
            listOfNotNull(movie.poster).distinct()
        } else if (isHorizontal) {
            horizontalArtworkCandidates(movie, preferContinueWatchingArtwork = artworkPreference != null)
        } else if (artworkPreference != null && (movie.type == "series" || movie.type == "tv" || movie.type == "anime")) {
            listOfNotNull(movie.continueWatchingBackground).distinct()
        } else {
            listOfNotNull(movie.poster).distinct()
        }
    }
    val focusCandidates = remember(movie.focusGifUrl, baseCandidates) {
        val gif = movie.focusGifUrl?.takeIf { it.isNotBlank() }
        if (gif != null && !isCatalogFolder) {
            (listOf(gif) + listOf(movie.poster, movie.background).filterNotNull()).distinct()
        } else baseCandidates
    }
    val imageCandidates = if (isFocused && !movie.focusGifUrl.isNullOrBlank() && !isCatalogFolder) focusCandidates else baseCandidates
    var imageIndex by remember(movie.id, movie.type, movie.poster, movie.background, movie.continueWatchingPoster, movie.continueWatchingBackground, isHorizontal, artworkPreference) { mutableIntStateOf(0) }
    val imageRequest = remember(imageCandidates, imageIndex, isHorizontal, deviceType) {
        val requestWidth = if (deviceType == DeviceType.Mobile) {
            if (isHorizontal) 512 else 288
        } else {
            if (isHorizontal) 640 else 320
        }
        val requestHeight = if (deviceType == DeviceType.Mobile) {
            if (isHorizontal) 288 else 432
        } else {
            if (isHorizontal) 360 else 480
        }
        ImageRequest.Builder(context)
            .data(imageCandidates.getOrNull(imageIndex))
            .crossfade(deviceType == DeviceType.TV)
            .memoryCacheKey(imageCandidates.getOrNull(imageIndex))
            .diskCacheKey(imageCandidates.getOrNull(imageIndex))
            .size(requestWidth, requestHeight)
            .build()
    }
    if (isEpisodeStyle) {
        ContinueWatchingEpisodeCardContent(
            movie = movie,
            isFocused = isFocused,
            artworkPreference = artworkPreference,
            lang = lang,
            isContinueWatchingCard = isContinueWatchingCard,
            hideContinueWatchingTitles = hideContinueWatchingTitles,
            loadArtwork = loadArtwork
        )
        return
    }
    val outerModifier = remember(cornerRadius) {
        Modifier.fillMaxSize().clip(RoundedCornerShape(cornerRadius)).background(Color.White.copy(alpha = 0.05f))
    }
    val showLogo = isHorizontal && showHorizontalLogo && !isCatalogFolder && (movie.timeOffset ?: 0L) <= 0L && !movie.logo.isNullOrBlank()
    val logoRequest = remember(movie.logo, showLogo) {
        if (!showLogo || movie.logo.isNullOrBlank()) null
        else ImageRequest.Builder(context)
            .data(movie.logo)
            .crossfade(deviceType == DeviceType.TV)
            .memoryCacheKey("home-logo:${movie.logo}")
            .diskCacheKey(movie.logo)
            .build()
    }

    val dimensionsKnown = contentWidth != Dp.Unspecified && contentHeight != Dp.Unspecified
    if (dimensionsKnown) {
        val logoMaxWidth = contentWidth * 0.42f
        val logoMaxHeight = contentHeight * 0.30f
        Box(modifier = outerModifier) {
            CardBody(
                movie, imageRequest, imageCandidates, isUpcoming, isFocused,
                showLogo, logoRequest, logoMaxWidth, logoMaxHeight,
                isContinueWatchingCard, hideContinueWatchingTitles, lang, deviceType, loadArtwork
            )
        }
    } else {
        BoxWithConstraints(modifier = outerModifier) {
            CardBody(
                movie, imageRequest, imageCandidates, isUpcoming, isFocused,
                showLogo, logoRequest, maxWidth * 0.42f, maxHeight * 0.30f,
                isContinueWatchingCard, hideContinueWatchingTitles, lang, deviceType, loadArtwork
            )
        }
    }
}

@Composable
private fun BoxScope.CardBody(
    movie: Meta,
    imageRequest: coil3.request.ImageRequest,
    imageCandidates: List<String>,
    isUpcoming: Boolean,
    isFocused: Boolean,
    showLogo: Boolean,
    logoRequest: coil3.request.ImageRequest?,
    logoMaxWidth: Dp,
    logoMaxHeight: Dp,
    isContinueWatchingCard: Boolean,
    hideContinueWatchingTitles: Boolean,
    lang: String?,
    deviceType: DeviceType,
    loadArtwork: Boolean
) {
    var imageFailed by remember(movie.id, imageRequest) { mutableStateOf(imageCandidates.isEmpty()) }
    if (loadArtwork && !imageFailed) {
        AsyncImage(
            model = imageRequest,
            contentDescription = null,
            modifier = Modifier.fillMaxSize().alpha(if (isUpcoming && !isFocused) 0.6f else 1.0f),
            contentScale = ContentScale.Crop,
            onError = { imageFailed = true }
        )
    } else {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(
                movie.coverEmoji?.takeIf { it.isNotBlank() } ?: movie.name.take(1).uppercase(),
                color = Color.White.copy(if (movie.coverEmoji.isNullOrBlank()) 0.2f else 0.82f),
                fontSize = if (movie.coverEmoji.isNullOrBlank()) 48.sp else 42.sp,
                fontWeight = FontWeight.Black
            )
        }
    }
    // Draw the gradient overlay only when something sits on top (logo/progress/continue-watching);
    // avoids a full-size overdraw on plain posters.
    val isProgressCard = (movie.timeOffset ?: 0L) > 0L && (movie.duration ?: 0L) > 0L
    val hasBottomOverlay = showLogo || deviceType == DeviceType.TV && (isContinueWatchingCard || isProgressCard)
    if (hasBottomOverlay) {
        Box(modifier = Modifier.fillMaxSize().background(CARD_BOTTOM_GRADIENT))
    }
    if (loadArtwork && showLogo && logoRequest != null) {
        AsyncImage(
            model = logoRequest,
            contentDescription = null,
            modifier = Modifier
                .align(Alignment.BottomStart)
                .padding(start = 9.dp, bottom = 9.dp)
                .widthIn(max = logoMaxWidth)
                .heightIn(max = logoMaxHeight),
            contentScale = ContentScale.Fit
        )
    }
    if (isContinueWatchingCard) {
        ContinueWatchingLogo(movie = movie, loadArtwork = loadArtwork, modifier = Modifier.align(Alignment.BottomStart))
    }
    val timeOffset = movie.timeOffset ?: 0L
    val duration = movie.duration ?: 0L
    if (timeOffset > 0 && duration > 0) {
        val progress = (timeOffset / duration.toFloat()).coerceIn(0f, 1f)
        if (isContinueWatchingCard) {
            Box(modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(4.dp).background(Color.White.copy(0.2f))) {
                Box(modifier = Modifier.fillMaxWidth(progress).fillMaxHeight().background(Color(0xFFE50914)))
            }
        } else if (deviceType == DeviceType.Mobile) {
            Box(modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(3.dp).background(Color.White.copy(0.22f))) {
                Box(modifier = Modifier.fillMaxWidth(progress).fillMaxHeight().background(Color.White))
            }
        } else {
            val remainingMs = duration - timeOffset
            Column(modifier = Modifier.align(Alignment.BottomCenter).fillMaxWidth().padding(8.dp)) {
                Text(text = formatRemainingTime(remainingMs, lang), color = Color.White.copy(0.7f), fontSize = 9.sp, fontWeight = FontWeight.Bold, modifier = Modifier.padding(bottom = 3.dp))
                Spacer(Modifier.height(2.dp))
                Box(modifier = Modifier.fillMaxWidth().height(3.dp).clip(RoundedCornerShape(1.5.dp)).background(Color.White.copy(0.2f))) {
                    Box(modifier = Modifier.fillMaxWidth(progress).fillMaxHeight().background(Color.White))
                }
            }
        }
    }
    if (isContinueWatchingCard && movie.isUpNextContinueItem()) {
        UpNextBadge(lang = lang, modifier = Modifier.align(Alignment.TopEnd).padding(8.dp))
    }
    if (isContinueWatchingCard && hideContinueWatchingTitles && !movie.isUpNextContinueItem()) {
        ContinueWatchingRemainingBadge(
            text = formatRemainingTime(((movie.duration ?: 0L) - (movie.timeOffset ?: 0L)).coerceAtLeast(0L), lang),
            modifier = Modifier.align(Alignment.TopEnd).padding(8.dp)
        )
    }
}

@Composable
internal fun ContinueWatchingEpisodeCardContent(
    movie: Meta,
    isFocused: Boolean,
    artworkPreference: String?,
    lang: String? = "en",
    isContinueWatchingCard: Boolean = false,
    hideContinueWatchingTitles: Boolean = false,
    loadArtwork: Boolean = true
) {
    val isUpNext = movie.isUpNextContinueItem()
    val deviceType = LocalDeviceType.current
    val progress = ((movie.timeOffset ?: 0L).toFloat() / (movie.duration ?: 1L).toFloat()).coerceIn(0f, 1f)
    val remainingMs = ((movie.duration ?: 0L) - (movie.timeOffset ?: 0L)).coerceAtLeast(0L)
    val seriesArtwork = when (artworkPreference) {
        "episode" -> movie.continueWatchingBackground
        "poster" -> movie.poster
        "background" -> movie.background
        else -> movie.continueWatchingBackground
    }
    val artwork = if (movie.type == "series" || movie.type == "tv" || movie.type == "anime") {
        seriesArtwork
    } else {
        movie.background
    }
    val context = LocalContext.current
    val artworkRequest = remember(artwork) {
        ImageRequest.Builder(context)
            .data(artwork)
            .crossfade(false)
            .memoryCacheKey(artwork)
            .diskCacheKey(artwork)
            .size(512, 288)
            .build()
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .clip(RoundedCornerShape(if (isContinueWatchingCard) 0.dp else 12.dp))
            .background(Color(0xFF141922))
    ) {
        if (loadArtwork) {
            AsyncImage(
                model = artworkRequest,
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
        }
        if (deviceType == DeviceType.TV || !isContinueWatchingCard) {
            Box(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .height(if (deviceType == DeviceType.Mobile) 68.dp else 112.dp)
                    .background(if (isFocused) EPISODE_CARD_GRADIENT_FOCUSED else EPISODE_CARD_GRADIENT)
            )
        }
        if (!isUpNext && !isContinueWatchingCard) Column(
            modifier = Modifier
                .align(Alignment.BottomStart)
                .fillMaxWidth()
                .padding(12.dp)
        ) {
            Text(
                text = formatRemainingTime(remainingMs, lang),
                color = Color.White.copy(alpha = 0.82f),
                fontSize = 10.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(bottom = 6.dp)
            )
            Box(
                modifier = Modifier
                .fillMaxWidth()
                .height(4.dp)
                .clip(RoundedCornerShape(if (isContinueWatchingCard) 0.dp else 999.dp))
                .background(Color.White.copy(alpha = 0.20f))
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth(progress)
                        .fillMaxHeight()
                        .background(if (isContinueWatchingCard) Color(0xFFE50914) else Color.White)
                )
            }
        }
        if (isContinueWatchingCard && isUpNext) {
            UpNextBadge(
                lang = lang,
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(8.dp)
            )
        }
        if (isContinueWatchingCard && hideContinueWatchingTitles && !isUpNext) {
            ContinueWatchingRemainingBadge(
                text = formatRemainingTime(remainingMs, lang),
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(8.dp)
            )
        }
        if (isContinueWatchingCard) {
            ContinueWatchingLogo(movie = movie, loadArtwork = loadArtwork, modifier = Modifier.align(Alignment.BottomStart))
            if (!isUpNext) ContinueWatchingProgressBar(
                progress = progress,
                modifier = Modifier.align(Alignment.BottomCenter)
            )
        }
    }
}

@Composable
private fun ContinueWatchingProgressBar(progress: Float, modifier: Modifier = Modifier) {
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(4.dp)
            .background(Color(0xFF212121))
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth(progress)
                .fillMaxHeight()
                .background(Color(0xFFE50914))
        )
    }
}

@Composable
private fun CenterPlayOverlay(modifier: Modifier = Modifier) {
    Box(
        modifier = modifier
            .size(42.dp)
            .clip(RoundedCornerShape(999.dp))
            .background(Color.Black.copy(alpha = 0.58f)),
        contentAlignment = Alignment.Center
    ) {
        androidx.compose.material3.Icon(
            imageVector = FluxaIcons.PlayArrow,
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier
                .size(23.dp)
                .padding(start = 2.dp)
        )
    }
}

@Composable
private fun UpNextBadge(lang: String?, modifier: Modifier = Modifier) {
    Text(
        text = AppStrings.t(lang, "auto.up_next"),
        color = Color.White,
        fontSize = 10.sp,
        fontWeight = FontWeight.Black,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier
            .background(Color.Black.copy(alpha = 0.88f))
            .padding(horizontal = 8.dp, vertical = 5.dp)
    )
}

@Composable
private fun ContinueWatchingRemainingBadge(text: String, modifier: Modifier = Modifier) {
    Text(
        text = text,
        color = Color.White,
        fontSize = 10.sp,
        fontWeight = FontWeight.Black,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier
            .background(Color.Black.copy(alpha = 0.88f))
            .padding(horizontal = 8.dp, vertical = 5.dp)
    )
}

@Composable
private fun ContinueWatchingLogo(movie: Meta, loadArtwork: Boolean = true, modifier: Modifier = Modifier) {
    if (!loadArtwork) return
    val context = LocalContext.current
    val deviceType = LocalDeviceType.current
    val logo = movie.logo?.takeIf { it.isNotBlank() } ?: return
    val logoRequest = remember(logo, deviceType) {
        ImageRequest.Builder(context)
            .data(logo)
            .crossfade(deviceType == DeviceType.TV)
            .memoryCacheKey("home-continue-logo:$logo")
            .diskCacheKey(logo)
            .size(184, 68)
            .build()
    }
    AsyncImage(
        model = logoRequest,
        contentDescription = null,
        modifier = modifier
            .padding(start = 4.dp, bottom = 7.dp)
            .widthIn(max = 92.dp)
            .heightIn(max = 34.dp),
        contentScale = ContentScale.Fit
    )
}
