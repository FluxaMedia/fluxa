import SwiftUI

@main
struct FluxaIosApp: App {
    private let coreVersion: String
    private let headlessRuntime: FluxaAppleHeadlessRuntime
    private let catalogStartup: FluxaAppleCatalogStartup
    private let detailObserver: FluxaAppleDetailNotificationObserver
    private let searchObserver: FluxaAppleSearchNotificationObserver
    @State private var hasStartedCatalog = false

    init() {
        let runtime = requireFluxaAppleHeadlessRuntime()
        coreVersion = FluxaRustCoreRuntime.version()
        headlessRuntime = runtime
        catalogStartup = FluxaAppleCatalogStartup(runtime: runtime)
        detailObserver = FluxaAppleDetailNotificationObserver(
            startup: FluxaAppleDetailStartup(runtime: runtime)
        )
        searchObserver = FluxaAppleSearchNotificationObserver(
            startup: FluxaAppleSearchStartup(runtime: runtime)
        )
    }

    var body: some Scene {
        WindowGroup {
            FluxaRootView()
                .ignoresSafeArea()
                .task {
                    guard !hasStartedCatalog else {
                        return
                    }
                    hasStartedCatalog = true
                    await catalogStartup.refresh()
                }
        }
    }
}
