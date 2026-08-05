package com.fluxa.app.desktop.settings

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
import java.util.prefs.Preferences

class DesktopSettingsDataSource : SettingsDataSource {
    private val prefs: Preferences = Preferences.userRoot().node("com/fluxa/app/desktop/settings")
    private val state = MutableStateFlow(readState())

    override fun observeSettings(): Flow<SettingsUiState> = state.asStateFlow()

    override suspend fun refreshContentFeeds() = Unit

    override suspend fun updateGeneral(value: SettingsGeneralUiModel) {
        prefs.put(K.language, value.language)
        prefs.put(K.startPage, value.startPage)
        prefs.putBoolean(K.backgroundPlayback, value.backgroundPlayback)
        state.value = state.value.copy(general = value)
    }

    override suspend fun updateAppearance(value: SettingsAppearanceUiModel) {
        prefs.putLong(K.accentColorArgb, value.accentColorArgb)
        prefs.putBoolean(K.amoledMode, value.amoledMode)
        prefs.putBoolean(K.liquidGlassMode, value.liquidGlassMode)
        prefs.putBoolean(K.animationsEnabled, value.animationsEnabled)
        prefs.putBoolean(K.floatingBottomBar, value.floatingBottomBar)
        prefs.putBoolean(K.bottomBarLabels, value.bottomBarLabels)
        prefs.putBoolean(K.topNavigationBar, value.topNavigationBar)
        state.value = state.value.copy(appearance = value)
    }

    override suspend fun updateAppearanceHome(value: SettingsAppearanceHomeUiModel) {
        prefs.put(K.cardCornerPreset, value.cardCornerPreset)
        prefs.put(K.interfaceDensity, value.interfaceDensity)
        prefs.put(K.posterWidthPreset, value.posterWidthPreset)
        prefs.putBoolean(K.posterLandscapeMode, value.posterLandscapeMode)
        prefs.putBoolean(K.posterHideTitles, value.posterHideTitles)
        prefs.putBoolean(K.homeSeasonPostersOnHero, value.homeSeasonPostersOnHero)
        prefs.putBoolean(K.trailerOnHomeHeroEnabled, value.trailerOnHomeHeroEnabled)
        prefs.putInt(K.trailerOnHomeHeroDelaySeconds, value.trailerOnHomeHeroDelaySeconds)
        prefs.putBoolean(K.continueWatchingHorizontal, value.continueWatchingHorizontal)
        prefs.putBoolean(K.continueWatchingEnabled, value.continueWatchingEnabled)
        prefs.putBoolean(K.continueWatchingHideTitles, value.continueWatchingHideTitles)
        state.value = state.value.copy(appearanceHome = value)
    }

    override suspend fun updateAppearanceDetail(value: SettingsAppearanceDetailUiModel) {
        prefs.putBoolean(K.blurUnwatchedEpisodes, value.blurUnwatchedEpisodes)
        prefs.put(K.detailSeasonSelectorMode, value.detailSeasonSelectorMode)
        prefs.putBoolean(K.detailSeasonPostersOnHero, value.detailSeasonPostersOnHero)
        prefs.put(K.episodeCardsLayout, value.episodeCardsLayout)
        state.value = state.value.copy(appearanceDetail = value)
    }

