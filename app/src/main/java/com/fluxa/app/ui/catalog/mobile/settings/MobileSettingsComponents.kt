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
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Offset
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
import com.fluxa.app.BuildConfig
import com.fluxa.app.ui.catalog.UpdateManager
import kotlinx.coroutines.launch
import java.util.Locale

internal data class MobileSettingsPalette(
    val background: Color,
    val card: Color,
    val text: Color,
    val mutedText: Color,
    val divider: Color,
    val border: Color,
    val accent: Color,
    val onAccent: Color,
    val cardRadius: Dp,
    val rowVerticalPadding: Dp
)

internal val LocalMobileSettingsPalette = staticCompositionLocalOf {
    mobileSettingsPalette(
        amoledMode = false,
        accentColorArgb = Color.White.toArgb(),
        cardCornerPreset = "medium",
        interfaceDensity = "medium"
    )
}

internal fun mobileSettingsPalette(
    amoledMode: Boolean,
    accentColorArgb: Int,
    cardCornerPreset: String,
    interfaceDensity: String
): MobileSettingsPalette {
    val accent = Color(accentColorArgb)
    val accentLuma = accent.red * 0.299f + accent.green * 0.587f + accent.blue * 0.114f
    val onAccent = if (accentLuma > 0.68f) Color.Black else Color.White
    val text = Color.White
    return MobileSettingsPalette(
        background = if (amoledMode) Color.Black else FluxaColors.background,
        card = if (amoledMode) FluxaColors.backgroundAmoled else FluxaColors.surfaceCard,
        text = text,
        mutedText = text.copy(alpha = 0.42f),
        divider = text.copy(alpha = 0.055f),
        border = text.copy(alpha = 0.05f),
        accent = accent,
        onAccent = onAccent,
        cardRadius = mobileCornerRadius(cardCornerPreset),
        rowVerticalPadding = mobileDensityPadding(interfaceDensity)
    )
}

private data class MobileSettingsCategory(
    val id: String,
    val title: String,
    val subtitle: String,
    val icon: ImageVector
)

private data class MobileSettingsSection(
    val label: String,
    val categories: List<MobileSettingsCategory>
)

