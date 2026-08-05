package com.fluxa.app.shared.feature.settings

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material.icons.filled.Movie
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import kotlinx.coroutines.delay
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
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
    val scrollStates = remember { mutableStateMapOf<SettingsCategory, ScrollState>() }
    var highlightLabel by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(highlightLabel) {
        if (highlightLabel != null) {
            delay(2_000L)
            highlightLabel = null
        }
    }
    val navigateAndHighlight: (SettingsSearchEntry) -> Unit = { entry ->
        highlightLabel = entry.label
        onPushCategory(entry.category)
    }
    val accentColor = Color(state.appearance.accentColorArgb.toInt())

    if (deviceType == com.fluxa.app.ui.catalog.DeviceType.TV) {
        val selectedCategory = if (category == SettingsCategory.Hub) SettingsCategory.Account else category
        CompositionLocalProvider(LocalSettingsAccentColor provides accentColor) {
        Row(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
            Column(
                modifier = Modifier.width(300.dp).fillMaxSize().padding(24.dp)
            ) {
                Text(AppStrings.t(lang, "nav.settings"), style = MaterialTheme.typography.titleLarge, color = Color.White)
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
            AnimatedContent(
                targetState = selectedCategory,
                transitionSpec = { fadeIn(tween(180)).togetherWith(fadeOut(tween(120))) },
                label = "settings-tv-category-transition",
                modifier = Modifier.weight(1f).fillMaxSize()
            ) { animatedCategory ->
                Column(
                    modifier = Modifier.fillMaxSize().padding(24.dp).verticalScroll(scrollStates.getOrPut(animatedCategory) { ScrollState(0) })
                ) {
                    Text(settingsCategoryTitle(animatedCategory, lang), style = MaterialTheme.typography.titleLarge, color = Color.White)
                    Spacer(Modifier.height(16.dp))
                    CompositionLocalProvider(LocalSettingsHighlightLabel provides highlightLabel) {
                        SettingsCategoryContent(animatedCategory, state, lang, brandIcons, onAction, onPushCategory, onSwitchProfilesRequested, navigateAndHighlight)
                    }
                    Spacer(Modifier.height(120.dp))
                }
            }
        }
        }
        return
    }

    var previousDepth by remember { mutableStateOf(backStack.size) }
    val forward = backStack.size >= previousDepth
    SideEffect { previousDepth = backStack.size }

    CompositionLocalProvider(LocalSettingsAccentColor provides accentColor) {
    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        Column(modifier = Modifier.fillMaxSize().windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Top))) {
            Box(modifier = Modifier.padding(horizontal = 16.dp)) {
                SettingsTopBarTitle(
                    title = settingsCategoryTitle(category, lang),
                    showBack = category != SettingsCategory.Hub,
                    onBack = { if (backStack.isEmpty()) onBackRequested() else onPopCategory() },
                    badge = settingsCategoryBadge(category)
                )
            }
            SettingsTopBarDivider()
            AnimatedContent(
                targetState = category,
                transitionSpec = {
                    val direction = if (forward) 1 else -1
                    (slideInHorizontally(tween(220)) { direction * it } + fadeIn(tween(220)))
                        .togetherWith(slideOutHorizontally(tween(220)) { -direction * it } + fadeOut(tween(220)))
                },
                label = "settings-category-transition",
                modifier = Modifier.fillMaxSize()
            ) { animatedCategory ->
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .verticalScroll(scrollStates.getOrPut(animatedCategory) { ScrollState(0) })
                    .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Bottom))
                    .padding(horizontal = 16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                CompositionLocalProvider(LocalSettingsHighlightLabel provides highlightLabel) {
                    SettingsCategoryContent(animatedCategory, state, lang, brandIcons, onAction, onPushCategory, onSwitchProfilesRequested, navigateAndHighlight)
                }
                Spacer(Modifier.height(120.dp))
            }
            }
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
    onSwitchProfiles: () -> Unit,
    onNavigateSearchResult: (SettingsSearchEntry) -> Unit
) {
    when (category) {
        SettingsCategory.Hub -> SettingsHubContent(
            state = state,
            lang = lang,
            onNavigate = onNavigate,
            onSwitchProfiles = onSwitchProfiles,
            onAction = onAction,
            onNavigateSearchResult = onNavigateSearchResult
        )
        SettingsCategory.Account -> SettingsAccountContent(
            model = state.account,
            lang = lang,
            brandIcons = brandIcons,
            onAction = onAction,
            onNavigate = onNavigate,
            continueWatchingSource = state.appearanceHome.continueWatchingSource,
            onContinueWatchingSourceChanged = { onAction(SettingsAction.AppearanceHomeChanged(state.appearanceHome.copy(continueWatchingSource = it))) }
        )
        SettingsCategory.TmdbFeatures -> SettingsTmdbFeaturesContent(state.account, lang, onAction)
        SettingsCategory.MdblistApi -> SettingsMdblistApiContent(state.account, lang, onAction)
        SettingsCategory.Notifications -> SettingsNotificationsContent(state.notifications, lang, onAction)
        SettingsCategory.General -> SettingsGeneralContent(state.general, lang, onAction)
        SettingsCategory.Appearance -> SettingsAppearanceContent(state.appearance, lang, onAction, onNavigate = onNavigate)
        SettingsCategory.AppearanceHome -> SettingsAppearanceHomeContent(state.appearanceHome, lang, onAction)
        SettingsCategory.AppearanceDetail -> SettingsAppearanceDetailContent(state.appearanceDetail, lang, onAction)
        SettingsCategory.Playback -> SettingsPlaybackCoreContent(state.playback, state.subtitles, lang, onAction, onNavigate = onNavigate)
        SettingsCategory.PlaybackStream -> SettingsPlaybackStreamContent(state.playback, lang, onAction, onNavigate = onNavigate)
        SettingsCategory.PlaybackSkip -> SettingsPlaybackSkipContent(state.playback, lang, onAction)
        SettingsCategory.Subtitles -> SettingsSubtitlesContent(state.subtitles, lang, onAction)
        SettingsCategory.Advanced -> SettingsAdvancedContent(state.advanced, lang, onAction)
        SettingsCategory.Content -> SettingsContentCategoryContent(state.content, lang, onAction)
        SettingsCategory.Downloads -> SettingsDownloadsContent(state.downloads, lang, onAction)
        SettingsCategory.Developer -> SettingsDeveloperContent(state.developer, lang)
        SettingsCategory.AccountStremio -> SettingsAccountDetailContent(SettingsAccountProvider.Stremio, state.account, lang, onAction)
        SettingsCategory.AccountNuvio -> SettingsAccountDetailContent(SettingsAccountProvider.Nuvio, state.account, lang, onAction)
        SettingsCategory.AccountTrakt -> SettingsAccountDetailContent(SettingsAccountProvider.Trakt, state.account, lang, onAction)
        SettingsCategory.AccountSimkl -> SettingsAccountDetailContent(SettingsAccountProvider.Simkl, state.account, lang, onAction)
        SettingsCategory.AccountAnilist -> SettingsAccountDetailContent(SettingsAccountProvider.Anilist, state.account, lang, onAction)
    }
}

