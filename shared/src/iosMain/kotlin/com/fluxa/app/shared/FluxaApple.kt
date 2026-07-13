package com.fluxa.app.shared

import androidx.compose.runtime.Composable
import androidx.compose.ui.window.ComposeUIViewController
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.platform.AppleCatalogHomeDataSource
import com.fluxa.app.shared.platform.AppleDetailDataSource
import com.fluxa.app.shared.platform.AppleFluxaPlatformServices
import platform.UIKit.UIViewController

object FluxaApple {
    internal val catalogHomeDataSource = AppleCatalogHomeDataSource()
    internal val detailDataSource = AppleDetailDataSource()
    internal val platformServices = AppleFluxaPlatformServices(catalogHomeDataSource, detailDataSource)

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
}

@Composable
private fun FluxaAppleApp() {
    FluxaAppHost(
        platformServices = FluxaApple.platformServices,
        onCatalogAction = { action ->
            when (action) {
                CatalogAction.Refresh,
                is CatalogAction.LoadMore,
                is CatalogAction.ItemSelected,
                is CatalogAction.PlayRequested,
                is CatalogAction.ResumeRequested -> Unit
            }
        }
    )
}
