import SwiftUI

@main
struct FluxaIosApp: App {
    private let coreVersion: String
    private let headlessRuntime: FluxaAppleHeadlessRuntime
    private let appRuntime: FluxaAppleAppRuntime
    private let catalogStartup: FluxaAppleCatalogStartup
    private let catalogObserver: FluxaAppleCatalogNotificationObserver
    private let detailObserver: FluxaAppleDetailNotificationObserver
    private let searchObserver: FluxaAppleSearchNotificationObserver
    private let discoverObserver: FluxaAppleDiscoverNotificationObserver
    private let calendarObserver: FluxaAppleCalendarNotificationObserver
    private let libraryObserver: FluxaAppleLibraryNotificationObserver
    private let authObserver: FluxaAppleAuthNotificationObserver

    init() {
        let runtime = requireFluxaAppleHeadlessRuntime()
        coreVersion = FluxaRustCoreRuntime.version()
        headlessRuntime = runtime
        appRuntime = FluxaAppleAppRuntime(runtime: runtime)
        catalogStartup = FluxaAppleCatalogStartup(coordinator: appRuntime.coordinator)
        catalogObserver = FluxaAppleCatalogNotificationObserver(startup: catalogStartup)
        detailObserver = FluxaAppleDetailNotificationObserver(
            startup: FluxaAppleDetailStartup(coordinator: appRuntime.coordinator)
        )
        searchObserver = FluxaAppleSearchNotificationObserver(
            startup: FluxaAppleSearchStartup(coordinator: appRuntime.coordinator)
        )
        discoverObserver = FluxaAppleDiscoverNotificationObserver(
            startup: FluxaAppleDiscoverStartup(coordinator: appRuntime.coordinator)
        )
        calendarObserver = FluxaAppleCalendarNotificationObserver(
            startup: FluxaAppleCalendarStartup(coordinator: appRuntime.coordinator)
        )
        libraryObserver = FluxaAppleLibraryNotificationObserver(
            startup: FluxaAppleLibraryStartup(coordinator: appRuntime.coordinator)
        )
        authObserver = FluxaAppleAuthNotificationObserver(startup: FluxaAppleAuthStartup())
    }

    var body: some Scene {
        WindowGroup {
            FluxaRootView()
                .ignoresSafeArea()
        }
    }
}
