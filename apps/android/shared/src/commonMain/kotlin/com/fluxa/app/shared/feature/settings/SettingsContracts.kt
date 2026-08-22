package com.fluxa.app.shared.feature.settings

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map

enum class SettingsCategory {
    Hub, Account, TmdbFeatures, MdblistApi, Notifications, General, Appearance, AppearanceHome, AppearanceDetail,
    Playback, PlaybackStream, PlaybackSkip, Subtitles, Advanced, Content, Downloads, Developer, DeviceCapabilities,
    AccountStremio, AccountNuvio, AccountTrakt, AccountSimkl, AccountAnilist
}

data class SettingsAccountUiModel(
    val displayName: String = "",
    val email: String = "",
    val stremioEmail: String? = null,
    val nuvioEmail: String? = null,
    val avatarUrl: String? = null,
    val hasStremio: Boolean = false,
    val hasNuvio: Boolean = false,
    val hasTrakt: Boolean = false,
    val hasSimkl: Boolean = false,
    val hasAnilist: Boolean = false,
    val traktUsername: String? = null,
    val simklUsername: String? = null,
    val anilistUsername: String? = null,
    val syncCwSourceOfTruth: String = "",
    val syncCwRanking: String = "last_watched",
    val integrationLibrarySource: String = "local",
    val continueWatchingDays: Int = 0,
    val traktCommentsEnabled: Boolean = false,
    val hasAnySync: Boolean = false,
    val syncFailedProviders: Set<String> = emptySet(),
    val syncingProviders: Set<String> = emptySet(),
    val connectErrors: Map<String, String> = emptyMap(),
    val stremioLastSyncAt: Long = 0L,
    val nuvioLastSyncAt: Long = 0L,
    val traktLastSyncAt: Long = 0L,
    val simklLastSyncAt: Long = 0L,
    val anilistLastSyncAt: Long = 0L,
    val stremioItemCount: Int = 0,
    val stremioContinueWatchingCount: Int = 0,
    val stremioLibraryCount: Int = 0,
    val stremioAddonCount: Int = 0,
    val nuvioItemCount: Int = 0,
    val nuvioContinueWatchingCount: Int = 0,
    val nuvioLibraryCount: Int = 0,
    val nuvioAddonCount: Int = 0,
    val traktItemCount: Int = 0,
    val traktContinueWatchingCount: Int = 0,
    val continueWatchingCount: Int = 0,
    val traktLibraryCount: Int = 0,
    val simklItemCount: Int = 0,
    val simklContinueWatchingCount: Int = 0,
    val simklLibraryCount: Int = 0,
    val anilistItemCount: Int = 0,
    val anilistContinueWatchingCount: Int = 0,
    val anilistLibraryCount: Int = 0,
    val addonCount: Int = 0,
    val tmdbApiKey: String? = null,
    val mdblistApiKey: String? = null,
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
    val themeId: String = "fluxa-dark",
    val themeJson: String = "",
    val amoledMode: Boolean = false,
    val liquidGlassMode: Boolean = false,
    val animationsEnabled: Boolean = true,
    val floatingBottomBar: Boolean = false,
    val bottomBarLabels: Boolean = false,
    val topNavigationBar: Boolean = false
)

data class SettingsAppearanceHomeUiModel(
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
    val continueWatchingSource: String = "local",
    val continueWatchingCardCornerPreset: String = "medium",
    val continueWatchingInterfaceDensity: String = "medium",
    val continueWatchingWidthPreset: String = "medium",
    val upcomingRowEnabled: Boolean = false
)

data class SettingsAppearanceDetailUiModel(
    val detailScreenStyle: String = "cinematic",
    val detailPreferClearlogo: Boolean = true,
    val detailShowEpisodeDescriptions: Boolean = true,
    val detailShowCast: Boolean = true,
    val detailShowRecommendations: Boolean = true,
    val detailCollapsingHero: Boolean = true,
    val trailerOnDetailHeroEnabled: Boolean = false,
    val trailerOnDetailHeroDelaySeconds: Int = 4,
    val blurUnwatchedEpisodes: Boolean = false,
    val detailSeasonSelectorMode: String = "dropdown",
    val detailSeasonPostersOnHero: Boolean = false,
    val episodeCardsLayout: String = "carousel"
)

