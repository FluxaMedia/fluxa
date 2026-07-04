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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.key.*
import androidx.compose.ui.layout.*
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
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
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.distinctUntilChanged
import androidx.compose.runtime.snapshotFlow
import java.text.SimpleDateFormat
import java.util.*
import kotlin.math.abs

private val HERO_GRADIENT = Brush.verticalGradient(
    0.00f to Color.Black.copy(alpha = 0.45f),
    0.36f to Color.Transparent,
    0.72f to Color.Black.copy(alpha = 0.54f),
    1.00f to Color.Black
)

@Composable
internal fun MobileStreamingHeroPager(
    movies: List<Meta>,
    currentMovie: Meta?,
    currentLogoUrl: String?,
    pagerState: androidx.compose.foundation.pager.PagerState,
    lang: String,
    isWatchlisted: Boolean,
    onPlayClick: () -> Unit,
    onInfoClick: () -> Unit,
    onToggleWatchlist: () -> Unit,
    seasonPostersOnHero: Boolean = true
) {
    val realPage = if (movies.isNotEmpty()) pagerState.currentPage.floorMod(movies.size) else 0
    val movie = movies.getOrNull(realPage) ?: currentMovie ?: return
    val pageLogoUrl = currentLogoUrl.takeIf { currentMovie?.id == movie.id } ?: movie.logo

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(FluxaDimensions.mobileBillboardHeight)
    ) {
        HorizontalPager(
            state = pagerState,
            modifier = Modifier.matchParentSize(),
            beyondViewportPageCount = 0,
            flingBehavior = PagerDefaults.flingBehavior(
                state = pagerState,
                snapAnimationSpec = tween(durationMillis = FluxaDimensions.AnimDuration.heroSnap, easing = FastOutSlowInEasing)
            )
        ) { page ->
            val pageMovie = movies.getOrNull(page.floorMod(movies.size)) ?: return@HorizontalPager
            MobileHeroBackground(pageMovie, seasonPostersOnHero)
        }

        MobileHeroForeground(
            movie = movie,
            logoUrl = pageLogoUrl,
            lang = lang,
            isWatchlisted = isWatchlisted,
            onPlayClick = onPlayClick,
            onInfoClick = onInfoClick,
            onToggleWatchlist = onToggleWatchlist
        )

        MobileHeroDots(
            activeIndex = realPage,
            pagerState = pagerState,
            count = movies.size,
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .padding(bottom = 14.dp)
        )
    }
}

@Composable
internal fun MobileStreamingHero(
    movie: Meta?,
    logoUrl: String?,
    lang: String,
    isWatchlisted: Boolean,
    onPlayClick: () -> Unit,
    onInfoClick: () -> Unit,
    onToggleWatchlist: () -> Unit,
    index: Int,
    poolSize: Int,
    seasonPostersOnHero: Boolean = true
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(FluxaDimensions.mobileBillboardHeight)
    ) {
        if (movie != null) {
            MobileHeroBackground(movie, seasonPostersOnHero)
            MobileHeroForeground(
                movie = movie,
                logoUrl = logoUrl,
                lang = lang,
                isWatchlisted = isWatchlisted,
                onPlayClick = onPlayClick,
                onInfoClick = onInfoClick,
                onToggleWatchlist = onToggleWatchlist
            )
            if (poolSize > 1) {
                MobileHeroDots(
                    activeIndex = index,
                    count = poolSize,
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .padding(bottom = 14.dp)
                )
            }
        }
    }
}

