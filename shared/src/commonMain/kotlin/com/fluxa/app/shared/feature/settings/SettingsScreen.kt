package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.AccountCircle
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Code
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.Extension
import androidx.compose.material.icons.automirrored.filled.LibraryBooks
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material.icons.filled.Palette
import androidx.compose.material.icons.filled.OndemandVideo
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Tune
import androidx.compose.material3.Icon
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import kotlinx.coroutines.delay
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors

data class SettingsBrandIcons(
    val stremio: @Composable () -> Unit = {},
    val nuvio: @Composable () -> Unit = {},
    val trakt: @Composable () -> Unit = {},
    val simkl: @Composable () -> Unit = {},
    val anilist: @Composable () -> Unit = {}
)

private val SETTINGS_TV_RAIL_CATEGORIES = listOf(
    SettingsCategory.Account,
    SettingsCategory.Notifications,
    SettingsCategory.General,
    SettingsCategory.Appearance,
    SettingsCategory.Playback,
    SettingsCategory.Content,
    SettingsCategory.Downloads,
    SettingsCategory.Developer
)

@Composable
fun SettingsScreen(
    state: SettingsUiState,
    language: String?,
    onAction: (SettingsAction) -> Unit,
    onSwitchProfilesRequested: () -> Unit,
    onBackRequested: () -> Unit,
    backStack: List<SettingsCategory> = emptyList(),
    onPushCategory: (SettingsCategory) -> Unit = {},
    onPopCategory: () -> Unit = {},
    onSelectCategory: (SettingsCategory) -> Unit = {},
    deviceType: com.fluxa.app.ui.catalog.DeviceType = com.fluxa.app.ui.catalog.DeviceType.Mobile,
    brandIcons: SettingsBrandIcons = SettingsBrandIcons(),
    modifier: Modifier = Modifier
) {
    val category = backStack.lastOrNull() ?: SettingsCategory.Hub
    val lang = language

    if (deviceType == com.fluxa.app.ui.catalog.DeviceType.TV) {
        val selectedCategory = if (category == SettingsCategory.Hub) SettingsCategory.Account else category
        Row(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
            Column(
                modifier = Modifier.width(300.dp).fillMaxSize().padding(24.dp)
            ) {
                Text(AppStrings.t(lang, "nav.settings"), color = Color.White, fontWeight = FontWeight.Black, fontSize = 22.sp)
                Spacer(Modifier.height(16.dp))
                Column(modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                    SETTINGS_TV_RAIL_CATEGORIES.forEach { railCategory ->
                        SettingsTvRailRow(
                            label = settingsCategoryTitle(railCategory, lang),
                            selected = railCategory == selectedCategory
                        ) { onSelectCategory(railCategory) }
                    }
                }
            }
            Column(
                modifier = Modifier.weight(1f).fillMaxSize().padding(24.dp).verticalScroll(remember(selectedCategory) { ScrollState(0) })
            ) {
                Text(settingsCategoryTitle(selectedCategory, lang), color = Color.White, fontWeight = FontWeight.Black, fontSize = 22.sp)
                Spacer(Modifier.height(16.dp))
                SettingsCategoryContent(selectedCategory, state, lang, brandIcons, onAction, onPushCategory, onSwitchProfilesRequested)
                Spacer(Modifier.height(120.dp))
            }
        }
        return
    }

    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        Column(modifier = Modifier.fillMaxSize().padding(horizontal = 20.dp)) {
            SettingsTopBar(
                title = settingsCategoryTitle(category, lang),
                onBack = { if (backStack.isEmpty()) onBackRequested() else onPopCategory() }
            )
            Column(modifier = Modifier.fillMaxSize().verticalScroll(remember(category) { ScrollState(0) })) {
                SettingsCategoryContent(category, state, lang, brandIcons, onAction, onPushCategory, onSwitchProfilesRequested)
                Spacer(Modifier.height(120.dp))
            }
        }
    }
}

@Composable
private fun SettingsCategoryContent(
    category: SettingsCategory,
    state: SettingsUiState,
    lang: String?,
    brandIcons: SettingsBrandIcons,
    onAction: (SettingsAction) -> Unit,
    onNavigate: (SettingsCategory) -> Unit,
    onSwitchProfiles: () -> Unit
) {
    when (category) {
        SettingsCategory.Hub -> SettingsHubContent(
            state = state,
            lang = lang,
            onNavigate = onNavigate,
            onSwitchProfiles = onSwitchProfiles,
            onAction = onAction
        )
        SettingsCategory.Account -> SettingsAccountContent(state.account, lang, brandIcons, onAction, onNavigate = onNavigate)
        SettingsCategory.TmdbFeatures -> SettingsTmdbFeaturesContent(state.account, lang, onAction)
        SettingsCategory.Notifications -> SettingsNotificationsContent(state.notifications, lang, onAction)
        SettingsCategory.General -> SettingsGeneralContent(state.general, lang, onAction)
        SettingsCategory.Appearance -> SettingsAppearanceContent(state.appearance, lang, onAction, onNavigate = onNavigate)
        SettingsCategory.AppearanceHome -> SettingsAppearanceHomeContent(state.appearanceHome, lang, onAction, onNavigate = onNavigate)
        SettingsCategory.AppearanceHomeHero -> SettingsAppearanceHomeHeroContent(state.appearanceHome, lang, onAction)
        SettingsCategory.AppearanceHomeContinueWatching -> SettingsAppearanceHomeContinueWatchingContent(state.appearanceHome, lang, onAction)
        SettingsCategory.AppearanceHomeNavigation -> SettingsAppearanceHomeNavigationContent(state.appearanceHome, lang, onAction)
        SettingsCategory.AppearanceDetail -> SettingsAppearanceDetailContent(lang, onNavigate = onNavigate)
        SettingsCategory.AppearanceDetailHero -> SettingsAppearanceDetailHeroContent(state.appearanceDetail, lang, onAction)
        SettingsCategory.AppearanceDetailEpisodes -> SettingsAppearanceDetailEpisodesContent(state.appearanceDetail, lang, onAction)
        SettingsCategory.Playback -> SettingsPlaybackContent(state.playback, lang, onAction, onNavigate = onNavigate)
        SettingsCategory.Subtitles -> SettingsSubtitlesContent(state.subtitles, lang, onAction)
        SettingsCategory.Advanced -> SettingsAdvancedContent(state.advanced, lang, onAction)
        SettingsCategory.Content -> SettingsContentCategoryContent(state.content, lang, onAction)
        SettingsCategory.Downloads -> SettingsDownloadsContent(state.downloads, lang, onAction)
        SettingsCategory.Developer -> SettingsDeveloperContent(state.developer, lang)
    }
}