    override suspend fun updatePlayback(value: SettingsPlaybackUiModel) {
        prefs.put(K.preferredPlayer, value.preferredPlayer)
        prefs.put(K.mpvCustomOptions, value.mpvCustomOptions)
        prefs.putBoolean(K.animeUseMpv, value.animeUseMpv)
        prefs.putBoolean(K.animePreferJapaneseAudio, value.animePreferJapaneseAudio)
        prefs.putFloat(K.playbackSpeed, value.playbackSpeed)
        prefs.putInt(K.seekForwardSeconds, value.seekForwardSeconds)
        prefs.putInt(K.seekBackwardSeconds, value.seekBackwardSeconds)
        prefs.putBoolean(K.holdToSpeedEnabled, value.holdToSpeedEnabled)
        prefs.putFloat(K.holdSpeed, value.holdSpeed)
        prefs.put(K.streamSourceSelectionMode, value.streamSourceSelectionMode)
        prefs.put(K.streamSourceRegexPattern, value.streamSourceRegexPattern)
        prefs.put(K.autoplayMode, value.autoplayMode)
        prefs.putBoolean(K.autoPlayNextEpisode, value.autoPlayNextEpisode)
        prefs.putInt(K.autoPlayCountdownSecs, value.autoPlayCountdownSecs)
        prefs.putBoolean(K.autoRetryNextSource, value.autoRetryNextSource)
        prefs.putBoolean(K.tryBingeGroup, value.tryBingeGroup)
        prefs.putFloat(K.nextEpisodeThresholdPercent, value.nextEpisodeThresholdPercent)
        prefs.putFloat(K.watchedThresholdPercent, value.watchedThresholdPercent)
        prefs.putBoolean(K.useSkipSegments, value.useSkipSegments)
        prefs.put(K.introDbApiKey, value.introDbApiKey)
        prefs.putBoolean(K.useChapterSkip, value.useChapterSkip)
        prefs.putBoolean(K.autoSkipIntro, value.autoSkipIntro)
        prefs.putBoolean(K.contentWarningsEnabled, value.contentWarningsEnabled)
        state.value = state.value.copy(playback = value)
    }

    override suspend fun updateSubtitles(value: SettingsSubtitlesUiModel) {
        prefs.put(K.preferredAudioLanguage, value.preferredAudioLanguage)
        prefs.put(K.secondaryAudioLanguage, value.secondaryAudioLanguage)
        prefs.put(K.preferredSubtitleLanguage, value.preferredSubtitleLanguage)
        prefs.put(K.secondarySubtitleLanguage, value.secondarySubtitleLanguage)
        prefs.putBoolean(K.autoEnableSubtitles, value.autoEnableSubtitles)
        prefs.putBoolean(K.subtitleShadow, value.subtitleShadow)
        prefs.putFloat(K.subtitleSize, value.subtitleSize)
        prefs.putLong(K.subtitleColorArgb, value.subtitleColorArgb)
        prefs.putFloat(K.subtitleTextOpacity, value.subtitleTextOpacity)
        prefs.putLong(K.subtitleBackgroundColorArgb, value.subtitleBackgroundColorArgb)
        prefs.putFloat(K.subtitleBackgroundOpacity, value.subtitleBackgroundOpacity)
        prefs.putLong(K.subtitleOutlineColorArgb, value.subtitleOutlineColorArgb)
        prefs.putFloat(K.subtitleOutlineOpacity, value.subtitleOutlineOpacity)
        state.value = state.value.copy(subtitles = value)
    }

    override suspend fun updateAdvanced(value: SettingsAdvancedUiModel) {
        prefs.putInt(K.playerBufferCacheMb, value.playerBufferCacheMb)
        prefs.putInt(K.playerForwardBufferSeconds, value.playerForwardBufferSeconds)
        prefs.putInt(K.playerBackBufferSeconds, value.playerBackBufferSeconds)
        prefs.put(K.audioDecoderMode, value.audioDecoderMode)
        prefs.putBoolean(K.tunneledPlayback, value.tunneledPlayback)
        state.value = state.value.copy(advanced = value)
    }

    override suspend fun updateAddons(value: SettingsAddonsUiModel) {
        prefs.put(K.torrentSpeedPreset, value.torrentSpeedPreset)
        prefs.putBoolean(K.torrentWifiOnly, value.torrentWifiOnly)
        state.value = state.value.copy(addons = value)
    }

    override suspend fun updateDownloads(value: SettingsDownloadsUiModel) {
        prefs.put(K.downloadSourceSelectionMode, value.downloadSourceSelectionMode)
        prefs.put(K.downloadSourceRegexPattern, value.downloadSourceRegexPattern)
        prefs.put(K.downloadSubtitleLanguage, value.downloadSubtitleLanguage)
        state.value = state.value.copy(downloads = value)
    }

    override suspend fun updateSystem(value: SettingsSystemUiModel) {
        prefs.putBoolean(K.automaticUpdates, value.automaticUpdates)
        state.value = state.value.copy(system = value.copy(appVersionLabel = APP_VERSION_LABEL))
    }

