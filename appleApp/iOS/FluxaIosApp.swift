import SwiftUI

@main
struct FluxaIosApp: App {
    private let coreVersion = FluxaRustCoreRuntime.version()
    private let headlessRuntime = requireFluxaAppleHeadlessRuntime()

    var body: some Scene {
        WindowGroup {
            FluxaRootView()
                .ignoresSafeArea()
        }
    }
}
