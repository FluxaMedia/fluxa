@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.safeAccentColorArgb
import com.fluxa.app.data.local.safeLanguage
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Video
import com.fluxa.app.shared.feature.player.PlayerSidebarShell
import com.fluxa.app.shared.feature.player.TrackItem

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
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
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun EpisodeSidebar(
    meta: Meta,
    currentId: String,
    deviceType: DeviceType,
    viewModel: HomeViewModel,
    activeProfile: UserProfile?,
    onSelect: (String, String?) -> Unit,
    onClose: (() -> Unit)? = null
) {
    var episodes by remember { mutableStateOf<List<Video>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var showSeasonMenu by remember { mutableStateOf(false) }
    var hasExtras by remember(meta.id) { mutableStateOf(false) }
    val currentSeasonFromId = remember(currentId) {
        val parts = currentId.split(":")
        if (parts.size >= 3) parts[parts.size - 2].toIntOrNull() ?: 1 else 1
    }
    var selectedSeason by remember(currentId) { mutableIntStateOf(currentSeasonFromId) }
    val availableSeasons = remember(meta.seasonsCount, hasExtras, currentSeasonFromId) {
        buildList {
            val count = maxOf(meta.seasonsCount ?: 1, currentSeasonFromId + 2)
            addAll(1..count.coerceAtLeast(1))
            if (hasExtras || currentSeasonFromId == 0) add(0)
        }.distinct().sortedWith(compareBy<Int> { if (it == 0) 1 else 0 }.thenBy { it })
    }

    LaunchedEffect(meta.id) {
        hasExtras = runCatching {
            viewModel.getSeasonEpisodes(meta.id, 0, activeProfile?.safeLanguage ?: "en").isNotEmpty()
        }.getOrDefault(false)
    }

    LaunchedEffect(currentId, selectedSeason) {
        isLoading = true
        episodes = viewModel.getSeasonEpisodes(meta.id, selectedSeason, activeProfile?.safeLanguage ?: "en")
        isLoading = false
    }

    val lang = activeProfile?.safeLanguage ?: "en"
    val accentColor = Color(activeProfile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt())

    PlayerSidebarShell(
        title = AppStrings.t(lang, "auto.episodes"),
        subtitle = "",
        deviceType = deviceType,
        onClose = onClose
    ) {
            Box {
                val chevronRotation by animateFloatAsState(if (showSeasonMenu) 180f else 0f, label = "seasonChevron")
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(14.dp))
                        .background(Color.White.copy(alpha = 0.06f))
                        .clickable { showSeasonMenu = true }
                        .padding(horizontal = 14.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = if (selectedSeason == 0) AppStrings.t(lang, "auto.specials") else AppStrings.format(lang, "format.season_number", selectedSeason),
                        color = Color.White,
                        fontSize = 13.sp,
                        fontWeight = FontWeight.SemiBold,
                        modifier = Modifier.weight(1f)
                    )
                    Icon(
                        FluxaIcons.KeyboardArrowDown,
                        null,
                        tint = Color.White,
                        modifier = Modifier.size(18.dp).rotate(chevronRotation)
                    )
                }
                DropdownMenu(
                    expanded = showSeasonMenu,
                    onDismissRequest = { showSeasonMenu = false },
                    modifier = Modifier.background(Color(0xFF121922))
                ) {
                    availableSeasons.forEach { season ->
                        val isSeasonSelected = season == selectedSeason
                        DropdownMenuItem(
                            text = {
                                Text(
                                    if (season == 0) AppStrings.t(lang, "auto.specials") else AppStrings.format(lang, "format.season_number", season),
                                    color = if (isSeasonSelected) accentColor else Color.White,
                                    fontWeight = if (isSeasonSelected) FontWeight.Bold else FontWeight.Normal,
                                    fontSize = 13.sp
                                )
                            },
                            trailingIcon = if (isSeasonSelected) {
                                { Icon(FluxaIcons.Check, null, tint = accentColor, modifier = Modifier.size(16.dp)) }
                            } else null,
                            onClick = {
                                selectedSeason = season
                                showSeasonMenu = false
                            }
                        )
                    }
                }
            }
            Spacer(Modifier.height(14.dp))
            if (isLoading) {
                Text(AppStrings.t(lang, "auto.loading"), color = Color.Gray)
            } else {
                LazyColumn(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    items(episodes, key = { it.id }) { ep ->
                        if (deviceType == DeviceType.Mobile) {
                            MobilePlayerEpisodeRow(
                                episode = ep,
                                isSelected = ep.id == currentId,
                                lang = lang,
                                accentColor = accentColor,
                                onClick = {
                                    val episodeLabel = buildString {
                                        append("S")
                                        append(ep.season ?: selectedSeason)
                                        append(" E")
                                        append(ep.number ?: 0)
                                        ep.name?.takeIf { it.isNotBlank() }?.let {
                                            append(" ")
                                            append(it.uppercase())
                                        }
                                    }
                                    onSelect(ep.id, episodeLabel)
                                }
                            )
                        } else {
                            TrackItem(
                                modifier = Modifier.animateItem(),
                                title = "${ep.number}. ${ep.name ?: AppStrings.format(lang, "format.episode_number", ep.number ?: 0)}",
                                isSelected = ep.id == currentId,
                                onClick = {
                                    val episodeLabel = buildString {
                                        append("S")
                                        append(ep.season ?: selectedSeason)
                                        append(" E")
                                        append(ep.number ?: 0)
                                        ep.name?.takeIf { it.isNotBlank() }?.let {
                                            append(" ")
                                            append(it.uppercase())
                                        }
                                    }
                                    onSelect(ep.id, episodeLabel)
                                },
                                subtitle = ep.overview?.takeIf { it.isNotBlank() },
                                deviceType = deviceType,
                                leadingIcon = FluxaIcons.Tv
                            )
                        }
                    }
                }
            }
    }
}
