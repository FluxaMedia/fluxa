@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.ui.catalog

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import androidx.tv.material3.Text
import coil3.compose.AsyncImage
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta

@Composable
internal fun ContinueWatchingHighlight(
    movie: Meta,
    lang: String,
    label: String,
    focusRequester: FocusRequester? = null,
    upFocusRequester: FocusRequester? = null,
    onClick: () -> Unit,
    onFocus: () -> Unit
) {
    Column(modifier = Modifier.padding(start = 42.dp, top = 12.dp, bottom = 8.dp)) {
        Text(
            text = label,
            color = Color.White,
            fontSize = 30.sp,
            fontWeight = FontWeight.SemiBold
        )
        Spacer(modifier = Modifier.height(14.dp))
        WideContinueCard(
            movie = movie,
            lang = lang,
            focusRequester = focusRequester,
            upFocusRequester = upFocusRequester,
            onClick = onClick,
            onFocus = onFocus
        )
    }
}

@Composable
private fun WideContinueCard(
    movie: Meta,
    lang: String,
    focusRequester: FocusRequester? = null,
    upFocusRequester: FocusRequester? = null,
    onClick: () -> Unit,
    onFocus: () -> Unit
) {
    var isFocused by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isFocused) 1.035f else 1f, tween(180), label = "continueScale")

    Row(
        modifier = Modifier
            .width(360.dp)
            .clip(RoundedCornerShape(18.dp))
            .background(Color.White.copy(alpha = 0.07f))
            .border(
                width = if (isFocused) 2.dp else 1.dp,
                color = if (isFocused) Color(0xFF9AD4FF) else Color.White.copy(alpha = 0.08f),
                shape = RoundedCornerShape(18.dp)
            )
            .scale(scale)
            .then(
                if (focusRequester != null || upFocusRequester != null) {
                    Modifier
                        .let { base -> if (focusRequester != null) base.focusRequester(focusRequester) else base }
                        .focusProperties {
                            if (upFocusRequester != null) up = upFocusRequester
                        }
                } else {
                    Modifier
                }
            )
            .onFocusChanged {
                isFocused = it.isFocused
                if (it.isFocused) onFocus()
            }
            .clickable { onClick() }
            .focusable()
            .padding(8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        AsyncImage(
            model = movie.continueWatchingBackground,
            contentDescription = movie.name,
            modifier = Modifier
                .width(156.dp)
                .height(88.dp)
                .clip(RoundedCornerShape(14.dp)),
            contentScale = ContentScale.Crop
        )
        Spacer(modifier = Modifier.width(14.dp))
        Column(modifier = Modifier.weight(1f)) {
            movie.lastEpisodeName?.takeIf { it.isNotBlank() }?.let {
                Text(
                    text = it,
                    color = Color.White.copy(alpha = 0.72f),
                    fontSize = 13.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Spacer(modifier = Modifier.height(2.dp))
            }
            Text(
                text = movie.name,
                color = Color.White,
                fontSize = 23.sp,
                fontWeight = FontWeight.Medium,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )
            Spacer(modifier = Modifier.height(8.dp))
            val remaining = movie.remainingWatchLabel(lang)
            if (remaining != null) {
                HeroBadge(text = remaining, background = Color(0xFF4A6AFB), content = Color.White)
            }
        }
    }
}

@Composable
internal fun HomeShelfRow(
    title: String,
    items: List<Meta>,
    cardLayout: String,
    artworkPreference: String?,
    activeProfile: UserProfile?,
    topTenEnabled: Boolean,
    onItemClick: (Meta) -> Unit,
    onItemFocus: (Meta) -> Unit,
    firstItemFocusRequester: FocusRequester? = null,
    upFocusRequester: FocusRequester? = null,
    onNeedMore: () -> Unit
) {
    Column(modifier = Modifier.padding(top = 18.dp)) {
        Text(
            text = title,
            color = Color.White,
            fontSize = 22.sp,
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.padding(start = 42.dp, bottom = 12.dp)
        )
        val rowHeight = when (cardLayout) {
            "episode" -> 226.dp
            "horizontal" -> horizontalCardHeight(activeProfile?.safePosterWidthPreset ?: "medium", DeviceType.TV) + 24.dp
            else -> posterCardHeight(activeProfile?.safePosterWidthPreset ?: "medium") + 82.dp
        }
        LazyRow(
            modifier = Modifier.height(rowHeight),
            contentPadding = PaddingValues(start = 42.dp, end = 50.dp),
            horizontalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            itemsIndexed(items, key = { _, movie -> "${movie.type}:${movie.id}" }) { index, movie ->
                if (index >= items.lastIndex - 4) {
                    LaunchedEffect(title, index) { onNeedMore() }
                }
                ShelfCard(
                    movie = movie,
                    cardLayout = cardLayout,
                    artworkPreference = artworkPreference,
                    activeProfile = activeProfile,
                    topTenRank = if (topTenEnabled && index < 10) index + 1 else null,
                    focusRequester = if (index == 0) firstItemFocusRequester else null,
                    upFocusRequester = if (index == 0) upFocusRequester else null,
                    onClick = { onItemClick(movie) },
                    onFocus = { onItemFocus(movie) }
                )
            }
        }
    }
}

@Composable
private fun ShelfCard(
    movie: Meta,
    cardLayout: String,
    artworkPreference: String?,
    activeProfile: UserProfile?,
    topTenRank: Int? = null,
    focusRequester: FocusRequester? = null,
    upFocusRequester: FocusRequester? = null,
    onClick: () -> Unit,
    onFocus: () -> Unit
) {
    var isFocused by remember { mutableStateOf(false) }
    val animationDuration = when {
        activeProfile?.safeAnimationsEnabled == false -> 0
        else -> 220
    }
    val scale by animateFloatAsState(
        if (isFocused) 1.08f else 1f,
        tween(animationDuration),
        label = "shelfScale"
    )

    val cardWidth = when (cardLayout) {
        "episode" -> 336.dp
        "horizontal" -> horizontalCardWidth(activeProfile?.safePosterWidthPreset ?: "medium", DeviceType.TV)
        else -> posterCardWidth(activeProfile?.safePosterWidthPreset ?: "medium") + 42.dp
    }
    val cardHeight = when (cardLayout) {
        "episode" -> 210.dp
        "horizontal" -> horizontalCardHeight(activeProfile?.safePosterWidthPreset ?: "medium", DeviceType.TV)
        else -> posterCardHeight(activeProfile?.safePosterWidthPreset ?: "medium") + 64.dp
    }

    val isWideTopTenCard = cardLayout == "horizontal" || cardLayout == "episode"
    val rankNumberBoxWidth = when {
        topTenRank == null -> 0.dp
        topTenRank >= 10 -> if (isWideTopTenCard) cardWidth * 0.98f else cardWidth * 1.18f
        topTenRank == 1 -> if (isWideTopTenCard) cardWidth * 0.46f else cardWidth * 0.54f
        else -> if (isWideTopTenCard) cardWidth * 0.72f else cardWidth * 0.86f
    }
    val rankPosterOverlap = when {
        topTenRank == null -> 0.dp
        topTenRank >= 10 -> if (isWideTopTenCard) cardWidth * 0.30f else cardWidth * 0.34f
        topTenRank == 1 -> if (isWideTopTenCard) cardWidth * 0.12f else cardWidth * 0.14f
        else -> if (isWideTopTenCard) cardWidth * 0.22f else cardWidth * 0.25f
    }
    val outerWidth = if (topTenRank != null) {
        rankNumberBoxWidth + cardWidth - rankPosterOverlap
    } else {
        cardWidth
    }
    val topTenFontSize = when {
        !isWideTopTenCard -> 226.sp
        cardLayout == "episode" -> 158.sp
        else -> 150.sp
    }
    val topTenNumberYOffset = if (isWideTopTenCard) 1.dp else 2.dp

    Box(
        modifier = Modifier
            .width(outerWidth)
            .height(cardHeight)
            .scale(scale)
            .then(
                if (focusRequester != null || upFocusRequester != null) {
                    Modifier
                        .let { base -> if (focusRequester != null) base.focusRequester(focusRequester) else base }
                        .focusProperties {
                            if (upFocusRequester != null) up = upFocusRequester
                        }
                } else {
                    Modifier
                }
            )
            .onFocusChanged {
                isFocused = it.isFocused
                if (it.isFocused) onFocus()
            }
            .clickable { onClick() }
            .focusable()
    ) {
        topTenRank?.let { rank ->
            Box(
                modifier = Modifier
                    .align(Alignment.TopStart)
                    .width(rankNumberBoxWidth)
                    .height(cardHeight)
                    .zIndex(0f),
                contentAlignment = Alignment.BottomEnd
            ) {
                TopTenRankNumber(
                    rank = rank,
                    fontSize = topTenFontSize,
                    modifier = Modifier.offset(
                        x = when {
                            rank == 1 -> 8.dp
                            rank >= 10 -> 0.dp
                            else -> 3.dp
                        },
                        y = topTenNumberYOffset
                    )
                )
            }
        }

        Box(
            modifier = Modifier
                .align(Alignment.TopEnd)
                .width(cardWidth)
                .height(cardHeight)
                .clip(RoundedCornerShape(posterCornerRadius(activeProfile?.safeCardCornerPreset ?: "soft")))
                .background(Color.White.copy(alpha = 0.03f))
                .zIndex(1f)
        ) {
            if (cardLayout == "episode") {
                MovieCardContent(
                    movie = movie,
                    isUpcoming = false,
                    isFocused = isFocused,
                    cardLayout = cardLayout,
                    artworkPreference = artworkPreference,
                    cornerRadius = posterCornerRadius(activeProfile?.safeCardCornerPreset ?: "soft"),
                    hideTitles = activeProfile?.safePosterHideTitles == true
                )
            } else {
                val imageCandidates = remember(movie.id, movie.poster, movie.background, cardLayout) {
                    if (cardLayout == "horizontal") horizontalArtworkCandidates(movie) else listOfNotNull(movie.poster)
                }
                var imageIndex by remember(movie.id, movie.poster, movie.background, cardLayout) { mutableIntStateOf(0) }
                AsyncImage(
                    model = imageCandidates.getOrNull(imageIndex),
                    contentDescription = movie.name,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                    onError = {
                        imageIndex = imageCandidates.size
                    }
                )
                if (activeProfile?.safePosterHideTitles != true) Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .align(Alignment.BottomStart)
                        .background(
                            Brush.verticalGradient(
                                colors = listOf(
                                    Color.Transparent,
                                    Color.Black.copy(alpha = 0.86f)
                                )
                            )
                        )
                        .padding(horizontal = 12.dp, vertical = 10.dp)
                ) {
                    Text(
                        text = movie.name,
                        color = Color.White,
                        fontSize = if (cardLayout == "horizontal") 16.sp else 15.sp,
                        fontWeight = if (isFocused) FontWeight.SemiBold else FontWeight.Medium,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }

            if (isFocused) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .border(3.dp, Color.White, RoundedCornerShape(posterCornerRadius(activeProfile?.safeCardCornerPreset ?: "soft")))
                )
            }
        }
    }
}

internal fun Meta.remainingWatchLabel(lang: String? = null): String? {
    val total = duration ?: return null
    val watched = timeOffset ?: return null
    val remainingMs = (total - watched).coerceAtLeast(0L)
    val mins = (remainingMs / 60000L).toInt()
    if (mins <= 0) return null
    return if (mins >= 60) {
        val hours = mins / 60
        val rest = mins % 60
        if (rest == 0) {
            AppStrings.format(lang, "format.remaining_hours", hours)
        } else {
            AppStrings.format(lang, "format.remaining_hours_minutes", hours, rest)
        }
    } else {
        AppStrings.format(lang, "format.remaining_minutes", mins)
    }
}
