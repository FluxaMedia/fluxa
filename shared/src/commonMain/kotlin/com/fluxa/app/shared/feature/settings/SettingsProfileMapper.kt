package com.fluxa.app.shared.feature.settings

import com.fluxa.app.data.local.*
import com.fluxa.app.domain.discovery.MetadataFeedOption
import com.fluxa.app.domain.discovery.effectiveHomeMetadataFeedSelection
import com.fluxa.app.domain.discovery.effectiveMetadataFeedSelection
import com.fluxa.app.domain.discovery.isMetadataFeedEnabled
import com.fluxa.app.domain.discovery.metadataFeedHomeTitle
import com.fluxa.app.domain.discovery.orderedMetadataFeeds

const val SETTINGS_HERO_FEED_LIMIT: Int = 2

data class SettingsAccountMetrics(
    val syncingProviders: Set<String> = emptySet(),
    val connectErrors: Map<String, String> = emptyMap(),
    val providerLastSyncAt: Map<String, Long> = emptyMap(),
    val continueWatchingCount: Int = 0,
    val stremioItemCount: Int = 0,
    val stremioContinueWatchingCount: Int = 0,
    val stremioLibraryCount: Int = 0,
    val stremioAddonCount: Int = 0,
    val nuvioItemCount: Int = 0,
    val nuvioContinueWatchingCount: Int = 0,
    val nuvioLibraryCount: Int = 0,
    val nuvioAddonCount: Int = 0,
    val traktItemCount: Int? = null,
    val traktContinueWatchingCount: Int? = null,
    val traktLibraryCount: Int? = null,
    val simklItemCount: Int = 0,
    val simklContinueWatchingCount: Int = 0,
    val simklLibraryCount: Int = 0,
    val anilistItemCount: Int = 0,
    val anilistContinueWatchingCount: Int = 0,
    val anilistLibraryCount: Int = 0,
    val addonCount: Int? = null,
)

data class SettingsDeveloperSnapshot(
    val lastProbeUpdatedAt: String? = null,
    val lastProbeTitle: String? = null,
    val lastProbeUrl: String? = null,
    val technicalInfo: String = "",
)

