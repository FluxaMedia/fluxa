@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Intent
import android.net.Uri
import androidx.compose.animation.*
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.foundation.Image
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import coil3.compose.AsyncImagePainter
import coil3.request.ImageRequest
import coil3.request.transformations
import com.fluxa.app.R
import java.text.SimpleDateFormat
import java.util.*

@Composable
fun DetailHeaderContentOfficial(
    detail: MetaDetail?,
    selectedEpisode: Video?,
    isInWatchlist: Boolean,
    feedback: Boolean?,
    lang: String,
    onBack: () -> Unit,
    onToggleWatchlist: () -> Unit,
    onFeedback: (Boolean) -> Unit,
    onPlayClick: () -> Unit
) {
    val deviceType = LocalDeviceType.current
    val horizontalPadding = if (deviceType == DeviceType.TV) 58.dp else 16.dp
    val playButtonFocusRequester = remember { FocusRequester() }
    LaunchedEffect(Unit) {
        if (deviceType != DeviceType.TV) return@LaunchedEffect
        repeat(15) {
            try {
                playButtonFocusRequester.requestFocus()
                return@LaunchedEffect
            } catch (e: Exception) {
                withFrameNanos {}
            }
        }
    }

    Column(modifier = Modifier.padding(start = horizontalPadding, top = if (deviceType == DeviceType.TV) 40.dp else 24.dp).fillMaxWidth()) {
        var logoLoadFailed by remember { mutableStateOf(false) }
        val resolvedLogo = detail?.logo
        
        if (!resolvedLogo.isNullOrEmpty() && !logoLoadFailed) {
            val context = LocalContext.current
            val logoRequest = remember(context, resolvedLogo) {
                ImageRequest.Builder(context)
                    .data(resolvedLogo)
                    .memoryCacheKey("detail-logo:$resolvedLogo")
                    .diskCacheKey(resolvedLogo)
                    .transformations(TrimTransparentEdgesTransformation())
                    .build()
            }
            AsyncImage(
                model = logoRequest,
                contentDescription = null,
                modifier = Modifier.height(if (deviceType == DeviceType.TV) 130.dp else 80.dp).widthIn(max = 500.dp),
                contentScale = ContentScale.Fit,
                alignment = Alignment.CenterStart,
                onError = { logoLoadFailed = true }
            )
        } else {
            Text(
                text = detail?.name?.uppercase() ?: stringResource(R.string.app_name).uppercase(),
                style = if (deviceType == DeviceType.TV) MaterialTheme.typography.displayLarge else MaterialTheme.typography.displayMedium,
                fontWeight = FontWeight.Black,
                color = Color.White,
                letterSpacing = (-2).sp,
                lineHeight = if(deviceType == DeviceType.TV) 60.sp else 44.sp,
                modifier = Modifier.padding(bottom = 8.dp)
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Start
        ) {
            detail?.ratings?.forEach { r -> OfficialRatingBadge(r.source, r.value.toString()) }
        }

        Spacer(modifier = Modifier.height(16.dp))

        val metaParts = mutableListOf<String>()
        detail?.releaseInfo?.let { metaParts.add(it) }
        if (detail?.type == "series") {
            detail.seasonsCount?.let { metaParts.add("${it} ${AppStrings.t(lang, "auto.season")}") }
        }
        detail?.runtime?.let { metaParts.add(formatRuntimeLabel(it, lang)) }
        
        Row(verticalAlignment = Alignment.CenterVertically) {
            metaParts.forEachIndexed { i, part ->
                Text(text = part, color = Color.White, fontSize = 15.sp, fontWeight = FontWeight.Bold)
                if (i < metaParts.size - 1) {
                    Text(text = "  ", color = Color.White.copy(0.4f), fontSize = 15.sp, fontWeight = FontWeight.Black, modifier = Modifier.padding(horizontal = 6.dp))
                }
            }
            val platforms = detail?.platforms.orEmpty()
            if (platforms.isNotEmpty()) {
                Text(text = "  ", color = Color.White.copy(0.4f), fontSize = 15.sp, fontWeight = FontWeight.Black, modifier = Modifier.padding(horizontal = 6.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    platforms.forEach { logo ->
                        AsyncImage(
                            model = logo,
                            contentDescription = null,
                            modifier = Modifier.height(16.dp),
                            contentScale = ContentScale.Fit,
                            colorFilter = androidx.compose.ui.graphics.ColorFilter.tint(Color.White)
                        )
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        val scheduleLabel = remember(detail?.id, detail?.videos, lang) { releaseScheduleLabel(detail, lang) }
        scheduleLabel?.let {
            Text(
                text = it,
                color = FluxaColors.releaseGreen,
                fontSize = 14.sp,
                fontWeight = FontWeight.SemiBold
            )
            Spacer(modifier = Modifier.height(12.dp))
        }

        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.horizontalScroll(rememberScrollState())) {
            detail?.ageRating?.takeIf { it.isNotBlank() }?.let { ageRating ->
                Box(modifier = Modifier.border(1.dp, Color.White.copy(0.4f), RoundedCornerShape(2.dp)).padding(horizontal = 6.dp, vertical = 2.dp)) {
                    Text(text = ageRating, color = Color.White, fontSize = 11.sp, fontWeight = FontWeight.Bold)
                }
                Spacer(Modifier.width(12.dp))
            }
            detail?.genres?.forEach { genre ->
                Box(modifier = Modifier.padding(end = 8.dp).background(Color(0xFF1A1A1A), RoundedCornerShape(4.dp)).padding(horizontal = 10.dp, vertical = 4.dp)) {
                    Text(text = genre, color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.Bold)
                }
            }
        }

        Spacer(modifier = Modifier.height(32.dp))

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
            val playButtonText = AppStrings.t(lang, "auto.play")
            
            Surface(
                onClick = onPlayClick,
                modifier = Modifier
                    .height(48.dp)
                    .then(
                        if (deviceType == DeviceType.TV) Modifier.widthIn(min = 140.dp, max = 240.dp)
                        else Modifier.wrapContentWidth()
                    )
                    .focusRequester(playButtonFocusRequester)
                    .run { if (deviceType == DeviceType.Mobile) clickable { onPlayClick() } else this },
                shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(12.dp)),
                colors = ClickableSurfaceDefaults.colors(containerColor = Color.White, focusedContainerColor = Color.White, contentColor = Color.Black, focusedContentColor = Color.Black)
            ) {
                Box(contentAlignment = Alignment.Center, modifier = Modifier.fillMaxHeight().padding(horizontal = if(deviceType == DeviceType.TV) 24.dp else 16.dp)) {
                    Text(
                        text = playButtonText,
                        fontWeight = FontWeight.Black,
                        fontSize = 14.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        textAlign = TextAlign.Center
                    )
                }
            }
            
            ActionCircleBtn(if (isInWatchlist) FluxaIcons.Check else FluxaIcons.Add, onToggleWatchlist, tint = if(isInWatchlist) Color(0xFF00E054) else Color.White)
            ActionCircleBtn(FluxaIcons.ThumbUp, { onFeedback(true) }, tint = if(feedback == true) Color.White else Color.White.copy(0.35f))
            ActionCircleBtn(FluxaIcons.ThumbDown, { onFeedback(false) }, tint = if(feedback == false) Color.White else Color.White.copy(0.35f))
        }
    }
}

@Composable
fun OfficialRatingBadge(source: String, text: String) {
    val logoRes = remember(source) {
        when (source.lowercase().trim()) {
            "imdb" -> R.drawable.imdb_logo
            "tmdb" -> R.drawable.ic_tmdb
            "trakt" -> R.drawable.ic_trakt
            "simkl" -> R.drawable.ic_simkl
            else -> null
        }
    }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.Start,
        modifier = Modifier.padding(end = 24.dp)
    ) {
        if (logoRes != null) {
            Image(
                painter = painterResource(id = logoRes),
                contentDescription = null,
                modifier = Modifier.height(16.dp).widthIn(max = 52.dp),
                contentScale = ContentScale.Fit
            )
        } else {
            Box(modifier = Modifier.size(24.dp).background(Color.White.copy(0.1f), RoundedCornerShape(4.dp)), contentAlignment = Alignment.Center) {
                Text(text = source.take(1).uppercase(), color = Color.White, fontSize = 10.sp, fontWeight = FontWeight.Black)
            }
            Spacer(Modifier.width(8.dp))
            Text(
                text = source,
                color = Color.White.copy(alpha = 0.65f),
                fontWeight = FontWeight.Bold,
                fontSize = 12.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.widthIn(max = 96.dp)
            )
        }

        Spacer(Modifier.width(6.dp))

        Text(
            text = text,
            color = Color.White,
            fontWeight = FontWeight.Black,
            fontSize = 17.sp
        )
    }
}

@Composable
fun ActionCircleBtn(icon: ImageVector, onClick: () -> Unit, tint: Color = Color.White) {
    var isFocused by remember { mutableStateOf(false) }
    val deviceType = LocalDeviceType.current
    Surface(
        onClick = onClick,
        modifier = Modifier
            .size(52.dp)
            .onFocusChanged { isFocused = it.isFocused }
            .run { if (deviceType == DeviceType.Mobile) clickable { onClick() } else this },
        shape = ClickableSurfaceDefaults.shape(CircleShape),
        colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(0.1f), focusedContainerColor = Color.White)
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) { Icon(icon, null, tint = if (isFocused) Color.Black else tint, modifier = Modifier.size(26.dp)) }
    }
}