@Composable
private fun SettingsTvRailRow(label: String, selected: Boolean, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(androidx.compose.foundation.shape.RoundedCornerShape(10.dp))
            .background(if (selected) Color.White.copy(alpha = 0.12f) else Color.Transparent)
            .clickable(onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 12.dp)
    ) {
        Text(
            label,
            color = if (selected) Color.White else Color.White.copy(alpha = 0.6f),
            fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium
        )
    }
}

private fun settingsCategoryTitle(category: SettingsCategory, lang: String?): String = when (category) {
    SettingsCategory.Hub -> AppStrings.t(lang, "nav.settings")
    SettingsCategory.Account -> AppStrings.t(lang, "auto.account")
    SettingsCategory.TmdbFeatures -> AppStrings.t(lang, "settings.apis")
    SettingsCategory.Notifications -> AppStrings.t(lang, "settings.notifications_title")
    SettingsCategory.General -> AppStrings.t(lang, "auto.general")
    SettingsCategory.Appearance -> AppStrings.t(lang, "auto.appearance")
    SettingsCategory.AppearanceHome -> AppStrings.t(lang, "settings.appearance_home_screen")
    SettingsCategory.AppearanceHomeHero -> AppStrings.t(lang, "settings.hero_banner")
    SettingsCategory.AppearanceHomeContinueWatching -> AppStrings.t(lang, "auto.continue_watching")
    SettingsCategory.AppearanceHomeNavigation -> AppStrings.t(lang, "settings.navigation")
    SettingsCategory.AppearanceDetail -> AppStrings.t(lang, "settings.appearance_detail_screen")
    SettingsCategory.AppearanceDetailHero -> AppStrings.t(lang, "settings.hero_banner")
    SettingsCategory.AppearanceDetailEpisodes -> AppStrings.t(lang, "settings.episodes")
    SettingsCategory.Playback -> AppStrings.t(lang, "auto.playback")
    SettingsCategory.Subtitles -> AppStrings.t(lang, "auto.subtitles")
    SettingsCategory.Advanced -> AppStrings.t(lang, "settings.advanced_settings")
    SettingsCategory.Content -> AppStrings.t(lang, "auto.catalogs")
    SettingsCategory.Downloads -> AppStrings.t(lang, "auto.downloads")
    SettingsCategory.Developer -> AppStrings.t(lang, "settings.developer")
}

@Composable
private fun SettingsTopBar(title: String, onBack: () -> Unit) {
    Row(modifier = Modifier.fillMaxWidth().padding(vertical = 16.dp), verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier.size(40.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.05f)).clickable(onClick = onBack),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                contentDescription = null,
                tint = Color.White,
                modifier = Modifier.size(20.dp)
            )
        }
        Spacer(Modifier.width(12.dp))
        Text(title, color = Color.White, fontWeight = FontWeight.Black, fontSize = 22.sp)
    }
}

@Composable
private fun SettingsHubContent(
    state: SettingsUiState,
    lang: String?,
    onNavigate: (SettingsCategory) -> Unit,
    onSwitchProfiles: () -> Unit,
    onAction: (SettingsAction) -> Unit
) {
    var searchQuery by remember { mutableStateOf("") }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 6.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(Color.White.copy(alpha = 0.06f))
            .padding(horizontal = 14.dp, vertical = 10.dp)
    ) {
        Icon(
            imageVector = Icons.Filled.Search,
            contentDescription = null,
            tint = Color.White.copy(alpha = 0.4f),
            modifier = Modifier.size(18.dp)
        )
        Spacer(Modifier.width(10.dp))
        Box(modifier = Modifier.weight(1f)) {
            if (searchQuery.isEmpty()) {
                Text(AppStrings.t(lang, "auto.search"), color = Color.White.copy(alpha = 0.4f))
            }
            androidx.compose.foundation.text.BasicTextField(
                value = searchQuery,
                onValueChange = { searchQuery = it },
                singleLine = true,
                textStyle = androidx.compose.ui.text.TextStyle(color = Color.White, fontSize = 16.sp),
                cursorBrush = androidx.compose.ui.graphics.SolidColor(Color.White),
                modifier = Modifier.fillMaxWidth()
            )
        }
    }
    if (searchQuery.isNotBlank()) {
        val results = settingsSearchResults(lang, searchQuery)
        if (results.isEmpty()) {
            Text(
                AppStrings.t(lang, "settings.search_no_results"),
                color = Color.White.copy(alpha = 0.5f),
                modifier = Modifier.padding(vertical = 12.dp)
            )
        } else {
            SettingsGroupCard {
                results.forEach { entry ->
                    SettingsNavRow(entry.label, value = settingsCategoryTitle(entry.category, lang)) { onNavigate(entry.category) }
                }
            }
        }
        return
    }

    SettingsGroupCard {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().clickable { onNavigate(SettingsCategory.Account) }.padding(vertical = 12.dp)
        ) {
            Box(
                modifier = Modifier.size(44.dp).clip(CircleShape).background(Color.White.copy(alpha = 0.1f)),
                contentAlignment = Alignment.Center
            ) {
                if (!state.account.avatarUrl.isNullOrBlank()) {
                    com.fluxa.app.shared.image.FluxaRemoteImage(
                        imageUrl = state.account.avatarUrl,
                        cacheKey = "settings-avatar:${state.account.avatarUrl}",
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize().clip(CircleShape),
                        contentScale = androidx.compose.ui.layout.ContentScale.Crop
                    )
                } else {
                    com.fluxa.app.shared.feature.profile.ProfileDefaultAvatar(modifier = Modifier.size(26.dp))
                }
            }
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(state.account.displayName, color = Color.White, fontWeight = FontWeight.Bold, fontSize = 16.sp)
                Text(AppStrings.t(lang, "auto.account"), color = Color.White.copy(alpha = 0.5f), fontSize = 13.sp)
            }
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = Color.White.copy(alpha = 0.35f),
                modifier = Modifier.size(20.dp)
            )
        }
        SettingsNavRow(AppStrings.t(lang, "settings.notifications_title"), icon = Icons.Filled.Notifications) { onNavigate(SettingsCategory.Notifications) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.section_preferences"))
    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "auto.general"), icon = Icons.Filled.Tune) { onNavigate(SettingsCategory.General) }
        SettingsNavRow(AppStrings.t(lang, "auto.appearance"), icon = Icons.Filled.Palette) { onNavigate(SettingsCategory.Appearance) }
        SettingsNavRow(AppStrings.t(lang, "auto.playback"), icon = Icons.Filled.OndemandVideo) { onNavigate(SettingsCategory.Playback) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.section_content"))
    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "auto.catalogs"), icon = Icons.AutoMirrored.Filled.LibraryBooks) { onNavigate(SettingsCategory.Content) }
        SettingsNavRow(AppStrings.t(lang, "auto.add_ons"), icon = Icons.Filled.Extension) { onAction(SettingsAction.ManageAddonsRequested) }
        SettingsNavRow(AppStrings.t(lang, "auto.downloads"), icon = Icons.Filled.Download) { onNavigate(SettingsCategory.Downloads) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.section_system"))
    SettingsGroupCard {
        SettingsToggleRow(
            label = AppStrings.t(lang, "settings.automatic_updates"),
            description = AppStrings.t(lang, "settings.automatic_updates_desc"),
            value = state.system.automaticUpdates,
            onValueChanged = { onAction(SettingsAction.SystemChanged(state.system.copy(automaticUpdates = it))) }
        )
        SettingsActionRow(AppStrings.t(lang, "settings.check_for_updates")) { onAction(SettingsAction.CheckForUpdateRequested) }
        SettingsNavRow(AppStrings.t(lang, "settings.developer"), icon = Icons.Filled.Code) { onNavigate(SettingsCategory.Developer) }
    }

    Spacer(Modifier.height(12.dp))
    Text(
        state.system.appVersionLabel,
        color = Color.White.copy(alpha = 0.35f),
        fontSize = 11.sp,
        modifier = Modifier.fillMaxWidth(),
        textAlign = androidx.compose.ui.text.style.TextAlign.Center
    )
}

