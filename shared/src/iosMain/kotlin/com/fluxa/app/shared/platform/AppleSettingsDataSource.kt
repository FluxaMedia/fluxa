package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.settings.SettingsAccountUiModel
import com.fluxa.app.shared.feature.settings.SettingsAddonsUiModel
import com.fluxa.app.shared.feature.settings.SettingsAdvancedUiModel
import com.fluxa.app.shared.feature.settings.SettingsAppearanceDetailUiModel
import com.fluxa.app.shared.feature.settings.SettingsAppearanceHomeUiModel
import com.fluxa.app.shared.feature.settings.SettingsAppearanceUiModel
import com.fluxa.app.shared.feature.settings.SettingsContentUiModel
import com.fluxa.app.shared.feature.settings.SettingsDataSource
import com.fluxa.app.shared.feature.settings.SettingsDownloadsUiModel
import com.fluxa.app.shared.feature.settings.SettingsGeneralUiModel
import com.fluxa.app.shared.feature.settings.SettingsNotificationsUiModel
import com.fluxa.app.shared.feature.settings.SettingsPlaybackUiModel
import com.fluxa.app.shared.feature.settings.SettingsSubtitlesUiModel
import com.fluxa.app.shared.feature.settings.SettingsSystemUiModel
import com.fluxa.app.shared.feature.settings.SettingsUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import platform.Foundation.NSBundle
import platform.Foundation.NSUserDefaults

class AppleSettingsDataSource : SettingsDataSource {
    private val defaults = NSUserDefaults.standardUserDefaults
    private val appVersionLabel: String =
        (NSBundle.mainBundle.infoDictionary?.get("CFBundleShortVersionString") as? String).orEmpty()
    private val state = MutableStateFlow(readState())

    override fun observeSettings(): Flow<SettingsUiState> = state.asStateFlow()

    override suspend fun refreshContentFeeds() = Unit

    override suspend fun updateGeneral(value: SettingsGeneralUiModel) {
        defaults.setObject(value.language, K.language)
        defaults.setObject(value.startPage, K.startPage)
        defaults.setBool(value.backgroundPlayback, K.backgroundPlayback)
        state.value = state.value.copy(general = value)
    }

    override suspend fun updateAppearance(value: SettingsAppearanceUiModel) {
        defaults.setDouble(value.accentColorArgb.toDouble(), K.accentColorArgb)
        defaults.setBool(value.amoledMode, K.amoledMode)
        defaults.setBool(value.animationsEnabled, K.animationsEnabled)
        defaults.setBool(value.floatingBottomBar, K.floatingBottomBar)
        defaults.setBool(value.bottomBarLabels, K.bottomBarLabels)
        state.value = state.value.copy(appearance = value)
    }

    override suspend fun updateAppearanceHome(value: SettingsAppearanceHomeUiModel) {
        defaults.setBool(value.topBarEnabled, K.homeTopBarEnabled)
        defaults.setObject(value.cardCornerPreset, K.cardCornerPreset)
        defaults.setObject(value.interfaceDensity, K.interfaceDensity)
        defaults.setObject(value.posterWidthPreset, K.posterWidthPreset)
        defaults.setBool(value.posterLandscapeMode, K.posterLandscapeMode)
        defaults.setBool(value.posterHideTitles, K.posterHideTitles)
        defaults.setBool(value.homeSeasonPostersOnHero, K.homeSeasonPostersOnHero)
        defaults.setBool(value.trailerOnHomeHeroEnabled, K.trailerOnHomeHeroEnabled)
        defaults.setInteger(value.trailerOnHomeHeroDelaySeconds.toLong(), K.trailerOnHomeHeroDelaySeconds)
        defaults.setBool(value.continueWatchingHorizontal, K.continueWatchingHorizontal)
        defaults.setBool(value.continueWatchingEnabled, K.continueWatchingEnabled)
        defaults.setBool(value.continueWatchingHideTitles, K.continueWatchingHideTitles)
        state.value = state.value.copy(appearanceHome = value)
    }