@Composable
fun SeasonPill(seasonNumber: Int, isSelected: Boolean, accentColor: Color, lang: String, onClick: () -> Unit) {
    var isFocused by remember { mutableStateOf(false) }
    val deviceType = LocalDeviceType.current
    val containerColor = if (isSelected) Color.White else if (isFocused) Color.White.copy(alpha = 0.2f) else Color.White.copy(alpha = 0.05f)
    val contentColor = if (isSelected) Color.Black else Color.White
    val seasonName = if (seasonNumber == 0) (AppStrings.t(lang, "auto.specials")) else "${AppStrings.t(lang, "auto.season")} $seasonNumber"
    Surface(
        onClick = onClick,
        modifier = Modifier.width(140.dp).height(50.dp).onFocusChanged { isFocused = it.isFocused }.run { if (deviceType == DeviceType.Mobile) clickable { onClick() } else this },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(8.dp)),
        colors = ClickableSurfaceDefaults.colors(containerColor = containerColor, contentColor = contentColor, focusedContainerColor = Color.White, focusedContentColor = Color.Black)
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(text = seasonName, fontSize = 14.sp, fontWeight = FontWeight.Bold)
        }
    }
}

@Composable
fun EpisodeCard(
    episode: Video,
    isSelected: Boolean,
    isWatched: Boolean = false,
    blurUnwatched: Boolean = false,
    progress: Float = 0f,
    durationLabel: String? = null,
    accentColor: Color,
    lang: String,
    onFocus: () -> Unit,
    onClick: () -> Unit,
    onDownloadClick: (() -> Unit)? = null
) {
    var isFocused by remember { mutableStateOf(false) }
    var pressStartTime by remember { mutableStateOf(0L) }
    val deviceType = LocalDeviceType.current
    val isUpcoming = detailIsUpcoming(episode.released)
    val thumbnailRequest = rememberEpisodeThumbnailRequest(episode)
    val runtimeText = durationLabel?.takeIf { it.isNotBlank() }?.let { formatRuntimeLabel(it, lang) }
    val thumbnailAlpha = when {
        isUpcoming -> 0.4f
        blurUnwatched && !isWatched -> 0.5f
        !blurUnwatched && isWatched -> 0.45f
        else -> 1.0f
    }
    Column(
        modifier = Modifier
            .width(300.dp)
            .onFocusChanged { if (it.isFocused) { isFocused = true; onFocus() } else isFocused = false }
            .onPreviewKeyEvent { event ->
                if (onDownloadClick == null || (event.key != Key.DirectionCenter && event.key != Key.Enter)) return@onPreviewKeyEvent false
                when (event.type) {
                    KeyEventType.KeyDown -> {
                        if (pressStartTime == 0L) pressStartTime = System.currentTimeMillis()
                        false
                    }
                    KeyEventType.KeyUp -> {
                        val heldMs = System.currentTimeMillis() - pressStartTime
                        pressStartTime = 0L
                        if (heldMs >= 500L) { onDownloadClick.invoke(); true } else false
                    }
                    else -> false
                }
            }
    ) {
        Surface(
            onClick = onClick,
            modifier = Modifier
                .fillMaxWidth()
                .height(170.dp)
                .run { if (deviceType == DeviceType.Mobile) clickable { onClick() } else this },
            shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(12.dp)),
            border = ClickableSurfaceDefaults.border(focusedBorder = Border(androidx.compose.foundation.BorderStroke(3.dp, Color.White))),
            scale = ClickableSurfaceDefaults.scale(focusedScale = 1.05f)
        ) {
            var thumbnailFailed by remember(thumbnailRequest) { mutableStateOf(false) }
            Box {
                if (thumbnailRequest != null && !thumbnailFailed) {
                    AsyncImage(
                        model = thumbnailRequest,
                        contentDescription = null,
                        contentScale = ContentScale.Crop,
                        modifier = Modifier.fillMaxSize().alpha(thumbnailAlpha),
                        onState = { state -> if (state is AsyncImagePainter.State.Error) thumbnailFailed = true }
                    )
                }
                else { Box(modifier = Modifier.fillMaxSize().background(Color.White.copy(0.05f)), contentAlignment = Alignment.Center) { Icon(FluxaIcons.Movie, null, tint = Color.White.copy(0.1f), modifier = Modifier.size(48.dp)) } }
                if (isWatched) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.Center)
                            .size(44.dp)
                            .clip(CircleShape)
                            .background(Color.Black.copy(alpha = 0.38f)),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(FluxaIcons.CheckCircle, null, tint = Color.White, modifier = Modifier.size(24.dp))
                    }
                }
            }
        }
        if (progress > 0f || isWatched) {
            Spacer(Modifier.height(8.dp))
            EpisodeProgressBar(progress = if (isWatched) 1f else progress, accentColor = accentColor)
        }
        Spacer(Modifier.height(10.dp))
        Text(
            text = "${episode.number ?: 0}. ${episode.name.orEmpty()}",
            color = if (isFocused) Color.White else Color.White.copy(alpha = 0.88f),
            fontSize = 15.sp,
            fontWeight = FontWeight.Bold,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis
        )
        runtimeText?.let {
            Spacer(Modifier.height(4.dp))
            Text(
                text = it,
                color = Color.White.copy(alpha = 0.62f),
                fontSize = 12.sp,
                fontWeight = FontWeight.Medium
            )
        }
    }
}