@Composable
private fun SettingsAccountContent(
    model: SettingsAccountUiModel,
    lang: String?,
    brandIcons: SettingsBrandIcons,
    onAction: (SettingsAction) -> Unit,
    onNavigate: (SettingsCategory) -> Unit
) {
    var selectedProvider by remember { mutableStateOf<SettingsAccountProvider?>(null) }
    var confirmingDisconnect by remember { mutableStateOf(false) }
    SettingsSectionHeader(AppStrings.t(lang, "auto.account_sync"))
    SettingsGroupCard {
        val syncFailedLabel = AppStrings.t(lang, "integration.sync_failed")
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.stremio"),
            connected = model.hasStremio,
            connectedLabel = AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.stremio,
            hasSyncFailure = "stremio" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { selectedProvider = SettingsAccountProvider.Stremio }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.nuvio"),
            connected = model.hasNuvio,
            connectedLabel = AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.nuvio,
            hasSyncFailure = "nuvio" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { selectedProvider = SettingsAccountProvider.Nuvio }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.trakt"),
            connected = model.hasTrakt,
            connectedLabel = AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.trakt,
            hasSyncFailure = "trakt" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { selectedProvider = SettingsAccountProvider.Trakt }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.simkl"),
            connected = model.hasSimkl,
            connectedLabel = AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.simkl,
            hasSyncFailure = "simkl" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { selectedProvider = SettingsAccountProvider.Simkl }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.anilist"),
            connected = model.hasAnilist,
            connectedLabel = AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.anilist,
            hasSyncFailure = "anilist" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { selectedProvider = SettingsAccountProvider.Anilist }
        if (model.hasAnySync) {
            SettingsActionRow(AppStrings.t(lang, "auto.disconnect"), destructive = true) { confirmingDisconnect = true }
        }
    }
    if (confirmingDisconnect) {
        androidx.compose.material3.AlertDialog(
            onDismissRequest = { confirmingDisconnect = false },
            title = { Text(AppStrings.t(lang, "auto.disconnect")) },
            text = { Text(AppStrings.t(lang, "settings.disconnect_confirm")) },
            confirmButton = {
                TextButton(onClick = {
                    confirmingDisconnect = false
                    onAction(SettingsAction.DisconnectSyncRequested)
                }) {
                    Text(AppStrings.t(lang, "auto.disconnect"), color = FluxaColors.errorRed)
                }
            },
            dismissButton = {
                TextButton(onClick = { confirmingDisconnect = false }) {
                    Text(AppStrings.t(lang, "common.cancel"))
                }
            }
        )
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.tmdb_api"))
    val tmdbConfigured = !model.tmdbApiKey.isNullOrBlank()
    SettingsGroupCard {
        Row(modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text(AppStrings.t(lang, "brand.tmdb"), color = Color.White)
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                if (tmdbConfigured) {
                    Icon(Icons.Filled.CheckCircle, contentDescription = null, tint = FluxaColors.successGreen, modifier = Modifier.size(14.dp))
                }
                Text(
                    AppStrings.t(lang, if (tmdbConfigured) "settings.tmdb_api_configured" else "settings.tmdb_api_not_configured"),
                    color = if (tmdbConfigured) FluxaColors.successGreen else Color.White.copy(alpha = 0.5f),
                    fontSize = 12.sp
                )
            }
        }
        SettingsSecretFieldRow(
            AppStrings.t(lang, "settings.tmdb_api_key"),
            model.tmdbApiKey.orEmpty(),
            placeholder = AppStrings.t(lang, "settings.tmdb_api_key_placeholder")
        ) {
            onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbApiKey = it)))
        }
        if (tmdbConfigured) {
            SettingsNavRow(AppStrings.t(lang, "settings.apis")) { onNavigate(SettingsCategory.TmdbFeatures) }
        }
    }

    selectedProvider?.let { provider ->
        key(provider) {
            SettingsAccountSheet(provider, model, lang, onDismiss = { selectedProvider = null }) {
                onAction(
                    when (provider) {
                        SettingsAccountProvider.Stremio -> SettingsAction.ConnectStremioRequested
                        SettingsAccountProvider.Nuvio -> SettingsAction.ConnectNuvioRequested
                        SettingsAccountProvider.Trakt -> SettingsAction.ConnectTraktRequested
                        SettingsAccountProvider.Simkl -> SettingsAction.ConnectSimklRequested
                        SettingsAccountProvider.Anilist -> SettingsAction.ConnectAnilistRequested
                    }
                )
            }
        }
    }
}

private enum class SettingsAccountProvider { Stremio, Nuvio, Trakt, Simkl, Anilist }

