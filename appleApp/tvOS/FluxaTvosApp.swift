import FluxaCore
import SwiftUI

@main
struct FluxaTvosApp: App {
    var body: some Scene {
        WindowGroup {
            Text(FluxaTvos.shared.homeTitle())
        }
    }
}