internal fun sortedDetailEpisodes(episodes: List<Video>, sort: String): List<Video> {
    val comparator = when (sort.substringBefore("_")) {
        "rating" -> compareBy<Video> { it.rating?.toDoubleOrNull() ?: -1.0 }
        "released" -> compareBy { it.released.orEmpty() }
        else -> compareBy<Video>({ it.number ?: Int.MAX_VALUE }, { it.name.orEmpty() })
    }
    return if (sort.endsWith("_desc")) episodes.sortedWith(comparator.reversed()) else episodes.sortedWith(comparator)
}

@Composable
internal fun rememberEpisodeThumbnailRequest(episode: Video): ImageRequest? {
    val context = LocalContext.current
    val thumbnailUrl = remember(episode.thumbnail) {
        episode.thumbnail?.trim()?.takeIf { it.isNotBlank() }
    }
    return remember(context, episode.id, thumbnailUrl) {
        thumbnailUrl?.let { url ->
            ImageRequest.Builder(context)
                .data(url)
                .memoryCacheKey("detail-episode:${episode.id}:$url")
                .diskCacheKey(url)
                .build()
        }
    }
}

internal fun episodeSortOptions(lang: String): List<Pair<String?, String>> = listOf(
    "number_asc" to AppStrings.t(lang, "sort.episode_number_asc"),
    "number_desc" to AppStrings.t(lang, "sort.episode_number_desc"),
    "rating_asc" to AppStrings.t(lang, "sort.rating_asc"),
    "rating_desc" to AppStrings.t(lang, "sort.rating_desc"),
    "released_asc" to AppStrings.t(lang, "sort.release_date_asc"),
    "released_desc" to AppStrings.t(lang, "sort.release_date_desc")
)

