package com.fluxa.app.ui.catalog

import android.content.SharedPreferences
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.domain.discovery.MetadataFeedOption
import com.fluxa.app.domain.discovery.buildMetadataFeedOptions
import com.fluxa.app.domain.discovery.effectiveHomeMetadataFeedSelection
import com.fluxa.app.domain.discovery.effectiveMetadataFeedSelection
import com.fluxa.app.domain.discovery.isMetadataFeedEnabled
import com.fluxa.app.domain.discovery.metadataFeedHomeTitle
import com.fluxa.app.domain.discovery.moveMetadataFeedOrder
import com.fluxa.app.domain.discovery.orderedMetadataFeeds
import com.fluxa.app.domain.discovery.toggleMetadataFeed
import com.fluxa.app.player.LastMediaDebugInfoStore
import com.fluxa.app.shared.feature.settings.SettingsAccountUiModel
import com.fluxa.app.shared.feature.settings.SettingsAddonsUiModel
import com.fluxa.app.shared.feature.settings.SettingsAdvancedUiModel
import com.fluxa.app.shared.feature.settings.SettingsAppearanceDetailUiModel
import com.fluxa.app.shared.feature.settings.SettingsAppearanceHomeUiModel
import com.fluxa.app.shared.feature.settings.SettingsAppearanceUiModel
import com.fluxa.app.shared.feature.settings.SettingsContentUiModel
import com.fluxa.app.shared.feature.settings.SettingsDataSource
import com.fluxa.app.shared.feature.settings.SettingsDeveloperUiModel
import com.fluxa.app.shared.feature.settings.SettingsDownloadsUiModel
import com.fluxa.app.shared.feature.settings.SettingsFeedItemUiModel
import com.fluxa.app.shared.feature.settings.SettingsGeneralUiModel
import com.fluxa.app.shared.feature.settings.SettingsNotificationsUiModel
import com.fluxa.app.shared.feature.settings.SettingsPlaybackUiModel
import com.fluxa.app.shared.feature.settings.SettingsSubtitlesUiModel
import com.fluxa.app.shared.feature.settings.SettingsSystemUiModel
import com.fluxa.app.shared.feature.settings.SettingsUiState
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.launch

private const val HERO_FEED_LIMIT = 2

