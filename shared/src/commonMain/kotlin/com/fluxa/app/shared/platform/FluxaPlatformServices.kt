package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.addonstore.AddonStoreDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.calendar.CalendarDataSource
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.discover.DiscoverDataSource
import com.fluxa.app.shared.feature.library.LibraryDataSource
import com.fluxa.app.shared.feature.player.PlaybackController
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.search.SearchDataSource

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

interface FluxaPlaybackServices {
    val playbackController: PlaybackController
}

interface FluxaAddonStoreServices {
    val addonStoreDataSource: AddonStoreDataSource
}

interface FluxaMobilePlatformServices :
    FluxaPlatformServices,
    FluxaSearchServices,
    FluxaDiscoverServices,
    FluxaCalendarServices,
    FluxaLibraryServices,
    FluxaDetailServices,
    FluxaProfileServices,
    FluxaPlaybackServices,
    FluxaAddonStoreServices