@Composable
fun EpisodeProgressBar(progress: Float, accentColor: Color) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(3.dp)
            .clip(RoundedCornerShape(999.dp))
            .background(Color.White.copy(alpha = 0.18f))
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth(progress.coerceIn(0f, 1f))
                .fillMaxHeight()
                .background(FluxaColors.progressFill)
        )
    }
}

@Composable
fun TrailerCard(trailer: DetailTrailer, accentColor: Color, onPlay: (() -> Unit)? = null) {
    val context = LocalContext.current
    val deviceType = LocalDeviceType.current
    val playableUrl = trailer.url.takeIf { it.isNotBlank() }
    val thumbnailRequest = remember(trailer.thumbnail) {
        trailer.thumbnail?.takeIf { it.isNotBlank() }?.let { url ->
            ImageRequest.Builder(context).data(url).memoryCacheKey("trailer:$url").diskCacheKey(url).build()
        }
    }
    val handleClick = {
        if (onPlay != null) {
            onPlay()
        } else if (playableUrl != null) {
            runCatching { context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(playableUrl))) }
        }
    }
    Surface(
        onClick = handleClick,
        modifier = Modifier
            .width(260.dp)
            .height(206.dp)
            .run { if (deviceType == DeviceType.Mobile) clickable { handleClick() } else this },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(12.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = Color.White.copy(alpha = 0.08f),
            contentColor = Color.White,
            focusedContainerColor = accentColor,
            focusedContentColor = Color.Black
        ),
        scale = ClickableSurfaceDefaults.scale(focusedScale = 1.05f),
        border = ClickableSurfaceDefaults.border(focusedBorder = Border(androidx.compose.foundation.BorderStroke(2.dp, accentColor)))
    ) {
        Column(modifier = Modifier.fillMaxSize()) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(146.dp)
                    .background(Color.Black.copy(alpha = 0.34f))
            ) {
                if (thumbnailRequest != null) {
                    AsyncImage(
                        model = thumbnailRequest,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop
                    )
                }
                Box(
                    modifier = Modifier
                        .align(Alignment.Center)
                        .size(44.dp)
                        .clip(CircleShape)
                        .background(Color.Black.copy(alpha = 0.52f)),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(FluxaIcons.PlayArrow, null, tint = Color.White, modifier = Modifier.size(24.dp))
                }
            }
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(60.dp)
                    .padding(horizontal = 12.dp, vertical = 8.dp)
            ) {
                Text(
                    text = trailer.type,
                    color = accentColor,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Black,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                if (trailer.title.isNotBlank() && !trailer.title.equals(trailer.type, ignoreCase = true)) {
                    Spacer(Modifier.height(4.dp))
                    Text(
                        text = trailer.title,
                        color = Color.White,
                        fontSize = 13.sp,
                        lineHeight = 16.sp,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
        }
    }
}

@Composable
fun StreamCard(stream: StreamUiModel, accentColor: Color, onClick: () -> Unit) {
    val deviceType = LocalDeviceType.current
    val sourceName = stream.header
    val rawTitle = stream.body
    Surface(
        onClick = onClick,
        modifier = Modifier
            .width(320.dp)
            .height(172.dp)
            .run { if (deviceType == DeviceType.Mobile) clickable { onClick() } else this },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(12.dp)),
        colors = ClickableSurfaceDefaults.colors(containerColor = Color.White.copy(0.08f), focusedContainerColor = Color.White, focusedContentColor = Color.Black, contentColor = Color.White),
        scale = ClickableSurfaceDefaults.scale(focusedScale = 1.05f)
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(12.dp)
        ) {
            Text(
                text = sourceName,
                fontWeight = FontWeight.Bold,
                fontSize = 13.sp,
                lineHeight = 15.sp,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )
            rawTitle?.let {
                Spacer(Modifier.height(5.dp))
                AddonStreamBodyText(
                    text = it,
                    bodyMaxLines = 6,
                    contentColor = Color.White.copy(alpha = 0.72f)
                )
            }
        }
    }
}

