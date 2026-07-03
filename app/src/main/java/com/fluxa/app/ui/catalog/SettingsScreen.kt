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
fun SettingsScreen(
    activeProfile: UserProfile?,
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
    onUpdateProfile: (UserProfile) -> Unit,
    onUpdateInfoChanged: (UpdateManager.UpdateInfo?) -> Unit = {},
    viewModel: HomeViewModel,
    tvNavActions: TvNavActions
) {
    val deviceType = LocalDeviceType.current
    val lang = activeProfile?.safeLanguage ?: "en"
    val useTopBar = activeProfile?.safeTvNavLayout == "top"
    val railGutter = if (useTopBar) 56.dp else 126.dp
    val contentTopPadding = if (useTopBar) 108.dp else 40.dp
    val accountTitle = activeProfile?.displayName ?: AppStrings.t(lang, "auto.guest")
    val userAddons by viewModel.userAddons.collectAsState()
    val totalWatchedContentDuration by viewModel.totalWatchedContentDuration.collectAsState()
    var selectedTab by remember { mutableStateOf("account") }

    val tabs = remember(lang, accountTitle) {
        listOf(
            SettingsTab("account", accountTitle, FluxaIcons.AccountCircle),
            SettingsTab("general", AppStrings.t(lang, "auto.general"), FluxaIcons.Settings),
            SettingsTab("appearance", AppStrings.t(lang, "auto.appearance"), FluxaIcons.Palette),
            SettingsTab("playback", AppStrings.t(lang, "auto.playback"), FluxaIcons.PlayCircle),
            SettingsTab("content", AppStrings.t(lang, "auto.catalogs"), FluxaIcons.MenuBook),
            SettingsTab("addons", AppStrings.t(lang, "auto.add_ons"), FluxaIcons.Extension),
            SettingsTab("downloads", AppStrings.t(lang, "auto.downloads"), FluxaIcons.Download),
            SettingsTab("system", AppStrings.t(lang, "auto.system"), FluxaIcons.Settings),
            SettingsTab("developer", AppStrings.t(lang, "settings.developer"), FluxaIcons.Memory)
        )
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                Brush.verticalGradient(
                    listOf(Color(0xFF0B0F16), Color(0xFF07090D), Color(0xFF050608))
                )
            )
    ) {
        if (deviceType == DeviceType.TV) {
            Box(modifier = Modifier.fillMaxSize()) {
            Row(modifier = Modifier.fillMaxSize().padding(start = railGutter, top = contentTopPadding)) {
                Column(
                    modifier = Modifier
                        .fillMaxHeight()
                        .width(320.dp)
                        .background(Color.Black.copy(alpha = 0.28f))
                        .verticalScroll(rememberScrollState())
                        .padding(28.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Text(
                        text = AppStrings.t(lang, "nav.settings"),
                        color = Color.White,
                        fontSize = 30.sp,
                        fontWeight = FontWeight.Black
                    )
                    Text(
                        text = AppStrings.t(lang, "auto.control_profile_playback_and_device_preferen"),
                        color = Color.White.copy(alpha = 0.58f),
                        fontSize = 13.sp
                    )

                    Spacer(Modifier.height(20.dp))

                    tabs.forEach { tab ->
                        SettingsSidebarItem(tab, selectedTab == tab.id) { selectedTab = tab.id }
                    }

                    Spacer(Modifier.height(20.dp))

                    SettingsSidebarItem(
                        SettingsTab("back", AppStrings.t(lang, "auto.back"), FluxaIcons.ArrowBack),
                        false
                    ) { onBack() }
                }

                Column(
                    modifier = Modifier
                        .fillMaxHeight()
                        .weight(1f)
                        .verticalScroll(rememberScrollState())
                        .padding(36.dp),
                    verticalArrangement = Arrangement.spacedBy(28.dp)
                ) {
                    SettingsContent(
                        tabId = selectedTab,
                        profile = activeProfile,
                        lang = lang,
                        onLogout = onLogout,
                        onConnectStremio = onConnectStremio,
                        onConnectTrakt = onConnectTrakt,
                        onConnectMal = onConnectMal,
                        onConnectSimkl = onConnectSimkl,
                        onManageAddons = onManageAddons,
                        onReboot = onReboot,
                        onUpdateProfile = onUpdateProfile,
                        viewModel = viewModel
                    )
                }
            }
            if (useTopBar) {
                TvHomeTopBar(
                    lang = lang,
                    selected = TvNavDestination.Settings,
                    onHomeClick = tvNavActions.onHome,
                    onSearchClick = tvNavActions.onSearch,
                    onWatchlistClick = tvNavActions.onWatchlist,
                    onExploreClick = tvNavActions.onExplore,
                    onProfileClick = {},
                    contentFocusRequester = null
                )
            } else {
                TvHomeNavRail(
                    lang = lang,
                    selected = TvNavDestination.Settings,
                    onHomeClick = tvNavActions.onHome,
                    onSearchClick = tvNavActions.onSearch,
                    onWatchlistClick = tvNavActions.onWatchlist,
                    onExploreClick = tvNavActions.onExplore,
                    onProfileClick = {},
                    contentFocusRequester = null
                )
            }
            }
        } else {
            var mobileCategory by remember { mutableStateOf<String?>(null) }
            
            androidx.activity.compose.BackHandler(enabled = mobileCategory != null) {
                mobileCategory = null
            }
            
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Color(0xFF070707))
            ) {
                if (mobileCategory == null) {
                    MobileSettingsHub(
                        profile = activeProfile ?: return@Box,
                        lang = lang,
                        totalWatchedContentDuration = totalWatchedContentDuration,
                        onNavigate = { mobileCategory = it },
                        onBack = onBack,
                        onLogout = onLogout,
                        onManageAddons = onManageAddons,
                        onUpdateInfoChanged = onUpdateInfoChanged
                    )
                } else mobileCategory?.let { category ->
                    MobileCategoryDetail(
                        category = category,
                        profile = activeProfile ?: return@Box,
                        lang = lang,
                        onBack = { mobileCategory = null },
                        onLogout = onLogout,
                        onConnectStremio = onConnectStremio,
                        onConnectTrakt = onConnectTrakt,
                        onConnectMal = onConnectMal,
                        onConnectSimkl = onConnectSimkl,
                        onManageAddons = onManageAddons,
                        onWatchlistClick = onWatchlistClick,
                        onOpenDownload = onOpenDownload,
                        onReboot = onReboot,
                        onNavigate = { mobileCategory = it },
                        viewModel = viewModel,
                        onUpdateProfile = onUpdateProfile
                    )
                }
            }
        }
    }
}









