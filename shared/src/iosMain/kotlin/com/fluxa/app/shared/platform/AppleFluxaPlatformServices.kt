package com.fluxa.app.shared.platform

class AppleFluxaPlatformServices(
    override val catalogHomeDataSource: AppleCatalogHomeDataSource,
    override val detailDataSource: AppleDetailDataSource
) : FluxaPlatformServices, FluxaDetailServices