    override suspend fun updateAppearanceDetail(value: SettingsAppearanceDetailUiModel) {
        defaults.setBool(value.trailerOnHero, K.trailerOnHero)
        defaults.setBool(value.blurUnwatchedEpisodes, K.blurUnwatchedEpisodes)
        defaults.setObject(value.detailSeasonSelectorMode, K.detailSeasonSelectorMode)
        defaults.setBool(value.detailSeasonPostersOnHero, K.detailSeasonPostersOnHero)
        defaults.setObject(value.episodeCardsLayout, K.episodeCardsLayout)
        state.value = state.value.copy(appearanceDetail = value)
    }

    override suspend fun updatePlayback(value: SettingsPlaybackUiModel) {
        defaults.setObject(value.preferredPlayer, K.preferredPlayer)
        defaults.setObject(value.mpvCustomOptions, K.mpvCustomOptions)
        defaults.setBool(value.animeUseMpv, K.animeUseMpv)
        defaults.setBool(value.animePreferJapaneseAudio, K.animePreferJapaneseAudio)
        defaults.setDouble(value.playbackSpeed.toDouble(), K.playbackSpeed)
        defaults.setInteger(value.seekForwardSeconds.toLong(), K.seekForwardSeconds)
        defaults.setInteger(value.seekBackwardSeconds.toLong(), K.seekBackwardSeconds)
        defaults.setBool(value.holdToSpeedEnabled, K.holdToSpeedEnabled)
        defaults.setDouble(value.holdSpeed.toDouble(), K.holdSpeed)
        defaults.setObject(value.streamSourceSelectionMode, K.streamSourceSelectionMode)
        defaults.setObject(value.streamSourceRegexPattern, K.streamSourceRegexPattern)
        defaults.setObject(value.autoplayMode, K.autoplayMode)
        defaults.setBool(value.autoPlayNextEpisode, K.autoPlayNextEpisode)
        defaults.setInteger(value.autoPlayCountdownSecs.toLong(), K.autoPlayCountdownSecs)
        defaults.setBool(value.autoRetryNextSource, K.autoRetryNextSource)
        defaults.setBool(value.tryBingeGroup, K.tryBingeGroup)
        defaults.setDouble(value.nextEpisodeThresholdPercent.toDouble(), K.nextEpisodeThresholdPercent)
        defaults.setDouble(value.watchedThresholdPercent.toDouble(), K.watchedThresholdPercent)
        defaults.setBool(value.useIntroDb, K.useIntroDb)
        defaults.setObject(value.introDbApiKey, K.introDbApiKey)
        defaults.setBool(value.useAniSkip, K.useAniSkip)
        defaults.setBool(value.useChapterSkip, K.useChapterSkip)
        defaults.setBool(value.autoSkipIntro, K.autoSkipIntro)
        state.value = state.value.copy(playback = value)
    }

    override suspend fun updateSubtitles(value: SettingsSubtitlesUiModel) {
        defaults.setObject(value.preferredAudioLanguage, K.preferredAudioLanguage)
        defaults.setObject(value.secondaryAudioLanguage, K.secondaryAudioLanguage)
        defaults.setObject(value.preferredSubtitleLanguage, K.preferredSubtitleLanguage)
        defaults.setObject(value.secondarySubtitleLanguage, K.secondarySubtitleLanguage)
        defaults.setBool(value.autoEnableSubtitles, K.autoEnableSubtitles)
        defaults.setBool(value.subtitleShadow, K.subtitleShadow)
        defaults.setDouble(value.subtitleSize.toDouble(), K.subtitleSize)
        defaults.setDouble(value.subtitleColorArgb.toDouble(), K.subtitleColorArgb)
        defaults.setDouble(value.subtitleTextOpacity.toDouble(), K.subtitleTextOpacity)
        defaults.setDouble(value.subtitleBackgroundColorArgb.toDouble(), K.subtitleBackgroundColorArgb)
        defaults.setDouble(value.subtitleBackgroundOpacity.toDouble(), K.subtitleBackgroundOpacity)
        defaults.setDouble(value.subtitleOutlineColorArgb.toDouble(), K.subtitleOutlineColorArgb)
        defaults.setDouble(value.subtitleOutlineOpacity.toDouble(), K.subtitleOutlineOpacity)
        state.value = state.value.copy(subtitles = value)
    }