data class SettingsPlaybackUiModel(
    val preferredPlayer: String = "internal",
    val externalPlayerTarget: String = "",
    val inAppPlayerOptions: List<SettingsChoiceOption> = emptyList(),
    val externalPlayerOptions: List<SettingsChoiceOption> = emptyList(),
    val mpvCustomOptions: String = "",
    val animeUpscalingMode: String = "off",
    val animeUpscalingQuality: String = "anime4k_m",
    val animeUpscalingModePreset: String = "a",
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
    val useSkipSegments: Boolean = false,
    val introDbApiKey: String = "",
    val useChapterSkip: Boolean = false,
    val autoSkipIntro: Boolean = false,
    val contentWarningsEnabled: Boolean = true
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
    val audioProcessingMode: String = "reference",
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
    val rememberLastProfile: Boolean = true,
    val appVersionLabel: String = ""
)

data class SettingsDeveloperUiModel(
    val lastProbeUpdatedAt: String? = null,
    val lastProbeTitle: String? = null,
    val lastProbeUrl: String? = null,
    val technicalInfo: String = ""
)

data class SettingsDeviceCapabilitiesUiModel(
    val deviceName: String = "",
    val platformVersion: String = "",
    val audioRoute: String = "",
    val audioPcmChannels: String = "",
    val audioPcmSampleRates: String = "",
    val audioPassthroughSupported: List<String> = emptyList(),
    val audioPassthroughNotDetected: List<String> = emptyList(),
    val audioFallback: String = "",
    val videoHardwareDecoders: List<String> = emptyList(),
    val videoSoftwareDecoders: List<String> = emptyList(),
    val videoNotDetected: List<String> = emptyList(),
    val hdrOutput: List<String> = emptyList(),
    val hdrNotDetected: List<String> = emptyList(),
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
    val deviceCapabilities: SettingsDeviceCapabilitiesUiModel = SettingsDeviceCapabilitiesUiModel(),
    val isLoading: Boolean = false
)

sealed interface SettingsAction {
    data class GeneralChanged(val value: SettingsGeneralUiModel) : SettingsAction
    data class AppearanceChanged(val value: SettingsAppearanceUiModel) : SettingsAction
    data class ThemeImported(val rawJson: String) : SettingsAction
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
    data object ManagePluginsRequested : SettingsAction
    data object ManageStreamBadgesRequested : SettingsAction
    data object ConnectStremioRequested : SettingsAction
    data object ConnectNuvioRequested : SettingsAction
    data class ConnectStremioWithCredentials(val email: String, val password: String) : SettingsAction
    data class ConnectNuvioWithCredentials(val email: String, val password: String) : SettingsAction
    data object ConnectTraktRequested : SettingsAction
    data object ConnectSimklRequested : SettingsAction
    data class SyncProviderRequested(val provider: String) : SettingsAction
    data object ConnectAnilistRequested : SettingsAction
    data object DisconnectSyncRequested : SettingsAction
    data class DisconnectProviderRequested(val provider: String) : SettingsAction
    data object SwitchProfilesRequested : SettingsAction
    data object CheckForUpdateRequested : SettingsAction
}

interface SettingsDataSource {
    fun observeSettings(): Flow<SettingsUiState>

    /** Lightweight global chrome subscription. Platforms can override this to avoid building the
     * complete Settings model when only appearance/navigation values are needed. */
    fun observeAppearance(): Flow<SettingsAppearanceUiModel> = observeSettings()
        .map { it.appearance }
        .distinctUntilChanged()

    /** Lightweight Home-only appearance subscription (Continue Watching labels/layout, hero flags, etc.). */
    fun observeAppearanceHome(): Flow<SettingsAppearanceHomeUiModel> = observeSettings()
        .map { it.appearanceHome }
        .distinctUntilChanged()

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
    suspend fun disconnectProvider(provider: String)
}
