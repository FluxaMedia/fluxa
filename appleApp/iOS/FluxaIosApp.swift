import SwiftUI

@main
struct FluxaIosApp: App {
    private let coreVersion = FluxaRustCoreRuntime.version()
    private let headlessRuntime = requireFluxaAppleHeadlessRuntime()
    private let catalogStartup: FluxaAppleCatalogStartup
    @State private var hasStartedCatalog = false

    init() {
        catalogStartup = FluxaAppleCatalogStartup(runtime: headlessRuntime)
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