@Composable
fun SourceFilterPill(name: String, isSelected: Boolean, onClick: () -> Unit) {
    var isFocused by remember { mutableStateOf(false) }
    val deviceType = LocalDeviceType.current
    val containerColor = if (isSelected) Color.White else if (isFocused) Color.White.copy(alpha = 0.2f) else Color.White.copy(alpha = 0.05f)
    val contentColor = if (isSelected) Color.Black else Color.White
    Surface(
        onClick = onClick,
        modifier = Modifier
            .onFocusChanged { isFocused = it.isFocused }
            .run { if (deviceType == DeviceType.Mobile) clickable { onClick() } else this },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(8.dp)),
        colors = ClickableSurfaceDefaults.colors(containerColor = containerColor, contentColor = contentColor, focusedContainerColor = Color.White, focusedContentColor = Color.Black)
    ) {
        Box(modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp), contentAlignment = Alignment.Center) {
            Text(text = name, fontSize = 13.sp, fontWeight = FontWeight.Bold)
        }
    }
}

@Composable
fun CastMemberCard(member: CastMember) {
    val size = if (LocalDeviceType.current == DeviceType.TV) 100.dp else 80.dp
    val cardWidth = if (LocalDeviceType.current == DeviceType.TV) 124.dp else 104.dp
    val context = LocalContext.current
    val photoRequest = remember(member.profilePath) {
        member.profilePath?.let { path ->
            ImageRequest.Builder(context).data(path).memoryCacheKey("cast:$path").diskCacheKey(path).build()
        }
    }
    Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.width(cardWidth)) {
        Box(modifier = Modifier.size(size).clip(CircleShape).background(Color.White.copy(alpha = 0.1f))) {
            if (photoRequest != null) AsyncImage(photoRequest, null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
            else Icon(FluxaIcons.Person, null, modifier = Modifier.size(size/2).align(Alignment.Center), tint = Color.Gray)
        }
        Spacer(modifier = Modifier.height(12.dp))
        Text(
            text = member.name,
            color = Color.White,
            fontSize = 12.sp,
            lineHeight = 15.sp,
            fontWeight = FontWeight.Bold,
            textAlign = TextAlign.Center,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis
        )
        member.character?.takeIf { it.isNotEmpty() }?.let { character ->
            Text(text = character, color = Color.White.copy(alpha = 0.5f), fontSize = 10.sp, textAlign = TextAlign.Center, maxLines = 1)
        }
    }
}

