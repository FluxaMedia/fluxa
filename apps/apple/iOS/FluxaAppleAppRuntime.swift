import Foundation

@MainActor
final class FluxaAppleAppRuntime {
    let coordinator: FluxaAppleHeadlessCoordinator

    init(runtime: FluxaAppleHeadlessRuntime) {
        let configurationStore = FluxaAppleAddonConfigurationStore()
        let homeHandler = FluxaAppleHomeEffectHandler(
            configurationStore: configurationStore
        )
        let handler = FluxaAppleCompositeEffectHandler(
            home: homeHandler,
            plugins: FluxaApplePluginsEffectHandler()
        )
        coordinator = FluxaAppleHeadlessCoordinator(
            runtime: runtime,
            executor: FluxaApplePlatformEffectExecutor(handler: handler)
        )
    }
}

/// Routes every iOS Core effect through the app's one coordinator/runtime.
/// Keep feature-specific I/O handlers, but never let a feature create its own
/// headless engine or effect-drain loop.
private final class FluxaAppleCompositeEffectHandler: FluxaApplePlatformEffectHandler {
    private let home: FluxaAppleHomeEffectHandler
    private let plugins: FluxaApplePluginsEffectHandler

    init(home: FluxaAppleHomeEffectHandler, plugins: FluxaApplePluginsEffectHandler) {
        self.home = home
        self.plugins = plugins
    }

    func execute(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        if effect.type == FluxaHeadlessEffectType.fetchPluginManifest {
            return try await plugins.execute(effect: effect)
        }
        return try await home.execute(effect: effect)
    }
}
