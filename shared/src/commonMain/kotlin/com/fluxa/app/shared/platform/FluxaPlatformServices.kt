package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.addonstore.AddonStoreDataSource
import com.fluxa.app.shared.feature.auth.AuthDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.calendar.CalendarDataSource
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.discover.DiscoverDataSource
import com.fluxa.app.shared.feature.library.LibraryDataSource
import com.fluxa.app.shared.feature.plugins.PluginsDataSource
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.search.SearchDataSource
import com.fluxa.app.shared.feature.settings.SettingsDataSource

interface FluxaPlatformServices {
    val catalogHomeDataSource: CatalogHomeDataSource
}

interface FluxaSearchServices {
    val searchDataSource: SearchDataSource
}

interface FluxaDetailServices {
    val detailDataSource: DetailDataSource
}

interface FluxaDiscoverServices {
    val discoverDataSource: DiscoverDataSource
}

interface FluxaCalendarServices {
    val calendarDataSource: CalendarDataSource
}

interface FluxaLibraryServices {
    val libraryDataSource: LibraryDataSource
}

interface FluxaProfileServices {
    val profileDataSource: ProfileDataSource
}

interface FluxaAddonStoreServices {
    val addonStoreDataSource: AddonStoreDataSource
}

interface FluxaPluginsServices {
    val pluginsDataSource: PluginsDataSource
}

interface FluxaAuthServices {
    val authDataSource: AuthDataSource
}

interface FluxaSettingsServices {
    val settingsDataSource: SettingsDataSource
}

interface FluxaMobilePlatformServices :
    FluxaPlatformServices,
    FluxaSearchServices,
    FluxaDiscoverServices,
    FluxaCalendarServices,
    FluxaLibraryServices,
    FluxaDetailServices,
    FluxaProfileServices,
    FluxaAddonStoreServices,
    FluxaPluginsServices,
    FluxaAuthServices,
    FluxaSettingsServices