@Composable
private fun MobileHeroBackground(movie: Meta, seasonPostersOnHero: Boolean) {
    val context = LocalContext.current
    val backgroundCandidates = remember(movie.id, movie.poster, movie.background, movie.seasonPosters, seasonPostersOnHero) {
        mobileHeroArtworkCandidates(movie, seasonPostersOnHero)
    }
    var backgroundIndex by remember(movie.id) { mutableIntStateOf(0) }
    LaunchedEffect(movie.id, backgroundCandidates) {
        backgroundIndex = 0
    }
    val backgroundUrl = backgroundCandidates.getOrNull(backgroundIndex)
    val backgroundRequest = remember(context, backgroundUrl) {
        ImageRequest.Builder(context)
            .data(backgroundUrl)
            .crossfade(false)
            .memoryCacheKey("home-hero-bg:$backgroundUrl")
            .diskCacheKey(backgroundUrl)
            .size(960, 640)
            .build()
    }

    Box(modifier = Modifier.fillMaxSize()) {
        AsyncImage(
            model = backgroundRequest,
            contentDescription = null,
            onError = {
                backgroundIndex = backgroundCandidates.size
            },
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop,
            alignment = Alignment.Center
        )

        Box(modifier = Modifier.fillMaxSize().background(HERO_GRADIENT))
        val ambient = rememberAmbientHeroColor(backgroundUrl)
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        0.55f to Color.Transparent,
                        0.84f to ambient.copy(alpha = 0.40f),
                        1f to Color.Transparent
                    )
                )
        )
    }
}

@Composable
private fun MobileHeroForeground(
    movie: Meta,
    logoUrl: String?,
    lang: String,
    isWatchlisted: Boolean,
    onPlayClick: () -> Unit,
    onInfoClick: () -> Unit,
    onToggleWatchlist: () -> Unit
) {
    val context = LocalContext.current
    val logoCandidates = remember(movie.id, logoUrl, movie.logo) {
        listOfNotNull(logoUrl, movie.logo).distinct()
    }
    var logoIndex by remember(movie.id) { mutableIntStateOf(0) }
    val heroLogo = logoCandidates.getOrNull(logoIndex)
    var showTextFallback by remember(movie.id) { mutableStateOf(logoCandidates.isEmpty()) }
    LaunchedEffect(movie.id, logoCandidates) {
        logoIndex = 0
        showTextFallback = logoCandidates.isEmpty()
    }
    val heroLogoRequest = remember(context, heroLogo) {
        heroLogo
            ?.takeIf { it.isNotBlank() }
            ?.let {
                ImageRequest.Builder(context)
                    .data(it)
                    .crossfade(false)
                    .memoryCacheKey("home-hero-logo:$it")
                    .diskCacheKey(it)
                    .build()
            }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .statusBarsPadding()
            .padding(horizontal = 20.dp, vertical = 14.dp)
    ) {
        Column(
            modifier = Modifier.fillMaxSize()
        ) {
            Spacer(Modifier.height(120.dp))
            Spacer(Modifier.weight(1f))
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 8.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 8.dp),
                    contentAlignment = Alignment.Center
                ) {
                    if (heroLogoRequest != null && !showTextFallback) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(0.70f)
                                .height(98.dp)
                                .padding(vertical = 14.dp)
                        ) {
                            AsyncImage(
                                model = heroLogoRequest,
                                contentDescription = null,
                                onSuccess = { showTextFallback = false },
                                onError = {
                                    showTextFallback = true
                                },
                                modifier = Modifier.matchParentSize(),
                                contentScale = ContentScale.Fit
                            )
                        }
                    } else {
                        Text(
                            text = movie.name,
                            color = Color.White,
                            fontSize = 40.sp,
                            lineHeight = 42.sp,
                            fontWeight = FontWeight.Black,
                            maxLines = 2
                        )
                    }
                        }
                    }

                    val heroTags = remember(movie.id, movie.type, movie.genres, movie.ageRating, lang) {
                        val typeLabel = if (movie.type == "movie") AppStrings.t(lang, "auto.movie") else AppStrings.t(lang, "auto.series")
                        val primaryGenre = movie.genres?.firstOrNull()?.takeIf { it.isNotBlank() }
                        val cert = movie.ageRating?.takeIf { it.isNotBlank() }
                        listOfNotNull(typeLabel, primaryGenre, cert).distinct().take(3)
                    }

                    if (heroTags.isNotEmpty()) {
                        Spacer(Modifier.height(10.dp))
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.Center,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            heroTags.forEachIndexed { index, tag ->
                                if (index > 0) {
                                    Spacer(Modifier.width(10.dp))
                                    Box(
                                        modifier = Modifier
                                            .width(1.dp)
                                            .height(14.dp)
                                            .background(Color.White.copy(alpha = 0.62f))
                                    )
                                    Spacer(Modifier.width(10.dp))
                                }
                                Text(
                                    text = tag,
                                    color = Color.White.copy(alpha = 0.95f),
                                    fontSize = 15.sp,
                                    fontWeight = FontWeight.Black,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                            }
                        }
                    }

                    Spacer(Modifier.height(14.dp))

                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.Center
                    ) {
                        Box(
                            modifier = Modifier.weight(1f),
                            contentAlignment = Alignment.Center
                        ) {
                            MobileHeroMiniAction(
                                icon = if (isWatchlisted) FluxaIcons.Check else FluxaIcons.Add,
                                label = AppStrings.t(lang, "auto.my_list"),
                                onClick = onToggleWatchlist
                            )
                        }
                        Box(
                            modifier = Modifier.weight(1f),
                            contentAlignment = Alignment.Center
                        ) {
                            MobileHeroPlayButton(
                                label = AppStrings.t(lang, "common.play"),
                                onClick = onPlayClick
                            )
                        }
                        Box(
                            modifier = Modifier.weight(1f),
                            contentAlignment = Alignment.Center
                        ) {
                            MobileHeroMiniAction(
                                icon = FluxaIcons.Info,
                                label = AppStrings.t(lang, "auto.info"),
                                onClick = onInfoClick
                            )
                        }
                    }
                    Spacer(Modifier.height(48.dp))
                }
            }
        }
}