@Composable
@OptIn(ExperimentalMaterial3Api::class)
private fun SettingsAccountSheet(
    provider: SettingsAccountProvider,
    model: SettingsAccountUiModel,
    lang: String?,
    onDismiss: () -> Unit,
    onSync: () -> Unit
) {
    val connected = when (provider) {
        SettingsAccountProvider.Stremio -> model.hasStremio
        SettingsAccountProvider.Nuvio -> model.hasNuvio
        SettingsAccountProvider.Trakt -> model.hasTrakt
        SettingsAccountProvider.Simkl -> model.hasSimkl
        SettingsAccountProvider.Anilist -> model.hasAnilist
    }
    val titleKey = when (provider) {
        SettingsAccountProvider.Stremio -> "brand.stremio"
        SettingsAccountProvider.Nuvio -> "brand.nuvio"
        SettingsAccountProvider.Trakt -> "brand.trakt"
        SettingsAccountProvider.Simkl -> "brand.simkl"
        SettingsAccountProvider.Anilist -> "brand.anilist"
    }
    val lastSyncAt = when (provider) {
        SettingsAccountProvider.Nuvio -> model.nuvioLastSyncAt
        SettingsAccountProvider.Trakt -> model.traktLastSyncAt
        SettingsAccountProvider.Simkl -> model.simklLastSyncAt
        SettingsAccountProvider.Stremio, SettingsAccountProvider.Anilist -> 0L
    }
    val email = if (provider == SettingsAccountProvider.Nuvio) model.nuvioEmail.orEmpty().ifBlank { model.email } else model.email
    val providerKey = when (provider) {
        SettingsAccountProvider.Stremio -> "stremio"
        SettingsAccountProvider.Nuvio -> "nuvio"
        SettingsAccountProvider.Trakt -> "trakt"
        SettingsAccountProvider.Simkl -> "simkl"
        SettingsAccountProvider.Anilist -> "anilist"
    }
    val hasSyncFailure = providerKey in model.syncFailedProviders
    val isSyncing = providerKey in model.syncingProviders
    var justSynced by remember { mutableStateOf(false) }
    LaunchedEffect(isSyncing) {
        if (isSyncing) {
            justSynced = false
        } else {
            justSynced = true
            delay(2_000L)
            justSynced = false
        }
    }
    ModalBottomSheet(onDismissRequest = onDismiss) {
        Column(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 24.dp, vertical = 12.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            Text(AppStrings.t(lang, titleKey), color = Color.White, fontSize = 24.sp, fontWeight = FontWeight.Bold)
            SettingsAccountStat(AppStrings.t(lang, "integration.account_info"), email)
            SettingsAccountStat(
                AppStrings.t(lang, "integration.sync_status"),
                when {
                    !connected -> AppStrings.t(lang, "auto.not_connected")
                    isSyncing -> AppStrings.t(lang, "integration.syncing")
                    justSynced -> AppStrings.t(lang, "integration.synced")
                    hasSyncFailure -> AppStrings.t(lang, "integration.sync_failed")
                    lastSyncAt <= 0L -> AppStrings.format(lang, "integration.status_ok", AppStrings.t(lang, "integration.never_synced"))
                    else -> AppStrings.format(lang, "integration.status_ok", AppStrings.t(lang, "integration.just_now"))
                }
            )
            SettingsAccountStat(AppStrings.t(lang, "integration.imported_items"), AppStrings.format(lang, "integration.item_count", if (provider == SettingsAccountProvider.Trakt) model.traktItemCount else model.addonCount))
            SettingsAccountStat(AppStrings.t(lang, "integration.continue_watching"), AppStrings.format(lang, "integration.item_count", if (provider == SettingsAccountProvider.Trakt) model.traktContinueWatchingCount else model.continueWatchingCount))
            SettingsAccountStat(AppStrings.t(lang, "integration.library_items"), AppStrings.format(lang, "integration.item_count", if (provider == SettingsAccountProvider.Trakt) model.traktLibraryCount else 0))
            SettingsAccountStat(AppStrings.t(lang, "integration.addons"), AppStrings.format(lang, "integration.item_count", model.addonCount))
            Button(
                onClick = onSync,
                enabled = !isSyncing,
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black)
            ) {
                Text(
                    when {
                        isSyncing -> AppStrings.t(lang, "integration.syncing")
                        justSynced -> AppStrings.t(lang, "integration.synced")
                        connected -> AppStrings.t(lang, "integration.sync_now")
                        else -> AppStrings.t(lang, "integration.connect")
                    }
                )
            }
        }
    }
}

@Composable
private fun SettingsAccountStat(label: String, value: String) {
    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
        Text(label, color = Color.White.copy(alpha = 0.6f), fontSize = 13.sp)
        Text(value, color = Color.White, fontSize = 13.sp, fontWeight = FontWeight.Bold)
    }
}

@Composable
private fun SettingsTmdbFeaturesContent(model: SettingsAccountUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_cast_images"), value = model.tmdbCastImagesEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbCastImagesEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_similar_results"), value = model.tmdbSimilarResultsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbSimilarResultsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_trailers"), value = model.tmdbTrailersEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbTrailersEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_recommendations"), value = model.tmdbRecommendationsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbRecommendationsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_collection_info"), value = model.tmdbCollectionInfoEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbCollectionInfoEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_episode_images"), value = model.tmdbEpisodeImagesEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbEpisodeImagesEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_logos_backdrops"), value = model.tmdbLogosBackdropsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbLogosBackdropsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_ratings"), value = model.tmdbRatingsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbRatingsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_basic_info"), value = model.tmdbBasicInfoEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbBasicInfoEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_details"), value = model.tmdbDetailsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbDetailsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_productions"), value = model.tmdbProductionsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbProductionsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_networks"), value = model.tmdbNetworksEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbNetworksEnabled = it))) }
    }
}

@Composable
private fun SettingsNotificationsContent(model: SettingsNotificationsUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.enable_notifications"), description = AppStrings.t(lang, "settings.enable_notifications_desc"), value = model.notificationsEnabled) {
            onAction(SettingsAction.NotificationsChanged(model.copy(notificationsEnabled = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.alert_new_episodes"), description = AppStrings.t(lang, "settings.alert_new_episodes_desc"), value = model.alertNewEpisodes) {
            onAction(SettingsAction.NotificationsChanged(model.copy(alertNewEpisodes = it)))
        }
    }
}

@Composable
private fun SettingsGeneralContent(model: SettingsGeneralUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val languageOptions = listOf(SettingsChoiceOption("en", "English"), SettingsChoiceOption("tr", "Türkçe"))
    val startPageOptions = listOf(
        SettingsChoiceOption("home", AppStrings.t(lang, "nav.home")),
        SettingsChoiceOption("discover", AppStrings.t(lang, "nav.discover")),
        SettingsChoiceOption("library", AppStrings.t(lang, "nav.library"))
    )
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "auto.language"), model.language, languageOptions) { onAction(SettingsAction.GeneralChanged(model.copy(language = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "auto.start_page"), model.startPage, startPageOptions) { onAction(SettingsAction.GeneralChanged(model.copy(startPage = it))) }
        SettingsToggleRow(AppStrings.t(lang, "auto.background_playback"), description = AppStrings.t(lang, "settings.background_playback_desc"), value = model.backgroundPlayback) {
            onAction(SettingsAction.GeneralChanged(model.copy(backgroundPlayback = it)))
        }
    }
}