@Composable
private fun SettingsHubContent(
    state: SettingsUiState,
    lang: String?,
    onNavigate: (SettingsCategory) -> Unit,
    onSwitchProfiles: () -> Unit,
    onAction: (SettingsAction) -> Unit,
    onNavigateSearchResult: (SettingsSearchEntry) -> Unit
) {
    var searchQuery by remember { mutableStateOf("") }
    val keyboardController = LocalSoftwareKeyboardController.current
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
                keyboardOptions = androidx.compose.foundation.text.KeyboardOptions(imeAction = ImeAction.Search),
                keyboardActions = androidx.compose.foundation.text.KeyboardActions(onSearch = { keyboardController?.hide() }),
                modifier = Modifier.fillMaxWidth()
            )
        }
        if (searchQuery.isNotEmpty()) {
            Box(
                modifier = Modifier.size(24.dp).clip(CircleShape).clickable { searchQuery = "" },
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.Filled.Clear,
                    contentDescription = null,
                    tint = Color.White.copy(alpha = 0.4f),
                    modifier = Modifier.size(16.dp)
                )
            }
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
                    SettingsNavRow(entry.label, value = settingsCategoryTitle(entry.category, lang)) { onNavigateSearchResult(entry) }
                }
            }
        }
        return
    }

    SettingsGroupCard {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().clickable { onNavigate(SettingsCategory.Account) }.padding(vertical = 14.dp)
        ) {
            Box(contentAlignment = Alignment.Center) {
                Box(
                    modifier = Modifier
                        .size(66.dp)
                        .background(
                            Brush.radialGradient(listOf(LocalSettingsAccentColor.current.copy(alpha = 0.3f), Color.Transparent)),
                            CircleShape
                        )
                )
                Box(
                    modifier = Modifier
                        .size(52.dp)
                        .clip(CircleShape)
                        .background(FluxaColors.surfaceRaised)
                        .border(1.5.dp, LocalSettingsAccentColor.current.copy(alpha = 0.4f), CircleShape),
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
                        com.fluxa.app.shared.feature.profile.ProfileDefaultAvatar(modifier = Modifier.size(28.dp))
                    }
                }
            }
            Spacer(Modifier.width(16.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(state.account.displayName, style = MaterialTheme.typography.titleMedium, color = Color.White)
                Text(AppStrings.t(lang, "auto.account"), color = Color.White.copy(alpha = 0.5f), fontSize = 13.sp)
            }
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = Color.White.copy(alpha = 0.35f),
                modifier = Modifier.size(20.dp)
            )
        }
        SettingsNavRow(
            AppStrings.t(lang, "settings.switch_profiles")
        ) { onSwitchProfiles() }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.section_preferences"))
    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "settings.notifications_title")) { onNavigate(SettingsCategory.Notifications) }
        SettingsNavRow(AppStrings.t(lang, "auto.general")) { onNavigate(SettingsCategory.General) }
        SettingsNavRow(AppStrings.t(lang, "auto.appearance")) { onNavigate(SettingsCategory.Appearance) }
        SettingsNavRow(AppStrings.t(lang, "auto.playback")) { onNavigate(SettingsCategory.Playback) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.section_content"))
    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "auto.catalogs")) { onNavigate(SettingsCategory.Content) }
        SettingsNavRow(AppStrings.t(lang, "auto.add_ons")) { onAction(SettingsAction.ManageAddonsRequested) }
        SettingsNavRow(AppStrings.t(lang, "settings.plugins.manage")) { onAction(SettingsAction.ManagePluginsRequested) }
        SettingsNavRow(AppStrings.t(lang, "auto.downloads")) { onNavigate(SettingsCategory.Downloads) }
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
        SettingsNavRow(AppStrings.t(lang, "settings.developer")) { onNavigate(SettingsCategory.Developer) }
    }

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
    onNavigate: (SettingsCategory) -> Unit,
    continueWatchingSource: String?,
    onContinueWatchingSourceChanged: (String) -> Unit
) {
    SettingsSectionHeader(AppStrings.t(lang, "auto.account_sync"))
    SettingsGroupCard {
        val syncFailedLabel = AppStrings.t(lang, "integration.sync_failed")
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.stremio"),
            connected = model.hasStremio,
            connectedLabel = if (model.email.isNotBlank()) AppStrings.format(lang, "settings.connected_as", model.email) else AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.stremio,
            hasSyncFailure = "stremio" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { onNavigate(SettingsCategory.AccountStremio) }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.nuvio"),
            connected = model.hasNuvio,
            connectedLabel = model.nuvioEmail?.takeIf { it.isNotBlank() }?.let { AppStrings.format(lang, "settings.connected_as", it) } ?: AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.nuvio,
            hasSyncFailure = "nuvio" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { onNavigate(SettingsCategory.AccountNuvio) }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.trakt"),
            connected = model.hasTrakt,
            connectedLabel = model.traktUsername?.takeIf { it.isNotBlank() }?.let { AppStrings.format(lang, "settings.connected_as", it) } ?: AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.trakt,
            hasSyncFailure = "trakt" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { onNavigate(SettingsCategory.AccountTrakt) }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.simkl"),
            connected = model.hasSimkl,
            connectedLabel = model.simklUsername?.takeIf { it.isNotBlank() }?.let { AppStrings.format(lang, "settings.connected_as", it) } ?: AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.simkl,
            hasSyncFailure = "simkl" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { onNavigate(SettingsCategory.AccountSimkl) }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.anilist"),
            connected = model.hasAnilist,
            connectedLabel = model.anilistUsername?.takeIf { it.isNotBlank() }?.let { AppStrings.format(lang, "settings.connected_as", it) } ?: AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.anilist,
            hasSyncFailure = "anilist" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { onNavigate(SettingsCategory.AccountAnilist) }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.sync_preferences"))
    SettingsGroupCard {
        val connectedSourceOptions = buildList {
            add(SettingsChoiceOption("stremio", AppStrings.t(lang, "settings.cw_source_of_truth_local")))
            if (model.hasNuvio) add(SettingsChoiceOption("nuvio", AppStrings.t(lang, "settings.cw_source_of_truth_nuvio")))
            if (model.hasTrakt) add(SettingsChoiceOption("trakt", AppStrings.t(lang, "settings.cw_source_of_truth_trakt")))
            if (model.hasSimkl) add(SettingsChoiceOption("simkl", AppStrings.t(lang, "settings.cw_source_of_truth_simkl")))
            if (model.hasAnilist) add(SettingsChoiceOption("anilist", AppStrings.t(lang, "settings.cw_source_of_truth_anilist")))
        }
        val connectedLibraryOptions = buildList {
            add(SettingsChoiceOption("local", AppStrings.t(lang, "settings.cw_source_of_truth_local")))
            if (model.hasStremio) add(SettingsChoiceOption("stremio", AppStrings.t(lang, "settings.cw_source_of_truth_stremio")))
            if (model.hasNuvio) add(SettingsChoiceOption("nuvio", AppStrings.t(lang, "settings.cw_source_of_truth_nuvio")))
            if (model.hasTrakt) add(SettingsChoiceOption("trakt", AppStrings.t(lang, "settings.cw_source_of_truth_trakt")))
            if (model.hasSimkl) add(SettingsChoiceOption("simkl", AppStrings.t(lang, "settings.cw_source_of_truth_simkl")))
            if (model.hasAnilist) add(SettingsChoiceOption("anilist", AppStrings.t(lang, "settings.cw_source_of_truth_anilist")))
        }
        SettingsChoiceRow(
            label = AppStrings.t(lang, "settings.continue_watching_source"),
            value = continueWatchingSource ?: "stremio",
            options = connectedSourceOptions
        ) { onContinueWatchingSourceChanged(it) }
        SettingsChoiceRow(
            label = AppStrings.t(lang, "settings.library_source_of_truth"),
            value = model.integrationLibrarySource,
            options = connectedLibraryOptions
        ) { onAction(SettingsAction.TmdbAccountChanged(model.copy(integrationLibrarySource = it))) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.apis"))
    val tmdbConfigured = !model.tmdbApiKey.isNullOrBlank()
    val mdblistConfigured = !model.mdblistApiKey.isNullOrBlank()
    SettingsGroupCard {
        SettingsNavRow(
            AppStrings.t(lang, "brand.tmdb"),
            value = AppStrings.t(lang, if (tmdbConfigured) "settings.tmdb_api_configured" else "settings.tmdb_api_not_configured")
        ) { onNavigate(SettingsCategory.TmdbFeatures) }
        SettingsNavRow(
            AppStrings.t(lang, "settings.mdblist_api"),
            value = AppStrings.t(lang, if (mdblistConfigured) "settings.tmdb_api_configured" else "settings.tmdb_api_not_configured")
        ) { onNavigate(SettingsCategory.MdblistApi) }
    }
}

@Composable
private fun SettingsMdblistApiContent(model: SettingsAccountUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsSectionHeader(AppStrings.t(lang, "settings.mdblist_api"))
    SettingsInlineSecretField(
        AppStrings.t(lang, "settings.mdblist_api_key"),
        model.mdblistApiKey.orEmpty(),
        placeholder = AppStrings.t(lang, "settings.mdblist_api_key_placeholder")
    ) {
        onAction(SettingsAction.TmdbAccountChanged(model.copy(mdblistApiKey = it)))
    }
}

internal enum class SettingsAccountProvider { Stremio, Nuvio, Trakt, Simkl, Anilist }

@Composable
private fun SettingsAccountDetailContent(
    provider: SettingsAccountProvider,
    model: SettingsAccountUiModel,
    lang: String?,
    onAction: (SettingsAction) -> Unit
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
    val email = when (provider) {
        SettingsAccountProvider.Nuvio -> model.nuvioEmail.orEmpty().ifBlank { model.email }
        SettingsAccountProvider.Simkl -> model.simklUsername.orEmpty().ifBlank { model.email }
        SettingsAccountProvider.Anilist -> model.anilistUsername.orEmpty().ifBlank { model.email }
        else -> model.email
    }
    val providerKey = when (provider) {
        SettingsAccountProvider.Stremio -> "stremio"
        SettingsAccountProvider.Nuvio -> "nuvio"
        SettingsAccountProvider.Trakt -> "trakt"
        SettingsAccountProvider.Simkl -> "simkl"
        SettingsAccountProvider.Anilist -> "anilist"
    }
    val hasSyncFailure = providerKey in model.syncFailedProviders
    val isSyncing = providerKey in model.syncingProviders
    val isCredentialProvider = provider == SettingsAccountProvider.Stremio || provider == SettingsAccountProvider.Nuvio
    var justSynced by remember(provider) { mutableStateOf(false) }
    var confirmingDisconnect by remember(provider) { mutableStateOf(false) }
    var showCredentialForm by remember(provider) { mutableStateOf(false) }
    LaunchedEffect(isSyncing) {
        if (isSyncing) {
            justSynced = false
        } else {
            justSynced = true
            delay(2_000L)
            justSynced = false
        }
    }
    LaunchedEffect(connected) {
        if (connected) showCredentialForm = false
    }
    val onSync = {
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
    val onSyncNow = {
        if (connected && (provider == SettingsAccountProvider.Simkl || provider == SettingsAccountProvider.Anilist)) {
            onAction(SettingsAction.SyncProviderRequested(providerKey))
        } else {
            onSync()
        }
    }
    val onConnect = {
        if (isCredentialProvider) {
            showCredentialForm = true
        } else {
            onSync()
        }
    }

    if (connected) {
        SettingsConnectedAccountCard(
            statusLabel = AppStrings.t(lang, "settings.connected_account"),
            email = email,
            badgeText = AppStrings.t(lang, "auto.connected")
        )
        if (hasSyncFailure) {
            SettingsGroupCard {
                SettingsInfoRow(AppStrings.t(lang, "common.error"), AppStrings.t(lang, "integration.sync_failed"))
            }
        }
    } else {
    SettingsSectionHeader(AppStrings.t(lang, "settings.sync_with"))
    SettingsGroupCard {
        SettingsActionRow(
            label = AppStrings.t(lang, titleKey),
            value = AppStrings.t(lang, "settings.connect_account"),
            onClick = onConnect
        )
        if (hasSyncFailure) {
            SettingsInfoRow(AppStrings.t(lang, "common.error"), AppStrings.t(lang, "integration.sync_failed"))
        }
    }
    }

    if (!connected && isCredentialProvider && showCredentialForm) {
        SettingsCredentialLoginForm(
            lang = lang,
            busy = isSyncing,
            errorMessage = model.connectErrors[providerKey]?.let { code ->
                when {
                    code == "invalid_credentials" && provider == SettingsAccountProvider.Stremio -> AppStrings.t(lang, "login.stremio_failed")
                    code == "invalid_credentials" -> AppStrings.t(lang, "auth.error.invalid_credentials")
                    else -> AppStrings.format(lang, "login.connection_error", code)
                }
            },
            onSubmit = { email, password ->
                onAction(
                    if (provider == SettingsAccountProvider.Stremio) {
                        SettingsAction.ConnectStremioWithCredentials(email, password)
                    } else {
                        SettingsAction.ConnectNuvioWithCredentials(email, password)
                    }
                )
            },
            onCancel = { showCredentialForm = false }
        )
    }

    if (connected) {
        SettingsSectionHeader(AppStrings.t(lang, "settings.provider_library"))
        SettingsGroupCard {
            SettingsInfoRow(
                AppStrings.t(lang, "integration.sync_now"),
                when {
                    isSyncing -> AppStrings.t(lang, "integration.syncing")
                    justSynced -> AppStrings.t(lang, "integration.synced")
                    lastSyncAt <= 0L -> AppStrings.t(lang, "integration.never_synced")
                    else -> AppStrings.t(lang, "integration.just_now")
                }
            )
            SettingsInfoRow(AppStrings.t(lang, "integration.account_info"), email)
            val importedItems = when (provider) {
                SettingsAccountProvider.Trakt -> model.traktItemCount
                SettingsAccountProvider.Simkl -> model.simklItemCount
                SettingsAccountProvider.Anilist -> model.anilistItemCount
                else -> model.addonCount
            }
            SettingsInfoRow(AppStrings.t(lang, "integration.imported_items"), AppStrings.format(lang, "integration.item_count", importedItems))
            val continueWatchingItems = when (provider) {
                SettingsAccountProvider.Trakt -> model.traktContinueWatchingCount
                SettingsAccountProvider.Simkl -> model.simklContinueWatchingCount
                SettingsAccountProvider.Anilist -> model.anilistContinueWatchingCount
                else -> null
            }
            if (continueWatchingItems != null) {
                SettingsInfoRow(AppStrings.t(lang, "integration.continue_watching"), AppStrings.format(lang, "integration.item_count", continueWatchingItems))
            }
            val libraryItems = when (provider) {
                SettingsAccountProvider.Trakt -> model.traktLibraryCount
                SettingsAccountProvider.Simkl -> model.simklLibraryCount
                SettingsAccountProvider.Anilist -> model.anilistLibraryCount
                else -> null
            }
            if (libraryItems != null) {
                SettingsInfoRow(AppStrings.t(lang, "integration.library_items"), AppStrings.format(lang, "integration.item_count", libraryItems))
            }
            if (provider == SettingsAccountProvider.Nuvio || provider == SettingsAccountProvider.Stremio) {
                SettingsInfoRow(AppStrings.t(lang, "integration.addons"), AppStrings.format(lang, "integration.item_count", model.addonCount))
            }
        }

        if (provider == SettingsAccountProvider.Trakt) {
            SettingsSectionHeader(AppStrings.t(lang, "brand.trakt"))
            SettingsGroupCard {
                SettingsChoiceRow(
                    label = AppStrings.t(lang, "settings.continue_watching_window"),
                    value = model.continueWatchingDays.toString(),
                    options = listOf(
                        SettingsChoiceOption("0", AppStrings.t(lang, "settings.continue_watching_window_all")),
                        SettingsChoiceOption("7", "7"),
                        SettingsChoiceOption("30", "30"),
                        SettingsChoiceOption("90", "90"),
                        SettingsChoiceOption("365", "365")
                    )
                ) { value -> onAction(SettingsAction.TmdbAccountChanged(model.copy(continueWatchingDays = value.toInt()))) }
                SettingsToggleRow(
                    label = AppStrings.t(lang, "settings.trakt_comments"),
                    description = AppStrings.t(lang, "settings.trakt_comments_desc"),
                    value = model.traktCommentsEnabled
                ) { onAction(SettingsAction.TmdbAccountChanged(model.copy(traktCommentsEnabled = it))) }
            }
        }

        SettingsPrimaryButton(AppStrings.t(lang, "integration.sync_now")) { onSyncNow() }
        SettingsDestructiveLink(AppStrings.t(lang, "integration.disconnect")) { confirmingDisconnect = true }
    }

    if (confirmingDisconnect) {
        androidx.compose.material3.AlertDialog(
            onDismissRequest = { confirmingDisconnect = false },
            title = { Text(AppStrings.t(lang, titleKey)) },
            text = { Text(AppStrings.t(lang, "settings.disconnect_confirm")) },
            confirmButton = {
                TextButton(onClick = {
                    confirmingDisconnect = false
                    onAction(SettingsAction.DisconnectProviderRequested(providerKey))
                }) {
                    Text(AppStrings.t(lang, "integration.disconnect"), color = FluxaColors.errorRed)
                }
            },
            dismissButton = {
                TextButton(onClick = { confirmingDisconnect = false }) {
                    Text(AppStrings.t(lang, "common.cancel"))
                }
            }
        )
    }
}

@Composable
private fun SettingsCredentialLoginForm(
    lang: String?,
    busy: Boolean,
    errorMessage: String?,
    onSubmit: (String, String) -> Unit,
    onCancel: () -> Unit
) {
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    val fieldColors = OutlinedTextFieldDefaults.colors(
        focusedTextColor = Color.White,
        unfocusedTextColor = Color.White,
        focusedBorderColor = Color.White.copy(alpha = 0.4f),
        unfocusedBorderColor = Color.White.copy(alpha = 0.15f),
        focusedLabelColor = Color.White.copy(alpha = 0.7f),
        unfocusedLabelColor = Color.White.copy(alpha = 0.4f),
        cursorColor = Color.White
    )
    SettingsGroupCard {
        Column(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            OutlinedTextField(
                value = email,
                onValueChange = { email = it },
                label = { Text(AppStrings.t(lang, "auth.field.email")) },
                singleLine = true,
                enabled = !busy,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
                colors = fieldColors,
                modifier = Modifier.fillMaxWidth()
            )
            OutlinedTextField(
                value = password,
                onValueChange = { password = it },
                label = { Text(AppStrings.t(lang, "auth.field.password")) },
                singleLine = true,
                enabled = !busy,
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                colors = fieldColors,
                modifier = Modifier.fillMaxWidth()
            )
            if (errorMessage != null) {
                Text(errorMessage, color = FluxaColors.errorRed, fontSize = 12.sp)
            }
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
                TextButton(onClick = onCancel, enabled = !busy, modifier = Modifier.weight(1f)) {
                    Text(AppStrings.t(lang, "common.cancel"))
                }
                Button(
                    onClick = { onSubmit(email.trim(), password) },
                    enabled = !busy && email.isNotBlank() && password.isNotBlank(),
                    colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black),
                    modifier = Modifier.weight(1f)
                ) {
                    if (busy) {
                        CircularProgressIndicator(modifier = Modifier.size(16.dp), color = Color.Black, strokeWidth = 2.dp)
                    } else {
                        Text(AppStrings.t(lang, "auth.nuvio.sign_in"))
                    }
                }
            }
        }
    }
}

@Composable
private fun SettingsTmdbFeaturesContent(model: SettingsAccountUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val allEnabled = model.tmdbCastImagesEnabled && model.tmdbSimilarResultsEnabled && model.tmdbTrailersEnabled &&
        model.tmdbRecommendationsEnabled && model.tmdbCollectionInfoEnabled && model.tmdbEpisodeImagesEnabled &&
        model.tmdbLogosBackdropsEnabled && model.tmdbRatingsEnabled && model.tmdbBasicInfoEnabled &&
        model.tmdbDetailsEnabled && model.tmdbProductionsEnabled && model.tmdbNetworksEnabled

    SettingsSectionHeader(AppStrings.t(lang, "settings.tmdb_api"))
    SettingsInlineSecretField(
        AppStrings.t(lang, "settings.tmdb_api_key"),
        model.tmdbApiKey.orEmpty(),
        placeholder = AppStrings.t(lang, "settings.tmdb_api_key_placeholder")
    ) {
        onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbApiKey = it)))
    }

    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_enable_all"), value = allEnabled) {
            onAction(
                SettingsAction.TmdbAccountChanged(
                    model.copy(
                        tmdbCastImagesEnabled = it, tmdbSimilarResultsEnabled = it, tmdbTrailersEnabled = it,
                        tmdbRecommendationsEnabled = it, tmdbCollectionInfoEnabled = it, tmdbEpisodeImagesEnabled = it,
                        tmdbLogosBackdropsEnabled = it, tmdbRatingsEnabled = it, tmdbBasicInfoEnabled = it,
                        tmdbDetailsEnabled = it, tmdbProductionsEnabled = it, tmdbNetworksEnabled = it
                    )
                )
            )
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.tmdb_group_media_info"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_basic_info"), description = AppStrings.t(lang, "settings.tmdb_basic_info_desc"), value = model.tmdbBasicInfoEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbBasicInfoEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_details"), description = AppStrings.t(lang, "settings.tmdb_details_desc"), value = model.tmdbDetailsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbDetailsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_productions"), description = AppStrings.t(lang, "settings.tmdb_productions_desc"), value = model.tmdbProductionsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbProductionsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_networks"), description = AppStrings.t(lang, "settings.tmdb_networks_desc"), value = model.tmdbNetworksEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbNetworksEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_collection_info"), description = AppStrings.t(lang, "settings.tmdb_collection_info_desc"), value = model.tmdbCollectionInfoEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbCollectionInfoEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_ratings"), description = AppStrings.t(lang, "settings.tmdb_ratings_desc"), value = model.tmdbRatingsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbRatingsEnabled = it))) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.tmdb_group_images"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_cast_images"), description = AppStrings.t(lang, "settings.tmdb_cast_images_desc"), value = model.tmdbCastImagesEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbCastImagesEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_episode_images"), description = AppStrings.t(lang, "settings.tmdb_episode_images_desc"), value = model.tmdbEpisodeImagesEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbEpisodeImagesEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_logos_backdrops"), description = AppStrings.t(lang, "settings.tmdb_logos_backdrops_desc"), value = model.tmdbLogosBackdropsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbLogosBackdropsEnabled = it))) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.tmdb_group_discovery"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_similar_results"), description = AppStrings.t(lang, "settings.tmdb_similar_results_desc"), value = model.tmdbSimilarResultsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbSimilarResultsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_recommendations"), description = AppStrings.t(lang, "settings.tmdb_recommendations_desc"), value = model.tmdbRecommendationsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbRecommendationsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_trailers"), description = AppStrings.t(lang, "settings.tmdb_trailers_desc"), value = model.tmdbTrailersEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbTrailersEnabled = it))) }
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
    SettingsSectionHeader(AppStrings.t(lang, "settings.section_appearance_theme"))
    SettingsGroupCard {
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(vertical = 10.dp)) {
            Text(AppStrings.t(lang, "auto.accent_color"), color = Color.White, modifier = Modifier.weight(1f))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                SETTINGS_COLOR_SWATCHES.forEach { swatch ->
                    var swatchFocused by remember { mutableStateOf(false) }
                    val selected = swatch == model.accentColorArgb
                    Box(
                        modifier = Modifier
                            .size(24.dp)
                            .clip(CircleShape)
                            .onFocusChanged { swatchFocused = it.isFocused }
                            .background(Color(swatch.toInt()))
                            .then(
                                if (selected || swatchFocused) Modifier.border(2.dp, Color.White, CircleShape) else Modifier
                            )
                            .clickable { onAction(SettingsAction.AppearanceChanged(model.copy(accentColorArgb = swatch))) },
                        contentAlignment = Alignment.Center
                    ) {
                        if (selected) {
                            val isLight = Color(swatch.toInt()).luminance() > 0.5f
                            Icon(
                                Icons.Filled.CheckCircle,
                                contentDescription = null,
                                tint = if (isLight) Color.Black else Color.White,
                                modifier = Modifier.size(14.dp)
                            )
                        }
                    }
                }
            }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.amoled"), value = model.amoledMode) { onAction(SettingsAction.AppearanceChanged(model.copy(amoledMode = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.liquid_glass"), description = AppStrings.t(lang, "settings.liquid_glass_desc"), value = model.liquidGlassMode) {
            onAction(SettingsAction.AppearanceChanged(model.copy(liquidGlassMode = it)))
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.section_appearance_layout"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "auto.disable_animations"), value = !model.animationsEnabled) {
            onAction(SettingsAction.AppearanceChanged(model.copy(animationsEnabled = !it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.floating_bottom_bar"), value = model.floatingBottomBar) {
            onAction(SettingsAction.AppearanceChanged(model.copy(floatingBottomBar = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.bottom_bar_labels"), value = model.bottomBarLabels) {
            onAction(SettingsAction.AppearanceChanged(model.copy(bottomBarLabels = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.top_navigation_bar"), description = AppStrings.t(lang, "settings.top_navigation_bar_desc"), value = model.topNavigationBar) {
            onAction(SettingsAction.AppearanceChanged(model.copy(topNavigationBar = it)))
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.section_appearance_screens"))
    SettingsGroupCard {
        SettingsNavRow(
            AppStrings.t(lang, "settings.appearance_home_screen"),
            description = AppStrings.t(lang, "settings.appearance_home_screen_desc")
        ) { onNavigate(SettingsCategory.AppearanceHome) }
        SettingsNavRow(
            AppStrings.t(lang, "settings.appearance_detail_screen"),
            description = AppStrings.t(lang, "settings.appearance_detail_screen_desc")
        ) { onNavigate(SettingsCategory.AppearanceDetail) }
    }
}

@Composable
private fun SettingsAppearanceHomeContent(model: SettingsAppearanceHomeUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
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
    SettingsPosterPreview(model)
    SettingsSectionHeader(AppStrings.t(lang, "settings.layout"))
    SettingsGroupCard {
        SettingsChoiceRow(AppStrings.t(lang, "auto.card_corners"), model.cardCornerPreset, cornerOptions) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(cardCornerPreset = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "auto.interface_density"), model.interfaceDensity, densityOptions) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(interfaceDensity = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "auto.poster_width"), model.posterWidthPreset, posterWidthOptions) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(posterWidthPreset = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.landscape_mode"), value = model.posterLandscapeMode) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(posterLandscapeMode = it))) }
        SettingsToggleRow(AppStrings.t(lang, "auto.hide_titles"), value = model.posterHideTitles) { onAction(SettingsAction.AppearanceHomeChanged(model.copy(posterHideTitles = it))) }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.hero_banner"))
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
    SettingsSectionHeader(AppStrings.t(lang, "auto.continue_watching"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "auto.continue_watching"), value = model.continueWatchingEnabled) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingEnabled = it)))
        }
        SettingsToggleRow(
            AppStrings.t(lang, "settings.upcoming_row"),
            description = AppStrings.t(lang, "settings.upcoming_row_desc"),
            value = model.upcomingRowEnabled
        ) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(upcomingRowEnabled = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.continue_watching_horizontal"), value = model.continueWatchingHorizontal) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingHorizontal = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.continue_watching_hide_titles"), value = model.continueWatchingHideTitles) {
            onAction(SettingsAction.AppearanceHomeChanged(model.copy(continueWatchingHideTitles = it)))
        }
    }
}

private fun posterCornerRadius(preset: String): androidx.compose.ui.unit.Dp = when (preset) {
    "sharp" -> 0.dp
    "classic" -> 4.dp
    "soft" -> 8.dp
    "rounded" -> 14.dp
    "pill" -> 22.dp
    else -> 8.dp
}

private fun posterWidth(preset: String): androidx.compose.ui.unit.Dp = when (preset) {
    "xsmall" -> 64.dp
    "small" -> 78.dp
    "medium" -> 94.dp
    "large" -> 112.dp
    "xlarge" -> 132.dp
    else -> 94.dp
}

private fun posterSpacing(preset: String): androidx.compose.ui.unit.Dp = when (preset) {
    "small" -> 6.dp
    "medium" -> 12.dp
    "large" -> 20.dp
    else -> 12.dp
}

@Composable
private fun SettingsPosterPreview(model: SettingsAppearanceHomeUiModel) {
    val shape = RoundedCornerShape(posterCornerRadius(model.cardCornerPreset))
    val width = posterWidth(model.posterWidthPreset)
    val aspectRatio = if (model.posterLandscapeMode) 16f / 9f else 2f / 3f
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 4.dp, bottom = 4.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .horizontalScroll(rememberScrollState())
            .padding(16.dp),
        horizontalArrangement = Arrangement.spacedBy(posterSpacing(model.interfaceDensity))
    ) {
        repeat(3) {
            Column(modifier = Modifier.width(width)) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .aspectRatio(aspectRatio)
                        .clip(shape)
                        .background(Color.White.copy(alpha = 0.1f))
                        .border(1.dp, Color.White.copy(alpha = 0.08f), shape),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        imageVector = Icons.Filled.Movie,
                        contentDescription = null,
                        tint = Color.White.copy(alpha = 0.25f),
                        modifier = Modifier.size(width / 3)
                    )
                }
                if (!model.posterHideTitles) {
                    Box(
                        modifier = Modifier
                            .padding(top = 6.dp)
                            .fillMaxWidth(0.75f)
                            .height(8.dp)
                            .clip(RoundedCornerShape(4.dp))
                            .background(Color.White.copy(alpha = 0.18f))
                    )
                }
            }
        }
    }
}

