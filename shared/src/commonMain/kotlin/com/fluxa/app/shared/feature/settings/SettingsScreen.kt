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
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import kotlinx.coroutines.delay
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.profile.ProfileAction
import com.fluxa.app.shared.feature.profile.ProfileUiState
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
    profileState: ProfileUiState? = null,
    onProfileAction: (ProfileAction) -> Unit = {},
    onBackRequested: () -> Unit,
    backStack: List<SettingsCategory> = emptyList(),
    onPushCategory: (SettingsCategory) -> Unit = {},
    onPopCategory: () -> Unit = {},
    onSelectCategory: (SettingsCategory) -> Unit = {},
    deviceType: com.fluxa.app.ui.catalog.DeviceType = com.fluxa.app.ui.catalog.DeviceType.Mobile,
    brandIcons: SettingsBrandIcons = SettingsBrandIcons(),
    onImportThemeRequested: ((String?) -> Unit) -> Unit = { onResult -> onResult(null) },
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

    if (deviceType == com.fluxa.app.ui.catalog.DeviceType.TV || deviceType == com.fluxa.app.ui.catalog.DeviceType.Desktop) {
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
                    SettingsTvRailRow(
                        label = AppStrings.t(lang, "auto.add_ons"),
                        selected = false
                    ) { onAction(SettingsAction.ManageAddonsRequested) }
                    SettingsTvRailRow(
                        label = AppStrings.t(lang, "settings.plugins.manage"),
                        selected = false
                    ) { onAction(SettingsAction.ManagePluginsRequested) }
                    SettingsTvRailRow(
                        label = AppStrings.t(lang, "settings.stream_badges.manage"),
                        selected = false
                    ) { onAction(SettingsAction.ManageStreamBadgesRequested) }
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
                        SettingsCategoryContent(
                            animatedCategory, state, lang, brandIcons, onAction, onPushCategory,
                            onSwitchProfilesRequested, profileState, onProfileAction, navigateAndHighlight,
                            onImportThemeRequested
                        )
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
                    SettingsCategoryContent(
                        animatedCategory, state, lang, brandIcons, onAction, onPushCategory,
                        onSwitchProfilesRequested, profileState, onProfileAction, navigateAndHighlight,
                        onImportThemeRequested
                    )
                }
                Spacer(Modifier.height(120.dp))
            }
            }
        }
    }
    }
}

@Composable
internal fun SettingsCategoryContent(
    category: SettingsCategory,
    state: SettingsUiState,
    lang: String?,
    brandIcons: SettingsBrandIcons,
    onAction: (SettingsAction) -> Unit,
    onNavigate: (SettingsCategory) -> Unit,
    onSwitchProfiles: () -> Unit,
    profileState: ProfileUiState?,
    onProfileAction: (ProfileAction) -> Unit,
    onNavigateSearchResult: (SettingsSearchEntry) -> Unit,
    onImportThemeRequested: ((String?) -> Unit) -> Unit
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
            profileState = profileState,
            onProfileAction = onProfileAction,
            onSwitchProfiles = onSwitchProfiles,
            continueWatchingSource = state.appearanceHome.continueWatchingSource,
            onContinueWatchingSourceChanged = { onAction(SettingsAction.AppearanceHomeChanged(state.appearanceHome.copy(continueWatchingSource = it))) }
        )
        SettingsCategory.TmdbFeatures -> SettingsTmdbFeaturesContent(state.account, lang, onAction)
        SettingsCategory.MdblistApi -> SettingsMdblistApiContent(state.account, lang, onAction)
        SettingsCategory.Notifications -> SettingsNotificationsContent(state.notifications, lang, onAction)
        SettingsCategory.General -> SettingsGeneralContent(state.general, lang, onAction)
        SettingsCategory.Appearance -> SettingsAppearanceContent(state.appearance, lang, onAction, onNavigate = onNavigate, onImportThemeRequested = onImportThemeRequested)
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
internal fun SettingsHubContent(
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
        SettingsNavRow(AppStrings.t(lang, "settings.stream_badges.manage")) { onAction(SettingsAction.ManageStreamBadgesRequested) }
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
        SettingsToggleRow(
            label = AppStrings.t(lang, "settings.remember_last_profile"),
            description = AppStrings.t(lang, "settings.remember_last_profile_desc"),
            value = state.system.rememberLastProfile,
            onValueChanged = { onAction(SettingsAction.SystemChanged(state.system.copy(rememberLastProfile = it))) }
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
