package com.fluxa.app.shared.platform

class AppleFluxaPlatformServices(
    override val catalogHomeDataSource: AppleCatalogHomeDataSource,
    override val detailDataSource: AppleDetailDataSource,
    override val searchDataSource: AppleSearchDataSource
) : FluxaPlatformServices, FluxaDetailServices, FluxaSearchServices