@Composable
private fun MobileHeroDots(
    modifier: Modifier = Modifier,
    activeIndex: Int? = null,
    pagerState: androidx.compose.foundation.pager.PagerState? = null,
    count: Int
) {
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        repeat(count) { page ->
            val selected = activeIndex?.let { it == page } ?: (pagerState?.currentPage == page)
            val distance = if (activeIndex != null) {
                if (selected) 0f else 1f
            } else pagerState?.let {
                kotlin.math.abs((it.currentPage - page) + it.currentPageOffsetFraction)
            } ?: if (selected) 0f else 1f
            val width = androidx.compose.ui.unit.lerp(22.dp, 8.dp, distance.coerceIn(0f, 1f))
            val alpha = 1f - (0.66f * distance.coerceIn(0f, 1f))
            Box(
                modifier = Modifier
                    .width(width)
                    .height(8.dp)
                    .clip(CircleShape)
                    .background(Color.White.copy(alpha = alpha.coerceIn(0.34f, 0.96f)))
            )
        }
    }
}

internal fun formatBillboardRuntime(runtime: String, lang: String): String {
    return AppStrings.runtimeLabel(lang, runtime)
}

internal fun Int.floorMod(size: Int): Int {
    if (size <= 0) return 0
    return ((this % size) + size) % size
}

internal fun shortestPageDelta(from: Int, to: Int, size: Int): Int {
    if (size <= 1) return to - from
    val forward = (to - from).floorMod(size)
    val backward = forward - size
    return if (kotlin.math.abs(forward) <= kotlin.math.abs(backward)) forward else backward
}

internal fun String?.toBillboardRuntimeLabel(lang: String): String? {
    val normalized = this?.trim()?.takeIf { it.isNotEmpty() } ?: return null
    if (normalized.equals("N/A", ignoreCase = true)) return null
    val formatted = formatBillboardRuntime(normalized, lang)
    return formatted.takeIf { it.isNotBlank() && !it.equals("N/A", ignoreCase = true) }
}


@Composable
private fun MobileHeroPlayButton(label: String, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .clip(RoundedCornerShape(10.dp))
            .background(Color.White)
            .clickable { onClick() }
            .padding(horizontal = 18.dp, vertical = 9.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        Icon(FluxaIcons.PlayArrow, null, tint = Color.Black, modifier = Modifier.size(18.dp))
        Text(label, color = Color.Black, fontSize = 16.sp, fontWeight = FontWeight.Bold, maxLines = 1)
    }
}

@Composable
private fun MobileHeroMiniAction(icon: ImageVector, label: String, onClick: () -> Unit) {
    Column(
        modifier = Modifier.clickable { onClick() },
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(2.dp)
    ) {
        Icon(
            icon,
            null,
            tint = Color.White,
            modifier = Modifier.size(24.dp)
        )
        Text(
            label,
            color = Color.White,
            fontSize = 12.sp,
            fontWeight = FontWeight.Black
        )
    }
}
