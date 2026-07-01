@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.animation.ExperimentalAnimationApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
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

@Composable
fun TrackSidebar(title: String, tracks: List<MediaTrack>, selected: MediaTrack?, deviceType: DeviceType, lang: String = "en", onSelect: (MediaTrack) -> Unit) {
    PlayerSidebarShell(
        title = title,
        subtitle = AppStrings.t(lang, "player.choose_preferred_source"),
        deviceType = deviceType
    ) {
        LazyColumn(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            items(tracks, key = { it.id }) { track ->
                TrackItem(
                    title = track.label,
                    isSelected = track == selected,
                    onClick = { onSelect(track) },
                    subtitle = track.language?.let { nativeLanguageName(it) },
                    deviceType = deviceType
                )
            }
        }
    }
}

@Composable
fun QuickSettingsSidebar(profile: UserProfile?, onUpdateProfile: (UserProfile) -> Unit, currentOffset: Long, onOffsetChange: (Long) -> Unit, deviceType: DeviceType, lang: String = profile?.safeLanguage ?: "en", onClose: () -> Unit) {
    PlayerSidebarShell(
        title = AppStrings.t(lang, "player.quick_settings_title"),
        subtitle = AppStrings.t(lang, "player.quick_settings_subtitle"),
        deviceType = deviceType,
        onClose = onClose
    ) {
            Text(AppStrings.t(lang, "player.subtitle_sync_title"), color = Color.White.copy(alpha = 0.62f), fontSize = 12.sp, fontWeight = FontWeight.Bold, modifier = Modifier.padding(bottom = 12.dp))
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(22.dp))
                    .background(Color.White.copy(alpha = 0.04f))
                    .border(1.dp, Color.White.copy(alpha = 0.08f), RoundedCornerShape(22.dp))
                    .padding(18.dp)
            ) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                SeekIconButton(FluxaIcons.Remove, deviceType) { onOffsetChange(currentOffset - 500) }
                Text(text = "${if(currentOffset >= 0) "+" else ""}${currentOffset/1000.0}s", color = Color.White, fontSize = 18.sp, fontWeight = FontWeight.Black, modifier = Modifier.weight(1f), textAlign = TextAlign.Center)
                SeekIconButton(FluxaIcons.Add, deviceType) { onOffsetChange(currentOffset + 500) }
            }
            }
            
            Spacer(Modifier.height(8.dp))
    }
}

@Composable
fun PlayerPremiumToggle(title: String, desc: String, isEnabled: Boolean, onToggle: () -> Unit) {
    Box(modifier = Modifier.fillMaxWidth().height(82.dp).clip(RoundedCornerShape(22.dp)).background(Color.White.copy(alpha = 0.05f)).border(1.dp, Color.White.copy(alpha = 0.08f), RoundedCornerShape(22.dp)).clickable { onToggle() }.padding(horizontal = 20.dp), contentAlignment = Alignment.CenterStart) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Column(modifier = Modifier.weight(1f)) {
                Text(title, color = Color.White, fontSize = 16.sp, fontWeight = FontWeight.Bold)
                Text(desc, color = Color.White.copy(alpha = 0.56f), fontSize = 12.sp)
            }
            Switch(checked = isEnabled, onCheckedChange = { onToggle() }, colors = SwitchDefaults.colors(checkedThumbColor = FluxaColors.accent, checkedTrackColor = FluxaColors.accent))
        }
    }
}