@Composable
private fun SettingsAppearanceContent(model: SettingsAppearanceUiModel, lang: String?, onAction: (SettingsAction) -> Unit, onNavigate: (SettingsCategory) -> Unit) {
    SettingsGroupCard {
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(vertical = 10.dp)) {
            Text(AppStrings.t(lang, "auto.accent_color"), color = Color.White, modifier = Modifier.weight(1f))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                SETTINGS_COLOR_SWATCHES.forEach { swatch ->
                    Box(
                        modifier = Modifier
                            .size(24.dp)
                            .clip(CircleShape)
                            .background(Color(swatch.toInt()))
                            .clickable { onAction(SettingsAction.AppearanceChanged(model.copy(accentColorArgb = swatch))) }
                    )
                }
            }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.amoled"), value = model.amoledMode) { onAction(SettingsAction.AppearanceChanged(model.copy(amoledMode = it))) }
        SettingsToggleRow(AppStrings.t(lang, "auto.disable_animations"), value = !model.animationsEnabled) {
            onAction(SettingsAction.AppearanceChanged(model.copy(animationsEnabled = !it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.floating_bottom_bar"), description = AppStrings.t(lang, "settings.floating_bottom_bar_desc"), value = model.floatingBottomBar) {
            onAction(SettingsAction.AppearanceChanged(model.copy(floatingBottomBar = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.bottom_bar_labels"), description = AppStrings.t(lang, "settings.bottom_bar_labels_desc"), value = model.bottomBarLabels) {
            onAction(SettingsAction.AppearanceChanged(model.copy(bottomBarLabels = it)))
        }
    }
    Spacer(Modifier.height(20.dp))
    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "settings.appearance_home_screen")) { onNavigate(SettingsCategory.AppearanceHome) }
        SettingsNavRow(AppStrings.t(lang, "settings.appearance_detail_screen")) { onNavigate(SettingsCategory.AppearanceDetail) }
    }
}

@Composable
private fun SettingsAppearanceHomeContent(model: SettingsAppearanceHomeUiModel, lang: String?, onAction: (SettingsAction) -> Unit, onNavigate: (SettingsCategory) -> Unit) {
    val cornerOptions = listOf(
        SettingsChoiceOption("sharp", AppStrings.t(lang, "auto.sharp")),
        SettingsChoiceOption("classic", AppStrings.t(lang, "auto.classic")),
        SettingsChoiceOption("soft", AppStrings.t(lang, "auto.soft")),
        SettingsChoiceOption("rounded", AppStrings.t(lang, "auto.rounded")),
        SettingsChoiceOption("pill", AppStrings.t(lang, "auto.extra_rounded"))
    )
    val densityOptions = listOf(
        SettingsChoiceOption("small", AppStrings.t(lang, "auto.small")),
        SettingsChoiceOption("medium", AppStrings.t(lang, "auto.medium")),
        SettingsChoiceOption("large", AppStrings.t(lang, "auto.large"))
    )
    val posterWidthOptions = listOf(
        SettingsChoiceOption("xsmall", AppStrings.t(lang, "auto.very_small")),
        SettingsChoiceOption("small", AppStrings.t(lang, "auto.small")),
        SettingsChoiceOption("medium", AppStrings.t(lang, "auto.medium")),
        SettingsChoiceOption("large", AppStrings.t(lang, "auto.large")),
        SettingsChoiceOption("xlarge", AppStrings.t(lang, "auto.very_large"))
    )
    SettingsSectionHeader(AppStrings.t(lang, "settings.layout"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "auto.card_corners"), model.cardCornerPreset, cornerOptions) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(cardCornerPreset = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "auto.interface_density"), model.interfaceDensity, densityOptions) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(interfaceDensity = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "auto.poster_width"), model.posterWidthPreset, posterWidthOptions) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(posterWidthPreset = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.landscape_mode"), value = model.posterLandscapeMode) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(posterLandscapeMode = it))) }
        SettingsToggleRow(AppStrings.t(lang, "auto.hide_titles"), value = model.posterHideTitles) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(posterHideTitles = it))) }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.sections"))
    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "settings.hero_banner")) { onNavigate(SettingsCategory.AppearanceHomeHero) }
        SettingsNavRow(AppStrings.t(lang, "auto.continue_watching")) { onNavigate(SettingsCategory.AppearanceHomeContinueWatching) }
        SettingsNavRow(AppStrings.t(lang, "settings.navigation")) { onNavigate(SettingsCategory.AppearanceHomeNavigation) }
    }
}

@Composable
private fun SettingsAppearanceHomeHeroContent(model: SettingsAppearanceHomeUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsGroupCard {
        SettingsToggleRow(
            AppStrings.t(lang, "settings.season_posters_on_hero"),
            description = AppStrings.t(lang, "settings.home_season_posters_on_hero_desc"),
            value = model.homeSeasonPostersOnHero
        ) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(homeSeasonPostersOnHero = it))) }
        SettingsToggleRow(
            AppStrings.t(lang, "settings.trailer_on_home_hero"),
            description = AppStrings.t(lang, "settings.trailer_on_home_hero_desc"),
            value = model.trailerOnHomeHeroEnabled
        ) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(trailerOnHomeHeroEnabled = it))) }
        if (model.trailerOnHomeHeroEnabled) {
            SettingsStepperRow(AppStrings.t(lang, "settings.trailer_on_home_hero_delay"), model.trailerOnHomeHeroDelaySeconds, min = 0, max = 15, formatValue = { "${it}s" }) {
                onAction(SettingsAction.AppearanceHomeChanged(model.copy(trailerOnHomeHeroDelaySeconds = it)))
            }
        }
    }
}

@Composable
private fun SettingsAppearanceHomeContinueWatchingContent(model: SettingsAppearanceHomeUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val continueWatchingSourceOptions = listOf(
        SettingsChoiceOption("fluxa", AppStrings.t(lang, "settings.continue_watching_source_fluxa")),
        SettingsChoiceOption("stremio", "Stremio"),
        SettingsChoiceOption("nuvio", "Nuvio"),
        SettingsChoiceOption("trakt", "Trakt"),
        SettingsChoiceOption("simkl", "Simkl"),
        SettingsChoiceOption("anilist", "AniList")
    )
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "auto.continue_watching"), value = model.continueWatchingEnabled) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingEnabled = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.continue_watching_horizontal"), value = model.continueWatchingHorizontal) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingHorizontal = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.continue_watching_hide_titles"), value = model.continueWatchingHideTitles) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingHideTitles = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "settings.continue_watching_source"), model.continueWatchingSource, continueWatchingSourceOptions) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingSource = it)))
        }
    }
}

