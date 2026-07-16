package com.fluxa.app.ui

import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistStore
import com.fluxa.app.shared.platform.FluxaMobilePlatformServices
import com.fluxa.app.ui.catalog.AndroidAddonStoreDataSource
import com.fluxa.app.ui.catalog.AndroidAuthDataSource
import com.fluxa.app.ui.catalog.AndroidCatalogHomeDataSource
import com.fluxa.app.ui.catalog.AndroidCalendarDataSource
import com.fluxa.app.ui.catalog.AndroidDetailDataSource
import com.fluxa.app.ui.catalog.AndroidDiscoverDataSource
import com.fluxa.app.ui.catalog.AndroidLibraryDataSource
import com.fluxa.app.ui.catalog.AndroidSearchDataSource
import com.fluxa.app.ui.catalog.AndroidSettingsDataSource
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
    offlineDownloadManager: OfflineDownloadManager,
    watchlistStore: WatchlistStore,
    appVersionLabel: String
) : FluxaMobilePlatformServices {
    override val catalogHomeDataSource = AndroidCatalogHomeDataSource(homeViewModel, activeProfile)
    override val calendarDataSource = AndroidCalendarDataSource(homeViewModel, activeProfile)
    override val searchDataSource = AndroidSearchDataSource(homeViewModel, activeProfile)
    override val discoverDataSource = AndroidDiscoverDataSource(homeViewModel, activeProfile)
    override val libraryDataSource = AndroidLibraryDataSource(
        homeViewModel = homeViewModel,
        profileManager = profileManager,
        activeProfile = activeProfile,
        onProfileChanged = onActiveProfileChanged,
        offlineDownloadManager = offlineDownloadManager,
        watchlistStore = watchlistStore,
        language = { activeProfile()?.language ?: "en" }
    )
    override val detailDataSource = AndroidDetailDataSource(detailViewModel, activeProfile)
    override val profileDataSource = AndroidProfileDataSource(profileManager)
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
    override val settingsDataSource = AndroidSettingsDataSource(
        homeViewModel = homeViewModel,
        profileManager = profileManager,
        activeProfile = activeProfile,
        onProfileChanged = { updated ->
            val reloadHome = activeProfile().requiresHomeReload(updated)
            onActiveProfileChanged(updated)
            homeViewModel.applyUpdatedProfile(updated, refreshHomeSideEffects = reloadHome)
            if (reloadHome) {
                homeViewModel.loadInitialData(updated, force = true)
            }
        },
        offlineDownloadManager = offlineDownloadManager,
        appVersionLabel = appVersionLabel,
        language = { activeProfile()?.language ?: "en" }
    )
}
