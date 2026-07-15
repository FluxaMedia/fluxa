@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class, androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.fluxa.app.ui

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.ui.catalog.FluxaIcons

import android.content.pm.ActivityInfo
import android.os.Bundle
import android.util.Log
import com.lagradost.cloudstream3.CommonActivity
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.Image
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.Icon
import androidx.compose.material3.TextButton
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.scale
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.viewModelScope
import coil3.compose.AsyncImage
import androidx.compose.ui.layout.ContentScale
import com.fluxa.app.ui.catalog.*
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.R
import androidx.media3.exoplayer.ExoPlayer
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

sealed class Screen {
    object Welcome : Screen()
    object Profiles : Screen()
    data class Login(val startOnNuvio: Boolean = false) : Screen()
    object Home : Screen()
    object Search : Screen()
    object Watchlist : Screen()
    object Calendar : Screen()
    data class Explore(val initialType: String = "movie", val initialGenre: String? = null) : Screen()
    object AddonStore : Screen()
    data class Settings(val initialSection: String? = null) : Screen()
    data class Detail(val type: String, val id: String, val initialProgress: Long? = null, val lastVideoId: String? = null, val lastStreamIndex: Int? = null, val autoPlay: Boolean = false, val targetSeason: Int? = null, val targetEpisode: Int? = null, val lastStreamUrl: String? = null, val lastStreamTitle: String? = null, val sourceAddonTransportUrl: String? = null, val sourceAddonCatalogType: String? = null, val initialMeta: Meta? = null) : Screen()
    data class Player(
        val meta: Meta,
        val videoId: String? = null,
        val initialProgress: Long = 0L,
        val streamIndex: Int = 0,
        val initialStreams: List<Stream> = emptyList(),
        val lastStreamUrl: String? = null,
        val lastStreamTitle: String? = null,
        val preferredBingeGroup: String? = null,
        val returnToSourcesOnError: Boolean = false,
        val showSourceSelection: Boolean = false
    ) : Screen()
}

internal fun navDirection(from: Screen, to: Screen): Int {
    return when {
        from is Screen.Player || to is Screen.Detail -> 1
        to is Screen.Home || to is Screen.Profiles -> -1
        else -> 1
    }
}

internal fun MetaDetail.asNavigationMeta() = Meta(
    id = id,
    name = name,
    type = type,
    poster = poster,
    background = background,
    logo = logo,
    description = description,
    imdbRating = imdbRating,
    releaseInfo = releaseInfo,
    released = released,
    originalLanguage = originalLanguage,
    originalName = originalName,
    videos = videos,
    trailers = trailers
)

@Composable
internal fun TraktIntegrationSheet(
    profile: UserProfile,
    lastContinueWatchingUpdatedAt: Long,
    syncing: Boolean,
    onDismiss: () -> Unit,
    onSyncNow: () -> Unit,
    onDisconnect: () -> Unit
) {
    val lang = profile.safeLanguage
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        containerColor = Color(0xFF111318),
        contentColor = Color.White
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 22.dp, vertical = 10.dp)
                .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Bottom)),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(14.dp)
            ) {
                Box(
                    modifier = Modifier
                        .size(52.dp)
                        .clip(CircleShape)
                        .background(Color.White.copy(alpha = 0.08f))
                        .padding(10.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Image(
                        painter = painterResource(id = R.drawable.ic_trakt),
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize()
                    )
                }
                Column(modifier = Modifier.weight(1f)) {
                    Text(AppStrings.t(lang, "brand.trakt"), color = Color.White, fontSize = 20.sp, fontWeight = FontWeight.Black)
                    Text(
                        text = AppStrings.t(lang, "integration.connected"),
                        color = Color.White.copy(alpha = 0.58f),
                        fontSize = 13.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                }
            }

            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                IntegrationStatRow(
                    label = AppStrings.t(lang, "integration.last_sync"),
                    value = formatIntegrationSyncTime(maxOf(profile.safeTraktLastSyncAt, lastContinueWatchingUpdatedAt), lang)
                )
                IntegrationStatRow(
                    label = AppStrings.t(lang, "integration.synced_data"),
                    value = AppStrings.format(
                        lang,
                        "integration.trakt_data_counts",
                        profile.safeTraktLastSyncedItems,
                        profile.safeTraktLastContinueWatchingCount,
                        profile.safeTraktLastWatchlistCount
                    )
                )
            }

            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                Button(
                    onClick = onSyncNow,
                    enabled = !syncing,
                    modifier = Modifier.weight(1f).height(52.dp),
                    colors = androidx.compose.material3.ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black),
                    shape = RoundedCornerShape(12.dp)
                ) {
                    if (syncing) {
                        CircularProgressIndicator(color = Color.Black, strokeWidth = 2.dp, modifier = Modifier.size(18.dp))
                    } else {
                        Text(AppStrings.t(lang, "integration.sync_now"), fontWeight = FontWeight.Black)
                    }
                }
                Button(
                    onClick = onDisconnect,
                    enabled = !syncing,
                    modifier = Modifier.weight(1f).height(52.dp),
                    colors = androidx.compose.material3.ButtonDefaults.buttonColors(containerColor = Color(0xFFFF4D4D).copy(alpha = 0.18f), contentColor = Color(0xFFFF8A8A)),
                    shape = RoundedCornerShape(12.dp)
                ) {
                    Text(AppStrings.t(lang, "integration.disconnect"), fontWeight = FontWeight.Black)
                }
            }
        }
    }
}

@Composable
internal fun SimpleIntegrationSheet(
    titleKey: String,
    iconRes: Int,
    lang: String,
    onDismiss: () -> Unit,
    onDisconnect: () -> Unit
) {
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        containerColor = Color(0xFF111318),
        contentColor = Color.White
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 22.dp, vertical = 10.dp)
                .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Bottom)),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(14.dp)
            ) {
                Box(
                    modifier = Modifier
                        .size(52.dp)
                        .clip(CircleShape)
                        .background(Color.White.copy(alpha = 0.08f))
                        .padding(10.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Image(
                        painter = painterResource(id = iconRes),
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize()
                    )
                }
                Column(modifier = Modifier.weight(1f)) {
                    Text(AppStrings.t(lang, titleKey), color = Color.White, fontSize = 20.sp, fontWeight = FontWeight.Black)
                    Text(
                        text = AppStrings.t(lang, "integration.connected"),
                        color = Color.White.copy(alpha = 0.58f),
                        fontSize = 13.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                }
            }

            Button(
                onClick = onDisconnect,
                modifier = Modifier.fillMaxWidth().height(52.dp),
                colors = androidx.compose.material3.ButtonDefaults.buttonColors(containerColor = Color(0xFFFF4D4D).copy(alpha = 0.18f), contentColor = Color(0xFFFF8A8A)),
                shape = RoundedCornerShape(12.dp)
            ) {
                Text(AppStrings.t(lang, "integration.disconnect"), fontWeight = FontWeight.Black)
            }
        }
    }
}

@Composable
internal fun IntegrationStatRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(Color.White.copy(alpha = 0.06f))
            .padding(horizontal = 14.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(label, color = Color.White.copy(alpha = 0.58f), fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
        Text(value, color = Color.White, fontSize = 13.sp, fontWeight = FontWeight.Black, textAlign = TextAlign.End)
    }
}

internal fun formatIntegrationSyncTime(timestamp: Long, lang: String): String {
    if (timestamp <= 0L) return AppStrings.t(lang, "integration.never_synced")
    val formatter = java.text.SimpleDateFormat("dd MMM yyyy HH:mm", java.util.Locale.getDefault())
    return formatter.format(java.util.Date(timestamp))
}