    override suspend fun updateAdvanced(value: SettingsAdvancedUiModel) {
        defaults.setInteger(value.playerBufferCacheMb.toLong(), K.playerBufferCacheMb)
        defaults.setInteger(value.playerForwardBufferSeconds.toLong(), K.playerForwardBufferSeconds)
        defaults.setInteger(value.playerBackBufferSeconds.toLong(), K.playerBackBufferSeconds)
        defaults.setObject(value.audioDecoderMode, K.audioDecoderMode)
        defaults.setBool(value.tunneledPlayback, K.tunneledPlayback)
        state.value = state.value.copy(advanced = value)
    }

    override suspend fun updateAddons(value: SettingsAddonsUiModel) {
        defaults.setObject(value.torrentSpeedPreset, K.torrentSpeedPreset)
        defaults.setBool(value.torrentWifiOnly, K.torrentWifiOnly)
        state.value = state.value.copy(addons = value)
    }

    override suspend fun updateDownloads(value: SettingsDownloadsUiModel) {
        defaults.setObject(value.downloadSourceSelectionMode, K.downloadSourceSelectionMode)
        defaults.setObject(value.downloadSourceRegexPattern, K.downloadSourceRegexPattern)
        defaults.setObject(value.downloadSubtitleLanguage, K.downloadSubtitleLanguage)
        state.value = state.value.copy(downloads = state.value.downloads.copy(
            downloadSourceSelectionMode = value.downloadSourceSelectionMode,
            downloadSourceRegexPattern = value.downloadSourceRegexPattern,
            downloadSubtitleLanguage = value.downloadSubtitleLanguage
        ))
    }

    override suspend fun updateSystem(value: SettingsSystemUiModel) {
        defaults.setBool(value.automaticUpdates, K.automaticUpdates)
        state.value = state.value.copy(system = value.copy(appVersionLabel = appVersionLabel))
    }

    override suspend fun updateTmdbAccount(value: SettingsAccountUiModel) {
        defaults.setObject(value.tmdbApiKey, K.tmdbApiKey)
        defaults.setBool(value.tmdbCastImagesEnabled, K.tmdbCastImagesEnabled)
        defaults.setBool(value.tmdbSimilarResultsEnabled, K.tmdbSimilarResultsEnabled)
        defaults.setBool(value.tmdbTrailersEnabled, K.tmdbTrailersEnabled)
        defaults.setBool(value.tmdbRecommendationsEnabled, K.tmdbRecommendationsEnabled)
        defaults.setBool(value.tmdbCollectionInfoEnabled, K.tmdbCollectionInfoEnabled)
        defaults.setBool(value.tmdbEpisodeImagesEnabled, K.tmdbEpisodeImagesEnabled)
        defaults.setBool(value.tmdbLogosBackdropsEnabled, K.tmdbLogosBackdropsEnabled)
        defaults.setBool(value.tmdbRatingsEnabled, K.tmdbRatingsEnabled)
        defaults.setBool(value.tmdbBasicInfoEnabled, K.tmdbBasicInfoEnabled)
        defaults.setBool(value.tmdbDetailsEnabled, K.tmdbDetailsEnabled)
        defaults.setBool(value.tmdbProductionsEnabled, K.tmdbProductionsEnabled)
        defaults.setBool(value.tmdbNetworksEnabled, K.tmdbNetworksEnabled)
        state.value = state.value.copy(account = state.value.account.copy(
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
        ))
    }

    override suspend fun updateNotifications(value: SettingsNotificationsUiModel) {
        defaults.setBool(value.notificationsEnabled, K.notificationsEnabled)
        defaults.setBool(value.alertNewEpisodes, K.alertNewEpisodes)
        state.value = state.value.copy(notifications = value)
    }

    override suspend fun updateShowHeroSection(value: Boolean) {
        defaults.setBool(value, K.showHeroSection)
        state.value = state.value.copy(content = state.value.content.copy(showHeroSection = value))
    }

