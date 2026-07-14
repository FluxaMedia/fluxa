package com.fluxa.app.shared

import androidx.compose.runtime.Composable
import androidx.compose.ui.window.ComposeUIViewController
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.platform.AppleAddonStoreDataSource
import com.fluxa.app.shared.platform.AppleAuthDataSource
import com.fluxa.app.shared.platform.AppleCatalogHomeDataSource
import com.fluxa.app.shared.platform.AppleCalendarDataSource
import com.fluxa.app.shared.platform.AppleDetailDataSource
import com.fluxa.app.shared.platform.AppleDiscoverDataSource
import com.fluxa.app.shared.platform.AppleFluxaPlatformServices
import com.fluxa.app.shared.platform.AppleLibraryDataSource
import com.fluxa.app.shared.platform.AppleProfileDataSource
import com.fluxa.app.shared.platform.AppleSearchDataSource
import com.fluxa.app.shared.platform.AppleSettingsDataSource
import platform.UIKit.UIViewController

object FluxaApple {
    internal val catalogHomeDataSource = AppleCatalogHomeDataSource()
    internal val detailDataSource = AppleDetailDataSource()
    internal val searchDataSource = AppleSearchDataSource()
    internal val discoverDataSource = AppleDiscoverDataSource()
    internal val calendarDataSource = AppleCalendarDataSource()
    internal val libraryDataSource = AppleLibraryDataSource()
    internal val profileDataSource = AppleProfileDataSource()
    internal val settingsDataSource = AppleSettingsDataSource()
    internal val addonStoreDataSource = AppleAddonStoreDataSource()
    internal val authDataSource = AppleAuthDataSource()
    internal val platformServices = AppleFluxaPlatformServices(
        catalogHomeDataSource,
        detailDataSource,
        searchDataSource,
        discoverDataSource,
        calendarDataSource,
        libraryDataSource,
        profileDataSource,
        settingsDataSource,
        addonStoreDataSource,
        authDataSource
    )

    fun rootViewController(): UIViewController = ComposeUIViewController {
        FluxaAppleApp()
    }

    fun updateCatalogHome(home: CatalogHomeUiState) {
        catalogHomeDataSource.update(home)
    }

    fun updateCatalogHomeJson(homeJson: String) {
        catalogHomeDataSource.updateJson(homeJson)
    }

    fun updateDetailJson(detailJson: String) {
        detailDataSource.updateJson(detailJson)
    }

    fun updateSearchJson(searchJson: String) {
        searchDataSource.updateJson(searchJson)
    }

    fun updateDiscoverJson(discoverJson: String) {
        discoverDataSource.updateJson(discoverJson)
    }

    fun updateLibraryJson(libraryJson: String) {
        libraryDataSource.updateJson(libraryJson)
    }

    fun updateCalendarJson(calendarJson: String) {
        calendarDataSource.updateJson(calendarJson)
    }

    fun updateAddonStoreJson(addonStoreJson: String) {
        addonStoreDataSource.updateJson(addonStoreJson)
    }

    fun updateAuthJson(authJson: String) {
        authDataSource.updateJson(authJson)
    }

}

@Composable
private fun FluxaAppleApp() {
    FluxaAppHost(platformServices = FluxaApple.platformServices)
}
