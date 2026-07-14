package com.fluxa.app.shared

import androidx.compose.runtime.Composable
import androidx.compose.ui.window.ComposeUIViewController
import com.fluxa.app.shared.platform.AppleCatalogHomeSnapshot
import com.fluxa.app.shared.platform.AppleSearchSnapshot
import com.fluxa.app.shared.platform.AppleDiscoverRequestSnapshot
import com.fluxa.app.shared.platform.AppleDiscoverSnapshot
import com.fluxa.app.shared.platform.AppleCalendarSnapshot
import com.fluxa.app.shared.platform.AppleAuthSubmitSnapshot
import com.fluxa.app.shared.platform.AppleAuthSnapshot
import com.fluxa.app.shared.platform.AppleLibrarySnapshot
import com.fluxa.app.shared.platform.AppleDetailRequestSnapshot
import com.fluxa.app.shared.platform.AppleDetailSnapshot
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

    fun setCatalogHomeRefreshHandler(handler: () -> Unit) {
        catalogHomeDataSource.setOnRefreshRequested(handler)
    }

    fun updateCatalogHome(snapshot: AppleCatalogHomeSnapshot) {
        catalogHomeDataSource.update(snapshot)
    }

    fun setSearchHandler(handler: (String) -> Unit) {
        searchDataSource.setOnSearchRequested(handler)
    }

    fun updateSearch(snapshot: AppleSearchSnapshot) {
        searchDataSource.update(snapshot)
    }

    fun setDiscoverHandler(handler: (AppleDiscoverRequestSnapshot) -> Unit) {
        discoverDataSource.setOnDiscoverRequested(handler)
    }

    fun updateDiscover(snapshot: AppleDiscoverSnapshot) {
        discoverDataSource.update(snapshot)
    }

    fun setCalendarMonthHandler(handler: (Int, Int) -> Unit) {
        calendarDataSource.setOnMonthRequested(handler)
    }

    fun updateCalendar(snapshot: AppleCalendarSnapshot) {
        calendarDataSource.update(snapshot)
    }

    fun setAuthSubmitHandler(handler: (AppleAuthSubmitSnapshot) -> Unit) {
        authDataSource.setOnSubmitRequested(handler)
    }

    fun updateAuth(snapshot: AppleAuthSnapshot) {
        authDataSource.update(snapshot)
    }

    fun setLibraryRefreshHandler(handler: () -> Unit) {
        libraryDataSource.setOnRefreshRequested(handler)
    }

    fun updateLibrary(snapshot: AppleLibrarySnapshot) {
        libraryDataSource.update(snapshot)
    }

    fun setDetailHandlers(
        load: (AppleDetailRequestSnapshot) -> Unit,
        watchlist: (AppleDetailRequestSnapshot) -> Unit
    ) {
        detailDataSource.setHandlers(load, watchlist)
    }

    fun updateDetail(snapshot: AppleDetailSnapshot) {
        detailDataSource.update(snapshot)
    }






}

@Composable
private fun FluxaAppleApp() {
    FluxaAppHost(platformServices = FluxaApple.platformServices)
}
