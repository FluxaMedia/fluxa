package com.fluxa.app.shared.feature.settings

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn

class SettingsStore(
    private val dataSource: SettingsDataSource,
    scope: CoroutineScope
) {
    val state: StateFlow<SettingsUiState> = dataSource.observeSettings()
        .stateIn(scope, SharingStarted.WhileSubscribed(5_000), SettingsUiState(isLoading = true))

    suspend fun refreshContentFeeds() = dataSource.refreshContentFeeds()

    suspend fun dispatch(action: SettingsAction) {
        when (action) {
            is SettingsAction.GeneralChanged -> dataSource.updateGeneral(action.value)
            is SettingsAction.AppearanceChanged -> dataSource.updateAppearance(action.value)
            is SettingsAction.AppearanceHomeChanged -> dataSource.updateAppearanceHome(action.value)
            is SettingsAction.AppearanceDetailChanged -> dataSource.updateAppearanceDetail(action.value)
            is SettingsAction.PlaybackChanged -> dataSource.updatePlayback(action.value)
            is SettingsAction.SubtitlesChanged -> dataSource.updateSubtitles(action.value)
            is SettingsAction.AdvancedChanged -> dataSource.updateAdvanced(action.value)
            is SettingsAction.AddonsChanged -> dataSource.updateAddons(action.value)
            is SettingsAction.DownloadsChanged -> dataSource.updateDownloads(action.value)
            is SettingsAction.SystemChanged -> dataSource.updateSystem(action.value)
            is SettingsAction.TmdbAccountChanged -> dataSource.updateTmdbAccount(action.value)
            is SettingsAction.NotificationsChanged -> dataSource.updateNotifications(action.value)
            is SettingsAction.ShowHeroSectionChanged -> dataSource.updateShowHeroSection(action.value)
            is SettingsAction.HeroFeedToggled -> dataSource.toggleHeroFeed(action.key)
            is SettingsAction.HeroFeedMoved -> dataSource.moveHeroFeed(action.key, action.direction)
            is SettingsAction.HomeFeedToggled -> dataSource.toggleHomeFeed(action.key)
            is SettingsAction.HomeFeedMoved -> dataSource.moveHomeFeed(action.key, action.direction)
            is SettingsAction.TopTenFeedToggled -> dataSource.toggleTopTenFeed(action.key)
            SettingsAction.ManageAddonsRequested -> Unit
            SettingsAction.ConnectStremioRequested -> Unit
            SettingsAction.ConnectNuvioRequested -> Unit
            SettingsAction.ConnectTraktRequested -> Unit
            SettingsAction.ConnectSimklRequested -> Unit
            SettingsAction.ConnectAnilistRequested -> Unit
            SettingsAction.DisconnectSyncRequested -> dataSource.disconnectSync()
            SettingsAction.SwitchProfilesRequested -> Unit
            SettingsAction.CheckForUpdateRequested -> Unit
            SettingsAction.ManageDownloadsRequested -> Unit
        }
    }
}
