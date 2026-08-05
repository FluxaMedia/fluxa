package com.fluxa.app.desktop

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import com.fluxa.app.data.local.AndroidWatchlistStore
import com.fluxa.app.data.local.ProfileCredentialStore
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.local.WatchlistStore
import com.fluxa.app.data.local.buildDesktopAppDatabase
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.data.repository.AddonPersistentCache
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.data.repository.DesktopPlatformFileStore
import com.fluxa.app.data.repository.DesktopPlatformKeyValueStore
import com.fluxa.app.data.repository.DesktopPlatformSecureStore
import com.fluxa.app.data.repository.HttpEffectExecutor
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.repository.RepositoryMemoryCache
import com.fluxa.app.data.repository.StremioAddonManifestClient
import com.fluxa.app.data.repository.StremioAddonResourceClient
import com.fluxa.app.desktop.addonstore.DesktopAddonRegistry
import com.fluxa.app.desktop.addonstore.DesktopAddonStoreDataSource
import com.fluxa.app.desktop.auth.DesktopAuthDataSource
import com.fluxa.app.desktop.auth.DesktopTraktAuthCoordinator
import com.fluxa.app.desktop.auth.buildDesktopNuvioService
import com.fluxa.app.desktop.detail.DesktopDetailDataSource
import com.fluxa.app.desktop.home.DesktopCatalogHomeDataSource
import com.fluxa.app.desktop.home.DesktopHomeCoordinator
import com.fluxa.app.desktop.library.DesktopLibraryDataSource
import com.fluxa.app.desktop.profile.DesktopProfileDataSource
import com.fluxa.app.desktop.search.DesktopSearchDataSource
import com.fluxa.app.desktop.settings.DesktopSettingsDataSource
import com.fluxa.app.shared.FluxaAppHost
import com.fluxa.app.shared.FluxaDestination
import com.fluxa.app.ui.catalog.DeviceType
import com.google.gson.Gson
import java.io.File

private const val DESKTOP_DEFAULT_PROFILE_ID = "desktop-default"

private fun buildDesktopAddonRepository(): AddonRepository {
    val gson = Gson()
    val cacheDir = File(System.getProperty("user.home"), ".fluxa/cache")
    val persistentCache = AddonPersistentCache(DesktopPlatformFileStore(cacheDir), gson)
    val memoryCache = RepositoryMemoryCache(gson)
    val httpEffectExecutor = HttpEffectExecutor()
    val httpClient = StremioService.sharedClient
    val manifestClient = StremioAddonManifestClient(memoryCache, persistentCache, httpEffectExecutor, httpClient)
    val resourceClient = StremioAddonResourceClient(StremioService.create(), memoryCache, persistentCache, manifestClient, httpEffectExecutor, httpClient)
    return AddonRepository(manifestClient, resourceClient)
}

private fun buildDesktopWatchlistManager(): WatchlistManager {
    val databaseFile = File(System.getProperty("user.home"), ".fluxa/fluxa.db")
    val database = buildDesktopAppDatabase(databaseFile)
    val manager = WatchlistManager(database.watchlistDao())
    manager.setActiveProfile(DESKTOP_DEFAULT_PROFILE_ID)
    return manager
}

private fun buildDesktopProfileManager(): ProfileManager {
    val gson = Gson()
    val secretsFile = File(System.getProperty("user.home"), ".fluxa/profile_secrets.properties")
    val prefsFile = File(System.getProperty("user.home"), ".fluxa/profile_prefs.properties")
    val credentialStore = ProfileCredentialStore(DesktopPlatformSecureStore(secretsFile), gson)
    return ProfileManager(DesktopPlatformKeyValueStore(prefsFile), gson, credentialStore)
}

