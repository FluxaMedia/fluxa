import FluxaCore
import SwiftUI

@main
struct FluxaTvosApp: App {
    private let headlessRuntime = requireFluxaAppleHeadlessRuntime()

    var body: some Scene {
        WindowGroup {
            Text(FluxaTvos.shared.homeTitle())
        }
    }
}
