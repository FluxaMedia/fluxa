import Foundation

@MainActor
final class FluxaAppleAppRuntime {
    let coordinator: FluxaAppleHeadlessCoordinator

    init(runtime: FluxaAppleHeadlessRuntime) {
        let configurationStore = FluxaAppleAddonConfigurationStore()
        let handler = FluxaAppleHomeEffectHandler(
            configurationStore: configurationStore,
            catalogBootstrap: FluxaAppleCatalogBootstrap()
        )
        coordinator = FluxaAppleHeadlessCoordinator(
            runtime: runtime,
            executor: FluxaApplePlatformEffectExecutor(handler: handler)
        )
    }
}