@Composable
internal fun MobileSettingsHub(
    profile: UserProfile,
    lang: String,
    totalWatchedContentDuration: Long,
    onNavigate: (String) -> Unit,
    onBack: () -> Unit,
    onLogout: () -> Unit,
    onManageAddons: () -> Unit,
    onUpdateInfoChanged: (UpdateManager.UpdateInfo?) -> Unit = {}
) {
    val coroutineScope = rememberCoroutineScope()
    var updateDialogMessage by remember { mutableStateOf<String?>(null) }
    var isCheckingUpdate by remember { mutableStateOf(false) }
    var availableUpdate by remember { mutableStateOf<UpdateManager.UpdateInfo?>(null) }
    val palette = mobileSettingsPalette(
        amoledMode = profile.safeAmoledMode,
        accentColorArgb = profile.safeAccentColorArgb,
        cardCornerPreset = profile.safeCardCornerPreset,
        interfaceDensity = profile.safeInterfaceDensity
    )

    LaunchedEffect(Unit) {
        val update = UpdateManager.checkUpdate()
        availableUpdate = update
        onUpdateInfoChanged(update)
    }

    fun checkForUpdates() {
        coroutineScope.launch {
            isCheckingUpdate = true
            updateDialogMessage = AppStrings.t(lang, "settings.checking_for_updates")
            val update = UpdateManager.checkUpdate()
            isCheckingUpdate = false
            availableUpdate = update
            onUpdateInfoChanged(update)
            updateDialogMessage = if (update != null)
                AppStrings.format(lang, "settings.update_available", update.versionName)
            else
                AppStrings.t(lang, "settings.up_to_date")
        }
    }

    val sections = listOf(
        MobileSettingsSection(
            label = AppStrings.t(lang, "settings.section_account"),
            categories = listOf(
                MobileSettingsCategory("account", AppStrings.t(lang, "auto.account_sync"), AppStrings.t(lang, "auto.account_devices_and_sync"), FluxaIcons.AccountCircle)
            )
        ),
        MobileSettingsSection(
            label = AppStrings.t(lang, "settings.section_preferences"),
            categories = listOf(
                MobileSettingsCategory("general", AppStrings.t(lang, "auto.general"), AppStrings.t(lang, "auto.language_theme_startup"), FluxaIcons.Settings),
                MobileSettingsCategory("appearance", AppStrings.t(lang, "auto.appearance"), AppStrings.t(lang, "auto.color_and_layout"), FluxaIcons.Palette),
                MobileSettingsCategory("playback", AppStrings.t(lang, "auto.playback"), AppStrings.t(lang, "auto.player_behavior_and_defaults"), FluxaIcons.PlayCircle)
            )
        ),
        MobileSettingsSection(
            label = AppStrings.t(lang, "settings.section_content"),
            categories = listOf(
                MobileSettingsCategory("content", AppStrings.t(lang, "auto.catalogs"), AppStrings.t(lang, "auto.categories_sources_and_ranking"), FluxaIcons.MenuBook),
                MobileSettingsCategory("addons", AppStrings.t(lang, "auto.add_ons"), AppStrings.t(lang, "auto.installed_add_ons_and_settings"), FluxaIcons.Extension),
                MobileSettingsCategory("downloads", AppStrings.t(lang, "auto.downloads"), AppStrings.t(lang, "auto.download_and_storage_settings"), FluxaIcons.Download)
            )
        ),
        MobileSettingsSection(
            label = AppStrings.t(lang, "settings.section_system"),
            categories = listOf(
                MobileSettingsCategory("developer", AppStrings.t(lang, "settings.developer"), AppStrings.t(lang, "settings.developer_desc"), FluxaIcons.Memory)
            )
        )
    )

    CompositionLocalProvider(LocalMobileSettingsPalette provides palette) {
        val colors = LocalMobileSettingsPalette.current
        updateDialogMessage?.let { message ->
            AlertDialog(
                onDismissRequest = { if (!isCheckingUpdate) updateDialogMessage = null },
                confirmButton = {
                    if (!isCheckingUpdate) {
                        TextButton(onClick = { updateDialogMessage = null }) {
                            Text(AppStrings.t(lang, "auto.ok"), color = colors.accent, fontWeight = FontWeight.Bold)
                        }
                    }
                },
                title = { Text(AppStrings.t(lang, "settings.check_for_updates"), color = colors.text, fontWeight = FontWeight.Bold) },
                text = { Text(message, color = colors.mutedText, fontSize = 14.sp) },
                containerColor = colors.card,
                shape = RoundedCornerShape(colors.cardRadius)
            )
        }
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .background(colors.background),
            contentPadding = PaddingValues(top = 30.dp, bottom = 150.dp, start = 12.dp, end = 12.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
        item {
            Text(
                text = AppStrings.t(lang, "nav.settings"),
                color = colors.text,
                fontFamily = FluxaDisplay,
                fontSize = 28.sp,
                fontWeight = FontWeight.Bold
            )
        }

        item {
            MobileSettingsProfileHeader(
                profile = profile,
                lang = lang,
                onClick = { onNavigate("account") }
            )
        }

        sections.forEach { section ->
            val isSystemSection = section.label == AppStrings.t(lang, "settings.section_system")
            item {
                Text(
                    text = section.label,
                    color = colors.mutedText,
                    fontFamily = FluxaDisplay,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.SemiBold,
                    letterSpacing = 0.6.sp,
                    modifier = Modifier.padding(start = 4.dp, top = 6.dp)
                )
            }
            item {
                MobileSettingsCard {
                    section.categories.forEachIndexed { index, category ->
                        MobileSettingsRow(
                            title = category.title,
                            subtitle = category.subtitle,
                            icon = category.icon,
                            onClick = {
                                when (category.id) {
                                    "addons" -> onManageAddons()
                                    "subtitles" -> onNavigate("subtitles")
                                    "advanced_settings" -> onNavigate("advanced_settings")
                                    "developer" -> onNavigate("developer")
                                    else -> onNavigate(category.id)
                                }
                            }
                        )
                        if (index != section.categories.lastIndex || isSystemSection) {
                            MobileSettingsRowDivider()
                        }
                    }
                    if (isSystemSection) {
                        MobileSettingsRow(
                            title = AppStrings.t(lang, "settings.check_for_updates"),
                            subtitle = if (availableUpdate != null)
                                AppStrings.format(lang, "settings.update_available", availableUpdate?.versionName.orEmpty())
                            else
                                AppStrings.t(lang, "settings.check_for_updates_desc"),
                            icon = FluxaIcons.SystemUpdate,
                            badge = availableUpdate != null,
                            onClick = { checkForUpdates() }
                        )
                    }
                }
            }
        }
        item {
            MobileWatchedContentCounter(
                duration = totalWatchedContentDuration,
                lang = lang
            )
        }
        item {
            Text(
                text = AppStrings.format(lang, "settings.version_footer", BuildConfig.VERSION_NAME, BuildConfig.VERSION_CODE),
                color = colors.mutedText,
                fontSize = 12.sp,
                fontWeight = FontWeight.Medium,
                modifier = Modifier.fillMaxWidth().padding(top = 4.dp, bottom = 12.dp),
                textAlign = androidx.compose.ui.text.style.TextAlign.Center
            )
        }
    }
    }
}

