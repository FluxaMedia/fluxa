package com.fluxa.app.desktop

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import com.fluxa.app.core.rust.FluxaDesktopHeadlessEnvironment
import com.fluxa.app.core.rust.FluxaHeadlessRuntimeFactory
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.data.repository.AddonPersistentCache
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.data.repository.DesktopPlatformFileStore
import com.fluxa.app.data.repository.HttpEffectExecutor
import com.fluxa.app.data.repository.RepositoryMemoryCache
import com.fluxa.app.data.repository.StremioAddonManifestClient
import com.fluxa.app.data.repository.StremioAddonResourceClient
import com.google.gson.Gson
import java.io.File
import kotlinx.coroutines.runBlocking

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

private fun runCatalogFetchSmokeTest() {
    val addonRepository = buildDesktopAddonRepository()
    val environment = FluxaDesktopHeadlessEnvironment(addonRepository)
    val runtime = FluxaHeadlessRuntimeFactory.createJni(environment)
    runBlocking {
        val result = runtime.dispatch(
            mapOf(
                "type" to "catalogPageRequested",
                "categoryId" to "cinemeta-top-movies",
                "transportUrl" to "https://v3-cinemeta.strem.io/manifest.json",
                "contentType" to "movie",
                "catalogId" to "top",
                "skip" to 0,
                "genre" to null,
                "search" to null,
                "remoteSource" to emptyList<Any>(),
                "profile" to null
            )
        )
        println("Desktop catalog fetch smoke test state: ${result.state}")
    }
    runtime.close()
}

fun main() {
    runCatalogFetchSmokeTest()
    application {
        Window(onCloseRequest = ::exitApplication, title = "Fluxa") {
            FluxaDesktopApp()
        }
    }
}

@Composable
private fun FluxaDesktopApp() {
    MaterialTheme {
        Surface(modifier = Modifier.fillMaxSize()) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text("Fluxa Desktop")
            }
        }
    }
}