internal fun UserProfile.sanitizedUpdate(
    language: String? = this.safeLanguage,
    cardLayout: String? = this.safeCardLayout,
    continueWatchingLayout: String? = this.safeContinueWatchingLayout,
    continueWatchingArtwork: String? = this.safeContinueWatchingArtwork,
    continueWatchingEnabled: Boolean? = this.safeContinueWatchingEnabled,
    continueWatchingHideTitles: Boolean? = this.safeContinueWatchingHideTitles,
    preferredSubtitleLanguage: String? = this.safePreferredSubtitleLanguage,
    preferredAudioLanguage: String? = this.safePreferredAudioLanguage,
    secondarySubtitleLanguage: String? = this.safeSecondarySubtitleLanguage,
    secondaryAudioLanguage: String? = this.safeSecondaryAudioLanguage,
    seekForwardSeconds: Int? = this.safeSeekForwardSeconds,
    seekBackwardSeconds: Int? = this.safeSeekBackwardSeconds,
    playerBufferCacheMb: Int? = this.safePlayerBufferCacheMb,
    playerForwardBufferSeconds: Int? = this.safePlayerForwardBufferSeconds,
    playerBackBufferSeconds: Int? = this.safePlayerBackBufferSeconds,
    timezoneConversionEnabled: Boolean? = this.safeTimezoneConversionEnabled,
    autoSkipIntro: Boolean? = this.safeAutoSkipIntro,
    autoPlayNextEpisode: Boolean? = this.safeAutoPlayNextEpisode,
    nextEpisodeThresholdPercent: Float? = this.safeNextEpisodeThresholdPercent,
    watchedThresholdPercent: Float? = this.safeWatchedThresholdPercent,
    preferredPlayer: String? = this.safePreferredPlayer,
    autoEnableSubtitles: Boolean? = this.safeAutoEnableSubtitles,
    subtitleSize: Float? = this.safeSubtitleSize,
    subtitleColor: Int? = this.safeSubtitleColor,
    subtitleBackgroundColor: Int? = this.safeSubtitleBackgroundColor,
    subtitleOutlineColor: Int? = this.safeSubtitleOutlineColor,
    subtitleTextOpacity: Float? = this.safeSubtitleTextOpacity,
    subtitleBackgroundOpacity: Float? = this.safeSubtitleBackgroundOpacity,
    subtitleOutlineOpacity: Float? = this.safeSubtitleOutlineOpacity,
    subtitleShadow: Boolean? = this.safeSubtitleShadow,
    torrentSpeedPreset: String? = this.safeTorrentSpeedPreset,
    torrentCachePreset: String? = this.safeTorrentCachePreset,
    appTheme: String? = this.safeAppTheme,
    accentColorArgb: Int? = this.safeAccentColorArgb,
    cardCornerPreset: String? = this.safeCardCornerPreset,
    interfaceDensity: String? = this.safeInterfaceDensity,
    amoledMode: Boolean? = this.safeAmoledMode,
    posterWidthPreset: String? = this.safePosterWidthPreset,
    posterLandscapeMode: Boolean? = this.safePosterLandscapeMode,
    posterHideTitles: Boolean? = this.safePosterHideTitles,
    detailEpisodeViewMode: String? = this.detailEpisodeViewMode ?: "modern",
    detailSeasonSelectorMode: String? = this.safeDetailSeasonSelectorMode,
    detailSeasonPostersOnHero: Boolean? = this.safeDetailSeasonPostersOnHero,
    homeSeasonPostersOnHero: Boolean? = this.safeHomeSeasonPostersOnHero,
    animationsEnabled: Boolean? = this.safeAnimationsEnabled,
    reduceMotion: Boolean? = this.safeReduceMotion,
    startPage: String? = this.safeStartPage,
    notificationsEnabled: Boolean? = this.safeNotificationsEnabled,
    alertNewEpisodes: Boolean? = this.safeAlertNewEpisodes,
    automaticUpdates: Boolean? = this.safeAutomaticUpdates,
    backgroundPlayback: Boolean? = this.safeBackgroundPlayback,
    pictureInPicture: Boolean? = this.safePictureInPicture,
    playbackSpeed: Float? = this.safePlaybackSpeed,
    holdToSpeedEnabled: Boolean? = this.safeHoldToSpeedEnabled,
    holdSpeed: Float? = this.safeHoldSpeed,
    audioDecoderMode: String? = this.safeAudioDecoderMode,
    dolbyVisionFallbackMode: String? = this.safeDolbyVisionFallbackMode,
    dvRpuMode: Int? = this.safeDvRpuMode,
    dvZeroLevel5: Boolean? = this.safeDvZeroLevel5,
    dvHdr10PlusMode: String? = this.safeDvHdr10PlusMode,
    tunneledPlayback: Boolean? = this.safeTunneledPlayback,
    useIntroDb: Boolean? = this.safeUseIntroDb,
    useAniSkip: Boolean? = this.safeUseAniSkip,
    defaultQuality: String? = this.safeDefaultQuality,
    mobileDataUsage: String? = this.safeMobileDataUsage,
    hdrPlayback: Boolean? = this.safeHdrPlayback,
    resumePlayback: Boolean? = this.safeResumePlayback,
    autoplayMode: String? = this.safeAutoplayMode,
    streamSourceSelectionMode: String? = this.safeStreamSourceSelectionMode,
    streamSourceRegexPattern: String? = this.safeStreamSourceRegexPattern,
    downloadSourceSelectionMode: String? = this.safeDownloadSourceSelectionMode,
    downloadSourceRegexPattern: String? = this.safeDownloadSourceRegexPattern,
    downloadSubtitleLanguage: String? = this.safeDownloadSubtitleLanguage,
    tryBingeGroup: Boolean? = this.safeTryBingeGroup,
    animeUseMpv: Boolean? = this.safeAnimeUseMpv,
    heroFeedToggles: List<String>? = this.heroFeedToggles,
    homeFeedToggles: List<String>? = this.homeFeedToggles,
    topTenFeedToggles: List<String>? = this.topTenFeedToggles,
    heroFeedOrder: List<String>? = this.heroFeedOrder,
    homeFeedOrder: List<String>? = this.homeFeedOrder,
    showHeroSection: Boolean? = this.safeShowHeroSection,
    libraryCollections: List<LibraryUserCollection>? = this.libraryCollections,
    cs3FeedsConfigured: Boolean? = this.cs3FeedsConfigured,
    traktAccessToken: String? = this.traktAccessToken,
    traktRefreshToken: String? = this.traktRefreshToken,
    traktLastSyncAt: Long? = this.traktLastSyncAt,
    traktLastSyncedItems: Int? = this.traktLastSyncedItems,
    traktLastContinueWatchingCount: Int? = this.traktLastContinueWatchingCount,
    traktLastWatchlistCount: Int? = this.traktLastWatchlistCount
): UserProfile {
    return UserProfile(
        id = id,
        email = email,
        authKey = authKey,
        colorArgb = colorArgb,
        isGuest = isGuest,
        avatarUrl = avatarUrl,
        pinHash = pinHash,
        streamingServerUrl = safeStreamingServerUrl,
        localAddons = safeInstalledLocalAddons,
        disabledLocalAddons = disabledLocalAddons,
        language = language,
        buildServerUrl = buildServerUrl,
        subtitleSize = subtitleSize,
        subtitleColor = subtitleColor,
        subtitleBackgroundColor = subtitleBackgroundColor,
        subtitleOutlineColor = subtitleOutlineColor,
        subtitleTextOpacity = subtitleTextOpacity,
        subtitleBackgroundOpacity = subtitleBackgroundOpacity,
        subtitleOutlineOpacity = subtitleOutlineOpacity,
        preferredSubtitleLanguage = preferredSubtitleLanguage,
        preferredAudioLanguage = preferredAudioLanguage,
        secondarySubtitleLanguage = secondarySubtitleLanguage,
        secondaryAudioLanguage = secondaryAudioLanguage,
        ambientLight = safeAmbientLight,
        audioDecoderMode = audioDecoderMode,
        preferredPlayer = preferredPlayer,
        traktAccessToken = traktAccessToken,
        traktRefreshToken = traktRefreshToken,
        traktTokenExpiresAt = traktTokenExpiresAt,
        traktLastSyncAt = traktLastSyncAt,
        traktLastSyncedItems = traktLastSyncedItems,
        traktLastContinueWatchingCount = traktLastContinueWatchingCount,
        traktLastWatchlistCount = traktLastWatchlistCount,
        cardLayout = cardLayout,
        continueWatchingLayout = continueWatchingLayout,
        continueWatchingArtwork = continueWatchingArtwork,
        continueWatchingEnabled = continueWatchingEnabled,
        continueWatchingHideTitles = continueWatchingHideTitles,
        subtitleShadow = subtitleShadow,
        autoEnableSubtitles = autoEnableSubtitles,
        autoSkipIntro = autoSkipIntro,
        autoPlayNextEpisode = autoPlayNextEpisode,
        nextEpisodeThresholdPercent = nextEpisodeThresholdPercent,
        watchedThresholdPercent = watchedThresholdPercent,
        seekForwardSeconds = seekForwardSeconds,
        seekBackwardSeconds = seekBackwardSeconds,
        playerBufferCacheMb = playerBufferCacheMb,
        playerForwardBufferSeconds = playerForwardBufferSeconds,
        playerBackBufferSeconds = playerBackBufferSeconds,
        timezoneConversionEnabled = timezoneConversionEnabled,
        torrentWifiOnly = safeTorrentWifiOnly,
        torrentMaxConnections = safeTorrentMaxConnections,
        torrentSpeedPreset = torrentSpeedPreset,
        torrentCachePreset = torrentCachePreset,
        appTheme = appTheme,
        accentColorArgb = accentColorArgb,
        cardCornerPreset = cardCornerPreset,
        interfaceDensity = interfaceDensity,
        amoledMode = amoledMode,
        posterWidthPreset = posterWidthPreset,
        posterLandscapeMode = posterLandscapeMode,
        posterHideTitles = posterHideTitles,
        detailEpisodeViewMode = detailEpisodeViewMode,
        detailSeasonSelectorMode = detailSeasonSelectorMode,
        detailSeasonPostersOnHero = detailSeasonPostersOnHero,
        homeSeasonPostersOnHero = homeSeasonPostersOnHero,
        animationsEnabled = animationsEnabled,
        reduceMotion = reduceMotion,
        startPage = startPage,
        notificationsEnabled = notificationsEnabled,
        alertNewEpisodes = alertNewEpisodes,
        automaticUpdates = automaticUpdates,
        backgroundPlayback = backgroundPlayback,
        pictureInPicture = pictureInPicture,
        playbackSpeed = playbackSpeed,
        holdToSpeedEnabled = holdToSpeedEnabled,
        holdSpeed = holdSpeed,
        dolbyVisionFallbackMode = dolbyVisionFallbackMode,
        dvRpuMode = dvRpuMode,
        dvZeroLevel5 = dvZeroLevel5,
        dvHdr10PlusMode = dvHdr10PlusMode,
        tunneledPlayback = tunneledPlayback,
        useIntroDb = useIntroDb,
        useAniSkip = useAniSkip,
        defaultQuality = defaultQuality,
        mobileDataUsage = mobileDataUsage,
        hdrPlayback = hdrPlayback,
        resumePlayback = resumePlayback,
        autoplayMode = autoplayMode,
        streamSourceSelectionMode = streamSourceSelectionMode,
        streamSourceRegexPattern = streamSourceRegexPattern,
        downloadSourceSelectionMode = downloadSourceSelectionMode,
        downloadSourceRegexPattern = downloadSourceRegexPattern,
        downloadSubtitleLanguage = downloadSubtitleLanguage,
        tryBingeGroup = tryBingeGroup,
        animeUseMpv = animeUseMpv,
        heroFeedToggles = heroFeedToggles,
        homeFeedToggles = homeFeedToggles,
        topTenFeedToggles = topTenFeedToggles,
        heroFeedOrder = heroFeedOrder,
        homeFeedOrder = homeFeedOrder,
        showHeroSection = showHeroSection,
        libraryCollections = libraryCollections,
        cs3FeedsConfigured = cs3FeedsConfigured
    )
}


























internal data class ChoiceOption(val value: String, val label: String)
internal data class SettingsTab(val id: String, val label: String, val icon: ImageVector)