fun UserProfile.toSettingsAccountUiModel(
    metrics: SettingsAccountMetrics = SettingsAccountMetrics(),
): SettingsAccountUiModel = SettingsAccountUiModel(
    displayName = profileName?.takeIf { it.isNotBlank() } ?: email,
    email = email,
    stremioEmail = stremioEmail,
    nuvioEmail = nuvioEmail,
    avatarUrl = avatarUrl,
    hasStremio = authKey.isNotBlank(),
    hasNuvio = !nuvioAccessToken.isNullOrBlank(),
    hasTrakt = !traktAccessToken.isNullOrBlank(),
    hasSimkl = !simklAccessToken.isNullOrBlank(),
    hasAnilist = !anilistAccessToken.isNullOrBlank(),
    traktUsername = traktUsername,
    simklUsername = simklUsername,
    anilistUsername = anilistUsername,
    syncCwSourceOfTruth = syncCwSourceOfTruth
        .takeIf { it in setOf("", "local", "nuvio", "trakt", "simkl", "anilist", "stremio") }
        ?: "",
    syncCwRanking = syncCwRanking
        .takeIf { it in setOf("last_watched", "most_recent_episode") }
        ?: "last_watched",
    integrationLibrarySource = integrationLibrarySource
        .takeIf { it in setOf("local", "nuvio", "trakt", "simkl", "anilist", "stremio") }
        ?: "local",
    continueWatchingDays = (continueWatchingDays ?: 0).takeIf { it == 0 || it in 1..365 } ?: 0,
    traktCommentsEnabled = traktCommentsEnabled ?: false,
    hasAnySync = authKey.isNotBlank() ||
        !nuvioAccessToken.isNullOrBlank() ||
        !traktAccessToken.isNullOrBlank() ||
        !simklAccessToken.isNullOrBlank() ||
        !anilistAccessToken.isNullOrBlank(),
    syncFailedProviders = externalSyncFailedProviders.orEmpty(),
    syncingProviders = metrics.syncingProviders,
    connectErrors = metrics.connectErrors,
    stremioLastSyncAt = metrics.providerLastSyncAt[ThirdPartyProviderId.STREMIO.key]
        ?: providerLastSyncAt(ThirdPartyProviderId.STREMIO),
    nuvioLastSyncAt = metrics.providerLastSyncAt[ThirdPartyProviderId.NUVIO.key]
        ?: providerLastSyncAt(ThirdPartyProviderId.NUVIO),
    traktLastSyncAt = metrics.providerLastSyncAt[ThirdPartyProviderId.TRAKT.key]
        ?: providerLastSyncAt(ThirdPartyProviderId.TRAKT),
    simklLastSyncAt = metrics.providerLastSyncAt[ThirdPartyProviderId.SIMKL.key]
        ?: providerLastSyncAt(ThirdPartyProviderId.SIMKL),
    anilistLastSyncAt = metrics.providerLastSyncAt[ThirdPartyProviderId.ANILIST.key]
        ?: providerLastSyncAt(ThirdPartyProviderId.ANILIST),
    stremioItemCount = metrics.stremioItemCount,
    stremioContinueWatchingCount = metrics.stremioContinueWatchingCount,
    stremioLibraryCount = metrics.stremioLibraryCount,
    stremioAddonCount = metrics.stremioAddonCount,
    nuvioItemCount = metrics.nuvioItemCount,
    nuvioContinueWatchingCount = metrics.nuvioContinueWatchingCount,
    nuvioLibraryCount = metrics.nuvioLibraryCount,
    nuvioAddonCount = metrics.nuvioAddonCount,
    traktItemCount = metrics.traktItemCount ?: safeTraktLastSyncedItems,
    traktContinueWatchingCount = metrics.traktContinueWatchingCount ?: safeTraktLastContinueWatchingCount,
    continueWatchingCount = metrics.continueWatchingCount,
    traktLibraryCount = metrics.traktLibraryCount ?: safeTraktLastWatchlistCount,
    simklItemCount = metrics.simklItemCount,
    simklContinueWatchingCount = metrics.simklContinueWatchingCount,
    simklLibraryCount = metrics.simklLibraryCount,
    anilistItemCount = metrics.anilistItemCount,
    anilistContinueWatchingCount = metrics.anilistContinueWatchingCount,
    anilistLibraryCount = metrics.anilistLibraryCount,
    addonCount = metrics.addonCount ?: safeLocalAddons.size,
    tmdbApiKey = tmdbApiKey,
    mdblistApiKey = mdblistApiKey,
    tmdbCastImagesEnabled = safeTmdbCastImagesEnabled,
    tmdbSimilarResultsEnabled = safeTmdbSimilarResultsEnabled,
    tmdbTrailersEnabled = safeTmdbTrailersEnabled,
    tmdbRecommendationsEnabled = safeTmdbRecommendationsEnabled,
    tmdbCollectionInfoEnabled = safeTmdbCollectionInfoEnabled,
    tmdbEpisodeImagesEnabled = safeTmdbEpisodeImagesEnabled,
    tmdbLogosBackdropsEnabled = safeTmdbLogosBackdropsEnabled,
    tmdbRatingsEnabled = safeTmdbRatingsEnabled,
    tmdbBasicInfoEnabled = safeTmdbBasicInfoEnabled,
    tmdbDetailsEnabled = safeTmdbDetailsEnabled,
    tmdbProductionsEnabled = safeTmdbProductionsEnabled,
    tmdbNetworksEnabled = safeTmdbNetworksEnabled,
)

fun buildSettingsContentUiModel(
    profile: UserProfile?,
    metadataOptions: List<MetadataFeedOption>,
    base: SettingsContentUiModel = SettingsContentUiModel(),
): SettingsContentUiModel {
    val heroOptions = orderedMetadataFeeds(metadataOptions, profile?.heroFeedOrder)
    val heroSelection = effectiveHomeMetadataFeedSelection(
        profile?.heroFeedToggles,
        heroOptions.map { it.key },
    )
    val heroFeeds = heroOptions.mapIndexed { index, option ->
        option.toSettingsFeedItem(
            selected = isMetadataFeedEnabled(heroSelection, option.key),
            canMoveUp = index > 0,
            canMoveDown = index < heroOptions.lastIndex,
        )
    }

    val homeOptions = orderedMetadataFeeds(metadataOptions, profile?.homeFeedOrder)
    val homeSelection = effectiveHomeMetadataFeedSelection(
        profile?.homeFeedToggles,
        homeOptions.map { it.key },
    )
    val homeFeeds = homeOptions.mapIndexed { index, option ->
        option.toSettingsFeedItem(
            selected = isMetadataFeedEnabled(homeSelection, option.key),
            canMoveUp = index > 0,
            canMoveDown = index < homeOptions.lastIndex,
        )
    }

    val visibleHomeKeys = homeFeeds.filter { it.selected }.map { it.key }
    val topTenSelection = effectiveMetadataFeedSelection(profile?.topTenFeedToggles, visibleHomeKeys)
    val topTenFeeds = homeOptions
        .filter { it.key in visibleHomeKeys }
        .map { option ->
            option.toSettingsFeedItem(
                selected = isMetadataFeedEnabled(topTenSelection, option.key),
            )
        }

    return base.copy(
        showHeroSection = profile?.safeShowHeroSection ?: base.showHeroSection,
        heroFeeds = heroFeeds,
        homeFeeds = homeFeeds,
        topTenFeeds = topTenFeeds,
        heroSelectionLimit = SETTINGS_HERO_FEED_LIMIT,
    )
}