    override suspend fun updateTmdbAccount(value: SettingsAccountUiModel) {
        prefs.put(K.tmdbApiKey, value.tmdbApiKey.orEmpty())
        prefs.putBoolean(K.tmdbCastImagesEnabled, value.tmdbCastImagesEnabled)
        prefs.putBoolean(K.tmdbSimilarResultsEnabled, value.tmdbSimilarResultsEnabled)
        prefs.putBoolean(K.tmdbTrailersEnabled, value.tmdbTrailersEnabled)
        prefs.putBoolean(K.tmdbRecommendationsEnabled, value.tmdbRecommendationsEnabled)
        prefs.putBoolean(K.tmdbCollectionInfoEnabled, value.tmdbCollectionInfoEnabled)
        prefs.putBoolean(K.tmdbEpisodeImagesEnabled, value.tmdbEpisodeImagesEnabled)
        prefs.putBoolean(K.tmdbLogosBackdropsEnabled, value.tmdbLogosBackdropsEnabled)
        prefs.putBoolean(K.tmdbRatingsEnabled, value.tmdbRatingsEnabled)
        prefs.putBoolean(K.tmdbBasicInfoEnabled, value.tmdbBasicInfoEnabled)
        prefs.putBoolean(K.tmdbDetailsEnabled, value.tmdbDetailsEnabled)
        prefs.putBoolean(K.tmdbProductionsEnabled, value.tmdbProductionsEnabled)
        prefs.putBoolean(K.tmdbNetworksEnabled, value.tmdbNetworksEnabled)
        state.value = state.value.copy(
            account = state.value.account.copy(
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
        )
    }

    override suspend fun updateNotifications(value: SettingsNotificationsUiModel) {
        prefs.putBoolean(K.notificationsEnabled, value.notificationsEnabled)
        prefs.putBoolean(K.alertNewEpisodes, value.alertNewEpisodes)
        state.value = state.value.copy(notifications = value)
    }

    override suspend fun updateShowHeroSection(value: Boolean) {
        prefs.putBoolean(K.showHeroSection, value)
        state.value = state.value.copy(content = state.value.content.copy(showHeroSection = value))
    }

    override suspend fun toggleHeroFeed(key: String) = Unit
    override suspend fun moveHeroFeed(key: String, direction: Int) = Unit
    override suspend fun toggleHomeFeed(key: String) = Unit
    override suspend fun moveHomeFeed(key: String, direction: Int) = Unit
    override suspend fun toggleTopTenFeed(key: String) = Unit
    override suspend fun disconnectSync() = Unit
    override suspend fun disconnectProvider(provider: String) = Unit

    private fun readState(): SettingsUiState {
        val defaults = SettingsUiState()
        return defaults.copy(
            account = defaults.account.copy(
                tmdbApiKey = prefs.get(K.tmdbApiKey, null),
                tmdbCastImagesEnabled = prefs.getBoolean(K.tmdbCastImagesEnabled, defaults.account.tmdbCastImagesEnabled),
                tmdbSimilarResultsEnabled = prefs.getBoolean(K.tmdbSimilarResultsEnabled, defaults.account.tmdbSimilarResultsEnabled),
                tmdbTrailersEnabled = prefs.getBoolean(K.tmdbTrailersEnabled, defaults.account.tmdbTrailersEnabled),
                tmdbRecommendationsEnabled = prefs.getBoolean(K.tmdbRecommendationsEnabled, defaults.account.tmdbRecommendationsEnabled),
                tmdbCollectionInfoEnabled = prefs.getBoolean(K.tmdbCollectionInfoEnabled, defaults.account.tmdbCollectionInfoEnabled),
                tmdbEpisodeImagesEnabled = prefs.getBoolean(K.tmdbEpisodeImagesEnabled, defaults.account.tmdbEpisodeImagesEnabled),
                tmdbLogosBackdropsEnabled = prefs.getBoolean(K.tmdbLogosBackdropsEnabled, defaults.account.tmdbLogosBackdropsEnabled),
                tmdbRatingsEnabled = prefs.getBoolean(K.tmdbRatingsEnabled, defaults.account.tmdbRatingsEnabled),
                tmdbBasicInfoEnabled = prefs.getBoolean(K.tmdbBasicInfoEnabled, defaults.account.tmdbBasicInfoEnabled),
                tmdbDetailsEnabled = prefs.getBoolean(K.tmdbDetailsEnabled, defaults.account.tmdbDetailsEnabled),
                tmdbProductionsEnabled = prefs.getBoolean(K.tmdbProductionsEnabled, defaults.account.tmdbProductionsEnabled),
                tmdbNetworksEnabled = prefs.getBoolean(K.tmdbNetworksEnabled, defaults.account.tmdbNetworksEnabled)
            ),
            notifications = defaults.notifications.copy(
                notificationsEnabled = prefs.getBoolean(K.notificationsEnabled, defaults.notifications.notificationsEnabled),
                alertNewEpisodes = prefs.getBoolean(K.alertNewEpisodes, defaults.notifications.alertNewEpisodes)
            ),
            general = defaults.general.copy(
                language = prefs.get(K.language, defaults.general.language),
                startPage = prefs.get(K.startPage, defaults.general.startPage),
                backgroundPlayback = prefs.getBoolean(K.backgroundPlayback, defaults.general.backgroundPlayback)
            ),
            appearance = defaults.appearance.copy(
                accentColorArgb = prefs.getLong(K.accentColorArgb, defaults.appearance.accentColorArgb),
                amoledMode = prefs.getBoolean(K.amoledMode, defaults.appearance.amoledMode),
                liquidGlassMode = prefs.getBoolean(K.liquidGlassMode, defaults.appearance.liquidGlassMode),
                animationsEnabled = prefs.getBoolean(K.animationsEnabled, defaults.appearance.animationsEnabled),
                floatingBottomBar = prefs.getBoolean(K.floatingBottomBar, defaults.appearance.floatingBottomBar),
                bottomBarLabels = prefs.getBoolean(K.bottomBarLabels, defaults.appearance.bottomBarLabels),
                topNavigationBar = prefs.getBoolean(K.topNavigationBar, defaults.appearance.topNavigationBar)
            ),
            appearanceHome = defaults.appearanceHome.copy(
                cardCornerPreset = prefs.get(K.cardCornerPreset, defaults.appearanceHome.cardCornerPreset),
                interfaceDensity = prefs.get(K.interfaceDensity, defaults.appearanceHome.interfaceDensity),
                posterWidthPreset = prefs.get(K.posterWidthPreset, defaults.appearanceHome.posterWidthPreset),
                posterLandscapeMode = prefs.getBoolean(K.posterLandscapeMode, defaults.appearanceHome.posterLandscapeMode),
                posterHideTitles = prefs.getBoolean(K.posterHideTitles, defaults.appearanceHome.posterHideTitles),
                homeSeasonPostersOnHero = prefs.getBoolean(K.homeSeasonPostersOnHero, defaults.appearanceHome.homeSeasonPostersOnHero),
                trailerOnHomeHeroEnabled = prefs.getBoolean(K.trailerOnHomeHeroEnabled, defaults.appearanceHome.trailerOnHomeHeroEnabled),
                trailerOnHomeHeroDelaySeconds = prefs.getInt(K.trailerOnHomeHeroDelaySeconds, defaults.appearanceHome.trailerOnHomeHeroDelaySeconds),
                continueWatchingHorizontal = prefs.getBoolean(K.continueWatchingHorizontal, defaults.appearanceHome.continueWatchingHorizontal),
                continueWatchingEnabled = prefs.getBoolean(K.continueWatchingEnabled, defaults.appearanceHome.continueWatchingEnabled),
                continueWatchingHideTitles = prefs.getBoolean(K.continueWatchingHideTitles, defaults.appearanceHome.continueWatchingHideTitles)
            ),
            appearanceDetail = defaults.appearanceDetail.copy(
                blurUnwatchedEpisodes = prefs.getBoolean(K.blurUnwatchedEpisodes, defaults.appearanceDetail.blurUnwatchedEpisodes),
                detailSeasonSelectorMode = prefs.get(K.detailSeasonSelectorMode, defaults.appearanceDetail.detailSeasonSelectorMode),
                detailSeasonPostersOnHero = prefs.getBoolean(K.detailSeasonPostersOnHero, defaults.appearanceDetail.detailSeasonPostersOnHero),
                episodeCardsLayout = prefs.get(K.episodeCardsLayout, defaults.appearanceDetail.episodeCardsLayout)
            ),
            playback = defaults.playback.copy(
                preferredPlayer = prefs.get(K.preferredPlayer, defaults.playback.preferredPlayer),
                mpvCustomOptions = prefs.get(K.mpvCustomOptions, defaults.playback.mpvCustomOptions),
                animeUseMpv = prefs.getBoolean(K.animeUseMpv, defaults.playback.animeUseMpv),
                animePreferJapaneseAudio = prefs.getBoolean(K.animePreferJapaneseAudio, defaults.playback.animePreferJapaneseAudio),
                playbackSpeed = prefs.getFloat(K.playbackSpeed, defaults.playback.playbackSpeed),
                seekForwardSeconds = prefs.getInt(K.seekForwardSeconds, defaults.playback.seekForwardSeconds),
                seekBackwardSeconds = prefs.getInt(K.seekBackwardSeconds, defaults.playback.seekBackwardSeconds),
                holdToSpeedEnabled = prefs.getBoolean(K.holdToSpeedEnabled, defaults.playback.holdToSpeedEnabled),
                holdSpeed = prefs.getFloat(K.holdSpeed, defaults.playback.holdSpeed),
                streamSourceSelectionMode = prefs.get(K.streamSourceSelectionMode, defaults.playback.streamSourceSelectionMode),
                streamSourceRegexPattern = prefs.get(K.streamSourceRegexPattern, defaults.playback.streamSourceRegexPattern),
                autoplayMode = prefs.get(K.autoplayMode, defaults.playback.autoplayMode),
                autoPlayNextEpisode = prefs.getBoolean(K.autoPlayNextEpisode, defaults.playback.autoPlayNextEpisode),
                autoPlayCountdownSecs = prefs.getInt(K.autoPlayCountdownSecs, defaults.playback.autoPlayCountdownSecs),
                autoRetryNextSource = prefs.getBoolean(K.autoRetryNextSource, defaults.playback.autoRetryNextSource),
                tryBingeGroup = prefs.getBoolean(K.tryBingeGroup, defaults.playback.tryBingeGroup),
                nextEpisodeThresholdPercent = prefs.getFloat(K.nextEpisodeThresholdPercent, defaults.playback.nextEpisodeThresholdPercent),
                watchedThresholdPercent = prefs.getFloat(K.watchedThresholdPercent, defaults.playback.watchedThresholdPercent),
                useSkipSegments = prefs.getBoolean(K.useSkipSegments, defaults.playback.useSkipSegments),
                introDbApiKey = prefs.get(K.introDbApiKey, defaults.playback.introDbApiKey),
                useChapterSkip = prefs.getBoolean(K.useChapterSkip, defaults.playback.useChapterSkip),
                autoSkipIntro = prefs.getBoolean(K.autoSkipIntro, defaults.playback.autoSkipIntro),
                contentWarningsEnabled = prefs.getBoolean(K.contentWarningsEnabled, defaults.playback.contentWarningsEnabled)
            ),
            subtitles = defaults.subtitles.copy(
                preferredAudioLanguage = prefs.get(K.preferredAudioLanguage, defaults.subtitles.preferredAudioLanguage),
                secondaryAudioLanguage = prefs.get(K.secondaryAudioLanguage, defaults.subtitles.secondaryAudioLanguage),
                preferredSubtitleLanguage = prefs.get(K.preferredSubtitleLanguage, defaults.subtitles.preferredSubtitleLanguage),
                secondarySubtitleLanguage = prefs.get(K.secondarySubtitleLanguage, defaults.subtitles.secondarySubtitleLanguage),
                autoEnableSubtitles = prefs.getBoolean(K.autoEnableSubtitles, defaults.subtitles.autoEnableSubtitles),
                subtitleShadow = prefs.getBoolean(K.subtitleShadow, defaults.subtitles.subtitleShadow),
                subtitleSize = prefs.getFloat(K.subtitleSize, defaults.subtitles.subtitleSize),
                subtitleColorArgb = prefs.getLong(K.subtitleColorArgb, defaults.subtitles.subtitleColorArgb),
                subtitleTextOpacity = prefs.getFloat(K.subtitleTextOpacity, defaults.subtitles.subtitleTextOpacity),
                subtitleBackgroundColorArgb = prefs.getLong(K.subtitleBackgroundColorArgb, defaults.subtitles.subtitleBackgroundColorArgb),
                subtitleBackgroundOpacity = prefs.getFloat(K.subtitleBackgroundOpacity, defaults.subtitles.subtitleBackgroundOpacity),
                subtitleOutlineColorArgb = prefs.getLong(K.subtitleOutlineColorArgb, defaults.subtitles.subtitleOutlineColorArgb),
                subtitleOutlineOpacity = prefs.getFloat(K.subtitleOutlineOpacity, defaults.subtitles.subtitleOutlineOpacity)
            ),
            advanced = defaults.advanced.copy(
                playerBufferCacheMb = prefs.getInt(K.playerBufferCacheMb, defaults.advanced.playerBufferCacheMb),
                playerForwardBufferSeconds = prefs.getInt(K.playerForwardBufferSeconds, defaults.advanced.playerForwardBufferSeconds),
                playerBackBufferSeconds = prefs.getInt(K.playerBackBufferSeconds, defaults.advanced.playerBackBufferSeconds),
                audioDecoderMode = prefs.get(K.audioDecoderMode, defaults.advanced.audioDecoderMode),
                tunneledPlayback = prefs.getBoolean(K.tunneledPlayback, defaults.advanced.tunneledPlayback)
            ),
            content = defaults.content.copy(
                showHeroSection = prefs.getBoolean(K.showHeroSection, defaults.content.showHeroSection)
            ),
            addons = defaults.addons.copy(
                torrentSpeedPreset = prefs.get(K.torrentSpeedPreset, defaults.addons.torrentSpeedPreset),
                torrentWifiOnly = prefs.getBoolean(K.torrentWifiOnly, defaults.addons.torrentWifiOnly)
            ),
            downloads = defaults.downloads.copy(
                downloadSourceSelectionMode = prefs.get(K.downloadSourceSelectionMode, defaults.downloads.downloadSourceSelectionMode),
                downloadSourceRegexPattern = prefs.get(K.downloadSourceRegexPattern, defaults.downloads.downloadSourceRegexPattern),
                downloadSubtitleLanguage = prefs.get(K.downloadSubtitleLanguage, defaults.downloads.downloadSubtitleLanguage)
            ),
            system = defaults.system.copy(
                automaticUpdates = prefs.getBoolean(K.automaticUpdates, defaults.system.automaticUpdates),
                appVersionLabel = APP_VERSION_LABEL
            )
        )
    }

    private object K {
        const val language = "language"
        const val startPage = "startPage"
        const val backgroundPlayback = "backgroundPlayback"
        const val accentColorArgb = "accentColorArgb"
        const val amoledMode = "amoledMode"
        const val liquidGlassMode = "liquidGlassMode"
        const val animationsEnabled = "animationsEnabled"
        const val floatingBottomBar = "floatingBottomBar"
        const val bottomBarLabels = "bottomBarLabels"
        const val topNavigationBar = "topNavigationBar"
        const val cardCornerPreset = "cardCornerPreset"
        const val interfaceDensity = "interfaceDensity"
        const val posterWidthPreset = "posterWidthPreset"
        const val posterLandscapeMode = "posterLandscapeMode"
        const val posterHideTitles = "posterHideTitles"
        const val homeSeasonPostersOnHero = "homeSeasonPostersOnHero"
        const val trailerOnHomeHeroEnabled = "trailerOnHomeHeroEnabled"
        const val trailerOnHomeHeroDelaySeconds = "trailerOnHomeHeroDelaySeconds"
        const val continueWatchingHorizontal = "continueWatchingHorizontal"
        const val continueWatchingEnabled = "continueWatchingEnabled"
        const val continueWatchingHideTitles = "continueWatchingHideTitles"
        const val blurUnwatchedEpisodes = "blurUnwatchedEpisodes"
        const val detailSeasonSelectorMode = "detailSeasonSelectorMode"
        const val detailSeasonPostersOnHero = "detailSeasonPostersOnHero"
        const val episodeCardsLayout = "episodeCardsLayout"
        const val preferredPlayer = "preferredPlayer"
        const val mpvCustomOptions = "mpvCustomOptions"
        const val animeUseMpv = "animeUseMpv"
        const val animePreferJapaneseAudio = "animePreferJapaneseAudio"
        const val playbackSpeed = "playbackSpeed"
        const val seekForwardSeconds = "seekForwardSeconds"
        const val seekBackwardSeconds = "seekBackwardSeconds"
        const val holdToSpeedEnabled = "holdToSpeedEnabled"
        const val holdSpeed = "holdSpeed"
        const val streamSourceSelectionMode = "streamSourceSelectionMode"
        const val streamSourceRegexPattern = "streamSourceRegexPattern"
        const val autoplayMode = "autoplayMode"
        const val autoPlayNextEpisode = "autoPlayNextEpisode"
        const val autoPlayCountdownSecs = "autoPlayCountdownSecs"
        const val autoRetryNextSource = "autoRetryNextSource"
        const val tryBingeGroup = "tryBingeGroup"
        const val nextEpisodeThresholdPercent = "nextEpisodeThresholdPercent"
        const val watchedThresholdPercent = "watchedThresholdPercent"
        const val useSkipSegments = "useSkipSegments"
        const val introDbApiKey = "introDbApiKey"
        const val useChapterSkip = "useChapterSkip"
        const val autoSkipIntro = "autoSkipIntro"
        const val contentWarningsEnabled = "contentWarningsEnabled"
        const val preferredAudioLanguage = "preferredAudioLanguage"
        const val secondaryAudioLanguage = "secondaryAudioLanguage"
        const val preferredSubtitleLanguage = "preferredSubtitleLanguage"
        const val secondarySubtitleLanguage = "secondarySubtitleLanguage"
        const val autoEnableSubtitles = "autoEnableSubtitles"
        const val subtitleShadow = "subtitleShadow"
        const val subtitleSize = "subtitleSize"
        const val subtitleColorArgb = "subtitleColorArgb"
        const val subtitleTextOpacity = "subtitleTextOpacity"
        const val subtitleBackgroundColorArgb = "subtitleBackgroundColorArgb"
        const val subtitleBackgroundOpacity = "subtitleBackgroundOpacity"
        const val subtitleOutlineColorArgb = "subtitleOutlineColorArgb"
        const val subtitleOutlineOpacity = "subtitleOutlineOpacity"
        const val playerBufferCacheMb = "playerBufferCacheMb"
        const val playerForwardBufferSeconds = "playerForwardBufferSeconds"
        const val playerBackBufferSeconds = "playerBackBufferSeconds"
        const val audioDecoderMode = "audioDecoderMode"
        const val tunneledPlayback = "tunneledPlayback"
        const val showHeroSection = "showHeroSection"
        const val torrentSpeedPreset = "torrentSpeedPreset"
        const val torrentWifiOnly = "torrentWifiOnly"
        const val downloadSourceSelectionMode = "downloadSourceSelectionMode"
        const val downloadSourceRegexPattern = "downloadSourceRegexPattern"
        const val downloadSubtitleLanguage = "downloadSubtitleLanguage"
        const val automaticUpdates = "automaticUpdates"
        const val tmdbApiKey = "tmdbApiKey"
        const val tmdbCastImagesEnabled = "tmdbCastImagesEnabled"
        const val tmdbSimilarResultsEnabled = "tmdbSimilarResultsEnabled"
        const val tmdbTrailersEnabled = "tmdbTrailersEnabled"
        const val tmdbRecommendationsEnabled = "tmdbRecommendationsEnabled"
        const val tmdbCollectionInfoEnabled = "tmdbCollectionInfoEnabled"
        const val tmdbEpisodeImagesEnabled = "tmdbEpisodeImagesEnabled"
        const val tmdbLogosBackdropsEnabled = "tmdbLogosBackdropsEnabled"
        const val tmdbRatingsEnabled = "tmdbRatingsEnabled"
        const val tmdbBasicInfoEnabled = "tmdbBasicInfoEnabled"
        const val tmdbDetailsEnabled = "tmdbDetailsEnabled"
        const val tmdbProductionsEnabled = "tmdbProductionsEnabled"
        const val tmdbNetworksEnabled = "tmdbNetworksEnabled"
        const val notificationsEnabled = "notificationsEnabled"
        const val alertNewEpisodes = "alertNewEpisodes"
    }

    private companion object {
        const val APP_VERSION_LABEL = "dev"
    }
}