class AndroidSettingsDataSource(
    private val homeViewModel: HomeViewModel,
    private val profileManager: ProfileManager,
    private val activeProfile: () -> UserProfile?,
    private val onProfileChanged: (UserProfile) -> Unit,
    private val appVersionLabel: String,
    private val language: () -> String
) : SettingsDataSource {

    private val profileState = MutableStateFlow(activeProfile())

    override fun observeSettings(): Flow<SettingsUiState> = combine(
        profileFlow(),
        homeViewModel.userAddons,
        homeViewModel.loadedCs3CatalogFeedOptions,
        LastMediaDebugInfoStore.state,
        homeViewModel.syncingProviders
    ) { profile, addons, cs3Options, debugInfo, syncingProviders ->
        if (profile == null) {
            SettingsUiState(isLoading = true)
        } else {
            buildState(profile, addons.let { buildMetadataFeedOptions(it, language()) } + cs3Options, debugInfo, syncingProviders)
        }
    }

    override suspend fun refreshContentFeeds() {
        homeViewModel.refreshInstalledAddons(forceRefresh = true)
    }

    private fun profileFlow(): Flow<UserProfile?> = callbackFlow {
        val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, _ -> profileState.value = activeProfile() }
        profileManager.registerOnChangeListener(listener)
        val forwardingJob = launch { profileState.collect { trySend(it) } }
        awaitClose {
            forwardingJob.cancel()
            profileManager.unregisterOnChangeListener(listener)
        }
    }

    private fun buildState(
        profile: UserProfile,
        metadataOptions: List<MetadataFeedOption>,
        debugInfo: com.fluxa.app.player.LastMediaDebugInfo,
        syncingProviders: Set<String> = emptySet()
    ): SettingsUiState {
        val heroOptions = orderedMetadataFeeds(metadataOptions, profile.heroFeedOrder)
        val heroSelection = effectiveHomeMetadataFeedSelection(profile.heroFeedToggles, heroOptions.map { it.key })
        val heroFeeds = heroOptions.mapIndexed { index, option ->
            SettingsFeedItemUiModel(
                key = option.key,
                label = metadataFeedHomeTitle(option.label),
                providerLabel = option.label,
                selected = isMetadataFeedEnabled(heroSelection, option.key),
                canMoveUp = index > 0,
                canMoveDown = index < heroOptions.lastIndex
            )
        }

        val homeOptions = orderedMetadataFeeds(metadataOptions, profile.homeFeedOrder)
        val homeSelection = effectiveHomeMetadataFeedSelection(profile.homeFeedToggles, homeOptions.map { it.key })
        val homeFeeds = homeOptions.mapIndexed { index, option ->
            SettingsFeedItemUiModel(
                key = option.key,
                label = metadataFeedHomeTitle(option.label),
                providerLabel = option.label,
                selected = isMetadataFeedEnabled(homeSelection, option.key),
                canMoveUp = index > 0,
                canMoveDown = index < homeOptions.lastIndex
            )
        }

        val visibleHomeKeys = homeFeeds.filter { it.selected }.map { it.key }
        val topTenSelection = effectiveMetadataFeedSelection(profile.topTenFeedToggles, visibleHomeKeys)
        val topTenFeeds = homeOptions.filter { it.key in visibleHomeKeys }.map { option ->
            SettingsFeedItemUiModel(
                key = option.key,
                label = metadataFeedHomeTitle(option.label),
                providerLabel = option.label,
                selected = isMetadataFeedEnabled(topTenSelection, option.key)
            )
        }

        return SettingsUiState(
            account = SettingsAccountUiModel(
                displayName = profile.profileName?.takeIf { it.isNotBlank() } ?: profile.email,
                email = profile.email,
                nuvioEmail = profile.nuvioEmail,
                avatarUrl = profile.avatarUrl,
                isGuest = profile.isGuest,
                hasStremio = profile.authKey.isNotBlank(),
                hasNuvio = !profile.nuvioAccessToken.isNullOrBlank(),
                hasTrakt = !profile.traktAccessToken.isNullOrBlank(),
                hasSimkl = !profile.simklAccessToken.isNullOrBlank(),
                hasAnilist = !profile.anilistAccessToken.isNullOrBlank(),
                hasAnySync = !profile.traktAccessToken.isNullOrBlank() ||
                    !profile.simklAccessToken.isNullOrBlank() ||
                    !profile.anilistAccessToken.isNullOrBlank(),
                syncFailedProviders = profile.externalSyncFailedProviders.orEmpty(),
                syncingProviders = syncingProviders,
                nuvioLastSyncAt = profile.nuvioLastSyncAt ?: 0L,
                traktLastSyncAt = profile.safeTraktLastSyncAt,
                simklLastSyncAt = profile.safeSimklLastSyncAt,
                traktItemCount = profile.safeTraktLastSyncedItems,
                traktContinueWatchingCount = profile.safeTraktLastContinueWatchingCount,
                continueWatchingCount = homeViewModel.currentContinueWatchingCount.value,
                traktLibraryCount = profile.safeTraktLastWatchlistCount,
                addonCount = profile.safeLocalAddons.size,
                tmdbApiKey = profile.tmdbApiKey,
                tmdbCastImagesEnabled = profile.safeTmdbCastImagesEnabled,
                tmdbSimilarResultsEnabled = profile.safeTmdbSimilarResultsEnabled,
                tmdbTrailersEnabled = profile.safeTmdbTrailersEnabled,
                tmdbRecommendationsEnabled = profile.safeTmdbRecommendationsEnabled,
                tmdbCollectionInfoEnabled = profile.safeTmdbCollectionInfoEnabled,
                tmdbEpisodeImagesEnabled = profile.safeTmdbEpisodeImagesEnabled,
                tmdbLogosBackdropsEnabled = profile.safeTmdbLogosBackdropsEnabled,
                tmdbRatingsEnabled = profile.safeTmdbRatingsEnabled,
                tmdbBasicInfoEnabled = profile.safeTmdbBasicInfoEnabled,
                tmdbDetailsEnabled = profile.safeTmdbDetailsEnabled,
                tmdbProductionsEnabled = profile.safeTmdbProductionsEnabled,
                tmdbNetworksEnabled = profile.safeTmdbNetworksEnabled
            ),
            notifications = SettingsNotificationsUiModel(
                notificationsEnabled = profile.safeNotificationsEnabled,
                alertNewEpisodes = profile.safeAlertNewEpisodes
            ),
            general = SettingsGeneralUiModel(
                language = profile.safeLanguage,
                startPage = profile.safeStartPage,
                backgroundPlayback = profile.safeBackgroundPlayback
            ),
            appearance = SettingsAppearanceUiModel(
                accentColorArgb = profile.safeAccentColorArgb.toLong() and 0xffffffffL,
                amoledMode = profile.safeAmoledMode,
                animationsEnabled = profile.safeAnimationsEnabled
            ),
            appearanceHome = SettingsAppearanceHomeUiModel(
                cardCornerPreset = profile.safeCardCornerPreset,
                interfaceDensity = profile.safeInterfaceDensity,
                posterWidthPreset = profile.safePosterWidthPreset,
                posterLandscapeMode = profile.safePosterLandscapeMode,
                posterHideTitles = profile.safePosterHideTitles,
                homeSeasonPostersOnHero = profile.safeHomeSeasonPostersOnHero,
                trailerOnHomeHeroEnabled = profile.safeTrailerOnHomeHeroEnabled,
                trailerOnHomeHeroDelaySeconds = profile.safeTrailerOnHomeHeroDelaySeconds,
                continueWatchingHorizontal = profile.safeContinueWatchingLayout != "vertical",
                continueWatchingEnabled = profile.safeContinueWatchingEnabled,
                continueWatchingHideTitles = profile.safeContinueWatchingHideTitles,
                continueWatchingSource = profile.safeContinueWatchingSource
            ),
            appearanceDetail = SettingsAppearanceDetailUiModel(
                trailerOnHero = profile.safeTrailerOnHero,
                blurUnwatchedEpisodes = profile.safeBlurUnwatchedEpisodes,
                detailSeasonSelectorMode = profile.safeDetailSeasonSelectorMode,
                detailSeasonPostersOnHero = profile.safeDetailSeasonPostersOnHero,
                episodeCardsLayout = profile.safeEpisodeCardsLayout
            ),
            playback = SettingsPlaybackUiModel(
                preferredPlayer = if (profile.safePreferredPlayer == "mpv") "mpv" else "internal",
                mpvCustomOptions = profile.safeMpvCustomOptions,
                animeUseMpv = profile.safeAnimeUseMpv,
                animePreferJapaneseAudio = profile.safeAnimePreferJapaneseAudio,
                playbackSpeed = profile.safePlaybackSpeed,
                seekForwardSeconds = profile.safeSeekForwardSeconds,
                seekBackwardSeconds = profile.safeSeekBackwardSeconds,
                holdToSpeedEnabled = profile.safeHoldToSpeedEnabled,
                holdSpeed = profile.safeHoldSpeed,
                streamSourceSelectionMode = profile.safeStreamSourceSelectionMode,
                streamSourceRegexPattern = profile.streamSourceRegexPattern.orEmpty(),
                autoplayMode = profile.safeAutoplayMode,
                autoPlayNextEpisode = profile.safeAutoPlayNextEpisode,
                autoPlayCountdownSecs = profile.safeAutoPlayCountdownSecs,
                autoRetryNextSource = profile.safeAutoRetryNextSource,
                tryBingeGroup = profile.safeTryBingeGroup,
                nextEpisodeThresholdPercent = profile.safeNextEpisodeThresholdPercent,
                watchedThresholdPercent = profile.safeWatchedThresholdPercent,
                useIntroDb = profile.safeUseIntroDb,
                introDbApiKey = profile.introDbApiKey.orEmpty(),
                useAniSkip = profile.safeUseAniSkip,
                useChapterSkip = profile.safeUseChapterSkip,
                autoSkipIntro = profile.safeAutoSkipIntro
            ),
            subtitles = SettingsSubtitlesUiModel(
                preferredAudioLanguage = profile.safePreferredAudioLanguage,
                secondaryAudioLanguage = profile.safeSecondaryAudioLanguage,
                preferredSubtitleLanguage = profile.safePreferredSubtitleLanguage,
                secondarySubtitleLanguage = profile.safeSecondarySubtitleLanguage,
                autoEnableSubtitles = profile.safeAutoEnableSubtitles,
                subtitleShadow = profile.safeSubtitleShadow,
                subtitleSize = profile.safeSubtitleSize,
                subtitleColorArgb = profile.safeSubtitleColor.toLong() and 0xffffffffL,
                subtitleTextOpacity = profile.safeSubtitleTextOpacity,
                subtitleBackgroundColorArgb = profile.safeSubtitleBackgroundColor.toLong() and 0xffffffffL,
                subtitleBackgroundOpacity = profile.safeSubtitleBackgroundOpacity,
                subtitleOutlineColorArgb = profile.safeSubtitleOutlineColor.toLong() and 0xffffffffL,
                subtitleOutlineOpacity = profile.safeSubtitleOutlineOpacity
            ),
            advanced = SettingsAdvancedUiModel(
                playerBufferCacheMb = profile.safePlayerBufferCacheMb,
                playerForwardBufferSeconds = profile.safePlayerForwardBufferSeconds,
                playerBackBufferSeconds = profile.safePlayerBackBufferSeconds,
                audioDecoderMode = profile.safeAudioDecoderMode,
                tunneledPlayback = profile.safeTunneledPlayback
            ),
            content = SettingsContentUiModel(
                showHeroSection = profile.safeShowHeroSection,
                heroFeeds = heroFeeds,
                homeFeeds = homeFeeds,
                topTenFeeds = topTenFeeds,
                heroSelectionLimit = HERO_FEED_LIMIT
            ),
            addons = SettingsAddonsUiModel(
                torrentSpeedPreset = profile.safeTorrentSpeedPreset,
                torrentWifiOnly = profile.safeTorrentWifiOnly
            ),
            downloads = SettingsDownloadsUiModel(
                downloadSourceSelectionMode = profile.safeDownloadSourceSelectionMode,
                downloadSourceRegexPattern = profile.downloadSourceRegexPattern.orEmpty(),
                downloadSubtitleLanguage = profile.safeDownloadSubtitleLanguage
            ),
            system = SettingsSystemUiModel(
                automaticUpdates = profile.safeAutomaticUpdates,
                appVersionLabel = appVersionLabel
            ),
            developer = SettingsDeveloperUiModel(
                lastProbeUpdatedAt = LastMediaDebugInfoStore.formattedUpdatedAt(debugInfo.updatedAtMs).takeIf { debugInfo.hasInfo },
                lastProbeTitle = debugInfo.title.takeIf { debugInfo.hasInfo },
                lastProbeUrl = debugInfo.url.takeIf { debugInfo.hasInfo },
                technicalInfo = debugInfo.technicalInfo
            ),
            isLoading = false
        )
    }

    private fun update(block: (UserProfile) -> UserProfile) {
        val profile = activeProfile() ?: return
        val updated = block(profile)
        profileManager.saveProfile(updated)
        profileState.value = updated
        onProfileChanged(updated)
    }

    override suspend fun updateGeneral(value: SettingsGeneralUiModel) = update {
        it.copy(language = value.language, startPage = value.startPage, backgroundPlayback = value.backgroundPlayback)
    }

    override suspend fun updateAppearance(value: SettingsAppearanceUiModel) = update {
        it.copy(
            accentColorArgb = value.accentColorArgb.toInt(),
            amoledMode = value.amoledMode,
            appTheme = if (value.amoledMode) "dark" else it.appTheme,
            animationsEnabled = value.animationsEnabled
        )
    }

    override suspend fun updateAppearanceHome(value: SettingsAppearanceHomeUiModel) = update {
        it.copy(
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
            continueWatchingSource = value.continueWatchingSource
        )
    }

    override suspend fun updateAppearanceDetail(value: SettingsAppearanceDetailUiModel) = update {
        it.copy(
            trailerOnHero = value.trailerOnHero,
            blurUnwatchedEpisodes = value.blurUnwatchedEpisodes,
            detailSeasonSelectorMode = value.detailSeasonSelectorMode,
            detailSeasonPostersOnHero = value.detailSeasonPostersOnHero,
            episodeCardsLayout = value.episodeCardsLayout
        )
    }

    override suspend fun updatePlayback(value: SettingsPlaybackUiModel) = update {
        it.copy(
            preferredPlayer = value.preferredPlayer,
            mpvCustomOptions = value.mpvCustomOptions.takeIf { opts -> opts.isNotBlank() },
            animeUseMpv = value.animeUseMpv,
            animePreferJapaneseAudio = value.animePreferJapaneseAudio,
            playbackSpeed = value.playbackSpeed,
            seekForwardSeconds = value.seekForwardSeconds,
            seekBackwardSeconds = value.seekBackwardSeconds,
            holdToSpeedEnabled = value.holdToSpeedEnabled,
            holdSpeed = value.holdSpeed,
            streamSourceSelectionMode = value.streamSourceSelectionMode,
            streamSourceRegexPattern = value.streamSourceRegexPattern.takeIf { pattern -> pattern.isNotBlank() },
            autoplayMode = value.autoplayMode,
            autoPlayNextEpisode = value.autoPlayNextEpisode,
            autoPlayCountdownSecs = value.autoPlayCountdownSecs,
            autoRetryNextSource = value.autoRetryNextSource,
            tryBingeGroup = value.tryBingeGroup,
            nextEpisodeThresholdPercent = value.nextEpisodeThresholdPercent,
            watchedThresholdPercent = value.watchedThresholdPercent,
            useIntroDb = value.useIntroDb,
            introDbApiKey = value.introDbApiKey.takeIf { key -> key.isNotBlank() },
            useAniSkip = value.useAniSkip,
            useChapterSkip = value.useChapterSkip,
            autoSkipIntro = value.autoSkipIntro
        )
    }

    override suspend fun updateSubtitles(value: SettingsSubtitlesUiModel) = update {
        it.copy(
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
            subtitleOutlineOpacity = value.subtitleOutlineOpacity
        )
    }

    override suspend fun updateAdvanced(value: SettingsAdvancedUiModel) = update {
        it.copy(
            playerBufferCacheMb = value.playerBufferCacheMb,
            playerForwardBufferSeconds = value.playerForwardBufferSeconds,
            playerBackBufferSeconds = value.playerBackBufferSeconds,
            audioDecoderMode = value.audioDecoderMode,
            tunneledPlayback = value.tunneledPlayback
        )
    }

    override suspend fun updateAddons(value: SettingsAddonsUiModel) = update {
        it.copy(torrentSpeedPreset = value.torrentSpeedPreset, torrentWifiOnly = value.torrentWifiOnly)
    }

    override suspend fun updateDownloads(value: SettingsDownloadsUiModel) = update {
        it.copy(
            downloadSourceSelectionMode = value.downloadSourceSelectionMode,
            downloadSourceRegexPattern = value.downloadSourceRegexPattern.takeIf { pattern -> pattern.isNotBlank() },
            downloadSubtitleLanguage = value.downloadSubtitleLanguage
        )
    }

    override suspend fun updateSystem(value: SettingsSystemUiModel) = update {
        it.copy(automaticUpdates = value.automaticUpdates)
    }

    override suspend fun updateTmdbAccount(value: SettingsAccountUiModel) = update {
        it.copy(
            tmdbApiKey = value.tmdbApiKey,
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
            tmdbNetworksEnabled = value.tmdbNetworksEnabled
        )
    }

    override suspend fun updateNotifications(value: SettingsNotificationsUiModel) = update {
        it.copy(notificationsEnabled = value.notificationsEnabled, alertNewEpisodes = value.alertNewEpisodes)
    }

    override suspend fun updateShowHeroSection(value: Boolean) = update {
        it.copy(showHeroSection = value)
    }

    private fun currentMetadataOptions(): List<MetadataFeedOption> =
        buildMetadataFeedOptions(homeViewModel.userAddons.value, language()) + homeViewModel.loadedCs3CatalogFeedOptions.value

    override suspend fun toggleHeroFeed(key: String) = update { profile ->
        val options = currentMetadataOptions()
        val ordered = orderedMetadataFeeds(options, profile.heroFeedOrder).map { it.key }
        profile.copy(heroFeedToggles = toggleMetadataFeed(profile.heroFeedToggles, ordered, key, HERO_FEED_LIMIT))
    }

    override suspend fun moveHeroFeed(key: String, direction: Int) = update { profile ->
        val options = currentMetadataOptions()
        profile.copy(heroFeedOrder = moveMetadataFeedOrder(options, profile.heroFeedOrder, key, direction))
    }

    override suspend fun toggleHomeFeed(key: String) = update { profile ->
        val options = currentMetadataOptions()
        val ordered = orderedMetadataFeeds(options, profile.homeFeedOrder).map { it.key }
        profile.copy(homeFeedToggles = toggleMetadataFeed(profile.homeFeedToggles, ordered, key))
    }

    override suspend fun moveHomeFeed(key: String, direction: Int) = update { profile ->
        val options = currentMetadataOptions()
        profile.copy(homeFeedOrder = moveMetadataFeedOrder(options, profile.homeFeedOrder, key, direction))
    }

    override suspend fun toggleTopTenFeed(key: String) = update { profile ->
        val options = currentMetadataOptions()
        val homeOrdered = orderedMetadataFeeds(options, profile.homeFeedOrder).map { it.key }
        val homeSelection = effectiveHomeMetadataFeedSelection(profile.homeFeedToggles, homeOrdered) ?: homeOrdered
        profile.copy(topTenFeedToggles = toggleMetadataFeed(profile.topTenFeedToggles, homeSelection, key))
    }

    override suspend fun disconnectSync() = update {
        it.copy(
            traktAccessToken = null,
            traktRefreshToken = null,
            simklAccessToken = null,
            anilistAccessToken = null,
            anilistRefreshToken = null,
            anilistTokenExpiresAt = null
        )
    }
}