fun UserProfile.toSettingsAppearanceHomeUiModel(): SettingsAppearanceHomeUiModel = SettingsAppearanceHomeUiModel(
    cardCornerPreset = safeCardCornerPreset,
    interfaceDensity = safeInterfaceDensity,
    posterWidthPreset = safePosterWidthPreset,
    posterLandscapeMode = safePosterLandscapeMode,
    posterHideTitles = safePosterHideTitles,
    homeSeasonPostersOnHero = safeHomeSeasonPostersOnHero,
    trailerOnHomeHeroEnabled = safeTrailerOnHomeHeroEnabled,
    trailerOnHomeHeroDelaySeconds = safeTrailerOnHomeHeroDelaySeconds,
    continueWatchingHorizontal = safeContinueWatchingLayout != "vertical",
    continueWatchingEnabled = safeContinueWatchingEnabled,
    continueWatchingHideTitles = safeContinueWatchingHideTitles,
    continueWatchingSource = safeContinueWatchingSource,
    upcomingRowEnabled = safeUpcomingRowEnabled,
)

fun UserProfile.toSettingsUiState(
    metadataOptions: List<MetadataFeedOption>,
    appVersionLabel: String,
    accountMetrics: SettingsAccountMetrics = SettingsAccountMetrics(),
    developerSnapshot: SettingsDeveloperSnapshot = SettingsDeveloperSnapshot(),
): SettingsUiState = SettingsUiState(
    account = toSettingsAccountUiModel(accountMetrics),
    notifications = SettingsNotificationsUiModel(
        notificationsEnabled = safeNotificationsEnabled,
        alertNewEpisodes = safeAlertNewEpisodes,
    ),
    general = SettingsGeneralUiModel(
        language = safeLanguage,
        startPage = safeStartPage,
        backgroundPlayback = safeBackgroundPlayback,
    ),
    appearance = SettingsAppearanceUiModel(
        accentColorArgb = safeAccentColorArgb.toLong() and 0xffffffffL,
        amoledMode = safeAmoledMode,
        liquidGlassMode = safeLiquidGlassMode,
        animationsEnabled = safeAnimationsEnabled,
        floatingBottomBar = floatingBottomBar ?: false,
        bottomBarLabels = bottomBarLabels ?: false,
        topNavigationBar = topNavigationBar ?: false,
    ),
    appearanceHome = toSettingsAppearanceHomeUiModel(),
    appearanceDetail = SettingsAppearanceDetailUiModel(
        detailScreenStyle = safeDetailScreenStyle,
        detailPreferClearlogo = safeDetailPreferClearlogo,
        detailShowEpisodeDescriptions = safeDetailShowEpisodeDescriptions,
        detailShowCast = safeDetailShowCast,
        detailShowRecommendations = safeDetailShowRecommendations,
        detailCollapsingHero = safeDetailCollapsingHero,
        trailerOnDetailHeroEnabled = safeTrailerOnDetailHeroEnabled,
        trailerOnDetailHeroDelaySeconds = safeTrailerOnDetailHeroDelaySeconds,
        blurUnwatchedEpisodes = safeBlurUnwatchedEpisodes,
        detailSeasonSelectorMode = safeDetailSeasonSelectorMode,
        detailSeasonPostersOnHero = safeDetailSeasonPostersOnHero,
        episodeCardsLayout = safeEpisodeCardsLayout,
    ),
    playback = SettingsPlaybackUiModel(
        preferredPlayer = when (preferredPlayer?.trim()?.lowercase()) {
            "external" -> "external"
            "mpv" -> "mpv"
            else -> if (safePreferredPlayer == "mpv") "mpv" else "internal"
        },
        externalPlayerTarget = externalPlayerTarget.orEmpty(),
        mpvCustomOptions = safeMpvCustomOptions,
        animeUseMpv = safeAnimeUseMpv,
        animePreferJapaneseAudio = safeAnimePreferJapaneseAudio,
        playbackSpeed = safePlaybackSpeed,
        seekForwardSeconds = safeSeekForwardSeconds,
        seekBackwardSeconds = safeSeekBackwardSeconds,
        holdToSpeedEnabled = safeHoldToSpeedEnabled,
        holdSpeed = safeHoldSpeed,
        streamSourceSelectionMode = safeStreamSourceSelectionMode,
        streamSourceRegexPattern = streamSourceRegexPattern.orEmpty(),
        autoplayMode = safeAutoplayMode,
        autoPlayNextEpisode = safeAutoPlayNextEpisode,
        autoPlayCountdownSecs = safeAutoPlayCountdownSecs,
        autoRetryNextSource = safeAutoRetryNextSource,
        tryBingeGroup = safeTryBingeGroup,
        nextEpisodeThresholdPercent = safeNextEpisodeThresholdPercent,
        watchedThresholdPercent = safeWatchedThresholdPercent,
        useSkipSegments = safeUseIntroDb || safeUseAniSkip,
        introDbApiKey = introDbApiKey.orEmpty(),
        useChapterSkip = safeUseChapterSkip,
        autoSkipIntro = safeAutoSkipIntro,
        contentWarningsEnabled = safeContentWarningsEnabled,
    ),
    subtitles = SettingsSubtitlesUiModel(
        preferredAudioLanguage = safePreferredAudioLanguage,
        secondaryAudioLanguage = safeSecondaryAudioLanguage,
        preferredSubtitleLanguage = safePreferredSubtitleLanguage,
        secondarySubtitleLanguage = safeSecondarySubtitleLanguage,
        autoEnableSubtitles = safeAutoEnableSubtitles,
        subtitleShadow = safeSubtitleShadow,
        subtitleSize = safeSubtitleSize,
        subtitleColorArgb = safeSubtitleColor.toLong() and 0xffffffffL,
        subtitleTextOpacity = safeSubtitleTextOpacity,
        subtitleBackgroundColorArgb = safeSubtitleBackgroundColor.toLong() and 0xffffffffL,
        subtitleBackgroundOpacity = safeSubtitleBackgroundOpacity,
        subtitleOutlineColorArgb = safeSubtitleOutlineColor.toLong() and 0xffffffffL,
        subtitleOutlineOpacity = safeSubtitleOutlineOpacity,
    ),
    advanced = SettingsAdvancedUiModel(
        playerBufferCacheMb = safePlayerBufferCacheMb,
        playerForwardBufferSeconds = safePlayerForwardBufferSeconds,
        playerBackBufferSeconds = safePlayerBackBufferSeconds,
        audioDecoderMode = safeAudioDecoderMode,
        tunneledPlayback = safeTunneledPlayback,
    ),
    content = buildSettingsContentUiModel(this, metadataOptions),
    addons = SettingsAddonsUiModel(
        torrentSpeedPreset = safeTorrentSpeedPreset,
        torrentWifiOnly = safeTorrentWifiOnly,
    ),
    downloads = SettingsDownloadsUiModel(
        downloadSourceSelectionMode = safeDownloadSourceSelectionMode,
        downloadSourceRegexPattern = downloadSourceRegexPattern.orEmpty(),
        downloadSubtitleLanguage = safeDownloadSubtitleLanguage,
    ),
    system = SettingsSystemUiModel(
        automaticUpdates = safeAutomaticUpdates,
        appVersionLabel = appVersionLabel,
    ),
    developer = SettingsDeveloperUiModel(
        lastProbeUpdatedAt = developerSnapshot.lastProbeUpdatedAt,
        lastProbeTitle = developerSnapshot.lastProbeTitle,
        lastProbeUrl = developerSnapshot.lastProbeUrl,
        technicalInfo = developerSnapshot.technicalInfo,
    ),
    isLoading = false,
)

private fun MetadataFeedOption.toSettingsFeedItem(
    selected: Boolean,
    canMoveUp: Boolean = false,
    canMoveDown: Boolean = false,
): SettingsFeedItemUiModel = SettingsFeedItemUiModel(
    key = key,
    label = metadataFeedHomeTitle(label),
    providerLabel = label,
    selected = selected,
    canMoveUp = canMoveUp,
    canMoveDown = canMoveDown,
)
