@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

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
import androidx.compose.foundation.lazy.items as mobileItems
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
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.*

@Composable
internal fun MobileDetailOverlays(
    showSeasonSelector: Boolean,
    seasons: List<Int>,
    selectedSeason: Int,
    lang: String,
    onSelectSeason: (Int) -> Unit,
    onDismissSeasonSelector: () -> Unit,
    episodeDrawer: Video?,
    actionEpisodes: List<Video>,
    watchedVideoIds: Set<String>,
    runtimeLabel: String?,
    onDismissEpisodeDrawer: () -> Unit,
    onMarkEpisodeWatched: (Video, Boolean) -> Unit,
    onSetEpisodesWatched: (List<Video>, Boolean) -> Unit,
    onDownloadEpisode: (Video?) -> Unit,
    seasonDrawer: MobileSeasonActionTarget?,
    onDismissSeasonDrawer: () -> Unit,
    isLoading: Boolean,
    detail: MetaDetail?,
    accentColor: Color = Color.White
) {
    if (showSeasonSelector) {
            ExploreOptionSelector(
                title = AppStrings.t(lang, "auto.select_season"),
                options = seasons.map { season ->
                    season.toString() to if (season == 0) {
                        AppStrings.t(lang, "auto.specials")
                    } else {
                        AppStrings.format(lang, "format.season_number", season)
                    }
                },
                selected = selectedSeason.toString(),
                onSelect = {
                    onSelectSeason(it?.toIntOrNull() ?: 1)
                    onDismissSeasonSelector()
                },
                onDismiss = { onDismissSeasonSelector() },
                accentColor = accentColor
            )
        }

        episodeDrawer?.let { episode ->
            val episodesThroughThisEpisode = remember(actionEpisodes, episode.id, selectedSeason) {
                episodesThroughEpisode(actionEpisodes, episode, selectedSeason)
            }
            val isThroughWatched = episodesThroughThisEpisode.isNotEmpty() &&
                episodesThroughThisEpisode.all { watchedVideoIds.contains(it.id) }
            MobileEpisodeActionsSheet(
                episode = episode,
                lang = lang,
                isWatched = watchedVideoIds.contains(episode.id),
                isThroughWatched = isThroughWatched,
                hasThroughEpisodes = episodesThroughThisEpisode.isNotEmpty(),
                durationLabel = runtimeLabel,
                onDismiss = { onDismissEpisodeDrawer() },
                onToggleWatched = {
                    onMarkEpisodeWatched(episode, !watchedVideoIds.contains(episode.id))
                    onDismissEpisodeDrawer()
                },
                onToggleThroughWatched = {
                    onSetEpisodesWatched(episodesThroughThisEpisode, !isThroughWatched)
                    onDismissEpisodeDrawer()
                }
            )
        }

        seasonDrawer?.let { target ->
            MobileSeasonActionsSheet(
                season = target.season,
                lang = lang,
                isSeasonWatched = target.seasonEpisodes.isNotEmpty() &&
                    target.seasonEpisodes.all { watchedVideoIds.contains(it.id) },
                isThroughWatched = target.throughEpisodes.isNotEmpty() &&
                    target.throughEpisodes.all { watchedVideoIds.contains(it.id) },
                hasSeasonEpisodes = target.seasonEpisodes.isNotEmpty(),
                hasThroughEpisodes = target.throughEpisodes.isNotEmpty(),
                onDismiss = { onDismissSeasonDrawer() },
                onToggleSeasonWatched = {
                    onSetEpisodesWatched(target.seasonEpisodes, !target.seasonEpisodes.all { watchedVideoIds.contains(it.id) })
                    onDismissSeasonDrawer()
                },
                onToggleThroughWatched = {
                    onSetEpisodesWatched(target.throughEpisodes, !target.throughEpisodes.all { watchedVideoIds.contains(it.id) })
                    onDismissSeasonDrawer()
                }
            )
        }

        if (isLoading && detail == null) {
            Box(
                modifier = Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.38f)),
                contentAlignment = Alignment.Center
            ) {
                CircularProgressIndicator(color = Color.White)
            }
        }
}
