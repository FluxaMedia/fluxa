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
internal fun MobileDetailInfoSection(
    detail: MetaDetail?,
    type: String,
    lang: String,
    isLoading: Boolean = false,
    selectedEpisode: Video?,
    selectedSeason: Int,
    titleText: String,
    scheduleLabel: String?,
    descriptionExpanded: Boolean,
    onDescriptionExpandedChange: (Boolean) -> Unit,
    onPlayPrimary: () -> Unit,
    isInWatchlist: Boolean,
    onToggleWatchlist: () -> Unit,
    onFeedback: (Boolean) -> Unit,
    accentColor: Color = Color(0xFFE50914)
) {
    Column(modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp)) {
                    val logoUrl = detail?.logo?.takeIf { it.isNotBlank() }
                    var logoLoadFailed by remember(logoUrl) { mutableStateOf(false) }
                    when {
                        isLoading && detail == null -> {
                            Box(
                                modifier = Modifier.height(48.dp),
                                contentAlignment = Alignment.CenterStart
                            ) {
                                CircularProgressIndicator(
                                    color = Color.White.copy(alpha = 0.7f),
                                    strokeWidth = 2.dp,
                                    modifier = Modifier.size(28.dp)
                                )
                            }
                        }
                        logoUrl != null && !logoLoadFailed -> {
                            AsyncImage(
                                model = logoUrl,
                                contentDescription = null,
                                modifier = Modifier.height(72.dp).widthIn(max = 320.dp),
                                contentScale = ContentScale.Fit,
                                alignment = Alignment.CenterStart,
                                onError = { logoLoadFailed = true }
                            )
                        }
                        else -> {
                            Text(titleText, color = Color.White, fontSize = 30.sp, lineHeight = 32.sp, fontWeight = FontWeight.Black)
                        }
                    }
                    scheduleLabel?.let {
                        Spacer(Modifier.height(8.dp))
                        Text(
                            text = it,
                            color = Color(0xFF45D483),
                            fontSize = 13.sp,
                            fontWeight = FontWeight.SemiBold
                        )
                    }
                    Spacer(Modifier.height(8.dp))

                    val metaBits = buildList {
                        detail?.releaseInfo?.let { add(it) }
                        detail?.ageRating?.let { add(it) }
                        if (detail?.type == "series") detail.seasonsCount?.let { add("$it ${AppStrings.t(lang, "auto.seasons")}") }
                        detail?.runtime?.let { add(formatRuntimeLabel(it, lang)) }
                    }
                    if (metaBits.isNotEmpty()) {
                        Text(
                            metaBits.joinToString("  "),
                            color = Color.White.copy(alpha = 0.82f),
                            fontSize = 13.sp,
                            fontWeight = FontWeight.SemiBold
                        )
                    }

                    detail?.genres?.takeIf { it.isNotEmpty() }?.let { genres ->
                        Spacer(Modifier.height(8.dp))
                        Text(
                            genres.joinToString("    "),
                            color = Color.White.copy(alpha = 0.65f),
                            fontSize = 13.sp
                        )
                    }

                    detail?.type?.let { contentType ->
                        Spacer(Modifier.height(6.dp))
                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(6.dp))
                                .background(accentColor.copy(alpha = 0.15f))
                                .padding(horizontal = 8.dp, vertical = 3.dp)
                        ) {
                            Text(
                                text = if (contentType == "series") AppStrings.t(lang, "auto.series_665fd5b7") else AppStrings.t(lang, "auto.movie_c1754804"),
                                color = accentColor,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold
                            )
                        }
                    }

                    detail?.ratings?.takeIf { it.isNotEmpty() }?.let { ratings ->
                        Spacer(Modifier.height(12.dp))
                        LazyRow(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            mobileItems(ratings) { rating ->
                                val ratingText = rating.value?.toString().orEmpty()
                                if (ratingText.isNotBlank()) OfficialRatingBadge(rating.source, ratingText)
                            }
                        }
                    }

                    detail?.awards?.takeIf { it.isNotBlank() }?.let { awards ->
                        Spacer(Modifier.height(8.dp))
                        Text(
                            text = awards,
                            color = Color(0xFFFFD700).copy(alpha = 0.85f),
                            fontSize = 12.sp,
                            lineHeight = 17.sp
                        )
                    }

                    Spacer(Modifier.height(14.dp))

                    val playLabel = if (type == "series" && selectedEpisode != null) {
                        val s = selectedEpisode.season ?: selectedSeason
                        val e = selectedEpisode.number
                        val epName = selectedEpisode.name
                        when {
                            e != null && !epName.isNullOrBlank() ->
                                AppStrings.format(lang, "format.play_series_episode", s, e, epName)
                            e != null ->
                                AppStrings.format(lang, "format.play_series_episode_no_name", s, e)
                            else -> AppStrings.t(lang, "common.play")
                        }
                    } else {
                        AppStrings.t(lang, "common.play")
                    }
                    MobilePrimaryButton(
                        label = playLabel,
                        onClick = onPlayPrimary
                    )

                    Spacer(Modifier.height(10.dp))

                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(10.dp))
                            .background(Color.White.copy(alpha = 0.10f))
                            .clickable { onToggleWatchlist() }
                            .padding(vertical = 12.dp),
                        horizontalArrangement = Arrangement.Center,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        androidx.compose.material3.Icon(
                            imageVector = if (isInWatchlist) FluxaIcons.Check else FluxaIcons.Add,
                            contentDescription = null,
                            tint = Color.White,
                            modifier = Modifier.size(20.dp)
                        )
                        Spacer(Modifier.width(6.dp))
                        Text(AppStrings.t(lang, if (isInWatchlist) "auto.in_list" else "auto.my_list"), color = Color.White, fontSize = 16.sp, fontWeight = FontWeight.Bold)
                    }

                    Spacer(Modifier.height(14.dp))

                    val activeOverview = detail?.description.orEmpty()
                    var overviewWasTruncated by remember(activeOverview) { mutableStateOf(false) }
                    Text(
                        text = activeOverview,
                        color = Color.White.copy(alpha = 0.88f),
                        fontSize = 14.sp,
                        lineHeight = 21.sp,
                        maxLines = if (descriptionExpanded) Int.MAX_VALUE else 4,
                        overflow = TextOverflow.Ellipsis,
                        onTextLayout = { layoutResult ->
                            if (!descriptionExpanded) {
                                overviewWasTruncated = layoutResult.hasVisualOverflow
                            }
                        }
                    )
                    if (overviewWasTruncated || descriptionExpanded) {
                        Text(
                            text = if (descriptionExpanded) {
                                AppStrings.t(lang, "auto.show_less")
                            } else {
                                AppStrings.t(lang, "auto.read_more_704a4641")
                            },
                            color = Color.White.copy(alpha = 0.68f),
                            fontSize = 13.sp,
                            fontWeight = FontWeight.SemiBold,
                            modifier = Modifier
                                .padding(top = 8.dp)
                                .clickable { onDescriptionExpandedChange(!descriptionExpanded) }
                        )
                    }

                    val creditsLine = buildString {
                        detail?.director?.take(2)?.joinToString(", ")?.let {
                            append(AppStrings.format(lang, "format.creator_line", it))
                        }
                    }
                    if (creditsLine.isNotBlank()) {
                        Spacer(Modifier.height(10.dp))
                        Text(
                            text = creditsLine,
                            color = Color.White.copy(alpha = 0.58f),
                            fontSize = 12.sp,
                            lineHeight = 18.sp
                        )
                    }

                    detail?.cast?.takeIf { it.isNotEmpty() }?.let { cast ->
                        Spacer(Modifier.height(14.dp))
                        LazyRow(
                            horizontalArrangement = Arrangement.spacedBy(16.dp),
                            contentPadding = PaddingValues(end = 8.dp)
                        ) {
                            mobileItems(cast.take(12)) { member ->
                                CastMemberCard(member)
                            }
                        }
                    }

                }
}
