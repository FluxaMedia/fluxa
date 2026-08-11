package com.fluxa.app.shared.feature.settings

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

/** Minimal typed storage boundary used by desktop and Apple settings. */
interface SettingsPreferenceStore {
    fun getString(key: String, default: String? = null): String?
    fun getBoolean(key: String, default: Boolean): Boolean
    fun getInt(key: String, default: Int): Int
    fun getLong(key: String, default: Long): Long
    fun getFloat(key: String, default: Float): Float

    fun putString(key: String, value: String?)
    fun putBoolean(key: String, value: Boolean)
    fun putInt(key: String, value: Int)
    fun putLong(key: String, value: Long)
    fun putFloat(key: String, value: Float)
}

/**
 * Shared persistence implementation for platforms that store settings as key/value pairs.
 * Platform data sources keep account/feed orchestration, while schema/defaults live here.
 */
abstract class PersistentSettingsDataSource(
    private val preferences: SettingsPreferenceStore,
    private val appVersionLabel: String,
) : SettingsDataSource {
    protected val storedState = MutableStateFlow(readStoredState())

    open override fun observeSettings(): Flow<SettingsUiState> = storedState.asStateFlow()

    override fun observeAppearanceHome(): Flow<SettingsAppearanceHomeUiModel> = storedState
        .map { it.appearanceHome }
        .distinctUntilChanged()

    open override suspend fun refreshContentFeeds() = Unit

    override suspend fun updateGeneral(value: SettingsGeneralUiModel) {
        preferences.putString(SettingsPreferenceKeys.LANGUAGE, value.language)
        preferences.putString(SettingsPreferenceKeys.START_PAGE, value.startPage)
        preferences.putBoolean(SettingsPreferenceKeys.BACKGROUND_PLAYBACK, value.backgroundPlayback)
        storedState.value = storedState.value.copy(general = value)
    }

    override suspend fun updateAppearance(value: SettingsAppearanceUiModel) {
        preferences.putLong(SettingsPreferenceKeys.ACCENT_COLOR_ARGB, value.accentColorArgb)
        preferences.putBoolean(SettingsPreferenceKeys.AMOLED_MODE, value.amoledMode)
        preferences.putBoolean(SettingsPreferenceKeys.LIQUID_GLASS_MODE, value.liquidGlassMode)
        preferences.putBoolean(SettingsPreferenceKeys.ANIMATIONS_ENABLED, value.animationsEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.FLOATING_BOTTOM_BAR, value.floatingBottomBar)
        preferences.putBoolean(SettingsPreferenceKeys.BOTTOM_BAR_LABELS, value.bottomBarLabels)
        preferences.putBoolean(SettingsPreferenceKeys.TOP_NAVIGATION_BAR, value.topNavigationBar)
        storedState.value = storedState.value.copy(appearance = value)
    }

    open override suspend fun updateAppearanceHome(value: SettingsAppearanceHomeUiModel) {
        preferences.putString(SettingsPreferenceKeys.CARD_CORNER_PRESET, value.cardCornerPreset)
        preferences.putString(SettingsPreferenceKeys.INTERFACE_DENSITY, value.interfaceDensity)
        preferences.putString(SettingsPreferenceKeys.POSTER_WIDTH_PRESET, value.posterWidthPreset)
        preferences.putString(SettingsPreferenceKeys.CONTINUE_WATCHING_CARD_CORNER_PRESET, value.continueWatchingCardCornerPreset)
        preferences.putString(SettingsPreferenceKeys.CONTINUE_WATCHING_INTERFACE_DENSITY, value.continueWatchingInterfaceDensity)
        preferences.putString(SettingsPreferenceKeys.CONTINUE_WATCHING_WIDTH_PRESET, value.continueWatchingWidthPreset)
        preferences.putBoolean(SettingsPreferenceKeys.POSTER_LANDSCAPE_MODE, value.posterLandscapeMode)
        preferences.putBoolean(SettingsPreferenceKeys.POSTER_HIDE_TITLES, value.posterHideTitles)
        preferences.putBoolean(SettingsPreferenceKeys.HOME_SEASON_POSTERS_ON_HERO, value.homeSeasonPostersOnHero)
        preferences.putBoolean(SettingsPreferenceKeys.TRAILER_ON_HOME_HERO_ENABLED, value.trailerOnHomeHeroEnabled)
        preferences.putInt(SettingsPreferenceKeys.TRAILER_ON_HOME_HERO_DELAY_SECONDS, value.trailerOnHomeHeroDelaySeconds)
        preferences.putBoolean(SettingsPreferenceKeys.CONTINUE_WATCHING_HORIZONTAL, value.continueWatchingHorizontal)
        preferences.putBoolean(SettingsPreferenceKeys.CONTINUE_WATCHING_ENABLED, value.continueWatchingEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.CONTINUE_WATCHING_HIDE_TITLES, value.continueWatchingHideTitles)
        preferences.putString(SettingsPreferenceKeys.CONTINUE_WATCHING_SOURCE, value.continueWatchingSource)
        preferences.putBoolean(SettingsPreferenceKeys.UPCOMING_ROW_ENABLED, value.upcomingRowEnabled)
        storedState.value = storedState.value.copy(appearanceHome = value)
    }

    override suspend fun updateAppearanceDetail(value: SettingsAppearanceDetailUiModel) {
        preferences.putString(SettingsPreferenceKeys.DETAIL_SCREEN_STYLE, value.detailScreenStyle)
        preferences.putBoolean(SettingsPreferenceKeys.DETAIL_PREFER_CLEARLOGO, value.detailPreferClearlogo)
        preferences.putBoolean(SettingsPreferenceKeys.DETAIL_SHOW_EPISODE_DESCRIPTIONS, value.detailShowEpisodeDescriptions)
        preferences.putBoolean(SettingsPreferenceKeys.DETAIL_SHOW_CAST, value.detailShowCast)
        preferences.putBoolean(SettingsPreferenceKeys.DETAIL_SHOW_RECOMMENDATIONS, value.detailShowRecommendations)
        preferences.putBoolean(SettingsPreferenceKeys.DETAIL_COLLAPSING_HERO, value.detailCollapsingHero)
        preferences.putBoolean(SettingsPreferenceKeys.TRAILER_ON_DETAIL_HERO_ENABLED, value.trailerOnDetailHeroEnabled)
        preferences.putInt(SettingsPreferenceKeys.TRAILER_ON_DETAIL_HERO_DELAY_SECONDS, value.trailerOnDetailHeroDelaySeconds)
        preferences.putBoolean(SettingsPreferenceKeys.BLUR_UNWATCHED_EPISODES, value.blurUnwatchedEpisodes)
        preferences.putString(SettingsPreferenceKeys.DETAIL_SEASON_SELECTOR_MODE, value.detailSeasonSelectorMode)
        preferences.putBoolean(SettingsPreferenceKeys.DETAIL_SEASON_POSTERS_ON_HERO, value.detailSeasonPostersOnHero)
        preferences.putString(SettingsPreferenceKeys.EPISODE_CARDS_LAYOUT, value.episodeCardsLayout)
        storedState.value = storedState.value.copy(appearanceDetail = value)
    }

    override suspend fun updatePlayback(value: SettingsPlaybackUiModel) {
        preferences.putString(SettingsPreferenceKeys.PREFERRED_PLAYER, value.preferredPlayer)
        preferences.putString(SettingsPreferenceKeys.EXTERNAL_PLAYER_TARGET, value.externalPlayerTarget)
        preferences.putString(SettingsPreferenceKeys.MPV_CUSTOM_OPTIONS, value.mpvCustomOptions)
        preferences.putString(SettingsPreferenceKeys.ANIME_UPSCALING_MODE, value.animeUpscalingMode)
        preferences.putString(SettingsPreferenceKeys.ANIME_UPSCALING_QUALITY, value.animeUpscalingQuality)
        preferences.putString(SettingsPreferenceKeys.ANIME_UPSCALING_MODE_PRESET, value.animeUpscalingModePreset)
        preferences.putBoolean(SettingsPreferenceKeys.ANIME_USE_MPV, value.animeUseMpv)
        preferences.putBoolean(SettingsPreferenceKeys.ANIME_PREFER_JAPANESE_AUDIO, value.animePreferJapaneseAudio)
        preferences.putFloat(SettingsPreferenceKeys.PLAYBACK_SPEED, value.playbackSpeed)
        preferences.putInt(SettingsPreferenceKeys.SEEK_FORWARD_SECONDS, value.seekForwardSeconds)
        preferences.putInt(SettingsPreferenceKeys.SEEK_BACKWARD_SECONDS, value.seekBackwardSeconds)
        preferences.putBoolean(SettingsPreferenceKeys.HOLD_TO_SPEED_ENABLED, value.holdToSpeedEnabled)
        preferences.putFloat(SettingsPreferenceKeys.HOLD_SPEED, value.holdSpeed)
        preferences.putString(SettingsPreferenceKeys.STREAM_SOURCE_SELECTION_MODE, value.streamSourceSelectionMode)
        preferences.putString(SettingsPreferenceKeys.STREAM_SOURCE_REGEX_PATTERN, value.streamSourceRegexPattern)
        preferences.putString(SettingsPreferenceKeys.AUTOPLAY_MODE, value.autoplayMode)
        preferences.putBoolean(SettingsPreferenceKeys.AUTO_PLAY_NEXT_EPISODE, value.autoPlayNextEpisode)
        preferences.putInt(SettingsPreferenceKeys.AUTO_PLAY_COUNTDOWN_SECONDS, value.autoPlayCountdownSecs)
        preferences.putBoolean(SettingsPreferenceKeys.AUTO_RETRY_NEXT_SOURCE, value.autoRetryNextSource)
        preferences.putBoolean(SettingsPreferenceKeys.TRY_BINGE_GROUP, value.tryBingeGroup)
        preferences.putFloat(SettingsPreferenceKeys.NEXT_EPISODE_THRESHOLD_PERCENT, value.nextEpisodeThresholdPercent)
        preferences.putFloat(SettingsPreferenceKeys.WATCHED_THRESHOLD_PERCENT, value.watchedThresholdPercent)
        preferences.putBoolean(SettingsPreferenceKeys.USE_SKIP_SEGMENTS, value.useSkipSegments)
        preferences.putString(SettingsPreferenceKeys.INTRO_DB_API_KEY, value.introDbApiKey)
        preferences.putBoolean(SettingsPreferenceKeys.USE_CHAPTER_SKIP, value.useChapterSkip)
        preferences.putBoolean(SettingsPreferenceKeys.AUTO_SKIP_INTRO, value.autoSkipIntro)
        preferences.putBoolean(SettingsPreferenceKeys.CONTENT_WARNINGS_ENABLED, value.contentWarningsEnabled)
        storedState.value = storedState.value.copy(playback = value)
    }

    override suspend fun updateSubtitles(value: SettingsSubtitlesUiModel) {
        preferences.putString(SettingsPreferenceKeys.PREFERRED_AUDIO_LANGUAGE, value.preferredAudioLanguage)
        preferences.putString(SettingsPreferenceKeys.SECONDARY_AUDIO_LANGUAGE, value.secondaryAudioLanguage)
        preferences.putString(SettingsPreferenceKeys.PREFERRED_SUBTITLE_LANGUAGE, value.preferredSubtitleLanguage)
        preferences.putString(SettingsPreferenceKeys.SECONDARY_SUBTITLE_LANGUAGE, value.secondarySubtitleLanguage)
        preferences.putBoolean(SettingsPreferenceKeys.AUTO_ENABLE_SUBTITLES, value.autoEnableSubtitles)
        preferences.putBoolean(SettingsPreferenceKeys.SUBTITLE_SHADOW, value.subtitleShadow)
        preferences.putFloat(SettingsPreferenceKeys.SUBTITLE_SIZE, value.subtitleSize)
        preferences.putLong(SettingsPreferenceKeys.SUBTITLE_COLOR_ARGB, value.subtitleColorArgb)
        preferences.putFloat(SettingsPreferenceKeys.SUBTITLE_TEXT_OPACITY, value.subtitleTextOpacity)
        preferences.putLong(SettingsPreferenceKeys.SUBTITLE_BACKGROUND_COLOR_ARGB, value.subtitleBackgroundColorArgb)
        preferences.putFloat(SettingsPreferenceKeys.SUBTITLE_BACKGROUND_OPACITY, value.subtitleBackgroundOpacity)
        preferences.putLong(SettingsPreferenceKeys.SUBTITLE_OUTLINE_COLOR_ARGB, value.subtitleOutlineColorArgb)
        preferences.putFloat(SettingsPreferenceKeys.SUBTITLE_OUTLINE_OPACITY, value.subtitleOutlineOpacity)
        storedState.value = storedState.value.copy(subtitles = value)
    }

    override suspend fun updateAdvanced(value: SettingsAdvancedUiModel) {
        preferences.putInt(SettingsPreferenceKeys.PLAYER_BUFFER_CACHE_MB, value.playerBufferCacheMb)
        preferences.putInt(SettingsPreferenceKeys.PLAYER_FORWARD_BUFFER_SECONDS, value.playerForwardBufferSeconds)
        preferences.putInt(SettingsPreferenceKeys.PLAYER_BACK_BUFFER_SECONDS, value.playerBackBufferSeconds)
        preferences.putString(SettingsPreferenceKeys.AUDIO_DECODER_MODE, value.audioDecoderMode)
        preferences.putBoolean(SettingsPreferenceKeys.TUNNELED_PLAYBACK, value.tunneledPlayback)
        storedState.value = storedState.value.copy(advanced = value)
    }

    override suspend fun updateAddons(value: SettingsAddonsUiModel) {
        preferences.putString(SettingsPreferenceKeys.TORRENT_SPEED_PRESET, value.torrentSpeedPreset)
        preferences.putBoolean(SettingsPreferenceKeys.TORRENT_WIFI_ONLY, value.torrentWifiOnly)
        storedState.value = storedState.value.copy(addons = value)
    }

    override suspend fun updateDownloads(value: SettingsDownloadsUiModel) {
        preferences.putString(SettingsPreferenceKeys.DOWNLOAD_SOURCE_SELECTION_MODE, value.downloadSourceSelectionMode)
        preferences.putString(SettingsPreferenceKeys.DOWNLOAD_SOURCE_REGEX_PATTERN, value.downloadSourceRegexPattern)
        preferences.putString(SettingsPreferenceKeys.DOWNLOAD_SUBTITLE_LANGUAGE, value.downloadSubtitleLanguage)
        storedState.value = storedState.value.copy(downloads = value)
    }

    override suspend fun updateSystem(value: SettingsSystemUiModel) {
        preferences.putBoolean(SettingsPreferenceKeys.AUTOMATIC_UPDATES, value.automaticUpdates)
        preferences.putBoolean(SettingsPreferenceKeys.REMEMBER_LAST_PROFILE, value.rememberLastProfile)
        storedState.value = storedState.value.copy(system = value.copy(appVersionLabel = appVersionLabel))
    }

    override suspend fun updateTmdbAccount(value: SettingsAccountUiModel) {
        preferences.putString(SettingsPreferenceKeys.TMDB_API_KEY, value.tmdbApiKey)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_CAST_IMAGES_ENABLED, value.tmdbCastImagesEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_SIMILAR_RESULTS_ENABLED, value.tmdbSimilarResultsEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_TRAILERS_ENABLED, value.tmdbTrailersEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_RECOMMENDATIONS_ENABLED, value.tmdbRecommendationsEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_COLLECTION_INFO_ENABLED, value.tmdbCollectionInfoEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_EPISODE_IMAGES_ENABLED, value.tmdbEpisodeImagesEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_LOGOS_BACKDROPS_ENABLED, value.tmdbLogosBackdropsEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_RATINGS_ENABLED, value.tmdbRatingsEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_BASIC_INFO_ENABLED, value.tmdbBasicInfoEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_DETAILS_ENABLED, value.tmdbDetailsEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_PRODUCTIONS_ENABLED, value.tmdbProductionsEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.TMDB_NETWORKS_ENABLED, value.tmdbNetworksEnabled)
        val current = storedState.value.account
        storedState.value = storedState.value.copy(
            account = current.copy(
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
                tmdbNetworksEnabled = value.tmdbNetworksEnabled,
            ),
        )
    }

    override suspend fun updateNotifications(value: SettingsNotificationsUiModel) {
        preferences.putBoolean(SettingsPreferenceKeys.NOTIFICATIONS_ENABLED, value.notificationsEnabled)
        preferences.putBoolean(SettingsPreferenceKeys.ALERT_NEW_EPISODES, value.alertNewEpisodes)
        storedState.value = storedState.value.copy(notifications = value)
    }

    override suspend fun updateShowHeroSection(value: Boolean) {
        preferences.putBoolean(SettingsPreferenceKeys.SHOW_HERO_SECTION, value)
        storedState.value = storedState.value.copy(
            content = storedState.value.content.copy(showHeroSection = value),
        )
    }

    open override suspend fun toggleHeroFeed(key: String) = Unit
    open override suspend fun moveHeroFeed(key: String, direction: Int) = Unit
    open override suspend fun toggleHomeFeed(key: String) = Unit
    open override suspend fun moveHomeFeed(key: String, direction: Int) = Unit
    open override suspend fun toggleTopTenFeed(key: String) = Unit
    open override suspend fun disconnectSync() = Unit
    open override suspend fun disconnectProvider(provider: String) = Unit

    private fun readStoredState(): SettingsUiState {
        val defaults = SettingsUiState()
        return defaults.copy(
            account = defaults.account.copy(
                tmdbApiKey = preferences.getString(SettingsPreferenceKeys.TMDB_API_KEY),
                tmdbCastImagesEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_CAST_IMAGES_ENABLED, defaults.account.tmdbCastImagesEnabled),
                tmdbSimilarResultsEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_SIMILAR_RESULTS_ENABLED, defaults.account.tmdbSimilarResultsEnabled),
                tmdbTrailersEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_TRAILERS_ENABLED, defaults.account.tmdbTrailersEnabled),
                tmdbRecommendationsEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_RECOMMENDATIONS_ENABLED, defaults.account.tmdbRecommendationsEnabled),
                tmdbCollectionInfoEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_COLLECTION_INFO_ENABLED, defaults.account.tmdbCollectionInfoEnabled),
                tmdbEpisodeImagesEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_EPISODE_IMAGES_ENABLED, defaults.account.tmdbEpisodeImagesEnabled),
                tmdbLogosBackdropsEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_LOGOS_BACKDROPS_ENABLED, defaults.account.tmdbLogosBackdropsEnabled),
                tmdbRatingsEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_RATINGS_ENABLED, defaults.account.tmdbRatingsEnabled),
                tmdbBasicInfoEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_BASIC_INFO_ENABLED, defaults.account.tmdbBasicInfoEnabled),
                tmdbDetailsEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_DETAILS_ENABLED, defaults.account.tmdbDetailsEnabled),
                tmdbProductionsEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_PRODUCTIONS_ENABLED, defaults.account.tmdbProductionsEnabled),
                tmdbNetworksEnabled = preferences.getBoolean(SettingsPreferenceKeys.TMDB_NETWORKS_ENABLED, defaults.account.tmdbNetworksEnabled),
            ),
            notifications = defaults.notifications.copy(
                notificationsEnabled = preferences.getBoolean(SettingsPreferenceKeys.NOTIFICATIONS_ENABLED, defaults.notifications.notificationsEnabled),
                alertNewEpisodes = preferences.getBoolean(SettingsPreferenceKeys.ALERT_NEW_EPISODES, defaults.notifications.alertNewEpisodes),
            ),
            general = defaults.general.copy(
                language = preferences.getString(SettingsPreferenceKeys.LANGUAGE, defaults.general.language) ?: defaults.general.language,
                startPage = preferences.getString(SettingsPreferenceKeys.START_PAGE, defaults.general.startPage) ?: defaults.general.startPage,
                backgroundPlayback = preferences.getBoolean(SettingsPreferenceKeys.BACKGROUND_PLAYBACK, defaults.general.backgroundPlayback),
            ),
            appearance = defaults.appearance.copy(
                accentColorArgb = preferences.getLong(SettingsPreferenceKeys.ACCENT_COLOR_ARGB, defaults.appearance.accentColorArgb),
                amoledMode = preferences.getBoolean(SettingsPreferenceKeys.AMOLED_MODE, defaults.appearance.amoledMode),
                liquidGlassMode = preferences.getBoolean(SettingsPreferenceKeys.LIQUID_GLASS_MODE, defaults.appearance.liquidGlassMode),
                animationsEnabled = preferences.getBoolean(SettingsPreferenceKeys.ANIMATIONS_ENABLED, defaults.appearance.animationsEnabled),
                floatingBottomBar = preferences.getBoolean(SettingsPreferenceKeys.FLOATING_BOTTOM_BAR, defaults.appearance.floatingBottomBar),
                bottomBarLabels = preferences.getBoolean(SettingsPreferenceKeys.BOTTOM_BAR_LABELS, defaults.appearance.bottomBarLabels),
                topNavigationBar = preferences.getBoolean(SettingsPreferenceKeys.TOP_NAVIGATION_BAR, defaults.appearance.topNavigationBar),
            ),
            appearanceHome = defaults.appearanceHome.copy(
                cardCornerPreset = preferences.getString(SettingsPreferenceKeys.CARD_CORNER_PRESET, defaults.appearanceHome.cardCornerPreset) ?: defaults.appearanceHome.cardCornerPreset,
                interfaceDensity = preferences.getString(SettingsPreferenceKeys.INTERFACE_DENSITY, defaults.appearanceHome.interfaceDensity) ?: defaults.appearanceHome.interfaceDensity,
                posterWidthPreset = preferences.getString(SettingsPreferenceKeys.POSTER_WIDTH_PRESET, defaults.appearanceHome.posterWidthPreset) ?: defaults.appearanceHome.posterWidthPreset,
                posterLandscapeMode = preferences.getBoolean(SettingsPreferenceKeys.POSTER_LANDSCAPE_MODE, defaults.appearanceHome.posterLandscapeMode),
                posterHideTitles = preferences.getBoolean(SettingsPreferenceKeys.POSTER_HIDE_TITLES, defaults.appearanceHome.posterHideTitles),
                homeSeasonPostersOnHero = preferences.getBoolean(SettingsPreferenceKeys.HOME_SEASON_POSTERS_ON_HERO, defaults.appearanceHome.homeSeasonPostersOnHero),
                trailerOnHomeHeroEnabled = preferences.getBoolean(SettingsPreferenceKeys.TRAILER_ON_HOME_HERO_ENABLED, defaults.appearanceHome.trailerOnHomeHeroEnabled),
                trailerOnHomeHeroDelaySeconds = preferences.getInt(SettingsPreferenceKeys.TRAILER_ON_HOME_HERO_DELAY_SECONDS, defaults.appearanceHome.trailerOnHomeHeroDelaySeconds),
                continueWatchingHorizontal = preferences.getBoolean(SettingsPreferenceKeys.CONTINUE_WATCHING_HORIZONTAL, defaults.appearanceHome.continueWatchingHorizontal),
                continueWatchingEnabled = preferences.getBoolean(SettingsPreferenceKeys.CONTINUE_WATCHING_ENABLED, defaults.appearanceHome.continueWatchingEnabled),
                continueWatchingHideTitles = preferences.getBoolean(SettingsPreferenceKeys.CONTINUE_WATCHING_HIDE_TITLES, defaults.appearanceHome.continueWatchingHideTitles),
                continueWatchingSource = preferences.getString(SettingsPreferenceKeys.CONTINUE_WATCHING_SOURCE, defaults.appearanceHome.continueWatchingSource) ?: defaults.appearanceHome.continueWatchingSource,
                continueWatchingCardCornerPreset = preferences.getString(SettingsPreferenceKeys.CONTINUE_WATCHING_CARD_CORNER_PRESET, defaults.appearanceHome.continueWatchingCardCornerPreset) ?: defaults.appearanceHome.continueWatchingCardCornerPreset,
                continueWatchingInterfaceDensity = preferences.getString(SettingsPreferenceKeys.CONTINUE_WATCHING_INTERFACE_DENSITY, defaults.appearanceHome.continueWatchingInterfaceDensity) ?: defaults.appearanceHome.continueWatchingInterfaceDensity,
                continueWatchingWidthPreset = preferences.getString(SettingsPreferenceKeys.CONTINUE_WATCHING_WIDTH_PRESET, defaults.appearanceHome.continueWatchingWidthPreset) ?: defaults.appearanceHome.continueWatchingWidthPreset,
                upcomingRowEnabled = preferences.getBoolean(SettingsPreferenceKeys.UPCOMING_ROW_ENABLED, defaults.appearanceHome.upcomingRowEnabled),
            ),
            appearanceDetail = defaults.appearanceDetail.copy(
                detailScreenStyle = preferences.getString(SettingsPreferenceKeys.DETAIL_SCREEN_STYLE, defaults.appearanceDetail.detailScreenStyle) ?: defaults.appearanceDetail.detailScreenStyle,
                detailPreferClearlogo = preferences.getBoolean(SettingsPreferenceKeys.DETAIL_PREFER_CLEARLOGO, defaults.appearanceDetail.detailPreferClearlogo),
                detailShowEpisodeDescriptions = preferences.getBoolean(SettingsPreferenceKeys.DETAIL_SHOW_EPISODE_DESCRIPTIONS, defaults.appearanceDetail.detailShowEpisodeDescriptions),
                detailShowCast = preferences.getBoolean(SettingsPreferenceKeys.DETAIL_SHOW_CAST, defaults.appearanceDetail.detailShowCast),
                detailShowRecommendations = preferences.getBoolean(SettingsPreferenceKeys.DETAIL_SHOW_RECOMMENDATIONS, defaults.appearanceDetail.detailShowRecommendations),
                detailCollapsingHero = preferences.getBoolean(SettingsPreferenceKeys.DETAIL_COLLAPSING_HERO, defaults.appearanceDetail.detailCollapsingHero),
                trailerOnDetailHeroEnabled = preferences.getBoolean(SettingsPreferenceKeys.TRAILER_ON_DETAIL_HERO_ENABLED, defaults.appearanceDetail.trailerOnDetailHeroEnabled),
                trailerOnDetailHeroDelaySeconds = preferences.getInt(SettingsPreferenceKeys.TRAILER_ON_DETAIL_HERO_DELAY_SECONDS, defaults.appearanceDetail.trailerOnDetailHeroDelaySeconds),
                blurUnwatchedEpisodes = preferences.getBoolean(SettingsPreferenceKeys.BLUR_UNWATCHED_EPISODES, defaults.appearanceDetail.blurUnwatchedEpisodes),
                detailSeasonSelectorMode = preferences.getString(SettingsPreferenceKeys.DETAIL_SEASON_SELECTOR_MODE, defaults.appearanceDetail.detailSeasonSelectorMode) ?: defaults.appearanceDetail.detailSeasonSelectorMode,
                detailSeasonPostersOnHero = preferences.getBoolean(SettingsPreferenceKeys.DETAIL_SEASON_POSTERS_ON_HERO, defaults.appearanceDetail.detailSeasonPostersOnHero),
                episodeCardsLayout = when (preferences.getString(SettingsPreferenceKeys.EPISODE_CARDS_LAYOUT, defaults.appearanceDetail.episodeCardsLayout)) {
                    "list" -> "list"
                    "two_column" -> "two_column"
                    "carousel", "horizontal", "grid" -> "carousel"
                    else -> defaults.appearanceDetail.episodeCardsLayout
                },
            ),
            playback = defaults.playback.copy(
                preferredPlayer = preferences.getString(SettingsPreferenceKeys.PREFERRED_PLAYER, defaults.playback.preferredPlayer) ?: defaults.playback.preferredPlayer,
                externalPlayerTarget = preferences.getString(SettingsPreferenceKeys.EXTERNAL_PLAYER_TARGET, defaults.playback.externalPlayerTarget) ?: defaults.playback.externalPlayerTarget,
                mpvCustomOptions = preferences.getString(SettingsPreferenceKeys.MPV_CUSTOM_OPTIONS, defaults.playback.mpvCustomOptions) ?: defaults.playback.mpvCustomOptions,
                animeUpscalingMode = preferences.getString(SettingsPreferenceKeys.ANIME_UPSCALING_MODE, defaults.playback.animeUpscalingMode) ?: defaults.playback.animeUpscalingMode,
                animeUpscalingQuality = preferences.getString(SettingsPreferenceKeys.ANIME_UPSCALING_QUALITY, defaults.playback.animeUpscalingQuality) ?: defaults.playback.animeUpscalingQuality,
                animeUpscalingModePreset = preferences.getString(SettingsPreferenceKeys.ANIME_UPSCALING_MODE_PRESET, defaults.playback.animeUpscalingModePreset) ?: defaults.playback.animeUpscalingModePreset,
                animeUseMpv = preferences.getBoolean(SettingsPreferenceKeys.ANIME_USE_MPV, defaults.playback.animeUseMpv),
                animePreferJapaneseAudio = preferences.getBoolean(SettingsPreferenceKeys.ANIME_PREFER_JAPANESE_AUDIO, defaults.playback.animePreferJapaneseAudio),
                playbackSpeed = preferences.getFloat(SettingsPreferenceKeys.PLAYBACK_SPEED, defaults.playback.playbackSpeed),
                seekForwardSeconds = preferences.getInt(SettingsPreferenceKeys.SEEK_FORWARD_SECONDS, defaults.playback.seekForwardSeconds),
                seekBackwardSeconds = preferences.getInt(SettingsPreferenceKeys.SEEK_BACKWARD_SECONDS, defaults.playback.seekBackwardSeconds),
                holdToSpeedEnabled = preferences.getBoolean(SettingsPreferenceKeys.HOLD_TO_SPEED_ENABLED, defaults.playback.holdToSpeedEnabled),
                holdSpeed = preferences.getFloat(SettingsPreferenceKeys.HOLD_SPEED, defaults.playback.holdSpeed),
                streamSourceSelectionMode = preferences.getString(SettingsPreferenceKeys.STREAM_SOURCE_SELECTION_MODE, defaults.playback.streamSourceSelectionMode) ?: defaults.playback.streamSourceSelectionMode,
                streamSourceRegexPattern = preferences.getString(SettingsPreferenceKeys.STREAM_SOURCE_REGEX_PATTERN, defaults.playback.streamSourceRegexPattern) ?: defaults.playback.streamSourceRegexPattern,
                autoplayMode = preferences.getString(SettingsPreferenceKeys.AUTOPLAY_MODE, defaults.playback.autoplayMode) ?: defaults.playback.autoplayMode,
                autoPlayNextEpisode = preferences.getBoolean(SettingsPreferenceKeys.AUTO_PLAY_NEXT_EPISODE, defaults.playback.autoPlayNextEpisode),
                autoPlayCountdownSecs = preferences.getInt(SettingsPreferenceKeys.AUTO_PLAY_COUNTDOWN_SECONDS, defaults.playback.autoPlayCountdownSecs),
                autoRetryNextSource = preferences.getBoolean(SettingsPreferenceKeys.AUTO_RETRY_NEXT_SOURCE, defaults.playback.autoRetryNextSource),
                tryBingeGroup = preferences.getBoolean(SettingsPreferenceKeys.TRY_BINGE_GROUP, defaults.playback.tryBingeGroup),
                nextEpisodeThresholdPercent = preferences.getFloat(SettingsPreferenceKeys.NEXT_EPISODE_THRESHOLD_PERCENT, defaults.playback.nextEpisodeThresholdPercent),
                watchedThresholdPercent = preferences.getFloat(SettingsPreferenceKeys.WATCHED_THRESHOLD_PERCENT, defaults.playback.watchedThresholdPercent),
                useSkipSegments = preferences.getBoolean(SettingsPreferenceKeys.USE_SKIP_SEGMENTS, defaults.playback.useSkipSegments),
                introDbApiKey = preferences.getString(SettingsPreferenceKeys.INTRO_DB_API_KEY, defaults.playback.introDbApiKey) ?: defaults.playback.introDbApiKey,
                useChapterSkip = preferences.getBoolean(SettingsPreferenceKeys.USE_CHAPTER_SKIP, defaults.playback.useChapterSkip),
                autoSkipIntro = preferences.getBoolean(SettingsPreferenceKeys.AUTO_SKIP_INTRO, defaults.playback.autoSkipIntro),
                contentWarningsEnabled = preferences.getBoolean(SettingsPreferenceKeys.CONTENT_WARNINGS_ENABLED, defaults.playback.contentWarningsEnabled),
            ),
            subtitles = defaults.subtitles.copy(
                preferredAudioLanguage = preferences.getString(SettingsPreferenceKeys.PREFERRED_AUDIO_LANGUAGE, defaults.subtitles.preferredAudioLanguage) ?: defaults.subtitles.preferredAudioLanguage,
                secondaryAudioLanguage = preferences.getString(SettingsPreferenceKeys.SECONDARY_AUDIO_LANGUAGE, defaults.subtitles.secondaryAudioLanguage) ?: defaults.subtitles.secondaryAudioLanguage,
                preferredSubtitleLanguage = preferences.getString(SettingsPreferenceKeys.PREFERRED_SUBTITLE_LANGUAGE, defaults.subtitles.preferredSubtitleLanguage) ?: defaults.subtitles.preferredSubtitleLanguage,
                secondarySubtitleLanguage = preferences.getString(SettingsPreferenceKeys.SECONDARY_SUBTITLE_LANGUAGE, defaults.subtitles.secondarySubtitleLanguage) ?: defaults.subtitles.secondarySubtitleLanguage,
                autoEnableSubtitles = preferences.getBoolean(SettingsPreferenceKeys.AUTO_ENABLE_SUBTITLES, defaults.subtitles.autoEnableSubtitles),
                subtitleShadow = preferences.getBoolean(SettingsPreferenceKeys.SUBTITLE_SHADOW, defaults.subtitles.subtitleShadow),
                subtitleSize = preferences.getFloat(SettingsPreferenceKeys.SUBTITLE_SIZE, defaults.subtitles.subtitleSize),
                subtitleColorArgb = preferences.getLong(SettingsPreferenceKeys.SUBTITLE_COLOR_ARGB, defaults.subtitles.subtitleColorArgb),
                subtitleTextOpacity = preferences.getFloat(SettingsPreferenceKeys.SUBTITLE_TEXT_OPACITY, defaults.subtitles.subtitleTextOpacity),
                subtitleBackgroundColorArgb = preferences.getLong(SettingsPreferenceKeys.SUBTITLE_BACKGROUND_COLOR_ARGB, defaults.subtitles.subtitleBackgroundColorArgb),
                subtitleBackgroundOpacity = preferences.getFloat(SettingsPreferenceKeys.SUBTITLE_BACKGROUND_OPACITY, defaults.subtitles.subtitleBackgroundOpacity),
                subtitleOutlineColorArgb = preferences.getLong(SettingsPreferenceKeys.SUBTITLE_OUTLINE_COLOR_ARGB, defaults.subtitles.subtitleOutlineColorArgb),
                subtitleOutlineOpacity = preferences.getFloat(SettingsPreferenceKeys.SUBTITLE_OUTLINE_OPACITY, defaults.subtitles.subtitleOutlineOpacity),
            ),
            advanced = defaults.advanced.copy(
                playerBufferCacheMb = preferences.getInt(SettingsPreferenceKeys.PLAYER_BUFFER_CACHE_MB, defaults.advanced.playerBufferCacheMb),
                playerForwardBufferSeconds = preferences.getInt(SettingsPreferenceKeys.PLAYER_FORWARD_BUFFER_SECONDS, defaults.advanced.playerForwardBufferSeconds),
                playerBackBufferSeconds = preferences.getInt(SettingsPreferenceKeys.PLAYER_BACK_BUFFER_SECONDS, defaults.advanced.playerBackBufferSeconds),
                audioDecoderMode = preferences.getString(SettingsPreferenceKeys.AUDIO_DECODER_MODE, defaults.advanced.audioDecoderMode) ?: defaults.advanced.audioDecoderMode,
                tunneledPlayback = preferences.getBoolean(SettingsPreferenceKeys.TUNNELED_PLAYBACK, defaults.advanced.tunneledPlayback),
            ),
            content = defaults.content.copy(
                showHeroSection = preferences.getBoolean(SettingsPreferenceKeys.SHOW_HERO_SECTION, defaults.content.showHeroSection),
            ),
            addons = defaults.addons.copy(
                torrentSpeedPreset = preferences.getString(SettingsPreferenceKeys.TORRENT_SPEED_PRESET, defaults.addons.torrentSpeedPreset) ?: defaults.addons.torrentSpeedPreset,
                torrentWifiOnly = preferences.getBoolean(SettingsPreferenceKeys.TORRENT_WIFI_ONLY, defaults.addons.torrentWifiOnly),
            ),
            downloads = defaults.downloads.copy(
                downloadSourceSelectionMode = preferences.getString(SettingsPreferenceKeys.DOWNLOAD_SOURCE_SELECTION_MODE, defaults.downloads.downloadSourceSelectionMode) ?: defaults.downloads.downloadSourceSelectionMode,
                downloadSourceRegexPattern = preferences.getString(SettingsPreferenceKeys.DOWNLOAD_SOURCE_REGEX_PATTERN, defaults.downloads.downloadSourceRegexPattern) ?: defaults.downloads.downloadSourceRegexPattern,
                downloadSubtitleLanguage = preferences.getString(SettingsPreferenceKeys.DOWNLOAD_SUBTITLE_LANGUAGE, defaults.downloads.downloadSubtitleLanguage) ?: defaults.downloads.downloadSubtitleLanguage,
            ),
            system = defaults.system.copy(
                automaticUpdates = preferences.getBoolean(SettingsPreferenceKeys.AUTOMATIC_UPDATES, defaults.system.automaticUpdates),
                rememberLastProfile = preferences.getBoolean(SettingsPreferenceKeys.REMEMBER_LAST_PROFILE, defaults.system.rememberLastProfile),
                appVersionLabel = appVersionLabel,
            ),
        )
    }
}

