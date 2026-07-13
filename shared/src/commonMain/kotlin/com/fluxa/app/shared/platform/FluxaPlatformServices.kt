package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.detail.DetailDataSource
import com.fluxa.app.shared.feature.player.PlaybackController
import com.fluxa.app.shared.feature.profile.ProfileDataSource
import com.fluxa.app.shared.feature.search.SearchDataSource

interface FluxaPlatformServices {
    val catalogHomeDataSource: CatalogHomeDataSource
    val searchDataSource: SearchDataSource
    val detailDataSource: DetailDataSource
    val profileDataSource: ProfileDataSource
    val playbackController: PlaybackController
}