@Composable
private fun SettingsAppearanceHomeNavigationContent(model: SettingsAppearanceHomeUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.home_top_bar"), description = AppStrings.t(lang, "settings.home_top_bar_desc"), value = model.topBarEnabled) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(topBarEnabled = it)))
        }
    }
}

@Composable
private fun SettingsAppearanceDetailContent(lang: String?, onNavigate: (SettingsCategory) -> Unit) {
    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "settings.hero_banner")) { onNavigate(SettingsCategory.AppearanceDetailHero) }
        SettingsNavRow(AppStrings.t(lang, "settings.episodes")) { onNavigate(SettingsCategory.AppearanceDetailEpisodes) }
    }
}

@Composable
private fun SettingsAppearanceDetailHeroContent(model: SettingsAppearanceDetailUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsGroupCard {
        SettingsToggleRow(
            AppStrings.t(lang, "settings.trailer_on_detail_hero"),
            description = AppStrings.t(lang, "settings.trailer_on_detail_hero_desc"),
            value = model.trailerOnDetailHeroEnabled
        ) { onAction(SettingsAction.AppearanceDetailChanged(model.copy(trailerOnDetailHeroEnabled = it))) }
        if (model.trailerOnDetailHeroEnabled) {
            SettingsStepperRow(AppStrings.t(lang, "settings.trailer_on_detail_hero_delay"), model.trailerOnDetailHeroDelaySeconds, min = 0, max = 15, formatValue = { "${it}s" }) {
                onAction(SettingsAction.AppearanceDetailChanged(model.copy(trailerOnDetailHeroDelaySeconds = it)))
            }
        }
        SettingsToggleRow(
            AppStrings.t(lang, "settings.season_posters_on_hero"),
            description = AppStrings.t(lang, "settings.detail_season_posters_on_hero_desc"),
            value = model.detailSeasonPostersOnHero
        ) { onAction(SettingsAction.AppearanceDetailChanged(model.copy(detailSeasonPostersOnHero = it))) }
    }
}

