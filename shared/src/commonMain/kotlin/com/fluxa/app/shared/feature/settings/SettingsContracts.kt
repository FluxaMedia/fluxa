package com.fluxa.app.shared.feature.settings

import kotlinx.coroutines.flow.Flow

enum class SettingsCategory {
    Hub, Account, TmdbFeatures, Notifications, General, Appearance, AppearanceHome, AppearanceDetail,
    Playback, Subtitles, Advanced, Content, Downloads, Developer
}

data class SettingsAccountUiModel(
    val displayName: String = "",
    val email: String = "",
    val nuvioEmail: String? = null,
    val avatarUrl: String? = null,
    val isGuest: Boolean = true,
    val hasStremio: Boolean = false,
    val hasNuvio: Boolean = false,
    val hasTrakt: Boolean = false,
    val hasSimkl: Boolean = false,
    val hasAnilist: Boolean = false,
    val hasAnySync: Boolean = false,
    val syncFailedProviders: Set<String> = emptySet(),
    val syncingProviders: Set<String> = emptySet(),
    val nuvioLastSyncAt: Long = 0L,
    val traktLastSyncAt: Long = 0L,
    val simklLastSyncAt: Long = 0L,
    val traktItemCount: Int = 0,
    val traktContinueWatchingCount: Int = 0,
    val continueWatchingCount: Int = 0,
    val traktLibraryCount: Int = 0,
    val addonCount: Int = 0,
    val tmdbApiKey: String? = null,
    val tmdbCastImagesEnabled: Boolean = true,
    val tmdbSimilarResultsEnabled: Boolean = true,
    val tmdbTrailersEnabled: Boolean = true,
    val tmdbRecommendationsEnabled: Boolean = true,
    val tmdbCollectionInfoEnabled: Boolean = true,
    val tmdbEpisodeImagesEnabled: Boolean = true,
    val tmdbLogosBackdropsEnabled: Boolean = true,
    val tmdbRatingsEnabled: Boolean = true,
    val tmdbBasicInfoEnabled: Boolean = true,
    val tmdbDetailsEnabled: Boolean = true,
    val tmdbProductionsEnabled: Boolean = true,
    val tmdbNetworksEnabled: Boolean = true
)

data class SettingsNotificationsUiModel(
    val notificationsEnabled: Boolean = true,
    val alertNewEpisodes: Boolean = true
)

data class SettingsGeneralUiModel(
    val language: String = "en",
    val startPage: String = "home",
    val backgroundPlayback: Boolean = true
)

data class SettingsAppearanceUiModel(
    val accentColorArgb: Long = 0xFFFFFFFFL,
    val amoledMode: Boolean = false,
    val animationsEnabled: Boolean = true,
    val floatingBottomBar: Boolean = false,
    val bottomBarLabels: Boolean = false
)

data class SettingsAppearanceHomeUiModel(
    val topBarEnabled: Boolean = true,
    val cardCornerPreset: String = "medium",
    val interfaceDensity: String = "normal",
    val posterWidthPreset: String = "medium",
    val posterLandscapeMode: Boolean = false,
    val posterHideTitles: Boolean = false,
    val homeSeasonPostersOnHero: Boolean = false,
    val trailerOnHomeHeroEnabled: Boolean = false,
    val trailerOnHomeHeroDelaySeconds: Int = 3,
    val continueWatchingHorizontal: Boolean = true,
    val continueWatchingEnabled: Boolean = true,
    val continueWatchingHideTitles: Boolean = false,
    val continueWatchingSource: String = "fluxa"
)

data class SettingsAppearanceDetailUiModel(
    val trailerOnHero: Boolean = false,
    val blurUnwatchedEpisodes: Boolean = false,
    val detailSeasonSelectorMode: String = "dropdown",
    val detailSeasonPostersOnHero: Boolean = false,
    val episodeCardsLayout: String = "modern"
)