@Composable
fun SimilarContentCard(item: SimilarItemUiModel, accentColor: Color, onClick: () -> Unit) {
    val deviceType = LocalDeviceType.current
    val context = LocalContext.current
    val posterRequest = remember(item.id, item.poster) {
        item.poster?.let { url ->
            ImageRequest.Builder(context).data(url).memoryCacheKey("similar:${item.id}").diskCacheKey(url).build()
        }
    }
    Surface(
        onClick = onClick,
        modifier = Modifier
            .width(150.dp)
            .height(220.dp)
            .run { if (deviceType == DeviceType.Mobile) clickable { onClick() } else this },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(8.dp)),
        scale = ClickableSurfaceDefaults.scale(focusedScale = 1.1f),
        border = ClickableSurfaceDefaults.border(focusedBorder = Border(androidx.compose.foundation.BorderStroke(3.dp, Color.White)))
    ) { AsyncImage(model = posterRequest, null, modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop) }
}

internal fun detailIsUpcoming(dateStr: String?): Boolean {
    return ReleaseDateUtils.isUpcoming(dateStr)
}

private fun parseReleaseDate(dateStr: String?): Date? {
    if (dateStr.isNullOrBlank()) return null
    return runCatching { SimpleDateFormat("yyyy-MM-dd", Locale.US).parse(dateStr) }.getOrNull()
}

