@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

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
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.runtime.staticCompositionLocalOf
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
import androidx.compose.ui.platform.LocalContext
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
import kotlinx.coroutines.delay
import java.util.Locale

@Composable
internal fun MobileCategoryDetail(
    category: String,
    profile: UserProfile,
    lang: String,
    onBack: () -> Unit,
    onLogout: () -> Unit,
    onConnectStremio: () -> Unit,
    onConnectTrakt: () -> Unit,
    onConnectMal: () -> Unit,
    onConnectSimkl: () -> Unit,
    onManageAddons: () -> Unit,
    onWatchlistClick: () -> Unit,
    onOpenDownload: (OfflineDownloadItem) -> Unit = {},
    onReboot: () -> Unit,
    onNavigate: (String) -> Unit = {},
    viewModel: HomeViewModel,
    onUpdateProfile: (UserProfile) -> Unit
) {
    var choiceDialog by remember { mutableStateOf<MobileChoiceDialogState?>(null) }
    var tmdbExpanded by remember { mutableStateOf(false) }
    val context = LocalContext.current
    val offlineDownloadManager = remember(context) { OfflineDownloadManager.getInstance(context) }
    val offlineDownloads by offlineDownloadManager.items.collectAsStateWithLifecycle()
    val categories by viewModel.categories.collectAsStateWithLifecycle()
    val userAddons by viewModel.userAddons.collectAsStateWithLifecycle()
    val loadedCs3ApiNames by viewModel.loadedCs3ApiNames.collectAsStateWithLifecycle()
    LaunchedEffect(category, profile.safeLocalAddons) {
        if (category == "content") {
            viewModel.refreshInstalledAddons(forceRefresh = true)
        }
    }
    var previewMeta by remember { mutableStateOf<Meta?>(null) }
    LaunchedEffect(categories) {
        val candidate = categories
            .flatMap { it.items }
            .firstOrNull { !it.poster.isNullOrBlank() || !it.background.isNullOrBlank() }
        if (candidate != null) previewMeta = candidate
    }
    val palette = mobileSettingsPalette(
        amoledMode = profile.safeAmoledMode,
        accentColorArgb = profile.safeAccentColorArgb,
        cardCornerPreset = profile.safeCardCornerPreset,
        interfaceDensity = profile.safeInterfaceDensity
    )
    
    val title = when(category) {
        "account" -> AppStrings.t(lang, "auto.account")
        "general" -> AppStrings.t(lang, "auto.general")
        "trakt" -> "Trakt"
        "appearance" -> AppStrings.t(lang, "auto.appearance")
        "appearance_home" -> AppStrings.t(lang, "settings.appearance_home_screen")
        "appearance_detail" -> AppStrings.t(lang, "settings.appearance_detail_screen")
        "content" -> AppStrings.t(lang, "auto.catalogs")
        "downloads" -> AppStrings.t(lang, "auto.downloads")
        "playback" -> AppStrings.t(lang, "auto.playback")
        "subtitles" -> AppStrings.t(lang, "auto.subtitles")
        "advanced_settings" -> AppStrings.t(lang, "settings.advanced_settings")
        "integrations" -> AppStrings.t(lang, "settings.check_for_updates")
        "developer" -> AppStrings.t(lang, "settings.developer")
        else -> AppStrings.t(lang, "nav.settings")
    }

    CompositionLocalProvider(LocalMobileSettingsPalette provides palette) {
    val colors = LocalMobileSettingsPalette.current
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(colors.background)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 30.dp, start = 12.dp, end = 12.dp, bottom = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Icon(
                FluxaIcons.ArrowBack,
                null,
                tint = colors.text.copy(alpha = 0.86f),
                modifier = Modifier
                    .size(26.dp)
                    .clip(CircleShape)
                    .clickable { onBack() }
                    .padding(3.dp)
            )
            Text(
                text = title,
                color = colors.text,
                fontSize = 30.sp,
                fontWeight = FontWeight.Black
            )
        }

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 8.dp, bottom = 150.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            when(category) {
                "general" -> {
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "auto.app")) {
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "auto.language"),
                                value = languageDisplayName(profile.safeLanguage, lang),
                                subtitle = AppStrings.t(lang, "settings.language_desc"),
                                onClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "auto.select_language"),
                                        options = listOf(
                                            ChoiceOption("tr", languageDisplayName("tr", lang)),
                                            ChoiceOption("en", languageDisplayName("en", lang))
                                        ),
                                        selected = profile.safeLanguage,
                                        onSelect = { onUpdateProfile(profile.copy(language = it)) }
                                    )
                                }
                            )
                            MobileChoiceRow(
                                AppStrings.t(lang, "auto.start_page"),
                                mobileStartPageLabel(profile.safeStartPage, lang),
                                subtitle = AppStrings.t(lang, "settings.start_page_desc")
                            ) {
                                choiceDialog = MobileChoiceDialogState(
                                    title = AppStrings.t(lang, "auto.start_page"),
                                    options = listOf(
                                        ChoiceOption("home", AppStrings.t(lang, "nav.home")),
                                        ChoiceOption("discover", AppStrings.t(lang, "nav.discover")),
                                        ChoiceOption("library", AppStrings.t(lang, "nav.library"))
                                    ),
                                    selected = profile.safeStartPage,
                                    onSelect = { onUpdateProfile(profile.copy(startPage = it)) }
                                )
                            }
                            MobileToggleRow(
                                title = AppStrings.t(lang, "auto.background_playback"),
                                subtitle = AppStrings.t(lang, "settings.background_playback_desc"),
                                checked = profile.safeBackgroundPlayback,
                                onToggle = { onUpdateProfile(profile.copy(backgroundPlayback = !profile.safeBackgroundPlayback)) }
                            )
                        }
                    }
                }
                "account" -> {
                    item {
                        MobileSettingsGroup(null) {
                            MobileActionRow(
                                title = AppStrings.t(lang, "settings.switch_profiles"),
                                value = AppStrings.t(lang, "settings.switch_profiles_desc"),
                                icon = FluxaIcons.AccountCircle,
                                onClick = onLogout,
                                prominent = true
                            )
                        }
                    }
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "settings.sync_with")) {
                            MobileActionRow(
                                title = AppStrings.t(lang, "brand.trakt"),
                                value = if (!profile.traktAccessToken.isNullOrBlank()) (AppStrings.t(lang, "auto.sync_active")) else (AppStrings.t(lang, "auto.connect_trakt_tv_account")),
                                iconRes = R.drawable.ic_trakt,
                                onClick = onConnectTrakt,
                                prominent = true
                            )
                            MobileActionRow(
                                title = AppStrings.t(lang, "brand.myanimelist"),
                                iconRes = R.drawable.ic_myanimelist,
                                value = if (!profile.malAccessToken.isNullOrBlank()) AppStrings.t(lang, "auto.sync_active") else AppStrings.t(lang, "auto.connect_mal_account"),
                                onClick = onConnectMal,
                                prominent = true
                            )
                            MobileActionRow(
                                title = AppStrings.t(lang, "brand.simkl"),
                                iconRes = R.drawable.ic_simkl,
                                value = if (!profile.simklAccessToken.isNullOrBlank()) AppStrings.t(lang, "auto.sync_active") else AppStrings.t(lang, "auto.connect_simkl_account"),
                                onClick = onConnectSimkl,
                                prominent = true
                            )
                            if (!profile.traktAccessToken.isNullOrBlank() || !profile.malAccessToken.isNullOrBlank() || !profile.simklAccessToken.isNullOrBlank()) {
                            MobileActionRow(
                                title = AppStrings.t(lang, "auto.disconnect"),
                                icon = FluxaIcons.Logout,
                                destructive = true,
                                    prominent = true,
                                    onClick = {
                                        onUpdateProfile(
                                            profile.copy(
                                                traktAccessToken = null,
                                                traktRefreshToken = null,
                                                traktLastSyncAt = null,
                                                traktLastSyncedItems = null,
                                                traktLastContinueWatchingCount = null,
                                                traktLastWatchlistCount = null,
                                                malAccessToken = null,
                                                malRefreshToken = null,
                                                simklAccessToken = null
                                            )
                                        )
                                    }
                            )
                            }
                        }
                    }
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "settings.apis")) {
                            MobileActionRow(
                                title = AppStrings.t(lang, "settings.tmdb_api"),
                                value = if (profile.safeTmdbApiKey.isNotBlank()) AppStrings.t(lang, "settings.tmdb_api_configured") else AppStrings.t(lang, "settings.tmdb_api_not_configured"),
                                iconRes = R.drawable.ic_tmdb,
                                onClick = { tmdbExpanded = !tmdbExpanded },
                                prominent = true
                            )
                            if (tmdbExpanded) {
                                MobileTmdbApiSettings(
                                    profile = profile,
                                    lang = lang,
                                    onUpdateProfile = onUpdateProfile
                                )
                            }
                        }
                    }
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "settings.notifications")) {
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.alert_new_episodes"),
                                subtitle = AppStrings.t(lang, "settings.alert_new_episodes_desc"),
                                checked = profile.safeAlertNewEpisodes,
                                onToggle = { onUpdateProfile(profile.sanitizedUpdate(alertNewEpisodes = !profile.safeAlertNewEpisodes)) }
                            )
                        }
                    }
                }
                "appearance" -> {
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "auto.accent_color")) {
                            MobileAccentDots(profile = profile, onUpdateProfile = onUpdateProfile)
                        }
                    }
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "auto.interface_3c5ec842")) {
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.amoled"),
                                subtitle = AppStrings.t(lang, "auto.makes_the_background_and_bottom_bar_pure_bla"),
                                checked = profile.safeAmoledMode,
                                onToggle = { onUpdateProfile(profile.sanitizedUpdate(appTheme = "dark", amoledMode = !profile.safeAmoledMode)) }
                            )
                            MobileToggleRow(
                                AppStrings.t(lang, "auto.disable_animations"),
                                subtitle = AppStrings.t(lang, "auto.turns_off_app_wide_transitions_and_card_anim"),
                                checked = !profile.safeAnimationsEnabled,
                                onToggle = { onUpdateProfile(profile.copy(animationsEnabled = !profile.safeAnimationsEnabled)) }
                            )
                        }
                    }
                    item {
                        MobileSettingsCard {
                            MobileSettingsRow(
                                title = AppStrings.t(lang, "settings.appearance_home_screen"),
                                subtitle = AppStrings.t(lang, "settings.appearance_home_screen_desc"),
                                icon = FluxaIcons.Home,
                                onClick = { onNavigate("appearance_home") }
                            )
                            MobileSettingsRow(
                                title = AppStrings.t(lang, "settings.appearance_detail_screen"),
                                subtitle = AppStrings.t(lang, "settings.appearance_detail_screen_desc"),
                                icon = FluxaIcons.Movie,
                                onClick = { onNavigate("appearance_detail") }
                            )
                        }
                    }
                }
                "appearance_home" -> {
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "auto.posters")) {
                            MobilePosterPreview(profile = profile, lang = lang, meta = previewMeta)
                            MobileChoiceRow(
                                AppStrings.t(lang, "auto.card_corners"),
                                mobilePosterCornerLabel(profile.safeCardCornerPreset, lang)
                            ) {
                                choiceDialog = MobileChoiceDialogState(
                                    title = AppStrings.t(lang, "auto.card_corners"),
                                    options = mobilePosterCornerOptions(lang),
                                    selected = profile.safeCardCornerPreset,
                                    onSelect = { onUpdateProfile(profile.copy(cardCornerPreset = it)) }
                                )
                            }
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "auto.interface_density"),
                                value = mobilePresetLabel(profile.safeInterfaceDensity, lang)
                            ) {
                                choiceDialog = MobileChoiceDialogState(
                                    title = AppStrings.t(lang, "auto.interface_density"),
                                    options = mobilePresetOptions(lang),
                                    selected = profile.safeInterfaceDensity,
                                    onSelect = { onUpdateProfile(profile.copy(interfaceDensity = it)) }
                                )
                            }
                            MobileChoiceRow(
                                AppStrings.t(lang, "auto.poster_width"),
                                mobilePosterWidthLabel(profile.safePosterWidthPreset, lang)
                            ) {
                                choiceDialog = MobileChoiceDialogState(
                                    title = AppStrings.t(lang, "auto.poster_width"),
                                    options = mobilePosterWidthOptions(lang),
                                    selected = profile.safePosterWidthPreset,
                                    onSelect = { onUpdateProfile(profile.copy(posterWidthPreset = it)) }
                                )
                            }
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.landscape_mode"),
                                checked = profile.safePosterLandscapeMode,
                                onToggle = { onUpdateProfile(profile.copy(posterLandscapeMode = !profile.safePosterLandscapeMode)) }
                            )
                            MobileToggleRow(
                                title = AppStrings.t(lang, "auto.hide_titles"),
                                checked = profile.safePosterHideTitles,
                                onToggle = { onUpdateProfile(profile.copy(posterHideTitles = !profile.safePosterHideTitles)) }
                            )
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.season_posters_on_hero"),
                                subtitle = AppStrings.t(lang, "settings.home_season_posters_on_hero_desc"),
                                checked = profile.safeHomeSeasonPostersOnHero,
                                onToggle = { onUpdateProfile(profile.copy(homeSeasonPostersOnHero = !profile.safeHomeSeasonPostersOnHero)) }
                            )
                        }
                    }
                    item {
                        MobileSettingsGroup(settingsContinueWatchingTitle(lang)) {
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.continue_watching_horizontal"),
                                checked = profile.safeContinueWatchingLayout != "vertical",
                                onToggle = {
                                    val layout = if (profile.safeContinueWatchingLayout == "vertical") "horizontal" else "vertical"
                                    onUpdateProfile(profile.copy(continueWatchingLayout = layout, continueWatchingArtwork = "background"))
                                }
                            )
                            MobileToggleRow(
                                title = AppStrings.t(lang, "auto.continue_watching"),
                                checked = profile.safeContinueWatchingEnabled,
                                onToggle = { onUpdateProfile(profile.copy(continueWatchingEnabled = !profile.safeContinueWatchingEnabled)) }
                            )
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.continue_watching_hide_titles"),
                                checked = profile.safeContinueWatchingHideTitles,
                                onToggle = { onUpdateProfile(profile.copy(continueWatchingHideTitles = !profile.safeContinueWatchingHideTitles)) }
                            )
                        }
                    }
                }
                "appearance_detail" -> {
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "auto.appearance")) {
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.trailer_on_hero"),
                                subtitle = AppStrings.t(lang, "settings.trailer_on_hero_desc"),
                                checked = profile.safeTrailerOnHero,
                                onToggle = { onUpdateProfile(profile.copy(trailerOnHero = !profile.safeTrailerOnHero)) }
                            )
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.blur_unwatched_episodes"),
                                subtitle = AppStrings.t(lang, "settings.blur_unwatched_episodes_desc"),
                                checked = profile.safeBlurUnwatchedEpisodes,
                                onToggle = { onUpdateProfile(profile.copy(blurUnwatchedEpisodes = !profile.safeBlurUnwatchedEpisodes)) }
                            )
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "settings.season_selector"),
                                value = mobileSeasonSelectorLabel(profile.safeDetailSeasonSelectorMode, lang),
                                subtitle = AppStrings.t(lang, "settings.season_selector_desc")
                            ) {
                                choiceDialog = MobileChoiceDialogState(
                                    title = AppStrings.t(lang, "settings.season_selector"),
                                    options = mobileSeasonSelectorOptions(lang),
                                    selected = profile.safeDetailSeasonSelectorMode,
                                    onSelect = { onUpdateProfile(profile.copy(detailSeasonSelectorMode = it)) }
                                )
                            }
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.season_posters_on_hero"),
                                subtitle = AppStrings.t(lang, "settings.detail_season_posters_on_hero_desc"),
                                checked = profile.safeDetailSeasonPostersOnHero,
                                onToggle = { onUpdateProfile(profile.copy(detailSeasonPostersOnHero = !profile.safeDetailSeasonPostersOnHero)) }
                            )
                        }
                    }
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "settings.episode_cards_layout")) {
                            MobileEpisodeLayoutPreview(
                                profile = profile,
                                lang = lang,
                                onSelect = { onUpdateProfile(profile.copy(episodeCardsLayout = it)) }
                            )
                        }
                    }
                }
                "content" -> {
                    val metadataFeedOptions = buildMetadataFeedOptions(userAddons, lang) +
                        buildCs3MetadataFeedOptions(loadedCs3ApiNames)
                    val heroFeedOptions = orderedMetadataFeeds(metadataFeedOptions, profile.heroFeedOrder)
                    val homeFeedOptions = orderedMetadataFeeds(metadataFeedOptions, profile.homeFeedOrder)
                    val metadataFeedKeys = metadataFeedOptions.map { it.key }
                    val heroSelectedKeys = effectiveMetadataFeedSelection(profile.heroFeedToggles, metadataFeedKeys) ?: metadataFeedKeys.take(2)
                    val homeSelectedKeys = effectiveMetadataFeedSelection(profile.homeFeedToggles, metadataFeedKeys)
                    val homeVisibleFeedOptions = homeFeedOptions.filter { isMetadataFeedEnabled(homeSelectedKeys, it.key) }
                    val topTenSelectedKeys = profile.safeTopTenFeedToggles
                    val heroProviderGroups = heroFeedOptions.groupBy { metadataFeedDisplay(it.label).provider }
                    item {
                        var heroFeedsExpanded by remember { mutableStateOf(false) }
                        MobileSettingsGroup(AppStrings.t(lang, "settings.hero_catalogs")) {
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.show_hero_section"),
                                subtitle = AppStrings.t(lang, "settings.show_hero_section_desc"),
                                checked = profile.safeShowHeroSection,
                                onToggle = { onUpdateProfile(profile.sanitizedUpdate(showHeroSection = !profile.safeShowHeroSection)) }
                            )
                            MobileExpandableHeaderRow(
                                title = AppStrings.t(lang, "settings.hero_catalogs"),
                                count = heroFeedOptions.size,
                                subtitle = if (settingsIsEnglish(lang)) {
                                    "Choose which catalog feeds can supply the rotating hero artwork."
                                } else {
                                    "Ana ekrandaki hareketli hero alann besleyen katalog kaynaklarn seçer."
                                },
                                expanded = heroFeedsExpanded,
                                onClick = { heroFeedsExpanded = !heroFeedsExpanded }
                            )
                            AnimatedVisibility(
                                visible = heroFeedsExpanded,
                                enter = expandVertically(animationSpec = tween(220)) + fadeIn(animationSpec = tween(180)),
                                exit = shrinkVertically(animationSpec = tween(180)) + fadeOut(animationSpec = tween(140))
                            ) {
                                Column {
                                heroProviderGroups.forEach { (provider, options) ->
                                    val groupKeys = options.map { it.key }
                                    val groupEnabled = groupKeys.all { heroSelectedKeys.contains(it) }
                                    MobileProviderToggleRow(
                                        title = provider,
                                        allLabel = AppStrings.t(lang, "auto.all"),
                                        checked = groupEnabled,
                                        onToggle = {
                                            val updatedKeys = setMetadataFeedGroupEnabled(heroSelectedKeys, metadataFeedKeys, groupKeys, !groupEnabled)
                                                .filter { it in metadataFeedKeys }
                                                .take(2)
                                            onUpdateProfile(profile.sanitizedUpdate(heroFeedToggles = updatedKeys))
                                        }
                                    )
                                    options.forEach { option ->
                                        val index = heroFeedOptions.indexOf(option)
                                        MobileOrderedToggleRow(
                                            title = option.label,
                                            checked = heroSelectedKeys.contains(option.key),
                                            onToggle = {
                                                onUpdateProfile(profile.sanitizedUpdate(heroFeedToggles = toggleMetadataFeed(heroSelectedKeys, metadataFeedKeys, option.key, maxEnabled = 2)))
                                            },
                                            canMoveUp = index > 0,
                                            canMoveDown = index < heroFeedOptions.lastIndex,
                                            onMoveUp = { onUpdateProfile(profile.sanitizedUpdate(heroFeedOrder = moveMetadataFeedOrder(metadataFeedOptions, profile.heroFeedOrder, option.key, -1))) },
                                            onMoveDown = { onUpdateProfile(profile.sanitizedUpdate(heroFeedOrder = moveMetadataFeedOrder(metadataFeedOptions, profile.heroFeedOrder, option.key, 1))) }
                                        )
                                    }
                                }
                                }
                            }
                        }
                    }
                    item {
                        val hasCs3Options = loadedCs3ApiNames.isNotEmpty()
                        var homeFeedsExpanded by remember { mutableStateOf(false) }
                        var topTenFeedsExpanded by remember { mutableStateOf(false) }
                        MobileSettingsGroup(AppStrings.t(lang, "settings.home_screen")) {
                            MobileExpandableHeaderRow(
                                title = AppStrings.t(lang, "settings.home_catalogs"),
                                count = homeFeedOptions.size,
                                subtitle = AppStrings.t(lang, "settings.home_catalogs_desc"),
                                expanded = homeFeedsExpanded,
                                onClick = { homeFeedsExpanded = !homeFeedsExpanded }
                            )
                            AnimatedVisibility(
                                visible = homeFeedsExpanded,
                                enter = expandVertically(animationSpec = tween(220)) + fadeIn(animationSpec = tween(180)),
                                exit = shrinkVertically(animationSpec = tween(180)) + fadeOut(animationSpec = tween(140))
                            ) {
                                Column {
                                    homeFeedOptions.forEachIndexed { index, option ->
                                        MobileOrderedToggleRow(
                                            title = option.label,
                                            checked = isMetadataFeedEnabled(homeSelectedKeys, option.key),
                                            onToggle = {
                                                onUpdateProfile(profile.sanitizedUpdate(
                                                    homeFeedToggles = toggleMetadataFeed(homeSelectedKeys, metadataFeedKeys, option.key),
                                                    cs3FeedsConfigured = if (hasCs3Options) true else profile.cs3FeedsConfigured
                                                ))
                                            },
                                            canMoveUp = index > 0,
                                            canMoveDown = index < homeFeedOptions.lastIndex,
                                            onMoveUp = { onUpdateProfile(profile.sanitizedUpdate(homeFeedOrder = moveMetadataFeedOrder(metadataFeedOptions, profile.homeFeedOrder, option.key, -1))) },
                                            onMoveDown = { onUpdateProfile(profile.sanitizedUpdate(homeFeedOrder = moveMetadataFeedOrder(metadataFeedOptions, profile.homeFeedOrder, option.key, 1))) }
                                        )
                                    }
                                }
                            }
                            MobileExpandableHeaderRow(
                                title = AppStrings.t(lang, "settings.top_10_catalogs"),
                                count = homeVisibleFeedOptions.size,
                                subtitle = AppStrings.t(lang, "settings.top_10_catalogs_desc"),
                                expanded = topTenFeedsExpanded,
                                onClick = { topTenFeedsExpanded = !topTenFeedsExpanded }
                            )
                            AnimatedVisibility(
                                visible = topTenFeedsExpanded,
                                enter = expandVertically(animationSpec = tween(220)) + fadeIn(animationSpec = tween(180)),
                                exit = shrinkVertically(animationSpec = tween(180)) + fadeOut(animationSpec = tween(140))
                            ) {
                                Column {
                                    homeVisibleFeedOptions.forEachIndexed { index, option ->
                                        MobileToggleRow(
                                            title = option.label,
                                            checked = topTenSelectedKeys.contains(option.key),
                                            onToggle = {
                                                onUpdateProfile(profile.sanitizedUpdate(topTenFeedToggles = toggleTopTenFeed(topTenSelectedKeys, option.key)))
                                            }
                                        )
                                        if (index != homeVisibleFeedOptions.lastIndex) {
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "playback" -> {
                    item {
                        MobileSettingsCard {
                            MobileSettingsRow(
                                title = AppStrings.t(lang, "auto.subtitles"),
                                subtitle = AppStrings.t(lang, "settings.subtitles_settings_desc"),
                                icon = FluxaIcons.Subtitles,
                                onClick = { onNavigate("subtitles") }
                            )
                            MobileSettingsRow(
                                title = AppStrings.t(lang, "settings.advanced_settings"),
                                subtitle = AppStrings.t(lang, "settings.advanced_settings_desc"),
                                icon = FluxaIcons.Speed,
                                onClick = { onNavigate("advanced_settings") }
                            )
                        }
                    }
                    mobilePlaybackSettingsSection(
                        profile = profile,
                        lang = lang,
                        onChoiceDialog = { choiceDialog = it },
                        onUpdateProfile = onUpdateProfile
                    )
                }
                "subtitles" -> {
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "settings.preferences")) {
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "settings.preferred_audio_language"),
                                value = languageDisplayName(profile.safePreferredAudioLanguage, lang),
                                subtitle = AppStrings.t(lang, "settings.preferred_audio_language_desc"),
                                onClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.preferred_audio_language"),
                                        options = audioLanguageOptions(lang),
                                        selected = profile.safePreferredAudioLanguage,
                                        onSelect = { onUpdateProfile(profile.sanitizedUpdate(preferredAudioLanguage = it)) }
                                    )
                                }
                            )
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "settings.secondary_audio_language"),
                                value = languageDisplayName(profile.safeSecondaryAudioLanguage, lang),
                                subtitle = AppStrings.t(lang, "settings.secondary_audio_language_desc"),
                                onClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.secondary_audio_language"),
                                        options = audioLanguageOptions(lang),
                                        selected = profile.safeSecondaryAudioLanguage,
                                        onSelect = { onUpdateProfile(profile.sanitizedUpdate(secondaryAudioLanguage = it)) }
                                    )
                                }
                            )
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "settings.preferred_subtitle_language"),
                                value = languageDisplayName(profile.safePreferredSubtitleLanguage, lang),
                                subtitle = AppStrings.t(lang, "settings.preferred_subtitle_language_desc"),
                                onClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.preferred_subtitle_language"),
                                        options = subtitleLanguageOptions(lang),
                                        selected = profile.safePreferredSubtitleLanguage,
                                        onSelect = { onUpdateProfile(profile.sanitizedUpdate(preferredSubtitleLanguage = it)) }
                                    )
                                }
                            )
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "settings.secondary_subtitle_language"),
                                value = languageDisplayName(profile.safeSecondarySubtitleLanguage, lang),
                                subtitle = AppStrings.t(lang, "settings.secondary_subtitle_language_desc"),
                                onClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.secondary_subtitle_language"),
                                        options = subtitleLanguageOptions(lang),
                                        selected = profile.safeSecondarySubtitleLanguage,
                                        onSelect = { onUpdateProfile(profile.sanitizedUpdate(secondarySubtitleLanguage = it)) }
                                    )
                                }
                            )
                        }
                    }
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "settings.subtitle.customize")) {
                            MobileToggleRow(
                                title = AppStrings.t(lang, "auto.auto_enable_subtitles_db2311e6"),
                                subtitle = AppStrings.t(lang, "auto.enable_subtitles_automatically_when_availabl"),
                                checked = profile.safeAutoEnableSubtitles,
                                onToggle = { onUpdateProfile(profile.copy(autoEnableSubtitles = !profile.safeAutoEnableSubtitles)) }
                            )
                            MobileSubtitlePreview(profile = profile, lang = lang)
                            MobileStepperRow(
                                title = AppStrings.t(lang, "auto.subtitle_size_7fc78c82"),
                                value = subtitleSizePercentLabel(profile.safeSubtitleSizePercent),
                                subtitle = AppStrings.t(lang, "auto.tune_readability_on_tv_and_mobile"),
                                onDecrease = { onUpdateProfile(profile.copy(subtitleSize = (profile.safeSubtitleSizePercent - 10f).coerceIn(50f, 200f))) },
                                onIncrease = { onUpdateProfile(profile.copy(subtitleSize = (profile.safeSubtitleSizePercent + 10f).coerceIn(50f, 200f))) }
                            )
                            MobileColorOpacityRow(
                                title = AppStrings.t(lang, "settings.subtitle_text"),
                                subtitle = AppStrings.t(lang, "settings.subtitle_text_desc"),
                                opacityTitle = AppStrings.t(lang, "auto.text_transparency"),
                                selectedColor = profile.safeSubtitleColor,
                                opacity = profile.safeSubtitleTextOpacity,
                                lang = lang,
                                onColorClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.subtitle_text"),
                                        options = mobileSubtitleColorOptions(lang),
                                        selected = profile.safeSubtitleColor.toString(),
                                        onSelect = { onUpdateProfile(profile.copy(subtitleColor = it.toInt())) }
                                    )
                                },
                                onOpacity = { onUpdateProfile(profile.copy(subtitleTextOpacity = it)) }
                            )
                            MobileColorOpacityRow(
                                title = AppStrings.t(lang, "settings.subtitle_background"),
                                subtitle = AppStrings.t(lang, "settings.subtitle_background_desc"),
                                opacityTitle = AppStrings.t(lang, "auto.background_transparency"),
                                selectedColor = profile.safeSubtitleBackgroundColor,
                                opacity = profile.safeSubtitleBackgroundOpacity,
                                lang = lang,
                                onColorClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.subtitle_background"),
                                        options = mobileSubtitleColorOptions(lang),
                                        selected = profile.safeSubtitleBackgroundColor.toString(),
                                        onSelect = { onUpdateProfile(profile.copy(subtitleBackgroundColor = it.toInt())) }
                                    )
                                },
                                onOpacity = { onUpdateProfile(profile.copy(subtitleBackgroundOpacity = it)) }
                            )
                            MobileColorOpacityRow(
                                title = AppStrings.t(lang, "settings.subtitle_outline"),
                                subtitle = AppStrings.t(lang, "settings.subtitle_outline_desc"),
                                opacityTitle = AppStrings.t(lang, "settings.subtitle.outline_opacity"),
                                selectedColor = profile.safeSubtitleOutlineColor,
                                opacity = profile.safeSubtitleOutlineOpacity,
                                lang = lang,
                                onColorClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.subtitle_outline"),
                                        options = mobileSubtitleColorOptions(lang),
                                        selected = profile.safeSubtitleOutlineColor.toString(),
                                        onSelect = { onUpdateProfile(profile.copy(subtitleOutlineColor = it.toInt())) }
                                    )
                                },
                                onOpacity = { onUpdateProfile(profile.copy(subtitleOutlineOpacity = it)) }
                            )
                        }
                    }
                }
                "advanced_settings" -> {
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "settings.advanced")) {
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "settings.buffer_cache"),
                                value = bufferCacheLabel(profile.safePlayerBufferCacheMb),
                                subtitle = AppStrings.t(lang, "settings.buffer_cache_desc"),
                                onClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.buffer_cache"),
                                        options = listOf(
                                            ChoiceOption("100", "100 MB"),
                                            ChoiceOption("500", "500 MB"),
                                            ChoiceOption("1000", "1 GB"),
                                            ChoiceOption("2000", "2 GB")
                                        ),
                                        selected = profile.safePlayerBufferCacheMb.toString(),
                                        onSelect = { onUpdateProfile(profile.sanitizedUpdate(playerBufferCacheMb = it.toInt())) }
                                    )
                                }
                            )
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "settings.forward_buffer"),
                                value = settingsSecondsLabel(lang, profile.safePlayerForwardBufferSeconds),
                                subtitle = AppStrings.t(lang, "settings.forward_buffer_desc"),
                                onClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.forward_buffer"),
                                        options = bufferSecondOptions(lang),
                                        selected = profile.safePlayerForwardBufferSeconds.toString(),
                                        onSelect = { onUpdateProfile(profile.sanitizedUpdate(playerForwardBufferSeconds = it.toInt())) }
                                    )
                                }
                            )
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "settings.back_buffer"),
                                value = settingsSecondsLabel(lang, profile.safePlayerBackBufferSeconds),
                                subtitle = AppStrings.t(lang, "settings.back_buffer_desc"),
                                onClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.back_buffer"),
                                        options = bufferSecondOptions(lang, includeZero = true),
                                        selected = profile.safePlayerBackBufferSeconds.toString(),
                                        onSelect = { onUpdateProfile(profile.sanitizedUpdate(playerBackBufferSeconds = it.toInt())) }
                                    )
                                }
                            )
                        }
                    }
                    item {
                        MobileSettingsGroup(AppStrings.t(lang, "settings.decoder")) {
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "settings.dv_fallback"),
                                value = dolbyVisionFallbackLabel(profile.safeDolbyVisionFallbackMode, lang),
                                subtitle = AppStrings.t(lang, "settings.dv_fallback_desc"),
                                onClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.dv_fallback"),
                                        options = dolbyVisionFallbackOptions(lang),
                                        selected = profile.safeDolbyVisionFallbackMode,
                                        onSelect = { onUpdateProfile(profile.sanitizedUpdate(dolbyVisionFallbackMode = it)) }
                                    )
                                }
                            )
                            if (profile.safeDolbyVisionFallbackMode == "convert_dv81") {
                                MobileChoiceRow(
                                    title = AppStrings.t(lang, "settings.dv_rpu_mode"),
                                    value = dvRpuModeLabel(profile.safeDvRpuMode, lang),
                                    subtitle = AppStrings.t(lang, "settings.dv_rpu_mode_desc"),
                                    onClick = {
                                        choiceDialog = MobileChoiceDialogState(
                                            title = AppStrings.t(lang, "settings.dv_rpu_mode"),
                                            options = dvRpuModeOptions(lang),
                                            selected = profile.safeDvRpuMode.toString(),
                                            onSelect = { onUpdateProfile(profile.sanitizedUpdate(dvRpuMode = it.toIntOrNull() ?: 2)) }
                                        )
                                    }
                                )
                                MobileToggleRow(
                                    title = AppStrings.t(lang, "settings.dv_zero_level5"),
                                    subtitle = AppStrings.t(lang, "settings.dv_zero_level5_desc"),
                                    checked = profile.safeDvZeroLevel5,
                                    onToggle = { onUpdateProfile(profile.sanitizedUpdate(dvZeroLevel5 = !profile.safeDvZeroLevel5)) }
                                )
                                MobileChoiceRow(
                                    title = AppStrings.t(lang, "settings.dv_hdr10plus_mode"),
                                    value = dvHdr10PlusModeOptions(lang).firstOrNull { it.value == profile.safeDvHdr10PlusMode }?.label
                                        ?: AppStrings.t(lang, "settings.dv_hdr10plus_mode_auto"),
                                    subtitle = AppStrings.t(lang, "settings.dv_hdr10plus_mode_desc"),
                                    onClick = {
                                        choiceDialog = MobileChoiceDialogState(
                                            title = AppStrings.t(lang, "settings.dv_hdr10plus_mode"),
                                            options = dvHdr10PlusModeOptions(lang),
                                            selected = profile.safeDvHdr10PlusMode,
                                            onSelect = { onUpdateProfile(profile.sanitizedUpdate(dvHdr10PlusMode = it)) }
                                        )
                                    }
                                )
                            }
                            MobileChoiceRow(
                                title = AppStrings.t(lang, "settings.audio_decoder_mode"),
                                value = audioDecoderModeLabel(profile.safeAudioDecoderMode, lang),
                                subtitle = AppStrings.t(lang, "settings.audio_decoder_mode_desc"),
                                onClick = {
                                    choiceDialog = MobileChoiceDialogState(
                                        title = AppStrings.t(lang, "settings.audio_decoder_mode"),
                                        options = audioDecoderModeOptions(lang),
                                        selected = profile.safeAudioDecoderMode,
                                        onSelect = { onUpdateProfile(profile.sanitizedUpdate(audioDecoderMode = it)) }
                                    )
                                }
                            )
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.tunneled_playback"),
                                subtitle = AppStrings.t(lang, "settings.tunneled_playback_desc"),
                                checked = profile.safeTunneledPlayback,
                                onToggle = { onUpdateProfile(profile.sanitizedUpdate(tunneledPlayback = !profile.safeTunneledPlayback)) }
                            )
                            MobileToggleRow(
                                title = AppStrings.t(lang, "settings.fps_counter"),
                                subtitle = AppStrings.t(lang, "settings.fps_counter_desc"),
                                checked = profile.safeShowFpsCounter,
                                onToggle = { onUpdateProfile(profile.sanitizedUpdate(showFpsCounter = !profile.safeShowFpsCounter)) }
                            )
                        }
                    }
                }
                "integrations" -> {
                    item {
                        MobileInfoRow(
                            title = AppStrings.t(lang, "auto.version_c1e9b8d0"),
                            value = AppStrings.format(lang, "settings.version_footer", BuildConfig.VERSION_NAME, BuildConfig.VERSION_CODE)
                        )
                    }
                    item {
                        MobileActionRow(
                            title = AppStrings.t(lang, "auto.restart_app_e2f23e42"),
                            value = AppStrings.t(lang, "auto.restart"),
                            onClick = onReboot
                        )
                    }
                }
                "developer" -> {
                    item {
                        MobileDeveloperSettings(lang)
                    }
                }
                "downloads" -> {
                    item {
                        LaunchedEffect(Unit) {
                            while (true) {
                                offlineDownloadManager.refresh()
                                delay(1000L)
                            }
                        }
                    }
                    item {
                        MobileChoiceRow(
                            title = AppStrings.t(lang, "settings.download_source_selection"),
                            value = streamSourceSelectionLabel(profile.safeDownloadSourceSelectionMode, lang),
                            subtitle = AppStrings.t(lang, "settings.download_source_selection_desc"),
                            onClick = {
                                choiceDialog = MobileChoiceDialogState(
                                    title = AppStrings.t(lang, "settings.download_source_selection"),
                                    options = streamSourceSelectionOptions(lang),
                                    selected = profile.safeDownloadSourceSelectionMode,
                                    onSelect = { onUpdateProfile(profile.sanitizedUpdate(downloadSourceSelectionMode = it)) }
                                )
                            }
                        )
                    }
                    if (profile.safeDownloadSourceSelectionMode == STREAM_SOURCE_MODE_REGEX) {
                        item {
                            MobileDownloadRegexPatternRow(
                                lang = lang,
                                value = profile.safeDownloadSourceRegexPattern,
                                onValueChange = { onUpdateProfile(profile.sanitizedUpdate(downloadSourceRegexPattern = it)) }
                            )
                        }
                    }
                    item {
                        MobileChoiceRow(
                            title = AppStrings.t(lang, "settings.download_subtitle"),
                            value = downloadSubtitleLabel(profile.safeDownloadSubtitleLanguage, lang),
                            subtitle = AppStrings.t(lang, "settings.download_subtitle_desc"),
                            onClick = {
                                choiceDialog = MobileChoiceDialogState(
                                    title = AppStrings.t(lang, "settings.download_subtitle"),
                                    options = downloadSubtitleOptions(lang),
                                    selected = profile.safeDownloadSubtitleLanguage,
                                    onSelect = { onUpdateProfile(profile.sanitizedUpdate(downloadSubtitleLanguage = it)) }
                                )
                            }
                        )
                    }
                    if (offlineDownloads.isEmpty()) {
                        item {
                            Text(AppStrings.t(lang, "downloads.empty"), color = Color.White.copy(alpha = 0.5f), fontSize = 14.sp)
                        }
                    } else {
                        items(offlineDownloads, key = { it.id }) { item ->
                            OfflineDownloadRow(
                                modifier = Modifier.animateItem(),
                                item = item,
                                lang = lang,
                                onCancel = {
                                    offlineDownloadManager.cancel(item.id)
                                },
                                onClick = {
                                    offlineDownloadManager.refresh()
                                    if (item.isPlayable) onOpenDownload(item)
                                }
                            )
                        }
                    }
                }
            }
        }
    }
    }

    CompositionLocalProvider(LocalMobileSettingsPalette provides palette) {
    choiceDialog?.let { dialog ->
        MobileChoiceDialog(
            title = dialog.title,
            options = dialog.options,
            selected = dialog.selected,
            onDismiss = { choiceDialog = null },
            onSelect = {
                dialog.onSelect(it)
                choiceDialog = null
            }
        )
    }
    }
}
