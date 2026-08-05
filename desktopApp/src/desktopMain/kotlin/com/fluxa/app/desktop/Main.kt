package com.fluxa.app.desktop

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.data.repository.AddonPersistentCache
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.data.repository.DesktopPlatformFileStore
import com.fluxa.app.data.repository.HttpEffectExecutor
import com.fluxa.app.data.repository.RepositoryMemoryCache
import com.fluxa.app.data.repository.StremioAddonManifestClient
import com.fluxa.app.data.repository.StremioAddonResourceClient
import com.fluxa.app.desktop.home.DesktopCatalogHomeDataSource
import com.fluxa.app.desktop.home.DesktopHomeCoordinator
import com.fluxa.app.desktop.search.DesktopSearchDataSource
import com.fluxa.app.shared.FluxaAppHost
import com.fluxa.app.ui.catalog.DeviceType
import com.google.gson.Gson
import java.io.File

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

fun main() = application {
    val addonRepository = remember { buildDesktopAddonRepository() }
    val catalogHomeDataSource = remember { DesktopCatalogHomeDataSource(DesktopHomeCoordinator(addonRepository)) }
    val searchDataSource = remember { DesktopSearchDataSource(addonRepository) }
    Window(
        onCloseRequest = ::exitApplication,
        title = "Fluxa",
        state = rememberWindowState(width = 1280.dp, height = 800.dp)
    ) {
        FluxaAppHost(
            catalogHomeDataSource = catalogHomeDataSource,
            searchDataSource = searchDataSource,
            deviceType = DeviceType.Desktop,
            showNavigationBar = true,
            modifier = Modifier.fillMaxSize()
        )
    }
}