internal fun formatEpisodeReleaseDate(dateStr: String?, lang: String): String? {
    val date = parseReleaseDate(dateStr) ?: return null
    val locale = AppStrings.locale(lang)
    return SimpleDateFormat("yyyy MMMM d", locale).format(date)
}

internal fun formatRuntimeLabel(runtimeLabel: String, lang: String): String {
    return AppStrings.runtimeLabel(lang, runtimeLabel)
}

internal fun releaseScheduleLabel(detail: MetaDetail?, lang: String): String? {
    val videos = detail?.videos.orEmpty()
    if (videos.isEmpty()) return null
    val normalizedStatus = detail?.status?.lowercase()?.trim().orEmpty()
    val hasFutureEpisode = videos.any { video -> parseReleaseDate(video.released)?.after(Date()) == true }
    val isFinished = normalizedStatus.contains("ended") ||
        normalizedStatus.contains("canceled") ||
        normalizedStatus.contains("cancelled")
    val isActivelyReleasing = hasFutureEpisode ||
        normalizedStatus.contains("returning") ||
        normalizedStatus.contains("continuing") ||
        normalizedStatus.contains("production") ||
        normalizedStatus.contains("planned")
    if (isFinished || !isActivelyReleasing) return null

    val now = Date()
    val futureDay = videos
        .mapNotNull { parseReleaseDate(it.released) }
        .filter { it.after(now) }
        .minByOrNull { it.time }
        ?.let { Calendar.getInstance().apply { time = it }.get(Calendar.DAY_OF_WEEK) }

    val dominantDay = futureDay ?: videos
        .mapNotNull { parseReleaseDate(it.released) }
        .groupingBy { date ->
            Calendar.getInstance().apply { time = date }.get(Calendar.DAY_OF_WEEK)
        }
        .eachCount()
        .maxByOrNull { it.value }
        ?.key
        ?: return null

    val calendar = Calendar.getInstance().apply {
        set(Calendar.DAY_OF_WEEK, dominantDay)
    }
    val locale = AppStrings.locale(lang)
    val dayName = SimpleDateFormat("EEEE", locale).format(calendar.time)
    return AppStrings.format(lang, "format.new_episodes_every", dayName.replaceFirstChar { if (it.isLowerCase()) it.titlecase(locale) else it.toString() })
}

internal fun episodeProgressFraction(progressMs: Long, runtimeLabel: String?): Float {
    if (progressMs <= 0L) return 0f
    val runtimeMinutes = Regex("""(\d+)""").find(runtimeLabel.orEmpty())?.groupValues?.getOrNull(1)?.toLongOrNull()
    val durationMs = (runtimeMinutes ?: 45L) * 60_000L
    return (progressMs.toFloat() / durationMs.toFloat()).coerceIn(0f, 1f)
}