    override suspend fun toggleHeroFeed(key: String) = Unit
    override suspend fun moveHeroFeed(key: String, direction: Int) = Unit
    override suspend fun toggleHomeFeed(key: String) = Unit
    override suspend fun moveHomeFeed(key: String, direction: Int) = Unit
    override suspend fun toggleTopTenFeed(key: String) = Unit
    override suspend fun disconnectSync() = Unit

    private fun readState(): SettingsUiState = SettingsUiState(
        account = SettingsAccountUiModel(
            tmdbApiKey = defaults.stringForKey(K.tmdbApiKey),
            tmdbCastImagesEnabled = defaults.boolOrDefault(K.tmdbCastImagesEnabled, true),
            tmdbSimilarResultsEnabled = defaults.boolOrDefault(K.tmdbSimilarResultsEnabled, true),
            tmdbTrailersEnabled = defaults.boolOrDefault(K.tmdbTrailersEnabled, true),
            tmdbRecommendationsEnabled = defaults.boolOrDefault(K.tmdbRecommendationsEnabled, true),
            tmdbCollectionInfoEnabled = defaults.boolOrDefault(K.tmdbCollectionInfoEnabled, true),
            tmdbEpisodeImagesEnabled = defaults.boolOrDefault(K.tmdbEpisodeImagesEnabled, true),
            tmdbLogosBackdropsEnabled = defaults.boolOrDefault(K.tmdbLogosBackdropsEnabled, true),
            tmdbRatingsEnabled = defaults.boolOrDefault(K.tmdbRatingsEnabled, true),
            tmdbBasicInfoEnabled = defaults.boolOrDefault(K.tmdbBasicInfoEnabled, true),
            tmdbDetailsEnabled = defaults.boolOrDefault(K.tmdbDetailsEnabled, true),
            tmdbProductionsEnabled = defaults.boolOrDefault(K.tmdbProductionsEnabled, true),
            tmdbNetworksEnabled = defaults.boolOrDefault(K.tmdbNetworksEnabled, true)
        ),
        notifications = SettingsNotificationsUiModel(
            notificationsEnabled = defaults.boolOrDefault(K.notificationsEnabled, true),
            alertNewEpisodes = defaults.boolOrDefault(K.alertNewEpisodes, true)
        ),
        general = SettingsGeneralUiModel(
            language = defaults.stringForKey(K.language) ?: "en",
            startPage = defaults.stringForKey(K.startPage) ?: "home",
            backgroundPlayback = defaults.boolOrDefault(K.backgroundPlayback, true)
        ),
        appearance = SettingsAppearanceUiModel(
            accentColorArgb = defaults.objectForKey(K.accentColorArgb)?.let { defaults.doubleForKey(K.accentColorArgb).toLong() } ?: 0xFFFFFFFFL,
            amoledMode = defaults.boolOrDefault(K.amoledMode, false),
            animationsEnabled = defaults.boolOrDefault(K.animationsEnabled, true),
            floatingBottomBar = defaults.boolOrDefault(K.floatingBottomBar, false),
            bottomBarLabels = defaults.boolOrDefault(K.bottomBarLabels, false)
        ),
        appearanceHome = SettingsAppearanceHomeUiModel(
            topBarEnabled = defaults.boolOrDefault(K.homeTopBarEnabled, true),
            cardCornerPreset = defaults.stringForKey(K.cardCornerPreset) ?: "medium",
            interfaceDensity = defaults.stringForKey(K.interfaceDensity) ?: "normal",
            posterWidthPreset = defaults.stringForKey(K.posterWidthPreset) ?: "medium",
            posterLandscapeMode = defaults.boolOrDefault(K.posterLandscapeMode, false),
            posterHideTitles = defaults.boolOrDefault(K.posterHideTitles, false),
            homeSeasonPostersOnHero = defaults.boolOrDefault(K.homeSeasonPostersOnHero, false),
            trailerOnHomeHeroEnabled = defaults.boolOrDefault(K.trailerOnHomeHeroEnabled, false),
            trailerOnHomeHeroDelaySeconds = defaults.objectForKey(K.trailerOnHomeHeroDelaySeconds)?.let { defaults.integerForKey(K.trailerOnHomeHeroDelaySeconds).toInt() } ?: 3,
            continueWatchingHorizontal = defaults.boolOrDefault(K.continueWatchingHorizontal, true),
            continueWatchingEnabled = defaults.boolOrDefault(K.continueWatchingEnabled, true),
            continueWatchingHideTitles = defaults.boolOrDefault(K.continueWatchingHideTitles, false)
        ),
        appearanceDetail = SettingsAppearanceDetailUiModel(
            trailerOnHero = defaults.boolOrDefault(K.trailerOnHero, false),
            blurUnwatchedEpisodes = defaults.boolOrDefault(K.blurUnwatchedEpisodes, false),
            detailSeasonSelectorMode = defaults.stringForKey(K.detailSeasonSelectorMode) ?: "dropdown",
            detailSeasonPostersOnHero = defaults.boolOrDefault(K.detailSeasonPostersOnHero, false),
            episodeCardsLayout = defaults.stringForKey(K.episodeCardsLayout) ?: "modern"
        ),
        playback = SettingsPlaybackUiModel(
            preferredPlayer = defaults.stringForKey(K.preferredPlayer) ?: "internal",
            mpvCustomOptions = defaults.stringForKey(K.mpvCustomOptions) ?: "",
            animeUseMpv = defaults.boolOrDefault(K.animeUseMpv, false),
            animePreferJapaneseAudio = defaults.boolOrDefault(K.animePreferJapaneseAudio, false),
            playbackSpeed = defaults.objectForKey(K.playbackSpeed)?.let { defaults.doubleForKey(K.playbackSpeed).toFloat() } ?: 1f,
            seekForwardSeconds = defaults.objectForKey(K.seekForwardSeconds)?.let { defaults.integerForKey(K.seekForwardSeconds).toInt() } ?: 10,
            seekBackwardSeconds = defaults.objectForKey(K.seekBackwardSeconds)?.let { defaults.integerForKey(K.seekBackwardSeconds).toInt() } ?: 10,
            holdToSpeedEnabled = defaults.boolOrDefault(K.holdToSpeedEnabled, true),
            holdSpeed = defaults.objectForKey(K.holdSpeed)?.let { defaults.doubleForKey(K.holdSpeed).toFloat() } ?: 2f,
            streamSourceSelectionMode = defaults.stringForKey(K.streamSourceSelectionMode) ?: "auto",
            streamSourceRegexPattern = defaults.stringForKey(K.streamSourceRegexPattern) ?: "",
            autoplayMode = defaults.stringForKey(K.autoplayMode) ?: "off",
            autoPlayNextEpisode = defaults.boolOrDefault(K.autoPlayNextEpisode, true),
            autoPlayCountdownSecs = defaults.objectForKey(K.autoPlayCountdownSecs)?.let { defaults.integerForKey(K.autoPlayCountdownSecs).toInt() } ?: 10,
            autoRetryNextSource = defaults.boolOrDefault(K.autoRetryNextSource, false),
            tryBingeGroup = defaults.boolOrDefault(K.tryBingeGroup, true),
            nextEpisodeThresholdPercent = defaults.objectForKey(K.nextEpisodeThresholdPercent)?.let { defaults.doubleForKey(K.nextEpisodeThresholdPercent).toFloat() } ?: 90f,
            watchedThresholdPercent = defaults.objectForKey(K.watchedThresholdPercent)?.let { defaults.doubleForKey(K.watchedThresholdPercent).toFloat() } ?: 90f,
            useIntroDb = defaults.boolOrDefault(K.useIntroDb, false),
            introDbApiKey = defaults.stringForKey(K.introDbApiKey) ?: "",
            useAniSkip = defaults.boolOrDefault(K.useAniSkip, false),
            useChapterSkip = defaults.boolOrDefault(K.useChapterSkip, false),
            autoSkipIntro = defaults.boolOrDefault(K.autoSkipIntro, false)
        ),
        subtitles = SettingsSubtitlesUiModel(
            preferredAudioLanguage = defaults.stringForKey(K.preferredAudioLanguage) ?: "none",
            secondaryAudioLanguage = defaults.stringForKey(K.secondaryAudioLanguage) ?: "none",
            preferredSubtitleLanguage = defaults.stringForKey(K.preferredSubtitleLanguage) ?: "none",
            secondarySubtitleLanguage = defaults.stringForKey(K.secondarySubtitleLanguage) ?: "none",
            autoEnableSubtitles = defaults.boolOrDefault(K.autoEnableSubtitles, false),
            subtitleShadow = defaults.boolOrDefault(K.subtitleShadow, true),
            subtitleSize = defaults.objectForKey(K.subtitleSize)?.let { defaults.doubleForKey(K.subtitleSize).toFloat() } ?: 100f,
            subtitleColorArgb = defaults.objectForKey(K.subtitleColorArgb)?.let { defaults.doubleForKey(K.subtitleColorArgb).toLong() } ?: 0xFFFFFFFFL,
            subtitleTextOpacity = defaults.objectForKey(K.subtitleTextOpacity)?.let { defaults.doubleForKey(K.subtitleTextOpacity).toFloat() } ?: 1f,
            subtitleBackgroundColorArgb = defaults.objectForKey(K.subtitleBackgroundColorArgb)?.let { defaults.doubleForKey(K.subtitleBackgroundColorArgb).toLong() } ?: 0x80000000L,
            subtitleBackgroundOpacity = defaults.objectForKey(K.subtitleBackgroundOpacity)?.let { defaults.doubleForKey(K.subtitleBackgroundOpacity).toFloat() } ?: 0.5f,
            subtitleOutlineColorArgb = defaults.objectForKey(K.subtitleOutlineColorArgb)?.let { defaults.doubleForKey(K.subtitleOutlineColorArgb).toLong() } ?: 0xFF000000L,
            subtitleOutlineOpacity = defaults.objectForKey(K.subtitleOutlineOpacity)?.let { defaults.doubleForKey(K.subtitleOutlineOpacity).toFloat() } ?: 1f
        ),
        advanced = SettingsAdvancedUiModel(
            playerBufferCacheMb = defaults.objectForKey(K.playerBufferCacheMb)?.let { defaults.integerForKey(K.playerBufferCacheMb).toInt() } ?: 100,
            playerForwardBufferSeconds = defaults.objectForKey(K.playerForwardBufferSeconds)?.let { defaults.integerForKey(K.playerForwardBufferSeconds).toInt() } ?: 30,
            playerBackBufferSeconds = defaults.objectForKey(K.playerBackBufferSeconds)?.let { defaults.integerForKey(K.playerBackBufferSeconds).toInt() } ?: 30,
            audioDecoderMode = defaults.stringForKey(K.audioDecoderMode) ?: "hw_prefer",
            tunneledPlayback = defaults.boolOrDefault(K.tunneledPlayback, false)
        ),
        content = com.fluxa.app.shared.feature.settings.SettingsContentUiModel(
            showHeroSection = defaults.boolOrDefault(K.showHeroSection, true)
        ),
        addons = SettingsAddonsUiModel(
            torrentSpeedPreset = defaults.stringForKey(K.torrentSpeedPreset) ?: "balanced",
            torrentWifiOnly = defaults.boolOrDefault(K.torrentWifiOnly, false)
        ),
        downloads = SettingsDownloadsUiModel(
            downloadSourceSelectionMode = defaults.stringForKey(K.downloadSourceSelectionMode) ?: "auto",
            downloadSourceRegexPattern = defaults.stringForKey(K.downloadSourceRegexPattern) ?: "",
            downloadSubtitleLanguage = defaults.stringForKey(K.downloadSubtitleLanguage) ?: "none"
        ),
        system = SettingsSystemUiModel(
            automaticUpdates = defaults.boolOrDefault(K.automaticUpdates, true),
            appVersionLabel = appVersionLabel
        )
    )

