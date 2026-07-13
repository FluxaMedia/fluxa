package com.fluxa.app.shared

import androidx.compose.runtime.Composable
import androidx.compose.ui.window.ComposeUIViewController
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.platform.AppleCatalogHomeDataSource
import com.fluxa.app.shared.platform.AppleDetailDataSource
import com.fluxa.app.shared.platform.AppleFluxaPlatformServices
import com.fluxa.app.shared.platform.AppleSearchDataSource
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import platform.Foundation.NSNotificationCenter
import platform.UIKit.UIViewController

object FluxaApple {
    internal val catalogHomeDataSource = AppleCatalogHomeDataSource()
    internal val detailDataSource = AppleDetailDataSource()
    internal val searchDataSource = AppleSearchDataSource()
    internal val platformServices = AppleFluxaPlatformServices(
        catalogHomeDataSource,
        detailDataSource,
        searchDataSource
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

    internal fun requestDetail(item: CatalogItemUiModel) {
        val request = buildJsonObject {
            put("id", item.id)
            put("type", item.type)
            item.source.addonTransportUrl?.let { put("addonTransportUrl", it) }
            item.source.catalogType?.let { put("catalogType", it) }
        }.toString()
        NSNotificationCenter.defaultCenter.postNotificationName(
            aName = "FluxaAppleDetailRequested",
            `object` = request,
            userInfo = null
        )
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
                is CatalogAction.PlayRequested,
                is CatalogAction.ResumeRequested -> Unit
                is CatalogAction.ItemSelected -> FluxaApple.requestDetail(action.item)
            }
        }
    )
}
