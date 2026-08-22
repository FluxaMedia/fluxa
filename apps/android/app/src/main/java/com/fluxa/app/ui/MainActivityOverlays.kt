@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.ui

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import com.fluxa.app.BuildConfig
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.ui.catalog.UpdateManager

@Composable
internal fun AppUpdateOverlay(
    update: UpdateManager.UpdateInfo?,
    deviceType: DeviceType,
    activeProfile: UserProfile?,
    isDownloading: Boolean,
    downloadProgress: Float,
    onUpdateNow: () -> Unit,
    onSkip: () -> Unit
) {
    update ?: return
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.85f))
            .zIndex(1000f)
            .clickable(
                interactionSource = remember { MutableInteractionSource() },
                indication = null,
                onClick = {}
            ),
        contentAlignment = Alignment.Center
    ) {
        Column(
            modifier = Modifier
                .width(if (deviceType == DeviceType.TV) 500.dp else 340.dp)
                .clip(RoundedCornerShape(24.dp))
                .background(Color(0xFF1A1A1A))
                .border(1.dp, Color.White.copy(alpha = 0.1f), RoundedCornerShape(24.dp))
                .padding(32.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Icon(FluxaIcons.SystemUpdate, null, tint = Color.White, modifier = Modifier.size(64.dp))
            Spacer(Modifier.height(24.dp))
            Text(AppStrings.t(activeProfile?.safeLanguage, "update.available"), color = Color.White, fontSize = 24.sp, fontWeight = FontWeight.Bold)
            Text(
                "${BuildConfig.VERSION_NAME} → ${update.versionName}",
                color = Color.Gray,
                fontSize = 14.sp
            )
            Spacer(Modifier.height(16.dp))
            Text(
                text = update.releaseNotes?.let(::formatReleaseNotes) ?: AppStrings.t(activeProfile?.safeLanguage, "update.default_notes"),
                color = Color.White.copy(alpha = 0.7f),
                textAlign = TextAlign.Start,
                fontSize = 14.sp,
                modifier = Modifier
                    .heightIn(max = 180.dp)
                    .verticalScroll(rememberScrollState())
            )
            Spacer(Modifier.height(32.dp))
            if (isDownloading) {
                val animatedProgress by animateFloatAsState(downloadProgress, label = "progress")
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    androidx.compose.material3.LinearProgressIndicator(
                        progress = { animatedProgress },
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(8.dp)
                            .clip(CircleShape),
                        color = Color.White,
                        trackColor = Color.White.copy(alpha = 0.1f)
                    )
                    Spacer(Modifier.height(12.dp))
                    Text(
                        AppStrings.format(activeProfile?.safeLanguage, "update.downloading_percent", (animatedProgress * 100).toInt()),
                        color = Color.White,
                        fontWeight = FontWeight.Bold
                    )
                }
            } else {
                Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    if (deviceType == DeviceType.TV) {
                        val updateNowFocusRequester = remember { FocusRequester() }
                        LaunchedEffect(update) { updateNowFocusRequester.requestFocus() }
                        androidx.tv.material3.Button(
                            onClick = onUpdateNow,
                            modifier = Modifier
                                .weight(1f)
                                .height(54.dp)
                                .focusRequester(updateNowFocusRequester),
                            colors = androidx.tv.material3.ButtonDefaults.colors(containerColor = Color.White, contentColor = Color.Black),
                            shape = androidx.tv.material3.ButtonDefaults.shape(RoundedCornerShape(12.dp))
                        ) {
                            Text(AppStrings.t(activeProfile?.safeLanguage, "update.update_now"), fontWeight = FontWeight.Bold)
                        }
                        androidx.tv.material3.Button(
                            onClick = onSkip,
                            modifier = Modifier.height(54.dp),
                            colors = androidx.tv.material3.ButtonDefaults.colors(containerColor = Color.White.copy(alpha = 0.05f)),
                            shape = androidx.tv.material3.ButtonDefaults.shape(RoundedCornerShape(12.dp))
                        ) {
                            Text(AppStrings.t(activeProfile?.safeLanguage, "update.skip"), color = Color.Gray)
                        }
                    } else {
                        androidx.compose.material3.Button(
                            onClick = onUpdateNow,
                            modifier = Modifier
                                .weight(1f)
                                .height(54.dp),
                            colors = androidx.compose.material3.ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black),
                            shape = RoundedCornerShape(12.dp)
                        ) {
                            Text(AppStrings.t(activeProfile?.safeLanguage, "update.update_now"), fontWeight = FontWeight.Bold)
                        }
                        androidx.compose.material3.Button(
                            onClick = onSkip,
                            modifier = Modifier.height(54.dp),
                            colors = androidx.compose.material3.ButtonDefaults.buttonColors(containerColor = Color.White.copy(alpha = 0.1f)),
                            shape = RoundedCornerShape(12.dp)
                        ) {
                            Text(AppStrings.t(activeProfile?.safeLanguage, "update.skip"), color = Color.Gray)
                        }
                    }
                }
            }
        }
    }
}

private fun formatReleaseNotes(raw: String): String = raw
    .lines()
    .joinToString("\n") { line ->
        val trimmed = line.trim()
        when {
            trimmed.startsWith("- ") || trimmed.startsWith("* ") -> "•  ${trimmed.drop(2)}"
            trimmed.startsWith("#") -> trimmed.trimStart('#').trim()
            else -> trimmed
        }
    }
    .trim()

@Composable
internal fun DirectLoadingOverlay(visible: Boolean) {
    if (!visible) return
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.52f))
            .zIndex(100f),
        contentAlignment = Alignment.Center
    ) {
        CircularProgressIndicator(color = Color.White)
    }
}
