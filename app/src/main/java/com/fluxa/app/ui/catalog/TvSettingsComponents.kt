@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.ui.res.painterResource
import com.fluxa.app.R
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.Surface
import coil3.compose.AsyncImage
import java.util.Locale

@Composable
internal fun SettingsContent(
    tabId: String,
    profile: UserProfile?,
    lang: String,
    onLogout: () -> Unit,
    onConnectStremio: () -> Unit,
    onConnectTrakt: () -> Unit,
    onConnectMal: () -> Unit,
    onConnectSimkl: () -> Unit,
    onManageAddons: () -> Unit,
    onReboot: () -> Unit,
    onUpdateProfile: (UserProfile) -> Unit,
    viewModel: HomeViewModel
) {
    val profileValue = profile ?: return
    when (tabId) {
        "account" -> AccountSettings(profileValue, lang, onLogout, onConnectTrakt, onConnectMal, onConnectSimkl, onUpdateProfile)
        "general" -> GeneralSettings(profileValue, lang, onUpdateProfile)
        "appearance" -> {
            val categories by viewModel.categories.collectAsStateWithLifecycle()
            var previewMeta by remember { mutableStateOf<Meta?>(null) }
            LaunchedEffect(categories) {
                val candidate = categories
                    .flatMap { it.items }
                    .firstOrNull { !it.poster.isNullOrBlank() || !it.background.isNullOrBlank() }
                if (candidate != null) previewMeta = candidate
            }
            AppearanceSettings(profileValue, lang, previewMeta, onUpdateProfile)
        }
        "content" -> {
            val categories by viewModel.categories.collectAsStateWithLifecycle()
            val userAddons by viewModel.userAddons.collectAsStateWithLifecycle()
            val cs3FeedOptions by viewModel.loadedCs3CatalogFeedOptions.collectAsStateWithLifecycle()
            ContentSettings(profileValue, lang, categories, userAddons, cs3FeedOptions, onUpdateProfile)
        }
        "playback" -> {
            var subScreen by remember { mutableStateOf<String?>(null) }
            when (subScreen) {
                "subtitles" -> TvSettingsSubScreen(AppStrings.t(lang, "auto.subtitles"), onBack = { subScreen = null }) {
                    SubtitleSettings(profileValue, lang, onUpdateProfile)
                }
                "advanced" -> TvSettingsSubScreen(AppStrings.t(lang, "settings.advanced_settings"), onBack = { subScreen = null }) {
                    AdvancedSettings(profileValue, lang, onUpdateProfile)
                }
                else -> PlaybackSettings(
                    profile = profileValue,
                    lang = lang,
                    onOpenSubtitles = { subScreen = "subtitles" },
                    onOpenAdvanced = { subScreen = "advanced" },
                    onUpdateProfile = onUpdateProfile
                )
            }
        }
        "downloads" -> DownloadSettings(profileValue, lang, onUpdateProfile)
        "addons" -> AddonSettings(profileValue, lang, onManageAddons, onUpdateProfile)
        "system" -> SystemSettings(profileValue, lang, onReboot, onLogout, onUpdateProfile)
        "developer" -> DeveloperSettings(lang)
    }
}

@Composable
internal fun TvSettingsSubScreen(title: String, onBack: () -> Unit, content: @Composable ColumnScope.() -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(28.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
            var focused by remember { mutableStateOf(false) }
            Surface(
                onClick = onBack,
                modifier = Modifier.size(44.dp).onFocusChanged { focused = it.isFocused },
                shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(14.dp)),
                colors = ClickableSurfaceDefaults.colors(
                    containerColor = if (focused) Color.White.copy(alpha = 0.16f) else Color.White.copy(alpha = 0.06f),
                    contentColor = Color.White
                )
            ) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Icon(FluxaIcons.ArrowBack, null, tint = Color.White)
                }
            }
            Text(title, color = Color.White, fontSize = 26.sp, fontWeight = FontWeight.Black)
        }
        content()
    }
}