    private fun NSUserDefaults.boolOrDefault(key: String, default: Boolean): Boolean =
        objectForKey(key)?.let { boolForKey(key) } ?: default

    private object K {
        const val language = "fluxa.apple.language"
        const val startPage = "fluxa.apple.settings.startPage"
        const val backgroundPlayback = "fluxa.apple.settings.backgroundPlayback"
        const val accentColorArgb = "fluxa.apple.settings.accentColorArgb"
        const val amoledMode = "fluxa.apple.settings.amoledMode"
        const val animationsEnabled = "fluxa.apple.settings.animationsEnabled"
        const val homeTopBarEnabled = "fluxa.apple.settings.homeTopBarEnabled"
        const val floatingBottomBar = "fluxa.apple.settings.floatingBottomBar"
        const val bottomBarLabels = "fluxa.apple.settings.bottomBarLabels"
        const val cardCornerPreset = "fluxa.apple.settings.cardCornerPreset"
        const val interfaceDensity = "fluxa.apple.settings.interfaceDensity"
        const val posterWidthPreset = "fluxa.apple.settings.posterWidthPreset"
        const val posterLandscapeMode = "fluxa.apple.settings.posterLandscapeMode"
        const val posterHideTitles = "fluxa.apple.settings.posterHideTitles"
        const val homeSeasonPostersOnHero = "fluxa.apple.settings.homeSeasonPostersOnHero"
        const val trailerOnHomeHeroEnabled = "fluxa.apple.settings.trailerOnHomeHeroEnabled"
        const val trailerOnHomeHeroDelaySeconds = "fluxa.apple.settings.trailerOnHomeHeroDelaySeconds"
        const val continueWatchingHorizontal = "fluxa.apple.settings.continueWatchingHorizontal"
        const val continueWatchingEnabled = "fluxa.apple.settings.continueWatchingEnabled"
        const val continueWatchingHideTitles = "fluxa.apple.settings.continueWatchingHideTitles"
        const val trailerOnHero = "fluxa.apple.settings.trailerOnHero"
        const val blurUnwatchedEpisodes = "fluxa.apple.settings.blurUnwatchedEpisodes"
        const val detailSeasonSelectorMode = "fluxa.apple.settings.detailSeasonSelectorMode"
        const val detailSeasonPostersOnHero = "fluxa.apple.settings.detailSeasonPostersOnHero"
        const val episodeCardsLayout = "fluxa.apple.settings.episodeCardsLayout"
        const val preferredPlayer = "fluxa.apple.settings.preferredPlayer"
        const val mpvCustomOptions = "fluxa.apple.settings.mpvCustomOptions"
        const val animeUseMpv = "fluxa.apple.settings.animeUseMpv"
        const val animePreferJapaneseAudio = "fluxa.apple.settings.animePreferJapaneseAudio"
        const val playbackSpeed = "fluxa.apple.settings.playbackSpeed"
        const val seekForwardSeconds = "fluxa.apple.settings.seekForwardSeconds"
        const val seekBackwardSeconds = "fluxa.apple.settings.seekBackwardSeconds"
        const val holdToSpeedEnabled = "fluxa.apple.settings.holdToSpeedEnabled"
        const val holdSpeed = "fluxa.apple.settings.holdSpeed"
        const val streamSourceSelectionMode = "fluxa.apple.settings.streamSourceSelectionMode"
        const val streamSourceRegexPattern = "fluxa.apple.settings.streamSourceRegexPattern"
        const val autoplayMode = "fluxa.apple.settings.autoplayMode"
        const val autoPlayNextEpisode = "fluxa.apple.settings.autoPlayNextEpisode"
        const val autoPlayCountdownSecs = "fluxa.apple.settings.autoPlayCountdownSecs"
        const val autoRetryNextSource = "fluxa.apple.settings.autoRetryNextSource"
        const val tryBingeGroup = "fluxa.apple.settings.tryBingeGroup"
        const val nextEpisodeThresholdPercent = "fluxa.apple.settings.nextEpisodeThresholdPercent"
        const val watchedThresholdPercent = "fluxa.apple.settings.watchedThresholdPercent"
        const val useIntroDb = "fluxa.apple.settings.useIntroDb"
        const val introDbApiKey = "fluxa.apple.settings.introDbApiKey"
        const val useAniSkip = "fluxa.apple.settings.useAniSkip"
        const val useChapterSkip = "fluxa.apple.settings.useChapterSkip"
        const val autoSkipIntro = "fluxa.apple.settings.autoSkipIntro"
        const val preferredAudioLanguage = "fluxa.apple.settings.preferredAudioLanguage"
        const val secondaryAudioLanguage = "fluxa.apple.settings.secondaryAudioLanguage"
        const val preferredSubtitleLanguage = "fluxa.apple.settings.preferredSubtitleLanguage"
        const val secondarySubtitleLanguage = "fluxa.apple.settings.secondarySubtitleLanguage"
        const val autoEnableSubtitles = "fluxa.apple.settings.autoEnableSubtitles"
        const val subtitleShadow = "fluxa.apple.settings.subtitleShadow"
        const val subtitleSize = "fluxa.apple.settings.subtitleSize"
        const val subtitleColorArgb = "fluxa.apple.settings.subtitleColorArgb"
        const val subtitleTextOpacity = "fluxa.apple.settings.subtitleTextOpacity"
        const val subtitleBackgroundColorArgb = "fluxa.apple.settings.subtitleBackgroundColorArgb"
        const val subtitleBackgroundOpacity = "fluxa.apple.settings.subtitleBackgroundOpacity"
        const val subtitleOutlineColorArgb = "fluxa.apple.settings.subtitleOutlineColorArgb"
        const val subtitleOutlineOpacity = "fluxa.apple.settings.subtitleOutlineOpacity"
        const val playerBufferCacheMb = "fluxa.apple.settings.playerBufferCacheMb"
        const val playerForwardBufferSeconds = "fluxa.apple.settings.playerForwardBufferSeconds"
        const val playerBackBufferSeconds = "fluxa.apple.settings.playerBackBufferSeconds"
        const val audioDecoderMode = "fluxa.apple.settings.audioDecoderMode"
        const val tunneledPlayback = "fluxa.apple.settings.tunneledPlayback"
        const val showHeroSection = "fluxa.apple.settings.showHeroSection"
        const val torrentSpeedPreset = "fluxa.apple.settings.torrentSpeedPreset"
        const val torrentWifiOnly = "fluxa.apple.settings.torrentWifiOnly"
        const val downloadSourceSelectionMode = "fluxa.apple.settings.downloadSourceSelectionMode"
        const val downloadSourceRegexPattern = "fluxa.apple.settings.downloadSourceRegexPattern"
        const val downloadSubtitleLanguage = "fluxa.apple.settings.downloadSubtitleLanguage"
        const val automaticUpdates = "fluxa.apple.settings.automaticUpdates"
        const val tmdbApiKey = "fluxa.apple.settings.tmdbApiKey"
        const val tmdbCastImagesEnabled = "fluxa.apple.settings.tmdbCastImagesEnabled"
        const val tmdbSimilarResultsEnabled = "fluxa.apple.settings.tmdbSimilarResultsEnabled"
        const val tmdbTrailersEnabled = "fluxa.apple.settings.tmdbTrailersEnabled"
        const val tmdbRecommendationsEnabled = "fluxa.apple.settings.tmdbRecommendationsEnabled"
        const val tmdbCollectionInfoEnabled = "fluxa.apple.settings.tmdbCollectionInfoEnabled"
        const val tmdbEpisodeImagesEnabled = "fluxa.apple.settings.tmdbEpisodeImagesEnabled"
        const val tmdbLogosBackdropsEnabled = "fluxa.apple.settings.tmdbLogosBackdropsEnabled"
        const val tmdbRatingsEnabled = "fluxa.apple.settings.tmdbRatingsEnabled"
        const val tmdbBasicInfoEnabled = "fluxa.apple.settings.tmdbBasicInfoEnabled"
        const val tmdbDetailsEnabled = "fluxa.apple.settings.tmdbDetailsEnabled"
        const val tmdbProductionsEnabled = "fluxa.apple.settings.tmdbProductionsEnabled"
        const val tmdbNetworksEnabled = "fluxa.apple.settings.tmdbNetworksEnabled"
        const val notificationsEnabled = "fluxa.apple.settings.notificationsEnabled"
        const val alertNewEpisodes = "fluxa.apple.settings.alertNewEpisodes"
    }
}