@Composable
private fun MobileWatchedContentCounter(duration: Long, lang: String) {
    val colors = LocalMobileSettingsPalette.current
    val parts = remember(duration) { watchedDurationParts(duration) }
    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Text(
            text = AppStrings.t(lang, "settings.watch_time").uppercase(AppStrings.locale(lang)),
            color = colors.text,
            fontFamily = FluxaDisplay,
            fontSize = 20.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.fillMaxWidth(),
            textAlign = androidx.compose.ui.text.style.TextAlign.Center
        )
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 12.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceEvenly
        ) {
            WatchedCounterPart(
                value = parts.months,
                label = AppStrings.t(lang, "unit.month_counter").uppercase(AppStrings.locale(lang)),
                modifier = Modifier.weight(1f)
            )
            WatchedCounterPart(
                value = parts.days,
                label = AppStrings.t(lang, "unit.day_counter").uppercase(AppStrings.locale(lang)),
                modifier = Modifier.weight(1f)
            )
            WatchedCounterPart(
                value = parts.hours,
                label = AppStrings.t(lang, "unit.hour_counter").uppercase(AppStrings.locale(lang)),
                modifier = Modifier.weight(1f)
            )
            WatchedCounterPart(
                value = parts.minutes,
                label = AppStrings.t(lang, "unit.minute_counter").uppercase(AppStrings.locale(lang)),
                modifier = Modifier.weight(1f)
            )
        }
    }
}

@Composable
private fun WatchedCounterPart(
    value: Long,
    label: String,
    modifier: Modifier = Modifier
) {
    val colors = LocalMobileSettingsPalette.current
    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(1.dp)
    ) {
        Text(
            text = value.toString(),
            color = colors.text,
            fontFamily = FluxaDisplay,
            fontSize = 20.sp,
            fontWeight = FontWeight.Bold,
            maxLines = 1
        )
        Text(
            text = label,
            color = colors.mutedText,
            fontSize = 11.sp,
            fontWeight = FontWeight.Medium,
            maxLines = 1,
            overflow = TextOverflow.Clip
        )
    }
}

private data class WatchedDurationParts(
    val months: Long,
    val days: Long,
    val hours: Long,
    val minutes: Long
)

private fun watchedDurationParts(durationMs: Long): WatchedDurationParts {
    val totalSeconds = (durationMs / 1000L).coerceAtLeast(0L)
    val totalMinutes = totalSeconds / 60L
    val minutesPerDay = 24L * 60L
    val minutesPerMonth = 30L * minutesPerDay
    val months = totalMinutes / minutesPerMonth
    val days = (totalMinutes % minutesPerMonth) / minutesPerDay
    val hours = (totalMinutes % minutesPerDay) / 60L
    val minutes = totalMinutes % 60L
    return WatchedDurationParts(
        months = months,
        days = days,
        hours = hours,
        minutes = minutes
    )
}

