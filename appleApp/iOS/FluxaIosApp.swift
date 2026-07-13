import SwiftUI

@main
struct FluxaIosApp: App {
    private let coreVersion = FluxaRustCoreRuntime.version()

    var body: some Scene {
        WindowGroup {
            FluxaRootView()
                .ignoresSafeArea()
        }
    }
}