fun main() = application {
    val addonRepository = remember { buildDesktopAddonRepository() }
    val addonRegistry = remember { DesktopAddonRegistry() }
    val watchlistManager = remember { buildDesktopWatchlistManager() }
    val watchlistStore: WatchlistStore = remember { AndroidWatchlistStore(watchlistManager) }
    val profileManager = remember { buildDesktopProfileManager() }
    val nuvioCoordinator = remember {
        NuvioAccountImportCoordinator(
            nuvioService = buildDesktopNuvioService(),
            profileManager = profileManager,
            watchlistManager = watchlistManager,
            addonRepository = addonRepository,
            supabaseUrl = com.fluxa.app.data.PlatformSecrets.nuvioSupabaseUrl,
            gson = Gson()
        )
    }
    val traktAuthCoordinator = remember { DesktopTraktAuthCoordinator(profileManager) {} }
    val catalogHomeDataSource = remember { DesktopCatalogHomeDataSource(DesktopHomeCoordinator(addonRepository)) }
    val searchDataSource = remember { DesktopSearchDataSource(addonRepository, addonRegistry) }
    val detailDataSource = remember { DesktopDetailDataSource(addonRepository, watchlistStore, addonRegistry) }
    val libraryDataSource = remember { DesktopLibraryDataSource(watchlistStore) }
    val settingsDataSource = remember { DesktopSettingsDataSource(profileManager, addonRegistry) }
    val addonStoreDataSource = remember { DesktopAddonStoreDataSource(addonRepository, addonRegistry) }
    val profileDataSource = remember { DesktopProfileDataSource(profileManager) }
    val authDataSource = remember {
        DesktopAuthDataSource(
            authService = StremioService.create(),
            nuvioCoordinator = nuvioCoordinator,
            profileManager = profileManager,
            onAuthenticated = { profile -> watchlistManager.setActiveProfile(profile.id) }
        )
    }
    var currentDestination by remember { mutableStateOf<FluxaDestination?>(null) }
    val coroutineScope = rememberCoroutineScope()
    Window(
        onCloseRequest = ::exitApplication,
        title = "Fluxa",
        state = rememberWindowState(width = 1280.dp, height = 800.dp)
    ) {
        FluxaAppHost(
            catalogHomeDataSource = catalogHomeDataSource,
            searchDataSource = searchDataSource,
            detailDataSource = detailDataSource,
            libraryDataSource = libraryDataSource,
            settingsDataSource = settingsDataSource,
            addonStoreDataSource = addonStoreDataSource,
            profileDataSource = profileDataSource,
            authDataSource = authDataSource,
            deviceType = DeviceType.Desktop,
            showNavigationBar = true,
            destination = currentDestination,
            onDestinationChanged = { currentDestination = it },
            onManageAddonsRequested = { currentDestination = FluxaDestination.AddonStore },
            onAddonStoreBackRequested = { currentDestination = FluxaDestination.Settings },
            onAuthBackRequested = { currentDestination = FluxaDestination.Settings },
            onAuthCompleted = { currentDestination = FluxaDestination.Settings },
            onProfileSelectionCompleted = { profileId -> watchlistManager.setActiveProfile(profileId) },
            onConnectStremioRequested = {
                val activeId = profileManager.getLastActiveProfileId()
                val active = profileManager.getProfiles().firstOrNull { it.id == activeId }
                if (active?.authKey.isNullOrBlank()) {
                    currentDestination = FluxaDestination.Auth
                }
            },
            onConnectNuvioRequested = {
                val activeId = profileManager.getLastActiveProfileId()
                val active = profileManager.getProfiles().firstOrNull { it.id == activeId }
                if (active?.nuvioAccessToken.isNullOrBlank()) {
                    currentDestination = FluxaDestination.Auth
                }
            },
            onConnectTraktRequested = {
                val activeId = profileManager.getLastActiveProfileId()
                if (activeId != null) {
                    traktAuthCoordinator.connect(activeId, coroutineScope)
                }
            },
            modifier = Modifier.fillMaxSize()
        )
    }
}