@Composable
private fun SettingsAppearanceDetailContent(model: SettingsAppearanceDetailUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val seasonSelectorOptions = listOf(
        SettingsChoiceOption("dropdown", AppStrings.t(lang, "settings.season_selector_dropdown")),
        SettingsChoiceOption("tabs", AppStrings.t(lang, "settings.season_selector_tabs")),
        SettingsChoiceOption("posters", AppStrings.t(lang, "settings.season_selector_posters"))
    )
    val episodeLayoutOptions = listOf(
        SettingsChoiceOption("list", AppStrings.t(lang, "settings.episode_layout_list")),
        SettingsChoiceOption("horizontal", AppStrings.t(lang, "settings.episode_layout_horizontal"))
    )
    SettingsEpisodeLayoutPreview(model)
    SettingsSectionHeader(AppStrings.t(lang, "settings.episodes"))
    SettingsGroupCard {
        SettingsToggleRow(
            AppStrings.t(lang, "settings.blur_unwatched_episodes"),
            value = model.blurUnwatchedEpisodes
        ) { onAction(SettingsAction.AppearanceDetailChanged(model.copy(blurUnwatchedEpisodes = it))) }
        SettingsChoiceRow(AppStrings.t(lang, "settings.season_selector"), model.detailSeasonSelectorMode, seasonSelectorOptions) {
            onAction(SettingsAction.AppearanceDetailChanged(model.copy(detailSeasonSelectorMode = it)))
        }
        SettingsChoiceRow(AppStrings.t(lang, "settings.episode_cards_layout"), model.episodeCardsLayout, episodeLayoutOptions) {
            onAction(SettingsAction.AppearanceDetailChanged(model.copy(episodeCardsLayout = it)))
        }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.hero_banner"))
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
private fun SettingsEpisodeLayoutPreview(model: SettingsAppearanceDetailUiModel) {
    val thumbAlpha = if (model.blurUnwatchedEpisodes) 0.35f else 0.85f
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 4.dp, bottom = 4.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(Color.White.copy(alpha = 0.04f))
            .padding(16.dp)
    ) {
        if (model.episodeCardsLayout == "horizontal") {
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                repeat(3) {
                    Column(modifier = Modifier.width(96.dp)) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .aspectRatio(16f / 9f)
                                .clip(RoundedCornerShape(8.dp))
                                .background(Color.White.copy(alpha = thumbAlpha * 0.14f)),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(Icons.Rounded.PlayArrow, contentDescription = null, tint = Color.White.copy(alpha = thumbAlpha), modifier = Modifier.size(20.dp))
                        }
                        Box(
                            modifier = Modifier
                                .padding(top = 6.dp)
                                .fillMaxWidth(0.7f)
                                .height(7.dp)
                                .clip(RoundedCornerShape(4.dp))
                                .background(Color.White.copy(alpha = 0.18f))
                        )
                    }
                }
            }
        } else {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
                repeat(2) {
                    Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
                        Box(
                            modifier = Modifier
                                .width(84.dp)
                                .aspectRatio(16f / 9f)
                                .clip(RoundedCornerShape(8.dp))
                                .background(Color.White.copy(alpha = thumbAlpha * 0.14f)),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(Icons.Rounded.PlayArrow, contentDescription = null, tint = Color.White.copy(alpha = thumbAlpha), modifier = Modifier.size(18.dp))
                        }
                        Spacer(Modifier.width(12.dp))
                        Column(modifier = Modifier.weight(1f)) {
                            Box(
                                modifier = Modifier
                                    .fillMaxWidth(0.6f)
                                    .height(8.dp)
                                    .clip(RoundedCornerShape(4.dp))
                                    .background(Color.White.copy(alpha = 0.2f))
                            )
                            Box(
                                modifier = Modifier
                                    .padding(top = 6.dp)
                                    .fillMaxWidth(0.9f)
                                    .height(6.dp)
                                    .clip(RoundedCornerShape(3.dp))
                                    .background(Color.White.copy(alpha = 0.1f))
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun SettingsPlaybackCoreContent(
    model: SettingsPlaybackUiModel,
    subtitles: SettingsSubtitlesUiModel,
    lang: String?,
    onAction: (SettingsAction) -> Unit,
    onNavigate: (SettingsCategory) -> Unit
) {
    val playerOptions = listOf(SettingsChoiceOption("internal", "ExoPlayer"), SettingsChoiceOption("mpv", "MPV"))
    val playbackSpeedOptions = listOf("0.75", "1.0", "1.25", "1.5").map { SettingsChoiceOption(it, "${it}x") }
    val seekOptions = listOf("10", "15", "30").map { SettingsChoiceOption(it, "${it}s") }
    val holdSpeedOptions = listOf("1.25", "1.5", "1.75", "2.0", "2.5", "3.0").map { SettingsChoiceOption(it, "${it}x") }

    SettingsSectionHeader(AppStrings.t(lang, "settings.section_playback_general"))
    SettingsGroupCard {
        SettingsNavRow(
            AppStrings.t(lang, "auto.subtitles"),
            value = languageOptionLabel(subtitles.preferredSubtitleLanguage, lang)
        ) { onNavigate(SettingsCategory.Subtitles) }
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

    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "settings.stream_settings")) { onNavigate(SettingsCategory.PlaybackStream) }
    }
}

@Composable
private fun SettingsPlaybackStreamContent(
    model: SettingsPlaybackUiModel,
    lang: String?,
    onAction: (SettingsAction) -> Unit,
    onNavigate: (SettingsCategory) -> Unit
) {
    val streamSourceModeOptions = listOf(
        SettingsChoiceOption("manual", AppStrings.t(lang, "settings.stream_source_manual")),
        SettingsChoiceOption("first", AppStrings.t(lang, "settings.stream_source_first")),
        SettingsChoiceOption("regex", AppStrings.t(lang, "settings.stream_source_regex"))
    )
    val autoplayCountdownOptions = listOf("5", "7", "10", "15").map { SettingsChoiceOption(it, "${it}s") }

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
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.progress_thresholds"))
    SettingsGroupCard {
        SettingsPercentSliderRow(AppStrings.t(lang, "settings.next_episode_threshold"), model.nextEpisodeThresholdPercent) {
            onAction(SettingsAction.PlaybackChanged(model.copy(nextEpisodeThresholdPercent = it)))
        }
        SettingsPercentSliderRow(AppStrings.t(lang, "settings.watched_threshold"), model.watchedThresholdPercent) {
            onAction(SettingsAction.PlaybackChanged(model.copy(watchedThresholdPercent = it)))
        }
    }

    SettingsGroupCard {
        SettingsNavRow(AppStrings.t(lang, "settings.skip_segments")) { onNavigate(SettingsCategory.PlaybackSkip) }
    }
}

@Composable
private fun SettingsPlaybackSkipContent(model: SettingsPlaybackUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsSectionHeader(AppStrings.t(lang, "settings.skip_segments"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.use_skip_segments"), value = model.useSkipSegments) { onAction(SettingsAction.PlaybackChanged(model.copy(useSkipSegments = it))) }
        if (model.useSkipSegments) {
            SettingsTextFieldRow(AppStrings.t(lang, "settings.introdb_api_key"), model.introDbApiKey) { onAction(SettingsAction.PlaybackChanged(model.copy(introDbApiKey = it))) }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.use_chapter_skip"), value = model.useChapterSkip) { onAction(SettingsAction.PlaybackChanged(model.copy(useChapterSkip = it))) }
        if (model.useSkipSegments) {
            SettingsToggleRow(AppStrings.t(lang, "settings.auto_skip"), value = model.autoSkipIntro) { onAction(SettingsAction.PlaybackChanged(model.copy(autoSkipIntro = it))) }
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.content_warnings_enabled"), value = model.contentWarningsEnabled) { onAction(SettingsAction.PlaybackChanged(model.copy(contentWarningsEnabled = it))) }
    }

    var confirmingReset by remember { mutableStateOf(false) }
    SettingsGroupCard {
        SettingsActionRow(AppStrings.t(lang, "settings.reset_to_defaults"), destructive = true) {
            confirmingReset = true
        }
    }
    if (confirmingReset) {
        SettingsResetConfirmDialog(lang, onDismiss = { confirmingReset = false }) {
            confirmingReset = false
            onAction(SettingsAction.PlaybackChanged(SettingsPlaybackUiModel()))
        }
    }
}

private fun formatFloat(value: Float): String {
    val rounded = (value * 100).toInt() / 100f
    return if (rounded == rounded.toInt().toFloat()) rounded.toInt().toString() + ".0" else rounded.toString()
}

@Composable
private fun SettingsResetConfirmDialog(lang: String?, onDismiss: () -> Unit, onConfirm: () -> Unit) {
    androidx.compose.material3.AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(AppStrings.t(lang, "settings.reset_to_defaults")) },
        text = { Text(AppStrings.t(lang, "settings.reset_to_defaults_confirm")) },
        confirmButton = {
            TextButton(onClick = onConfirm) {
                Text(AppStrings.t(lang, "settings.reset_to_defaults"), color = FluxaColors.errorRed)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(AppStrings.t(lang, "common.cancel"))
            }
        }
    )
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
        SettingsChoiceOption("manual", AppStrings.t(lang, "settings.stream_source_manual"), AppStrings.t(lang, "settings.stream_source_manual_desc")),
        SettingsChoiceOption("first", AppStrings.t(lang, "settings.stream_source_first"), AppStrings.t(lang, "settings.stream_source_first_desc")),
        SettingsChoiceOption("regex", AppStrings.t(lang, "settings.stream_source_regex"), AppStrings.t(lang, "settings.stream_source_regex_desc"))
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

    SettingsSectionHeader(AppStrings.t(lang, "settings.source_modes"))
    SettingsInlineChoiceCards(
        options = streamSourceModeOptions,
        selected = model.downloadSourceSelectionMode
    ) { onAction(SettingsAction.DownloadsChanged(model.copy(downloadSourceSelectionMode = it))) }
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