data class SettingsPlaybackUiModel(
    val preferredPlayer: String = "internal",
    val mpvCustomOptions: String = "",
    val animeUseMpv: Boolean = false,
    val animePreferJapaneseAudio: Boolean = false,
    val playbackSpeed: Float = 1f,
    val seekForwardSeconds: Int = 10,
    val seekBackwardSeconds: Int = 10,
    val holdToSpeedEnabled: Boolean = true,
    val holdSpeed: Float = 2f,
    val streamSourceSelectionMode: String = "auto",
    val streamSourceRegexPattern: String = "",
    val autoplayMode: String = "off",
    val autoPlayNextEpisode: Boolean = true,
    val autoPlayCountdownSecs: Int = 10,
    val autoRetryNextSource: Boolean = false,
    val tryBingeGroup: Boolean = true,
    val nextEpisodeThresholdPercent: Float = 90f,
    val watchedThresholdPercent: Float = 90f,
    val useIntroDb: Boolean = false,
    val introDbApiKey: String = "",
    val useAniSkip: Boolean = false,
    val useChapterSkip: Boolean = false,
    val autoSkipIntro: Boolean = false
)

data class SettingsSubtitlesUiModel(
    val preferredAudioLanguage: String = "none",
    val secondaryAudioLanguage: String = "none",
    val preferredSubtitleLanguage: String = "none",
    val secondarySubtitleLanguage: String = "none",
    val autoEnableSubtitles: Boolean = false,
    val subtitleShadow: Boolean = true,
    val subtitleSize: Float = 100f,
    val subtitleColorArgb: Long = 0xFFFFFFFFL,
    val subtitleTextOpacity: Float = 1f,
    val subtitleBackgroundColorArgb: Long = 0x80000000L,
    val subtitleBackgroundOpacity: Float = 0.5f,
    val subtitleOutlineColorArgb: Long = 0xFF000000L,
    val subtitleOutlineOpacity: Float = 1f
)

data class SettingsAdvancedUiModel(
    val playerBufferCacheMb: Int = 100,
    val playerForwardBufferSeconds: Int = 30,
    val playerBackBufferSeconds: Int = 30,
    val audioDecoderMode: String = "hw_prefer",
    val tunneledPlayback: Boolean = false
)

data class SettingsFeedItemUiModel(
    val key: String,
    val label: String,
    val providerLabel: String,
    val selected: Boolean,
    val canMoveUp: Boolean = false,
    val canMoveDown: Boolean = false
)

data class SettingsContentUiModel(
    val showHeroSection: Boolean = true,
    val heroFeeds: List<SettingsFeedItemUiModel> = emptyList(),
    val homeFeeds: List<SettingsFeedItemUiModel> = emptyList(),
    val topTenFeeds: List<SettingsFeedItemUiModel> = emptyList(),
    val heroSelectionLimit: Int = 2
)

data class SettingsAddonsUiModel(
    val torrentSpeedPreset: String = "balanced",
    val torrentWifiOnly: Boolean = false
)

data class SettingsDownloadsUiModel(
    val downloadSourceSelectionMode: String = "auto",
    val downloadSourceRegexPattern: String = "",
    val downloadSubtitleLanguage: String = "none"
)

data class SettingsSystemUiModel(
    val automaticUpdates: Boolean = true,
    val appVersionLabel: String = ""
)

data class SettingsDeveloperUiModel(
    val lastProbeUpdatedAt: String? = null,
    val lastProbeTitle: String? = null,
    val lastProbeUrl: String? = null,
    val technicalInfo: String = ""
)

data class SettingsUiState(
    val account: SettingsAccountUiModel = SettingsAccountUiModel(),
    val notifications: SettingsNotificationsUiModel = SettingsNotificationsUiModel(),
    val general: SettingsGeneralUiModel = SettingsGeneralUiModel(),
    val appearance: SettingsAppearanceUiModel = SettingsAppearanceUiModel(),
    val appearanceHome: SettingsAppearanceHomeUiModel = SettingsAppearanceHomeUiModel(),
    val appearanceDetail: SettingsAppearanceDetailUiModel = SettingsAppearanceDetailUiModel(),
    val playback: SettingsPlaybackUiModel = SettingsPlaybackUiModel(),
    val subtitles: SettingsSubtitlesUiModel = SettingsSubtitlesUiModel(),
    val advanced: SettingsAdvancedUiModel = SettingsAdvancedUiModel(),
    val content: SettingsContentUiModel = SettingsContentUiModel(),
    val addons: SettingsAddonsUiModel = SettingsAddonsUiModel(),
    val downloads: SettingsDownloadsUiModel = SettingsDownloadsUiModel(),
    val system: SettingsSystemUiModel = SettingsSystemUiModel(),
    val developer: SettingsDeveloperUiModel = SettingsDeveloperUiModel(),
    val isLoading: Boolean = false
)