object SettingsPreferenceKeys {
    const val LANGUAGE = "language"
    const val START_PAGE = "startPage"
    const val BACKGROUND_PLAYBACK = "backgroundPlayback"
    const val ACCENT_COLOR_ARGB = "accentColorArgb"
    const val AMOLED_MODE = "amoledMode"
    const val LIQUID_GLASS_MODE = "liquidGlassMode"
    const val ANIMATIONS_ENABLED = "animationsEnabled"
    const val FLOATING_BOTTOM_BAR = "floatingBottomBar"
    const val BOTTOM_BAR_LABELS = "bottomBarLabels"
    const val TOP_NAVIGATION_BAR = "topNavigationBar"
    const val CARD_CORNER_PRESET = "cardCornerPreset"
    const val INTERFACE_DENSITY = "interfaceDensity"
    const val POSTER_WIDTH_PRESET = "posterWidthPreset"
    const val CONTINUE_WATCHING_CARD_CORNER_PRESET = "continueWatchingCardCornerPreset"
    const val CONTINUE_WATCHING_INTERFACE_DENSITY = "continueWatchingInterfaceDensity"
    const val CONTINUE_WATCHING_WIDTH_PRESET = "continueWatchingWidthPreset"
    const val POSTER_LANDSCAPE_MODE = "posterLandscapeMode"
    const val POSTER_HIDE_TITLES = "posterHideTitles"
    const val HOME_SEASON_POSTERS_ON_HERO = "homeSeasonPostersOnHero"
    const val TRAILER_ON_HOME_HERO_ENABLED = "trailerOnHomeHeroEnabled"
    const val TRAILER_ON_HOME_HERO_DELAY_SECONDS = "trailerOnHomeHeroDelaySeconds"
    const val CONTINUE_WATCHING_HORIZONTAL = "continueWatchingHorizontal"
    const val CONTINUE_WATCHING_ENABLED = "continueWatchingEnabled"
    const val CONTINUE_WATCHING_HIDE_TITLES = "continueWatchingHideTitles"
    const val CONTINUE_WATCHING_SOURCE = "continueWatchingSource"
    const val UPCOMING_ROW_ENABLED = "upcomingRowEnabled"
    const val DETAIL_SCREEN_STYLE = "detailScreenStyle"
    const val DETAIL_PREFER_CLEARLOGO = "detailPreferClearlogo"
    const val DETAIL_SHOW_EPISODE_DESCRIPTIONS = "detailShowEpisodeDescriptions"
    const val DETAIL_SHOW_CAST = "detailShowCast"
    const val DETAIL_SHOW_RECOMMENDATIONS = "detailShowRecommendations"
    const val DETAIL_COLLAPSING_HERO = "detailCollapsingHero"
    const val TRAILER_ON_DETAIL_HERO_ENABLED = "trailerOnDetailHeroEnabled"
    const val TRAILER_ON_DETAIL_HERO_DELAY_SECONDS = "trailerOnDetailHeroDelaySeconds"
    const val BLUR_UNWATCHED_EPISODES = "blurUnwatchedEpisodes"
    const val DETAIL_SEASON_SELECTOR_MODE = "detailSeasonSelectorMode"
    const val DETAIL_SEASON_POSTERS_ON_HERO = "detailSeasonPostersOnHero"
    const val EPISODE_CARDS_LAYOUT = "episodeCardsLayout"
    const val PREFERRED_PLAYER = "preferredPlayer"
    const val EXTERNAL_PLAYER_TARGET = "externalPlayerTarget"
    const val MPV_CUSTOM_OPTIONS = "mpvCustomOptions"
    const val ANIME_UPSCALING_MODE = "animeUpscalingMode"
    const val ANIME_UPSCALING_QUALITY = "animeUpscalingQuality"
    const val ANIME_UPSCALING_MODE_PRESET = "animeUpscalingModePreset"
    const val ANIME_USE_MPV = "animeUseMpv"
    const val ANIME_PREFER_JAPANESE_AUDIO = "animePreferJapaneseAudio"
    const val PLAYBACK_SPEED = "playbackSpeed"
    const val SEEK_FORWARD_SECONDS = "seekForwardSeconds"
    const val SEEK_BACKWARD_SECONDS = "seekBackwardSeconds"
    const val HOLD_TO_SPEED_ENABLED = "holdToSpeedEnabled"
    const val HOLD_SPEED = "holdSpeed"
    const val STREAM_SOURCE_SELECTION_MODE = "streamSourceSelectionMode"
    const val STREAM_SOURCE_REGEX_PATTERN = "streamSourceRegexPattern"
    const val AUTOPLAY_MODE = "autoplayMode"
    const val AUTO_PLAY_NEXT_EPISODE = "autoPlayNextEpisode"
    const val AUTO_PLAY_COUNTDOWN_SECONDS = "autoPlayCountdownSecs"
    const val AUTO_RETRY_NEXT_SOURCE = "autoRetryNextSource"
    const val TRY_BINGE_GROUP = "tryBingeGroup"
    const val NEXT_EPISODE_THRESHOLD_PERCENT = "nextEpisodeThresholdPercent"
    const val WATCHED_THRESHOLD_PERCENT = "watchedThresholdPercent"
    const val USE_SKIP_SEGMENTS = "useSkipSegments"
    const val INTRO_DB_API_KEY = "introDbApiKey"
    const val USE_CHAPTER_SKIP = "useChapterSkip"
    const val AUTO_SKIP_INTRO = "autoSkipIntro"
    const val CONTENT_WARNINGS_ENABLED = "contentWarningsEnabled"
    const val PREFERRED_AUDIO_LANGUAGE = "preferredAudioLanguage"
    const val SECONDARY_AUDIO_LANGUAGE = "secondaryAudioLanguage"
    const val PREFERRED_SUBTITLE_LANGUAGE = "preferredSubtitleLanguage"
    const val SECONDARY_SUBTITLE_LANGUAGE = "secondarySubtitleLanguage"
    const val AUTO_ENABLE_SUBTITLES = "autoEnableSubtitles"
    const val SUBTITLE_SHADOW = "subtitleShadow"
    const val SUBTITLE_SIZE = "subtitleSize"
    const val SUBTITLE_COLOR_ARGB = "subtitleColorArgb"
    const val SUBTITLE_TEXT_OPACITY = "subtitleTextOpacity"
    const val SUBTITLE_BACKGROUND_COLOR_ARGB = "subtitleBackgroundColorArgb"
    const val SUBTITLE_BACKGROUND_OPACITY = "subtitleBackgroundOpacity"
    const val SUBTITLE_OUTLINE_COLOR_ARGB = "subtitleOutlineColorArgb"
    const val SUBTITLE_OUTLINE_OPACITY = "subtitleOutlineOpacity"
    const val PLAYER_BUFFER_CACHE_MB = "playerBufferCacheMb"
    const val PLAYER_FORWARD_BUFFER_SECONDS = "playerForwardBufferSeconds"
    const val PLAYER_BACK_BUFFER_SECONDS = "playerBackBufferSeconds"
    const val AUDIO_DECODER_MODE = "audioDecoderMode"
    const val TUNNELED_PLAYBACK = "tunneledPlayback"
    const val SHOW_HERO_SECTION = "showHeroSection"
    const val TORRENT_SPEED_PRESET = "torrentSpeedPreset"
    const val TORRENT_WIFI_ONLY = "torrentWifiOnly"
    const val DOWNLOAD_SOURCE_SELECTION_MODE = "downloadSourceSelectionMode"
    const val DOWNLOAD_SOURCE_REGEX_PATTERN = "downloadSourceRegexPattern"
    const val DOWNLOAD_SUBTITLE_LANGUAGE = "downloadSubtitleLanguage"
    const val AUTOMATIC_UPDATES = "automaticUpdates"
    const val REMEMBER_LAST_PROFILE = "rememberLastProfile"
    const val TMDB_API_KEY = "tmdbApiKey"
    const val TMDB_CAST_IMAGES_ENABLED = "tmdbCastImagesEnabled"
    const val TMDB_SIMILAR_RESULTS_ENABLED = "tmdbSimilarResultsEnabled"
    const val TMDB_TRAILERS_ENABLED = "tmdbTrailersEnabled"
    const val TMDB_RECOMMENDATIONS_ENABLED = "tmdbRecommendationsEnabled"
    const val TMDB_COLLECTION_INFO_ENABLED = "tmdbCollectionInfoEnabled"
    const val TMDB_EPISODE_IMAGES_ENABLED = "tmdbEpisodeImagesEnabled"
    const val TMDB_LOGOS_BACKDROPS_ENABLED = "tmdbLogosBackdropsEnabled"
    const val TMDB_RATINGS_ENABLED = "tmdbRatingsEnabled"
    const val TMDB_BASIC_INFO_ENABLED = "tmdbBasicInfoEnabled"
    const val TMDB_DETAILS_ENABLED = "tmdbDetailsEnabled"
    const val TMDB_PRODUCTIONS_ENABLED = "tmdbProductionsEnabled"
    const val TMDB_NETWORKS_ENABLED = "tmdbNetworksEnabled"
    const val NOTIFICATIONS_ENABLED = "notificationsEnabled"
    const val ALERT_NEW_EPISODES = "alertNewEpisodes"
}