@Composable
private fun SettingsAppearanceDetailEpisodesContent(model: SettingsAppearanceDetailUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val seasonSelectorOptions = listOf(
        SettingsChoiceOption("dropdown", AppStrings.t(lang, "settings.season_selector_dropdown")),
        SettingsChoiceOption("tabs", AppStrings.t(lang, "settings.season_selector_tabs")),
        SettingsChoiceOption("posters", AppStrings.t(lang, "settings.season_selector_posters"))
    )
    val episodeLayoutOptions = listOf(
        SettingsChoiceOption("list", AppStrings.t(lang, "settings.episode_layout_list")),
        SettingsChoiceOption("horizontal", AppStrings.t(lang, "settings.episode_layout_horizontal"))
    )
    SettingsGroupCard {
        SettingsToggleRow(
            AppStrings.t(lang, "settings.blur_unwatched_episodes"),
            description = AppStrings.t(lang, "settings.blur_unwatched_episodes_desc"),
            value = model.blurUnwatchedEpisodes
        ) { onAction(SettingsAction.AppearanceDetailChanged(model.copy(blurUnwatchedEpisodes = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "settings.season_selector"), model.detailSeasonSelectorMode, seasonSelectorOptions) {
            onAction(SettingsAction.AppearanceDetailChanged(model.copy(detailSeasonSelectorMode = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "settings.episode_cards_layout"), model.episodeCardsLayout, episodeLayoutOptions) {
            onAction(SettingsAction.AppearanceDetailChanged(model.copy(episodeCardsLayout = it)))
        }
    }
}

@Composable
private fun SettingsPlaybackContent(model: SettingsPlaybackUiModel, lang: String?, onAction: (SettingsAction) -> Unit, onNavigate: (SettingsCategory) -> Unit) {
    val playerOptions = listOf(SettingsChoiceOption("internal", "ExoPlayer"), SettingsChoiceOption("mpv", "MPV"))
    val playbackSpeedOptions = listOf("0.75", "1.0", "1.25", "1.5").map { SettingsChoiceOption(it, "${it}x") }
    val seekOptions = listOf("10", "15", "30").map { SettingsChoiceOption(it, "${it}s") }
    val holdSpeedOptions = listOf("1.25", "1.5", "1.75", "2.0", "2.5", "3.0").map { SettingsChoiceOption(it, "${it}x") }
    val streamSourceModeOptions = listOf(
        SettingsChoiceOption("manual", AppStrings.t(lang, "settings.stream_source_manual")),
        SettingsChoiceOption("first", AppStrings.t(lang, "settings.stream_source_first")),
        SettingsChoiceOption("regex", AppStrings.t(lang, "settings.stream_source_regex"))
    )
    val autoplayCountdownOptions = listOf("5", "7", "10", "15").map { SettingsChoiceOption(it, "${it}s") }

    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "auto.subtitles")) { onNavigate(SettingsCategory.Subtitles) }
        SettingsNavRow(AppStrings.t(lang, "settings.advanced_settings")) { onNavigate(SettingsCategory.Advanced) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "auto.playback"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "auto.player"), model.preferredPlayer, playerOptions) { onAction(SettingsAction.PlaybackChanged(model.copy(preferredPlayer = it))) }
        if (model.preferredPlayer == "mpv") {
            SettingsTextFieldRow(AppStrings.t(lang, "settings.mpv_custom_options"), model.mpvCustomOptions) { onAction(SettingsAction.PlaybackChanged(model.copy(mpvCustomOptions = it))) }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.anime_use_mpv"), value = model.animeUseMpv) { onAction(SettingsAction.PlaybackChanged(model.copy(animeUseMpv = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.anime_prefer_japanese_audio"), value = model.animePreferJapaneseAudio) {
            onAction(SettingsAction.PlaybackChanged(model.copy(animePreferJapaneseAudio = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "auto.playback_speed"), formatFloat(model.playbackSpeed), playbackSpeedOptions) {
            onAction(SettingsAction.PlaybackChanged(model.copy(playbackSpeed = it.toFloatOrNull() ?: model.playbackSpeed)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "auto.forward_rewind"), model.seekForwardSeconds.toString(), seekOptions) {
            val seconds = it.toIntOrNull() ?: model.seekForwardSeconds
            onAction(SettingsAction.PlaybackChanged(model.copy(seekForwardSeconds = seconds, seekBackwardSeconds = seconds)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.hold_to_speed"), value = model.holdToSpeedEnabled) { onAction(SettingsAction.PlaybackChanged(model.copy(holdToSpeedEnabled = it))) }
        if (model.holdToSpeedEnabled) {
            SettingsChoiceRow(AppStrings.t(lang, "settings.hold_speed"), formatFloat(model.holdSpeed), holdSpeedOptions) {
                onAction(SettingsAction.PlaybackChanged(model.copy(holdSpeed = it.toFloatOrNull() ?: model.holdSpeed)))
            }
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.stream_settings"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.stream_source_selection"), model.streamSourceSelectionMode, streamSourceModeOptions) {
            onAction(SettingsAction.PlaybackChanged(model.copy(streamSourceSelectionMode = it)))
        }
        if (model.streamSourceSelectionMode == "regex") {
            SettingsTextFieldRow(AppStrings.t(lang, "settings.regex_pattern"), model.streamSourceRegexPattern) {
                onAction(SettingsAction.PlaybackChanged(model.copy(streamSourceRegexPattern = it)))
            }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.auto_play_next_episode"), value = model.autoplayMode == "next_episode") {
            onAction(SettingsAction.PlaybackChanged(model.copy(autoplayMode = if (it) "next_episode" else "off", autoPlayNextEpisode = it)))
        }
        if (model.autoplayMode == "next_episode") {
            SettingsChoiceRow(AppStrings.t(lang, "settings.autoplay_countdown"), model.autoPlayCountdownSecs.toString(), autoplayCountdownOptions) {
                onAction(SettingsAction.PlaybackChanged(model.copy(autoPlayCountdownSecs = it.toIntOrNull() ?: model.autoPlayCountdownSecs)))
            }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.auto_retry_next_source"), value = model.autoRetryNextSource) { onAction(SettingsAction.PlaybackChanged(model.copy(autoRetryNextSource = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.try_binge_group"), value = model.tryBingeGroup) { onAction(SettingsAction.PlaybackChanged(model.copy(tryBingeGroup = it))) }
        SettingsPercentSliderRow(AppStrings.t(lang, "settings.next_episode_threshold"), model.nextEpisodeThresholdPercent) {
            onAction(SettingsAction.PlaybackChanged(model.copy(nextEpisodeThresholdPercent = it)))
        }
        SettingsPercentSliderRow(AppStrings.t(lang, "settings.watched_threshold"), model.watchedThresholdPercent) {
            onAction(SettingsAction.PlaybackChanged(model.copy(watchedThresholdPercent = it)))
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.skip_segments"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.use_introdb"), value = model.useIntroDb) { onAction(SettingsAction.PlaybackChanged(model.copy(useIntroDb = it))) }
        if (model.useIntroDb) {
            SettingsTextFieldRow(AppStrings.t(lang, "settings.introdb_api_key"), model.introDbApiKey) { onAction(SettingsAction.PlaybackChanged(model.copy(introDbApiKey = it))) }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.use_aniskip"), value = model.useAniSkip) { onAction(SettingsAction.PlaybackChanged(model.copy(useAniSkip = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.use_chapter_skip"), value = model.useChapterSkip) { onAction(SettingsAction.PlaybackChanged(model.copy(useChapterSkip = it))) }
        if (model.useIntroDb || model.useAniSkip) {
            SettingsToggleRow(AppStrings.t(lang, "settings.auto_skip"), value = model.autoSkipIntro) { onAction(SettingsAction.PlaybackChanged(model.copy(autoSkipIntro = it))) }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.content_warnings_enabled"), value = model.contentWarningsEnabled) { onAction(SettingsAction.PlaybackChanged(model.copy(contentWarningsEnabled = it))) }
    }

    Spacer(Modifier.height(20.dp))
    SettingsGroupCard {
        SettingsActionRow(AppStrings.t(lang, "settings.reset_to_defaults"), destructive = true) {
            onAction(SettingsAction.PlaybackChanged(SettingsPlaybackUiModel()))
        }
    }
}

private fun formatFloat(value: Float): String {
    val rounded = (value * 100).toInt() / 100f
    return if (rounded == rounded.toInt().toFloat()) rounded.toInt().toString() + ".0" else rounded.toString()
}

@Composable
private fun SettingsSubtitlesContent(model: SettingsSubtitlesUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val languageOptions = listOf("none", "original", "device_language", "en", "tr", "ja", "ko", "es", "fr", "de").map {
        SettingsChoiceOption(it, languageOptionLabel(it, lang))
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.preferences"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.preferred_audio_language"), model.preferredAudioLanguage, languageOptions) {
            onAction(SettingsAction.SubtitlesChanged(model.copy(preferredAudioLanguage = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "settings.secondary_audio_language"), model.secondaryAudioLanguage, languageOptions) {
            onAction(SettingsAction.SubtitlesChanged(model.copy(secondaryAudioLanguage = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "settings.preferred_subtitle_language"), model.preferredSubtitleLanguage, languageOptions) {
            onAction(SettingsAction.SubtitlesChanged(model.copy(preferredSubtitleLanguage = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "settings.secondary_subtitle_language"), model.secondarySubtitleLanguage, languageOptions) {
            onAction(SettingsAction.SubtitlesChanged(model.copy(secondarySubtitleLanguage = it)))
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.subtitle.customize"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.auto_enable_subtitles"), value = model.autoEnableSubtitles) {
            onAction(SettingsAction.SubtitlesChanged(model.copy(autoEnableSubtitles = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.subtitle_shadow"), value = model.subtitleShadow) { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleShadow = it))) }
        SettingsStepperRow(AppStrings.t(lang, "settings.subtitle_size"), model.subtitleSize.toInt(), step = 10, min = 50, max = 200, formatValue = { "$it%" }) {
            onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleSize = it.toFloat())))
        }
        SettingsColorOpacityRow(
            AppStrings.t(lang, "settings.subtitle_text"), model.subtitleColorArgb, model.subtitleTextOpacity,
            onColorChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleColorArgb = it))) },
            onOpacityChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleTextOpacity = it))) }
        )
        SettingsColorOpacityRow(
            AppStrings.t(lang, "settings.subtitle_background"), model.subtitleBackgroundColorArgb, model.subtitleBackgroundOpacity,
            onColorChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleBackgroundColorArgb = it))) },
            onOpacityChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleBackgroundOpacity = it))) }
        )
        SettingsColorOpacityRow(
            AppStrings.t(lang, "settings.subtitle_outline"), model.subtitleOutlineColorArgb, model.subtitleOutlineOpacity,
            onColorChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleOutlineColorArgb = it))) },
            onOpacityChanged = { onAction(SettingsAction.SubtitlesChanged(model.copy(subtitleOutlineOpacity = it))) }
        )
    }
}

private fun languageOptionLabel(value: String, lang: String?): String = when (value) {
    "none" -> AppStrings.t(lang, "settings.none")
    "original" -> AppStrings.t(lang, "settings.original")
    "device_language" -> AppStrings.t(lang, "settings.device_language")
    "en" -> "English"
    "tr" -> "Türkçe"
    else -> value.uppercase()
}

