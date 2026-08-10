package com.fluxa.app.shared.feature.settings

import com.fluxa.app.data.local.UserProfile

fun UserProfile.withGeneralSettings(value: SettingsGeneralUiModel): UserProfile = copy(
    language = value.language,
    startPage = value.startPage,
    backgroundPlayback = value.backgroundPlayback,
)

fun UserProfile.withAppearanceSettings(value: SettingsAppearanceUiModel): UserProfile = copy(
    accentColorArgb = value.accentColorArgb.toInt(),
    amoledMode = value.amoledMode,
    appTheme = if (value.amoledMode) "dark" else appTheme,
    liquidGlassMode = value.liquidGlassMode,
    animationsEnabled = value.animationsEnabled,
    floatingBottomBar = value.floatingBottomBar,
    bottomBarLabels = value.bottomBarLabels,
    topNavigationBar = value.topNavigationBar,
)

fun UserProfile.withAppearanceHomeSettings(value: SettingsAppearanceHomeUiModel): UserProfile = copy(
    cardCornerPreset = value.cardCornerPreset,
    interfaceDensity = value.interfaceDensity,
    posterWidthPreset = value.posterWidthPreset,
    posterLandscapeMode = value.posterLandscapeMode,
    posterHideTitles = value.posterHideTitles,
    homeSeasonPostersOnHero = value.homeSeasonPostersOnHero,
    trailerOnHomeHeroEnabled = value.trailerOnHomeHeroEnabled,
    trailerOnHomeHeroDelaySeconds = value.trailerOnHomeHeroDelaySeconds,
    continueWatchingLayout = if (value.continueWatchingHorizontal) "horizontal" else "vertical",
    continueWatchingEnabled = value.continueWatchingEnabled,
    continueWatchingHideTitles = value.continueWatchingHideTitles,
    continueWatchingSource = value.continueWatchingSource,
    upcomingRowEnabled = value.upcomingRowEnabled,
)

fun UserProfile.withAppearanceDetailSettings(value: SettingsAppearanceDetailUiModel): UserProfile = copy(
    detailScreenStyle = value.detailScreenStyle,
    detailPreferClearlogo = value.detailPreferClearlogo,
    detailShowEpisodeDescriptions = value.detailShowEpisodeDescriptions,
    detailShowCast = value.detailShowCast,
    detailShowRecommendations = value.detailShowRecommendations,
    detailCollapsingHero = value.detailCollapsingHero,
    trailerOnDetailHeroEnabled = value.trailerOnDetailHeroEnabled,
    trailerOnDetailHeroDelaySeconds = value.trailerOnDetailHeroDelaySeconds,
    blurUnwatchedEpisodes = value.blurUnwatchedEpisodes,
    detailSeasonSelectorMode = value.detailSeasonSelectorMode,
    detailSeasonPostersOnHero = value.detailSeasonPostersOnHero,
    episodeCardsLayout = value.episodeCardsLayout,
)

fun UserProfile.withPlaybackSettings(value: SettingsPlaybackUiModel): UserProfile = copy(
    preferredPlayer = value.preferredPlayer,
    externalPlayerTarget = value.externalPlayerTarget.takeIf(String::isNotBlank),
    mpvCustomOptions = value.mpvCustomOptions.takeIf(String::isNotBlank),
    animeUseMpv = value.animeUseMpv,
    animePreferJapaneseAudio = value.animePreferJapaneseAudio,
    playbackSpeed = value.playbackSpeed,
    seekForwardSeconds = value.seekForwardSeconds,
    seekBackwardSeconds = value.seekBackwardSeconds,
    holdToSpeedEnabled = value.holdToSpeedEnabled,
    holdSpeed = value.holdSpeed,
    streamSourceSelectionMode = value.streamSourceSelectionMode,
    streamSourceRegexPattern = value.streamSourceRegexPattern.takeIf(String::isNotBlank),
    autoplayMode = value.autoplayMode,
    autoPlayNextEpisode = value.autoPlayNextEpisode,
    autoPlayCountdownSecs = value.autoPlayCountdownSecs,
    autoRetryNextSource = value.autoRetryNextSource,
    tryBingeGroup = value.tryBingeGroup,
    nextEpisodeThresholdPercent = value.nextEpisodeThresholdPercent,
    watchedThresholdPercent = value.watchedThresholdPercent,
    useIntroDb = value.useSkipSegments,
    introDbApiKey = value.introDbApiKey.takeIf(String::isNotBlank),
    useAniSkip = value.useSkipSegments,
    useChapterSkip = value.useChapterSkip,
    autoSkipIntro = value.autoSkipIntro,
    contentWarningsEnabled = value.contentWarningsEnabled,
)

fun UserProfile.withSubtitleSettings(value: SettingsSubtitlesUiModel): UserProfile = copy(
    preferredAudioLanguage = value.preferredAudioLanguage,
    secondaryAudioLanguage = value.secondaryAudioLanguage,
    preferredSubtitleLanguage = value.preferredSubtitleLanguage,
    secondarySubtitleLanguage = value.secondarySubtitleLanguage,
    autoEnableSubtitles = value.autoEnableSubtitles,
    subtitleShadow = value.subtitleShadow,
    subtitleSize = value.subtitleSize,
    subtitleColor = value.subtitleColorArgb.toInt(),
    subtitleTextOpacity = value.subtitleTextOpacity,
    subtitleBackgroundColor = value.subtitleBackgroundColorArgb.toInt(),
    subtitleBackgroundOpacity = value.subtitleBackgroundOpacity,
    subtitleOutlineColor = value.subtitleOutlineColorArgb.toInt(),
    subtitleOutlineOpacity = value.subtitleOutlineOpacity,
)