sealed interface SettingsAction {
    data class GeneralChanged(val value: SettingsGeneralUiModel) : SettingsAction
    data class AppearanceChanged(val value: SettingsAppearanceUiModel) : SettingsAction
    data class AppearanceHomeChanged(val value: SettingsAppearanceHomeUiModel) : SettingsAction
    data class AppearanceDetailChanged(val value: SettingsAppearanceDetailUiModel) : SettingsAction
    data class PlaybackChanged(val value: SettingsPlaybackUiModel) : SettingsAction
    data class SubtitlesChanged(val value: SettingsSubtitlesUiModel) : SettingsAction
    data class AdvancedChanged(val value: SettingsAdvancedUiModel) : SettingsAction
    data class ShowHeroSectionChanged(val value: Boolean) : SettingsAction
    data class AddonsChanged(val value: SettingsAddonsUiModel) : SettingsAction
    data class DownloadsChanged(val value: SettingsDownloadsUiModel) : SettingsAction
    data class SystemChanged(val value: SettingsSystemUiModel) : SettingsAction
    data class TmdbAccountChanged(val value: SettingsAccountUiModel) : SettingsAction
    data class NotificationsChanged(val value: SettingsNotificationsUiModel) : SettingsAction
    data class HeroFeedToggled(val key: String) : SettingsAction
    data class HeroFeedMoved(val key: String, val direction: Int) : SettingsAction
    data class HomeFeedToggled(val key: String) : SettingsAction
    data class HomeFeedMoved(val key: String, val direction: Int) : SettingsAction
    data class TopTenFeedToggled(val key: String) : SettingsAction
    data object ManageAddonsRequested : SettingsAction
    data object ConnectStremioRequested : SettingsAction
    data object ConnectNuvioRequested : SettingsAction
    data object ConnectTraktRequested : SettingsAction
    data object ConnectSimklRequested : SettingsAction
    data object ConnectAnilistRequested : SettingsAction
    data object DisconnectSyncRequested : SettingsAction
    data object SwitchProfilesRequested : SettingsAction
    data object CheckForUpdateRequested : SettingsAction
}

interface SettingsDataSource {
    fun observeSettings(): Flow<SettingsUiState>
    suspend fun refreshContentFeeds()
    suspend fun updateGeneral(value: SettingsGeneralUiModel)
    suspend fun updateAppearance(value: SettingsAppearanceUiModel)
    suspend fun updateAppearanceHome(value: SettingsAppearanceHomeUiModel)
    suspend fun updateAppearanceDetail(value: SettingsAppearanceDetailUiModel)
    suspend fun updatePlayback(value: SettingsPlaybackUiModel)
    suspend fun updateSubtitles(value: SettingsSubtitlesUiModel)
    suspend fun updateAdvanced(value: SettingsAdvancedUiModel)
    suspend fun updateAddons(value: SettingsAddonsUiModel)
    suspend fun updateDownloads(value: SettingsDownloadsUiModel)
    suspend fun updateSystem(value: SettingsSystemUiModel)
    suspend fun updateTmdbAccount(value: SettingsAccountUiModel)
    suspend fun updateNotifications(value: SettingsNotificationsUiModel)
    suspend fun updateShowHeroSection(value: Boolean)
    suspend fun toggleHeroFeed(key: String)
    suspend fun moveHeroFeed(key: String, direction: Int)
    suspend fun toggleHomeFeed(key: String)
    suspend fun moveHomeFeed(key: String, direction: Int)
    suspend fun toggleTopTenFeed(key: String)
    suspend fun disconnectSync()
}
