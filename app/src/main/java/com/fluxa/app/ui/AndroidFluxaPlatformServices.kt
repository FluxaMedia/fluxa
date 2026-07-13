package com.fluxa.app.ui

import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.player.AndroidPlaybackController
import com.fluxa.app.shared.feature.player.PlayerContentUiModel
import com.fluxa.app.shared.platform.FluxaMobilePlatformServices
import com.fluxa.app.ui.catalog.AndroidAddonStoreDataSource
import com.fluxa.app.ui.catalog.AndroidAuthDataSource
import com.fluxa.app.ui.catalog.AndroidCatalogHomeDataSource
import com.fluxa.app.ui.catalog.AndroidCalendarDataSource
import com.fluxa.app.ui.catalog.AndroidDetailDataSource
import com.fluxa.app.ui.catalog.AndroidDiscoverDataSource
import com.fluxa.app.ui.catalog.AndroidLibraryDataSource
import com.fluxa.app.ui.catalog.AndroidSearchDataSource
import com.fluxa.app.ui.catalog.AppContainer
import com.fluxa.app.ui.catalog.DetailViewModel
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.profile.AndroidProfileDataSource

class AndroidFluxaPlatformServices(
    homeViewModel: HomeViewModel,
    detailViewModel: DetailViewModel,
    profileManager: ProfileManager,
    activeProfile: () -> UserProfile?,
    onActiveProfileChanged: (UserProfile) -> Unit,
    player: ExoPlayer,
    playerContent: () -> PlayerContentUiModel?
) : FluxaMobilePlatformServices {
    override val catalogHomeDataSource = AndroidCatalogHomeDataSource(homeViewModel, activeProfile)
    override val calendarDataSource = AndroidCalendarDataSource(homeViewModel, activeProfile)
    override val searchDataSource = AndroidSearchDataSource(homeViewModel, activeProfile)
    override val discoverDataSource = AndroidDiscoverDataSource(homeViewModel, activeProfile)
    override val libraryDataSource = AndroidLibraryDataSource(homeViewModel, activeProfile)
    override val detailDataSource = AndroidDetailDataSource(detailViewModel, activeProfile)
    override val profileDataSource = AndroidProfileDataSource(profileManager)
    override val playbackController = AndroidPlaybackController(player, playerContent)
    override val addonStoreDataSource = AndroidAddonStoreDataSource(
        pluginManager = AppContainer.pluginManager,
        repository = AppContainer.repository,
        profileManager = profileManager,
        homeViewModel = homeViewModel,
        activeProfile = activeProfile,
        onProfileChanged = onActiveProfileChanged
    )
    override val authDataSource = AndroidAuthDataSource(
        authService = AppContainer.authService,
        nuvioCoordinator = AppContainer.nuvioImportCoordinator,
        profileManager = profileManager,
        language = { activeProfile()?.language ?: "en" },
        onAuthenticated = onActiveProfileChanged
    )
}