fun UserProfile.withAdvancedSettings(value: SettingsAdvancedUiModel): UserProfile = copy(
    playerBufferCacheMb = value.playerBufferCacheMb,
    playerForwardBufferSeconds = value.playerForwardBufferSeconds,
    playerBackBufferSeconds = value.playerBackBufferSeconds,
    audioDecoderMode = value.audioDecoderMode,
    tunneledPlayback = value.tunneledPlayback,
)

fun UserProfile.withAddonSettings(value: SettingsAddonsUiModel): UserProfile = copy(
    torrentSpeedPreset = value.torrentSpeedPreset,
    torrentWifiOnly = value.torrentWifiOnly,
)

fun UserProfile.withDownloadSettings(value: SettingsDownloadsUiModel): UserProfile = copy(
    downloadSourceSelectionMode = value.downloadSourceSelectionMode,
    downloadSourceRegexPattern = value.downloadSourceRegexPattern.takeIf(String::isNotBlank),
    downloadSubtitleLanguage = value.downloadSubtitleLanguage,
)

fun UserProfile.withSystemSettings(value: SettingsSystemUiModel): UserProfile = copy(
    automaticUpdates = value.automaticUpdates,
)

fun UserProfile.withAccountSettings(value: SettingsAccountUiModel): UserProfile = copy(
    syncCwSourceOfTruth = value.syncCwSourceOfTruth,
    syncCwRanking = value.syncCwRanking,
    integrationLibrarySource = value.integrationLibrarySource,
    continueWatchingDays = value.continueWatchingDays,
    traktCommentsEnabled = value.traktCommentsEnabled,
    tmdbApiKey = value.tmdbApiKey,
    mdblistApiKey = value.mdblistApiKey,
    tmdbCastImagesEnabled = value.tmdbCastImagesEnabled,
    tmdbSimilarResultsEnabled = value.tmdbSimilarResultsEnabled,
    tmdbTrailersEnabled = value.tmdbTrailersEnabled,
    tmdbRecommendationsEnabled = value.tmdbRecommendationsEnabled,
    tmdbCollectionInfoEnabled = value.tmdbCollectionInfoEnabled,
    tmdbEpisodeImagesEnabled = value.tmdbEpisodeImagesEnabled,
    tmdbLogosBackdropsEnabled = value.tmdbLogosBackdropsEnabled,
    tmdbRatingsEnabled = value.tmdbRatingsEnabled,
    tmdbBasicInfoEnabled = value.tmdbBasicInfoEnabled,
    tmdbDetailsEnabled = value.tmdbDetailsEnabled,
    tmdbProductionsEnabled = value.tmdbProductionsEnabled,
    tmdbNetworksEnabled = value.tmdbNetworksEnabled,
)

fun UserProfile.withNotificationSettings(value: SettingsNotificationsUiModel): UserProfile = copy(
    notificationsEnabled = value.notificationsEnabled,
    alertNewEpisodes = value.alertNewEpisodes,
)

fun UserProfile.withAllSyncProvidersDisconnected(): UserProfile = copy(
    authKey = "",
    stremioUserId = null,
    stremioEmail = null,
    nuvioAccessToken = null,
    nuvioRefreshToken = null,
    nuvioTokenExpiresAt = null,
    nuvioUserId = null,
    nuvioEmail = null,
    nuvioProfileIndex = null,
    nuvioLastSyncAt = null,
    traktAccessToken = null,
    traktRefreshToken = null,
    traktTokenExpiresAt = null,
    traktUsername = null,
    traktLastSyncAt = null,
    traktLastSyncedItems = null,
    traktLastContinueWatchingCount = null,
    traktLastWatchlistCount = null,
    simklAccessToken = null,
    simklUsername = null,
    simklLastSyncAt = null,
    anilistAccessToken = null,
    anilistRefreshToken = null,
    anilistTokenExpiresAt = null,
    anilistUsername = null,
    continueWatchingSource = "local",
    integrationLibrarySource = "local",
    externalSyncFailedProviders = emptySet(),
    providerSyncTimestamps = emptyMap(),
)

fun UserProfile.withProviderDisconnected(provider: String): UserProfile {
    val providerKey = provider.trim().lowercase()
    val disconnected = when (providerKey) {
        "stremio" -> copy(
            authKey = "",
            stremioUserId = null,
            stremioEmail = null,
        )
        "nuvio" -> copy(
            nuvioAccessToken = null,
            nuvioRefreshToken = null,
            nuvioTokenExpiresAt = null,
            nuvioUserId = null,
            nuvioEmail = null,
            nuvioProfileIndex = null,
            nuvioLastSyncAt = null,
        )
        "trakt" -> copy(
            traktAccessToken = null,
            traktRefreshToken = null,
            traktTokenExpiresAt = null,
            traktUsername = null,
            traktLastSyncAt = null,
            traktLastSyncedItems = null,
            traktLastContinueWatchingCount = null,
            traktLastWatchlistCount = null,
        )
        "simkl" -> copy(
            simklAccessToken = null,
            simklUsername = null,
            simklLastSyncAt = null,
        )
        "anilist" -> copy(
            anilistAccessToken = null,
            anilistRefreshToken = null,
            anilistTokenExpiresAt = null,
            anilistUsername = null,
        )
        else -> this
    }
    return disconnected.copy(
        continueWatchingSource = if (continueWatchingSource?.trim()?.lowercase() == providerKey) {
            "local"
        } else {
            disconnected.continueWatchingSource
        },
        integrationLibrarySource = if (integrationLibrarySource.equals(providerKey, ignoreCase = true)) "local" else disconnected.integrationLibrarySource,
        externalSyncFailedProviders = disconnected.externalSyncFailedProviders.orEmpty() - providerKey,
        providerSyncTimestamps = disconnected.providerSyncTimestamps.orEmpty() - providerKey,
    )
}