@Composable
private fun SettingsAdvancedContent(model: SettingsAdvancedUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val bufferCacheOptions = listOf("100", "500", "1000", "2000").map { SettingsChoiceOption(it, "$it MB") }
    val bufferSecondOptions = listOf("0", "15", "30", "60", "120").map { SettingsChoiceOption(it, "${it}s") }
    val audioDecoderOptions = listOf(
        SettingsChoiceOption("hw_prefer", AppStrings.t(lang, "settings.audio_decoder_hw_prefer")),
        SettingsChoiceOption("hw_only", AppStrings.t(lang, "settings.audio_decoder_hw_only")),
        SettingsChoiceOption("sw_only", AppStrings.t(lang, "settings.audio_decoder_sw_only"))
    )
    SettingsSectionHeader(AppStrings.t(lang, "settings.advanced"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.buffer_cache"), model.playerBufferCacheMb.toString(), bufferCacheOptions) {
            onAction(SettingsAction.AdvancedChanged(model.copy(playerBufferCacheMb = it.toIntOrNull() ?: model.playerBufferCacheMb)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "settings.forward_buffer"), model.playerForwardBufferSeconds.toString(), bufferSecondOptions) {
            onAction(SettingsAction.AdvancedChanged(model.copy(playerForwardBufferSeconds = it.toIntOrNull() ?: model.playerForwardBufferSeconds)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "settings.back_buffer"), model.playerBackBufferSeconds.toString(), bufferSecondOptions) {
            onAction(SettingsAction.AdvancedChanged(model.copy(playerBackBufferSeconds = it.toIntOrNull() ?: model.playerBackBufferSeconds)))
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.decoder"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.audio_decoder_mode"), model.audioDecoderMode, audioDecoderOptions) {
            onAction(SettingsAction.AdvancedChanged(model.copy(audioDecoderMode = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.tunneled_playback"), value = model.tunneledPlayback) { onAction(SettingsAction.AdvancedChanged(model.copy(tunneledPlayback = it))) }
    }

    Spacer(Modifier.height(20.dp))
    SettingsGroupCard {
        SettingsActionRow(AppStrings.t(lang, "settings.reset_to_defaults"), destructive = true) {
            onAction(SettingsAction.AdvancedChanged(SettingsAdvancedUiModel()))
        }
    }
}

@Composable
private fun SettingsContentCategoryContent(model: SettingsContentUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.show_hero_section"), description = AppStrings.t(lang, "settings.show_hero_section_desc"), value = model.showHeroSection) {
            onAction(SettingsAction.ShowHeroSectionChanged(it))
        }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.hero_catalogs"))
    SettingsGroupCard {
        model.heroFeeds.forEach { feed ->
            SettingsOrderedToggleRow(
                label = feed.label,
                subtitle = feed.providerLabel,
                selected = feed.selected,
                canMoveUp = feed.canMoveUp,
                canMoveDown = feed.canMoveDown,
                onToggle = { onAction(SettingsAction.HeroFeedToggled(feed.key)) },
                onMoveUp = { onAction(SettingsAction.HeroFeedMoved(feed.key, -1)) },
                onMoveDown = { onAction(SettingsAction.HeroFeedMoved(feed.key, 1)) }
            )
        }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.home_catalogs"))
    SettingsGroupCard {
        model.homeFeeds.forEach { feed ->
            SettingsOrderedToggleRow(
                label = feed.label,
                subtitle = feed.providerLabel,
                selected = feed.selected,
                canMoveUp = feed.canMoveUp,
                canMoveDown = feed.canMoveDown,
                onToggle = { onAction(SettingsAction.HomeFeedToggled(feed.key)) },
                onMoveUp = { onAction(SettingsAction.HomeFeedMoved(feed.key, -1)) },
                onMoveDown = { onAction(SettingsAction.HomeFeedMoved(feed.key, 1)) }
            )
        }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.top_10_catalogs"))
    SettingsGroupCard {
        model.topTenFeeds.forEach { feed ->
            SettingsToggleRow(feed.label, value = feed.selected) { onAction(SettingsAction.TopTenFeedToggled(feed.key)) }
        }
    }
}

@Composable
private fun SettingsDownloadsContent(model: SettingsDownloadsUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val streamSourceModeOptions = listOf(
        SettingsChoiceOption("manual", AppStrings.t(lang, "settings.stream_source_manual")),
        SettingsChoiceOption("first", AppStrings.t(lang, "settings.stream_source_first")),
        SettingsChoiceOption("regex", AppStrings.t(lang, "settings.stream_source_regex"))
    )
    val downloadSubtitleOptions = listOf(
        SettingsChoiceOption("preferred", AppStrings.t(lang, "settings.download_subtitle_preferred")),
        SettingsChoiceOption("off", AppStrings.t(lang, "settings.download_subtitle_off")),
        SettingsChoiceOption("tr", "Türkçe"),
        SettingsChoiceOption("en", "English")
    )
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "settings.download_source_selection"), model.downloadSourceSelectionMode, streamSourceModeOptions) {
            onAction(SettingsAction.DownloadsChanged(model.copy(downloadSourceSelectionMode = it)))
        }
        if (model.downloadSourceSelectionMode == "regex") {
            SettingsTextFieldRow(AppStrings.t(lang, "settings.regex_pattern"), model.downloadSourceRegexPattern) {
                onAction(SettingsAction.DownloadsChanged(model.copy(downloadSourceRegexPattern = it)))
            }
        }
        SettingsChoiceRow(AppStrings.t(lang, "settings.download_subtitle"), model.downloadSubtitleLanguage, downloadSubtitleOptions) {
            onAction(SettingsAction.DownloadsChanged(model.copy(downloadSubtitleLanguage = it)))
        }
    }

}

@Composable
private fun SettingsDeveloperContent(model: SettingsDeveloperUiModel, lang: String?) {
    SettingsSectionHeader(AppStrings.t(lang, "settings.last_media_probe"))
    SettingsGroupCard {
    if (model.lastProbeUpdatedAt == null) {
        Text(AppStrings.t(lang, "settings.no_media_probe"), color = Color.White.copy(alpha = 0.5f))
    } else {
        SettingsInfoRow(AppStrings.t(lang, "settings.last_media_probe_updated"), model.lastProbeUpdatedAt)
        SettingsInfoRow(AppStrings.t(lang, "settings.last_media_probe_title"), model.lastProbeTitle.orEmpty())
        SettingsInfoRow(AppStrings.t(lang, "settings.last_media_probe_url"), model.lastProbeUrl.orEmpty())
    }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.media_file_data"))
    SettingsGroupCard {
        Text(model.technicalInfo, color = Color.White.copy(alpha = 0.7f), fontSize = 12.sp, modifier = Modifier.padding(vertical = 10.dp))
    }
}
