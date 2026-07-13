package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.detail.DetailDataSource
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

interface FluxaProfileServices {
    val profileDataSource: ProfileDataSource
}

interface FluxaPlaybackServices {
    val playbackController: PlaybackController
}

interface FluxaMobilePlatformServices :
    FluxaPlatformServices,
    FluxaSearchServices,
    FluxaDetailServices,
    FluxaProfileServices,
    FluxaPlaybackServices
