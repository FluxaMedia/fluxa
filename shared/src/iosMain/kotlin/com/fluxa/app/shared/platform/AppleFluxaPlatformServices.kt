package com.fluxa.app.shared.platform

class AppleFluxaPlatformServices(
    override val catalogHomeDataSource: AppleCatalogHomeDataSource,
    override val detailDataSource: AppleDetailDataSource,
    override val searchDataSource: AppleSearchDataSource,
    override val discoverDataSource: AppleDiscoverDataSource,
    override val calendarDataSource: AppleCalendarDataSource,
    override val libraryDataSource: AppleLibraryDataSource,
    override val profileDataSource: AppleProfileDataSource,
    override val settingsDataSource: AppleSettingsDataSource,
    override val addonStoreDataSource: AppleAddonStoreDataSource,
    override val authDataSource: AppleAuthDataSource
) : FluxaPlatformServices, FluxaDetailServices, FluxaSearchServices, FluxaDiscoverServices, FluxaCalendarServices,
    FluxaLibraryServices, FluxaProfileServices, FluxaSettingsServices, FluxaAddonStoreServices, FluxaAuthServices
