package com.fluxa.app.ui

import android.content.Context
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.ProfilePickerSettingsStore
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
import com.fluxa.app.ui.catalog.DetailViewModel
import com.fluxa.app.ui.catalog.HomeViewModel
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.profile.AndroidProfileDataSource
import com.fluxa.app.ui.settings.AndroidPluginsDataSource
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.data.platform.PlatformSecureStore
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.library.ThirdPartyProviderRepository
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.plugins.PluginRepositoryManager
import com.fluxa.app.shared.feature.localmedia.JvmLocalMediaLibraryService
import com.fluxa.app.shared.feature.localmedia.SmbMediaSourceReader
import com.fluxa.app.shared.feature.localmedia.WebDavMediaSourceReader
import com.fluxa.app.ui.localmedia.AndroidSafMediaSourceReader
import java.io.File

class AndroidFluxaPlatformServices(
    context: Context,
    homeViewModel: HomeViewModel,
    detailViewModel: DetailViewModel,
    profileManager: ProfileManager,
    profilePickerSettingsStore: ProfilePickerSettingsStore,
    activeProfile: () -> UserProfile?,
    onActiveProfileChanged: (UserProfile) -> Unit,
    offlineDownloadManager: OfflineDownloadManager,
    watchlistStore: WatchlistStore,
    repository: StremioRepository,
    addonRepository: AddonRepository,
    secureStore: PlatformSecureStore,
    pluginRepositoryManager: PluginRepositoryManager,
    pluginManager: PluginManager,
    authService: StremioService,
    nuvioImportCoordinator: NuvioAccountImportCoordinator,
    thirdPartyProviderRepository: ThirdPartyProviderRepository,
    appVersionLabel: String,
    deviceType: DeviceType = DeviceType.Mobile,
) : FluxaMobilePlatformServices {
    val localMediaLibrary = JvmLocalMediaLibraryService(
        addonRepository = addonRepository,
        stateFile = File(context.applicationContext.filesDir, "local-media/index.json"),
        sourceReaders = listOf(
            AndroidSafMediaSourceReader(context.applicationContext),
            SmbMediaSourceReader(),
            WebDavMediaSourceReader(),
        ),
        authKey = { activeProfile()?.authKey.orEmpty() },
        localAddons = { homeViewModel.userAddons.value.map { it.transportUrl } },
        language = { activeProfile()?.language ?: "en" },
        secureStore = secureStore,
    )

    override val catalogHomeDataSource = AndroidCatalogHomeDataSource(homeViewModel, activeProfile, profileManager, deviceType)
    override val calendarDataSource = AndroidCalendarDataSource(homeViewModel, activeProfile, deviceType)
    override val searchDataSource = AndroidSearchDataSource(homeViewModel, activeProfile, deviceType)
    override val discoverDataSource = AndroidDiscoverDataSource(homeViewModel, activeProfile, deviceType)
    override val libraryDataSource = AndroidLibraryDataSource(
        homeViewModel = homeViewModel,
        profileManager = profileManager,
        activeProfile = activeProfile,
        onProfileChanged = onActiveProfileChanged,
        offlineDownloadManager = offlineDownloadManager,
        watchlistStore = watchlistStore,
        language = { activeProfile()?.language ?: "en" },
        localMediaLibrary = localMediaLibrary,
        deviceType = deviceType,
    )
    override val detailDataSource = AndroidDetailDataSource(detailViewModel, activeProfile, localMediaLibrary, deviceType)
    override val profileDataSource = AndroidProfileDataSource(profileManager, profilePickerSettingsStore)
    override val addonStoreDataSource = AndroidAddonStoreDataSource(
        repository = repository,
        profileManager = profileManager,
        homeViewModel = homeViewModel,
        activeProfile = activeProfile,
        onProfileChanged = onActiveProfileChanged
    )
    override val pluginsDataSource = AndroidPluginsDataSource(
        pluginRepositoryManager = pluginRepositoryManager,
        pluginManager = pluginManager,
        nuvioCoordinator = nuvioImportCoordinator,
        profileManager = profileManager,
        language = { activeProfile()?.language ?: "en" }
    )
    override val authDataSource = AndroidAuthDataSource(
        authService = authService,
        nuvioCoordinator = nuvioImportCoordinator,
        pluginRepositoryManager = pluginRepositoryManager,
        profileManager = profileManager,
        language = { activeProfile()?.language ?: "en" },
        onAuthenticated = onActiveProfileChanged
    )
    override val settingsDataSource = AndroidSettingsDataSource(
        context = context.applicationContext,
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
        thirdPartyProviderRepository = thirdPartyProviderRepository,
        appVersionLabel = appVersionLabel,
        language = { activeProfile()?.language ?: "en" }
    )

    fun close() = localMediaLibrary.close()
}