@Composable
internal fun MobileSettingsCard(
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    Column(
        modifier = modifier
            .fillMaxWidth()
            .animateContentSize(animationSpec = tween(FluxaDimensions.AnimDuration.settingsExpand))
            .clip(RoundedCornerShape(colors.cardRadius))
            .background(colors.card)
            .border(1.dp, colors.border, RoundedCornerShape(colors.cardRadius))
            .padding(vertical = 8.dp),
        content = content
    )
}

@Composable
internal fun MobileSettingsRow(
    title: String,
    subtitle: String,
    icon: ImageVector? = null,
    logoUrl: String? = null,
    iconBg: Color = Color.Transparent,
    iconTint: Color = Color.White,
    badge: Boolean = false,
    onClick: () -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
            .padding(horizontal = 16.dp, vertical = colors.rowVerticalPadding + 4.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp)
    ) {
        Box(
            modifier = Modifier
                .size(38.dp)
                .clip(RoundedCornerShape(11.dp))
                .background(if (iconBg != Color.Transparent) iconBg else colors.accent.copy(alpha = 0.16f)),
            contentAlignment = Alignment.Center
        ) {
            if (logoUrl != null) {
                AsyncImage(
                    model = logoUrl,
                    contentDescription = null,
                    modifier = Modifier.size(20.dp)
                )
            } else if (icon != null) {
                Icon(
                    icon,
                    null,
                    tint = if (iconTint != Color.White) iconTint else colors.accent,
                    modifier = Modifier.size(19.dp)
                )
            }
            if (badge) {
                Box(
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .offset(x = 3.dp, y = (-3).dp)
                        .size(8.dp)
                        .clip(CircleShape)
                        .background(colors.accent)
                        .border(1.5.dp, colors.card, CircleShape)
                )
            }
        }
        Column(modifier = Modifier.weight(1f)) {
            Text(text = title, color = colors.text.copy(alpha = 0.94f), fontFamily = FluxaDisplay, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
            if (subtitle.isNotBlank()) {
                Text(
                    text = subtitle,
                    color = colors.text.copy(alpha = 0.5f),
                    fontSize = 12.sp,
                    lineHeight = 16.sp,
                    modifier = Modifier.padding(top = 1.dp)
                )
            }
        }
        Icon(FluxaIcons.ChevronRight, null, tint = colors.text.copy(alpha = 0.24f), modifier = Modifier.size(20.dp))
    }
}

@Composable
private fun MobileSettingsRowDivider() {
    val colors = LocalMobileSettingsPalette.current
    HorizontalDivider(
        modifier = Modifier.padding(start = 68.dp, end = 16.dp),
        thickness = 1.dp,
        color = colors.divider
    )
}

@Composable
private fun MobileSettingsProfileHeader(
    profile: UserProfile,
    lang: String,
    onClick: () -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(colors.cardRadius))
            .background(colors.card)
            .border(1.dp, colors.border, RoundedCornerShape(colors.cardRadius))
            .clickable { onClick() }
            .padding(horizontal = 16.dp, vertical = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp)
    ) {
        Box(
            modifier = Modifier
                .size(56.dp)
                .clip(CircleShape)
                .background(Color(profile.safeColorArgb)),
            contentAlignment = Alignment.Center
        ) {
            if (!profile.avatarUrl.isNullOrBlank()) {
                AsyncImage(
                    model = profile.avatarUrl,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize().clip(CircleShape),
                    contentScale = ContentScale.Crop
                )
            } else {
                DefaultProfileAvatar(modifier = Modifier.size(30.dp))
            }
        }
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = profile.displayName,
                color = colors.text,
                fontFamily = FluxaDisplay,
                fontSize = 17.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = AppStrings.t(lang, "auto.account_sync"),
                color = colors.text.copy(alpha = 0.5f),
                fontSize = 13.sp,
                modifier = Modifier.padding(top = 1.dp)
            )
        }
        Icon(FluxaIcons.ChevronRight, null, tint = colors.text.copy(alpha = 0.24f), modifier = Modifier.size(20.dp))
    }
}
